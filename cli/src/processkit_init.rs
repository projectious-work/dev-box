//! Install processkit content into a project at `aibox init` time.
//!
//! Called by `cmd_init` after the existing init pipeline (aibox.toml,
//! .devcontainer/, context/ scaffold) has succeeded. Reads the
//! [processkit] section from the just-written config, fetches the
//! configured source@version into the user's cache, walks the cache,
//! and copies files into the project per the mapping in
//! [`crate::processkit_install::install_action_for`].
//!
//! Writes the lock and manifest under `context/.aibox/`.
//!
//! ## Error policy
//!
//! The public entry point [`install_processkit`] propagates fetch and
//! I/O errors to its caller (`cmd_init`), which then decides whether to
//! warn-and-continue or fail hard. The install itself is best-effort
//! for individual files only in the sense that *unrecognized* files in
//! the cache are silently skipped (that's the install-mapping contract
//! in [`crate::processkit_install`]); once a file has been chosen for
//! install, any copy failure aborts the run.
//!
//! ## Idempotency
//!
//! Re-running on the same `(source, version)` with no manual edits to
//! installed files is a no-op for file content (same bytes land in the
//! same places), but the lock file is always rewritten with a fresh
//! `installed_at` timestamp so callers can tell when the last install
//! ran.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::config::{AiboxConfig, PROCESSKIT_VERSION_UNSET};
use crate::manifest::{self, ProcessKitLock, group_for_path};
use crate::processkit_install::{InstallAction, install_action_for};
use crate::processkit_source;

/// Result of a processkit install run, for reporting.
#[derive(Debug, Default, Clone)]
pub struct InstallReport {
    pub files_installed: usize,
    pub files_skipped: usize,
    pub groups_touched: usize,
    pub fetched_from: String,
    pub fetched_version: String,
    pub skipped_due_to_unset: bool,
}

/// Install processkit content into the given project root, based on the
/// given config. See module docs for error policy and idempotency notes.
pub fn install_processkit(
    project_root: &Path,
    config: &AiboxConfig,
) -> Result<InstallReport> {
    let pk = &config.processkit;

    // 1. Sentinel check — no fetch, no I/O, caller prints a skip message.
    if pk.version == PROCESSKIT_VERSION_UNSET {
        return Ok(InstallReport {
            fetched_from: pk.source.clone(),
            fetched_version: PROCESSKIT_VERSION_UNSET.to_string(),
            skipped_due_to_unset: true,
            ..Default::default()
        });
    }

    // 2. Fetch into cache.
    let fetched = processkit_source::fetch(
        &pk.source,
        &pk.version,
        pk.branch.as_deref(),
        &pk.src_path,
    )
    .with_context(|| {
        format!(
            "failed to fetch processkit {}@{}",
            pk.source, pk.version
        )
    })?;

    // 3. Walk and install.
    let (files_installed, files_skipped, groups_touched) =
        install_files_from_cache(&fetched.src_path, project_root)?;

    // 4. Build the manifest from the cache.
    let built_manifest = manifest::manifest_from_cache(&fetched.src_path)
        .with_context(|| {
            format!(
                "failed to build manifest from cache {}",
                fetched.src_path.display()
            )
        })?;

    // 5. Write the lock file (always — fresh installed_at every run).
    let lock = ProcessKitLock {
        source: pk.source.clone(),
        version: pk.version.clone(),
        src_path: pk.src_path.clone(),
        branch: pk.branch.clone(),
        resolved_commit: fetched.resolved_commit.clone(),
        installed_at: Utc::now().to_rfc3339(),
    };
    manifest::write_lock(project_root, &lock)
        .context("failed to write processkit.lock")?;

    // 6. Write the manifest file.
    manifest::write_manifest(project_root, &built_manifest)
        .context("failed to write processkit.manifest")?;

    // 7. Return the report.
    Ok(InstallReport {
        files_installed,
        files_skipped,
        groups_touched,
        fetched_from: pk.source.clone(),
        fetched_version: pk.version.clone(),
        skipped_due_to_unset: false,
    })
}

/// Walk `cache_src_path` recursively, consult the install mapping for
/// each file, and copy Install files into `project_root`. Returns
/// `(files_installed, files_skipped, groups_touched)`.
///
/// This function is extracted from [`install_processkit`] so it can be
/// exercised in unit tests with a synthetic cache directory, without
/// needing to run [`processkit_source::fetch`].
pub fn install_files_from_cache(
    cache_src_path: &Path,
    project_root: &Path,
) -> Result<(usize, usize, usize)> {
    if !cache_src_path.is_dir() {
        anyhow::bail!(
            "install_files_from_cache: {} is not a directory",
            cache_src_path.display()
        );
    }

    let mut files_installed = 0usize;
    let mut files_skipped = 0usize;
    let mut groups: BTreeSet<String> = BTreeSet::new();

    walk_and_install(
        cache_src_path,
        cache_src_path,
        project_root,
        &mut files_installed,
        &mut files_skipped,
        &mut groups,
    )?;

    Ok((files_installed, files_skipped, groups.len()))
}

/// Recursive walker mirroring `manifest::manifest_from_cache`'s skip rules.
fn walk_and_install(
    root: &Path,
    dir: &Path,
    project_root: &Path,
    files_installed: &mut usize,
    files_skipped: &mut usize,
    groups: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", path.display()))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();

        if should_skip_entry(&name_str) {
            continue;
        }

        if file_type.is_dir() {
            walk_and_install(
                root,
                &path,
                project_root,
                files_installed,
                files_skipped,
                groups,
            )?;
            continue;
        }
        if !file_type.is_file() {
            // Symlinks, fifos, etc. — ignore.
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .with_context(|| {
                format!(
                    "failed to relativize {} against {}",
                    path.display(),
                    root.display()
                )
            })?
            .to_path_buf();

        match install_action_for(&rel) {
            InstallAction::Install(target_rel) => {
                let dest = project_root.join(&target_rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create parent directory {}", parent.display())
                    })?;
                }
                fs::copy(&path, &dest).with_context(|| {
                    format!(
                        "failed to copy {} -> {}",
                        path.display(),
                        dest.display()
                    )
                })?;
                *files_installed += 1;
                if let Some(g) = group_for_path(&rel) {
                    groups.insert(g);
                }
            }
            InstallAction::Skip => {
                *files_skipped += 1;
            }
        }
    }
    Ok(())
}

/// Skip-rules matching `manifest::manifest_from_cache` (which we rely on
/// to build the manifest from the same cache).
fn should_skip_entry(name: &str) -> bool {
    if name == ".git" || name == "__pycache__" || name == ".fetch-complete" {
        return true;
    }
    if name.starts_with('.') {
        return true;
    }
    if name.ends_with(".pyc") {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build a synthetic processkit-shaped src tree under `root`. Mirrors
    /// the shape used by the manifest module's tests, plus extras to
    /// exercise skip rules (INDEX.md, skills/FORMAT.md, packages/...).
    fn synth_cache(root: &Path) {
        fs::create_dir_all(root.join("skills/event-log/templates")).unwrap();
        fs::create_dir_all(root.join("skills/workitem-management")).unwrap();
        fs::create_dir_all(root.join("primitives/schemas")).unwrap();
        fs::create_dir_all(root.join("primitives/state-machines")).unwrap();
        fs::create_dir_all(root.join("lib/processkit")).unwrap();
        fs::create_dir_all(root.join("processes")).unwrap();
        fs::create_dir_all(root.join("packages")).unwrap();

        // Installed files.
        fs::write(root.join("PROVENANCE.toml"), "version = \"v0.4.0\"\n").unwrap();
        fs::write(root.join("skills/event-log/SKILL.md"), "# event log\n").unwrap();
        fs::write(
            root.join("skills/event-log/templates/entry.yaml"),
            "name: x\n",
        )
        .unwrap();
        fs::write(
            root.join("skills/workitem-management/SKILL.md"),
            "# workitem mgmt\n",
        )
        .unwrap();
        fs::write(
            root.join("primitives/schemas/workitem.yaml"),
            "name: workitem\n",
        )
        .unwrap();
        fs::write(
            root.join("primitives/state-machines/workflow.yaml"),
            "name: workflow\n",
        )
        .unwrap();
        fs::write(root.join("primitives/FORMAT.md"), "# primitives format\n").unwrap();
        fs::write(root.join("lib/processkit/entity.py"), "print('x')\n").unwrap();
        fs::write(root.join("processes/release.md"), "# release\n").unwrap();

        // Skipped files.
        fs::write(root.join("INDEX.md"), "# top index\n").unwrap();
        fs::write(root.join("skills/INDEX.md"), "# skills index\n").unwrap();
        fs::write(root.join("skills/FORMAT.md"), "# skills format\n").unwrap();
        fs::write(root.join("primitives/INDEX.md"), "# primitives index\n").unwrap();
        fs::write(root.join("packages/minimal.yaml"), "name: minimal\n").unwrap();
        fs::write(root.join("packages/software.yaml"), "name: software\n").unwrap();
    }

    /// Returns a default AiboxConfig with `processkit.version` overridden.
    fn config_with_version(version: &str) -> AiboxConfig {
        use crate::config::{
            AddonsSection, AiSection, AiboxConfig, AiboxSection, AudioSection,
            ContainerSection, ContextSection, CustomizationSection, ProcessKitSection,
            SkillsSection,
        };
        AiboxConfig {
            aibox: AiboxSection {
                version: "0.14.4".to_string(),
                base: crate::config::BaseImage::Debian,
            },
            container: ContainerSection {
                name: "t".to_string(),
                hostname: "t".to_string(),
                user: "aibox".to_string(),
                post_create_command: None,
                keepalive: false,
            },
            context: ContextSection::default(),
            ai: AiSection {
                providers: vec![crate::config::AiProvider::Claude],
            },
            process: None,
            addons: AddonsSection::default(),
            skills: SkillsSection::default(),
            processkit: ProcessKitSection {
                version: version.to_string(),
                ..ProcessKitSection::default()
            },
            customization: CustomizationSection::default(),
            audio: AudioSection::default(),
        }
    }

    // -- sentinel skip ------------------------------------------------------

    #[test]
    fn install_processkit_skips_on_unset_sentinel() {
        let tmp = TempDir::new().unwrap();
        let cfg = config_with_version(PROCESSKIT_VERSION_UNSET);
        let report = install_processkit(tmp.path(), &cfg).unwrap();
        assert!(report.skipped_due_to_unset);
        assert_eq!(report.files_installed, 0);
        assert_eq!(report.files_skipped, 0);
        assert_eq!(report.fetched_version, PROCESSKIT_VERSION_UNSET);
        // No lock, no manifest — we did no I/O.
        assert!(!manifest::lock_path(tmp.path()).exists());
        assert!(!manifest::manifest_path(tmp.path()).exists());
    }

    // -- install_files_from_cache -------------------------------------------

    #[test]
    fn install_copies_skill_files_under_claude_skills() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(proj.join(".claude/skills/event-log/SKILL.md").exists());
        assert!(
            proj.join(".claude/skills/event-log/templates/entry.yaml").exists()
        );
        assert!(proj.join(".claude/skills/workitem-management/SKILL.md").exists());
    }

    #[test]
    fn install_copies_lib_files_under_claude_skills_lib() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(
            proj.join(".claude/skills/_lib/processkit/entity.py").exists(),
            "lib file should land under .claude/skills/_lib/processkit/"
        );
    }

    #[test]
    fn install_copies_primitive_schemas_under_aibox_dir() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(proj.join("context/.aibox/schemas/workitem.yaml").exists());
        assert!(
            proj.join("context/.aibox/state-machines/workflow.yaml").exists()
        );
        assert!(proj.join("context/.aibox/primitives-FORMAT.md").exists());
    }

    #[test]
    fn install_copies_provenance_with_renamed_path() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        let dest = proj.join("context/.aibox/processkit-provenance.toml");
        assert!(dest.exists(), "provenance should be installed at renamed path");
        let body = fs::read_to_string(dest).unwrap();
        assert!(body.contains("v0.4.0"));
    }

    #[test]
    fn install_skips_index_md_files() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        // None of the INDEX.md files should have been copied anywhere
        // under the project root.
        for rel in [
            ".claude/skills/INDEX.md",
            "context/.aibox/INDEX.md",
            "INDEX.md",
        ] {
            assert!(
                !proj.join(rel).exists(),
                "INDEX.md should not appear at {}",
                rel
            );
        }
    }

    #[test]
    fn install_skips_packages() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        // packages/* files should not exist anywhere under the project.
        assert!(!proj.join("packages").exists());
        // And definitely no accidental copy under .claude/skills or
        // context/.aibox.
        assert!(!proj.join(".claude/skills/packages").exists());
        assert!(!proj.join("context/.aibox/packages").exists());
    }

    #[test]
    fn install_writes_lock_and_manifest_via_helpers() {
        // Unit test for the lock/manifest write path — we call the
        // helpers directly since install_processkit's full path requires
        // a real fetch. This gives us end-to-end assurance that the
        // manifest/lock files land where expected.
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        let mf = manifest::manifest_from_cache(&cache).unwrap();
        manifest::write_manifest(&proj, &mf).unwrap();

        let lock = ProcessKitLock {
            source: "https://github.com/projectious-work/processkit.git".to_string(),
            version: "v0.4.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: Some("deadbeef".to_string()),
            installed_at: Utc::now().to_rfc3339(),
        };
        manifest::write_lock(&proj, &lock).unwrap();

        assert!(proj.join("context/.aibox/processkit.lock").exists());
        assert!(proj.join("context/.aibox/processkit.manifest").exists());

        // Round-trip: reread the manifest and confirm it has the
        // expected files.
        let back = manifest::read_manifest(&proj).unwrap().unwrap();
        assert!(back.files.contains_key("PROVENANCE.toml"));
        assert!(back.files.contains_key("skills/event-log/SKILL.md"));
        let entry = back.files.get("PROVENANCE.toml").unwrap();
        assert!(!entry.upstream_sha.is_empty());
    }

    #[test]
    fn install_counts_groups() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        let (installed, skipped, groups) =
            install_files_from_cache(&cache, &proj).unwrap();

        // 9 installed files (PROVENANCE, 2 event-log, workitem-mgmt SKILL,
        // workitem schema, workflow state machine, FORMAT, lib/entity.py,
        // release.md).
        assert_eq!(installed, 9, "unexpected installed count");
        // 6 skipped: 3 INDEX.md, skills/FORMAT.md, 2 packages.
        assert_eq!(skipped, 6, "unexpected skipped count");
        // Groups: PROVENANCE, skills/event-log, skills/workitem-management,
        // primitives/schemas/workitem, primitives/state-machines/workflow,
        // primitives (FORMAT.md), lib, processes/release — 8 total.
        assert_eq!(groups, 8, "unexpected group count");
    }

    #[test]
    fn install_creates_parent_directories() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        // Build a minimal cache with one deeply-nested file.
        fs::create_dir_all(cache.join("skills/deep/nested/path")).unwrap();
        fs::write(cache.join("PROVENANCE.toml"), "version = \"x\"\n").unwrap();
        fs::write(
            cache.join("skills/deep/nested/path/SKILL.md"),
            "# deep\n",
        )
        .unwrap();
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        let dest = proj.join(".claude/skills/deep/nested/path/SKILL.md");
        assert!(
            dest.exists(),
            "parent directories should have been created for {}",
            dest.display()
        );
    }

    #[test]
    fn install_overwrites_existing_files() {
        // Idempotency: run once, corrupt the dest, run again, confirm
        // final bytes match the source.
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();
        let dest = proj.join(".claude/skills/event-log/SKILL.md");
        assert_eq!(fs::read_to_string(&dest).unwrap(), "# event log\n");

        // Corrupt the installed file.
        fs::write(&dest, "tampered\n").unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "tampered\n");

        // Re-run install — destination should be restored.
        install_files_from_cache(&cache, &proj).unwrap();
        assert_eq!(
            fs::read_to_string(&dest).unwrap(),
            "# event log\n",
            "re-running install should overwrite tampered file"
        );
    }

    #[test]
    fn install_skips_git_and_pycache() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);

        // Add noise the walker must skip.
        fs::create_dir_all(cache.join(".git/objects")).unwrap();
        fs::write(cache.join(".git/objects/foo"), b"git internals").unwrap();
        fs::create_dir_all(cache.join("skills/event-log/__pycache__")).unwrap();
        fs::write(cache.join("skills/event-log/__pycache__/x.pyc"), b"junk").unwrap();
        fs::write(cache.join(".fetch-complete"), b"deadbeef").unwrap();

        fs::create_dir_all(&proj).unwrap();
        install_files_from_cache(&cache, &proj).unwrap();

        // Nothing from .git or __pycache__ should land in the project.
        assert!(!proj.join(".git").exists());
        assert!(
            !proj.join(".claude/skills/event-log/__pycache__").exists()
        );
    }
}
