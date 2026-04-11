//! Install content-source payload into a project at `aibox init` time.
//!
//! Called by `cmd_init` after the existing init pipeline (aibox.toml,
//! .devcontainer/, context/ scaffold) has succeeded. Reads the
//! `[processkit]` section from the just-written config (today's only
//! content source), fetches the configured `source@version` into the
//! user's cache, walks the cache, and copies files into the project
//! per the mapping in [`crate::content_install::install_action_for`].
//!
//! The fetcher and the install mapping are content-source-neutral —
//! they don't know or care that the content happens to be processkit.
//! When aibox grows additional content sources (community packs,
//! company forks, …), they will reuse this same machinery.
//!
//! Two pieces of state are written next to the live install:
//!
//! 1. **`aibox.lock`** at the project root — pinned `(source, version,
//!    commit)`. Cargo-style, top level, git-tracked.
//! 2. **`context/templates/processkit/<version>/...`** — a verbatim copy
//!    of the cache `<src_path>/` (modulo `.git`, `__pycache__`, dotfiles
//!    and `*.pyc`). This is the immutable "as-installed" reference used
//!    by the 3-way diff in `content_diff` to detect upstream-vs-local
//!    edits without needing a SHA manifest. (The path still mentions
//!    `processkit` because today's only content source is processkit;
//!    the layout will generalise to `<source-id>/<version>/` when
//!    multi-source support lands.)
//!
//! ## Error policy
//!
//! The public entry point [`install_content_source`] propagates fetch
//! and I/O errors to its caller (`cmd_init`), which then decides
//! whether to warn-and-continue or fail hard. The install itself is
//! best-effort for individual files only in the sense that
//! *unrecognized* files in the cache are silently skipped (that's the
//! install-mapping contract in [`crate::content_install`]); once a
//! file has been chosen for install, any copy failure aborts the run.
//!
//! ## Idempotency
//!
//! Re-running on the same `(source, version)` with no manual edits to
//! installed files is a no-op for file content (same bytes land in the
//! same places), but the lock file is always rewritten with a fresh
//! `installed_at` timestamp so callers can tell when the last install
//! ran. The templates dir for the version is wiped and re-copied so it
//! always reflects the current cache exactly.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;

use crate::config::{AiboxConfig, PROCESSKIT_VERSION_UNSET};
use crate::content_install::{InstallAction, install_action_for};
use crate::content_source;
use crate::context;
use crate::lock::{self, AiboxLock, group_for_path, should_skip_entry};
use crate::processkit_vocab::{
    self, SKILL_FILENAME, TEMPLATES_PROCESSKIT_DIR, mirror_packages_dir, mirror_skills_dir,
};

// ---------------------------------------------------------------------------
// [skills].include / [skills].exclude activation (BACK-118 / DEC-035)
// ---------------------------------------------------------------------------

/// Shape of a processkit `packages/<name>.yaml` file. Only the fields
/// aibox needs to compute the effective skill set are deserialized.
#[derive(Debug, Deserialize)]
struct PackageYaml {
    spec: PackageSpec,
}

#[derive(Debug, Deserialize)]
struct PackageSpec {
    #[serde(default)]
    extends: Vec<String>,
    #[serde(default)]
    includes: PackageIncludes,
}

#[derive(Debug, Default, Deserialize)]
struct PackageIncludes {
    #[serde(default)]
    skills: Vec<String>,
}

/// Compute the effective skill set for a project from the templates
/// mirror at the currently-pinned processkit version.
///
/// Composition rule (DEC-035 / BACK-118):
///
/// 1. Start with the union of every selected `[context].packages` package's
///    skill list, recursively expanding `extends:` (cycles → error).
/// 2. Add anything in `[skills].include`.
/// 3. Remove anything in `[skills].exclude`.
///
/// Returns `Ok(None)` when the templates mirror is missing (e.g.
/// processkit version is "unset" — no install has happened yet). Real
/// callers treat `None` as "install everything" (the v0.16.4 behavior).
///
/// Returns `Ok(Some(set))` when the mirror is present, even if the set
/// happens to be empty (e.g. user has only an `[skills].exclude` that
/// removes everything — silly but supported).
pub fn build_effective_skill_set(
    project_root: &Path,
    config: &AiboxConfig,
) -> Result<Option<HashSet<String>>> {
    if config.processkit.version == PROCESSKIT_VERSION_UNSET {
        return Ok(None);
    }
    let Some(packages_dir) = mirror_packages_dir(project_root, &config.processkit.version) else {
        return Ok(None);
    };

    // Step 1: walk every selected [context].packages package, recursively
    // expand `extends:`, collect the union of skill names.
    let mut effective: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = config.context.packages.iter().cloned().collect();
    let mut visited: HashSet<String> = HashSet::new();
    while let Some(pkg_name) = queue.pop_front() {
        if !visited.insert(pkg_name.clone()) {
            // Already processed — cycle protection.
            continue;
        }
        let pkg_path = packages_dir.join(format!("{}.yaml", pkg_name));
        if !pkg_path.is_file() {
            anyhow::bail!(
                "package '{}' selected in [context].packages does not exist at {}",
                pkg_name,
                pkg_path.display()
            );
        }
        let body = fs::read_to_string(&pkg_path)
            .with_context(|| format!("failed to read {}", pkg_path.display()))?;
        let parsed: PackageYaml = serde_yaml::from_str(&body)
            .with_context(|| format!("failed to parse package YAML {}", pkg_path.display()))?;

        for skill in parsed.spec.includes.skills {
            effective.insert(skill);
        }
        for parent in parsed.spec.extends {
            queue.push_back(parent);
        }
    }

    // Step 2: add user includes.
    for skill in &config.skills.include {
        effective.insert(skill.clone());
    }

    // Step 3: remove user excludes.
    for skill in &config.skills.exclude {
        effective.remove(skill);
    }

    // Step 4: enforce core skills (metadata.processkit.core: true).
    // Core skills are always installed regardless of include/exclude — they
    // are added back unconditionally after the exclude pass. `aibox doctor`
    // warns separately when a core skill appears in [skills].exclude.
    // See processkit/aibox#36 for the proposed convention.
    let skills_for_core =
        mirror_skills_dir(project_root, &config.processkit.version).unwrap_or_default();
    let core_skills = collect_core_skills(&skills_for_core);
    for skill in core_skills {
        effective.insert(skill);
    }

    Ok(Some(effective))
}

/// Walk `skills_dir` (the templates mirror's `skills/` subdirectory) and
/// return the names of skills whose frontmatter declares `core: true`.
///
/// Called by `build_effective_skill_set` to re-insert core skills after
/// the exclude pass, and by `validate_skill_overrides` to emit doctor
/// warnings when the user attempts to exclude a core skill.
///
/// Returns an empty vec when `skills_dir` does not exist (e.g. no mirror
/// yet on the first install).
pub fn collect_core_skills(skills_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(skills_dir) else {
        return Vec::new();
    };
    let mut core = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match entry.file_name().to_str() {
            Some(n) if !n.starts_with('_') && !n.starts_with('.') => n.to_string(),
            _ => continue,
        };
        let skill_file = path.join(SKILL_FILENAME);
        if !skill_file.exists() {
            continue;
        }
        if processkit_vocab::parse_skill_frontmatter(&skill_file)
            .map(|fm| fm.is_core())
            .unwrap_or(false)
        {
            core.push(name);
        }
    }
    core
}

/// Validate that every name in `[skills].include` and `[skills].exclude`
/// corresponds to a real skill. Also warns when a core skill is in exclude
/// (it will be installed regardless — see processkit/aibox#36).
/// Returns diagnostic strings (empty if all OK); used by `aibox doctor`.
pub fn validate_skill_overrides(project_root: &Path, config: &AiboxConfig) -> Result<Vec<String>> {
    let effective = match build_effective_skill_set(project_root, config)? {
        Some(set) => set,
        None => return Ok(Vec::new()),
    };
    // Build the universe = set BEFORE applying user overrides.
    let Some(mirror_skills_dir) = mirror_skills_dir(project_root, &config.processkit.version)
    else {
        return Ok(Vec::new());
    };
    let mut all_skills: HashSet<String> = HashSet::new();
    if let Ok(entries) = fs::read_dir(&mirror_skills_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && !name.starts_with('_')
                && !name.starts_with('.')
                && entry.path().is_dir()
            {
                all_skills.insert(name.to_string());
            }
        }
    }

    // Collect core skills for the exclude-core warning.
    let core_skills: HashSet<String> = collect_core_skills(&mirror_skills_dir)
        .into_iter()
        .collect();

    let mut warnings = Vec::new();
    for name in &config.skills.include {
        if !all_skills.contains(name) {
            warnings.push(format!(
                "[skills].include = \"{}\" — not a known processkit skill",
                name
            ));
        }
    }
    for name in &config.skills.exclude {
        if !all_skills.contains(name) && !effective.contains(name) {
            warnings.push(format!(
                "[skills].exclude = \"{}\" — not a known processkit skill",
                name
            ));
        } else if core_skills.contains(name) {
            warnings.push(format!(
                "[skills].exclude = \"{}\" — this is a core skill (metadata.processkit.core: true) \
                 and will be installed regardless of [skills].exclude",
                name
            ));
        }
    }
    Ok(warnings)
}

/// Result of a content-source install run, for reporting.
#[derive(Debug, Default, Clone)]
pub struct InstallReport {
    pub files_installed: usize,
    pub files_skipped: usize,
    pub groups_touched: usize,
    pub fetched_from: String,
    pub fetched_version: String,
    pub skipped_due_to_unset: bool,
}

/// Install content-source payload into the given project root, based
/// on the given config. Today this reads `config.processkit` (the only
/// configured content source); when multi-source support lands the
/// signature will accept a content-source descriptor instead.
///
/// See module docs for error policy and idempotency notes.
pub fn install_content_source(project_root: &Path, config: &AiboxConfig) -> Result<InstallReport> {
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
    let fetched = content_source::fetch(
        &pk.source,
        &pk.version,
        pk.branch.as_deref(),
        &pk.src_path,
        pk.release_asset_url_template.as_deref(),
    )
    .with_context(|| format!("failed to fetch processkit {}@{}", pk.source, pk.version))?;

    // 3. Walk cache and install live files. Templated files are
    //    rendered through the Class A substitution vocabulary at copy
    //    time — see DEC-032. Skill files are filtered through the
    //    effective skill set computed from [context].packages plus
    //    [skills].include/exclude — see DEC-035 / BACK-118.
    let template_vars = context::build_substitution_map(config);
    // The effective skill set requires the templates mirror, which
    // doesn't exist on the FIRST install (it's about to be created
    // below). For the very first install, we install ALL skills the
    // cache ships and let the next sync's filter take effect once the
    // mirror is in place. Subsequent installs read the OLD mirror
    // (still on disk from the previous install) for filtering.
    let effective_skills = build_effective_skill_set(project_root, config)
        .ok()
        .flatten();
    let (files_installed, files_skipped, groups_touched) = install_files_from_cache_with_vars(
        &fetched.src_path,
        project_root,
        &template_vars,
        effective_skills.as_ref(),
    )?;

    // 4. Copy the full cache into context/templates/processkit/<version>/
    //    so the 3-way diff has an immutable "as-installed" reference.
    //    Templated files (e.g. scaffolding/AGENTS.md) are rendered
    //    through the SAME Class A vocabulary that the live install
    //    used, so the mirror's templated content matches the live
    //    file SHA-for-SHA. See DEC-034.
    copy_templates_from_cache_with_vars(
        &fetched.src_path,
        project_root,
        &pk.version,
        &template_vars,
    )
    .context("failed to copy cache to templates dir")?;

    // 5. Write the top-level aibox.lock (always — fresh timestamps every run).
    //    The [aibox] section captures the CLI version that performed this
    //    install (DEC-037 — absorbs the legacy .aibox-version). The
    //    [processkit] section captures the install state of processkit.
    let now = Utc::now().to_rfc3339();
    let aibox_lock = AiboxLock {
        aibox: lock::AiboxLockSection {
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            synced_at: now.clone(),
        },
        processkit: Some(lock::ProcessKitLockSection {
            source: pk.source.clone(),
            version: pk.version.clone(),
            src_path: pk.src_path.clone(),
            branch: pk.branch.clone(),
            resolved_commit: fetched.resolved_commit.clone(),
            release_asset_sha256: fetched.release_asset_sha256.clone(),
            installed_at: now,
        }),
    };
    lock::write_lock(project_root, &aibox_lock).context("failed to write aibox.lock")?;

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
/// This is the **back-compat shim** for tests and any caller that
/// doesn't need template substitution. It calls
/// [`install_files_from_cache_with_vars`] with an empty vocabulary,
/// which means any [`InstallAction::InstallTemplated`] file is
/// installed verbatim (placeholders pass through). Real callers (i.e.
/// [`install_content_source`]) should use the `_with_vars` form so
/// templated files render correctly.
#[allow(dead_code)] // used by tests and by content_diff::tests
pub fn install_files_from_cache(
    cache_src_path: &Path,
    project_root: &Path,
) -> Result<(usize, usize, usize)> {
    let empty: HashMap<&'static str, String> = HashMap::new();
    install_files_from_cache_with_vars(cache_src_path, project_root, &empty, None)
}

/// Walk `cache_src_path` recursively, consult the install mapping for
/// each file, copy `Install` files verbatim, and render `InstallTemplated`
/// files through `template_vars` before writing. Returns
/// `(files_installed, files_skipped, groups_touched)`.
///
/// When `effective_skills` is `Some(set)`, skill files (anything
/// under `skills/<name>/`) are filtered by the set: skills whose
/// directory name is not in the set are skipped without writing
/// (counted as `files_skipped`). When `effective_skills` is `None`,
/// every skill the cache ships is installed (the v0.16.4 behavior).
pub fn install_files_from_cache_with_vars(
    cache_src_path: &Path,
    project_root: &Path,
    template_vars: &HashMap<&'static str, String>,
    effective_skills: Option<&HashSet<String>>,
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
        template_vars,
        effective_skills,
        &mut files_installed,
        &mut files_skipped,
        &mut groups,
    )?;

    Ok((files_installed, files_skipped, groups.len()))
}

/// Compute the templates dir for a given processkit version.
pub fn templates_dir_for_version(project_root: &Path, version: &str) -> PathBuf {
    project_root.join(TEMPLATES_PROCESSKIT_DIR).join(version)
}

/// Copy the entire cache `src_path/` (modulo `.git`, `__pycache__`,
/// dotfiles, and `*.pyc`) into `<project_root>/context/templates/processkit/<version>/`.
///
/// If a templates dir already exists for this version it is removed first
/// so the result is always a clean mirror of the cache. The full upstream
/// tree is preserved (including `INDEX.md`, `FORMAT.md`, `packages/...`,
/// and `PROVENANCE.toml`) so users can browse the reference directly with
/// any file viewer — no tooling required to see "what shipped".
///
/// **Templated files (DEC-034):** files that `install_action_for`
/// classifies as `InstallTemplated` (today: only `scaffolding/AGENTS.md`)
/// are **rendered through the Class A vocabulary** before being written
/// into the mirror, using the same substitution map as the live install.
/// This ensures the mirror's templated files contain the same rendered
/// content as their live counterparts, so the SHA-based three-way diff
/// in `content_diff::run_content_sync` works correctly for them. Pre-v0.16.5
/// the mirror held unrendered cache content, which caused all templated
/// files to false-positive as "ChangedLocally" forever — see DEC-032 for
/// the v0.16.4 limitation that this fix closes.
#[allow(dead_code)] // back-compat shim used by tests
pub fn copy_templates_from_cache(
    cache_src_path: &Path,
    project_root: &Path,
    version: &str,
) -> Result<()> {
    let empty: HashMap<&'static str, String> = HashMap::new();
    copy_templates_from_cache_with_vars(cache_src_path, project_root, version, &empty)
}

/// Same as [`copy_templates_from_cache`] but with an explicit Class A
/// substitution vocabulary used to render templated files into the
/// mirror. Real callers (i.e. `install_content_source`) use this form;
/// tests can pass an empty map to keep templated files unrendered
/// (matching the v0.16.4 behaviour for back-compat with existing tests).
pub fn copy_templates_from_cache_with_vars(
    cache_src_path: &Path,
    project_root: &Path,
    version: &str,
    template_vars: &HashMap<&'static str, String>,
) -> Result<()> {
    if !cache_src_path.is_dir() {
        anyhow::bail!(
            "copy_templates_from_cache: {} is not a directory",
            cache_src_path.display()
        );
    }
    let dest = templates_dir_for_version(project_root, version);
    if dest.exists() {
        fs::remove_dir_all(&dest)
            .with_context(|| format!("failed to clear stale templates dir {}", dest.display()))?;
    }
    fs::create_dir_all(&dest)
        .with_context(|| format!("failed to create templates dir {}", dest.display()))?;

    copy_dir_recursive_with_render(cache_src_path, cache_src_path, &dest, template_vars)?;
    Ok(())
}

/// Recursive directory copy that honours [`should_skip_entry`] AND
/// dispatches on [`install_action_for`] for templated files.
///
/// - Files matching `InstallTemplated` are read, rendered through
///   `template_vars`, and written to the mirror destination.
/// - Files matching `Install` or `Skip` are copied verbatim (the
///   mirror is the FULL upstream reference; nothing is filtered out
///   here).
fn copy_dir_recursive_with_render(
    root: &Path,
    src: &Path,
    dst: &Path,
    template_vars: &HashMap<&'static str, String>,
) -> Result<()> {
    for entry in
        fs::read_dir(src).with_context(|| format!("failed to read directory {}", src.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        if should_skip_entry(&name_str) {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        let ft = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", from.display()))?;
        if ft.is_dir() {
            fs::create_dir_all(&to)
                .with_context(|| format!("failed to create {}", to.display()))?;
            copy_dir_recursive_with_render(root, &from, &to, template_vars)?;
        } else if ft.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            // Compute the relative-to-cache-root path so we can
            // consult install_action_for for the templating decision.
            let rel = from
                .strip_prefix(root)
                .with_context(|| {
                    format!(
                        "failed to relativize {} against {}",
                        from.display(),
                        root.display()
                    )
                })?
                .to_path_buf();
            match install_action_for(&rel) {
                InstallAction::InstallTemplated(_) => {
                    let source = fs::read_to_string(&from).with_context(|| {
                        format!("failed to read templated source {}", from.display())
                    })?;
                    let rendered = context::render(&source, template_vars);
                    fs::write(&to, rendered).with_context(|| {
                        format!(
                            "failed to write rendered template into mirror {} -> {}",
                            from.display(),
                            to.display()
                        )
                    })?;
                }
                InstallAction::Install(_) | InstallAction::Skip => {
                    fs::copy(&from, &to).with_context(|| {
                        format!("failed to copy {} -> {}", from.display(), to.display())
                    })?;
                }
            }
        }
        // symlinks/fifos: ignore
    }
    Ok(())
}

/// Recursive walker mirroring the diff walker's skip rules.
#[allow(clippy::too_many_arguments)]
fn walk_and_install(
    root: &Path,
    dir: &Path,
    project_root: &Path,
    template_vars: &HashMap<&'static str, String>,
    effective_skills: Option<&HashSet<String>>,
    files_installed: &mut usize,
    files_skipped: &mut usize,
    groups: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?
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
                template_vars,
                effective_skills,
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

        // [skills] filter (DEC-035): if the rel path is under
        // skills/<name>/ AND we have an effective set AND <name>
        // isn't in it, skip the file. _lib/ and the top-level
        // FORMAT.md / INDEX.md under skills/ are NOT filtered (they
        // belong to "skills as a system", not to any specific skill).
        if let Some(set) = effective_skills {
            let parts: Vec<&str> = rel
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => s.to_str(),
                    _ => None,
                })
                .collect();
            if parts.len() >= 2 && parts[0] == "skills" {
                let skill_name = parts[1];
                // _lib is the shared lib, never filtered.
                if !skill_name.starts_with('_') && !set.contains(skill_name) {
                    *files_skipped += 1;
                    continue;
                }
            }
        }

        match install_action_for(&rel) {
            InstallAction::Install(target_rel) => {
                let dest = project_root.join(&target_rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create parent directory {}", parent.display())
                    })?;
                }
                fs::copy(&path, &dest).with_context(|| {
                    format!("failed to copy {} -> {}", path.display(), dest.display())
                })?;
                *files_installed += 1;
                if let Some(g) = group_for_path(&rel) {
                    groups.insert(g);
                }
            }
            InstallAction::InstallTemplated(target_rel) => {
                // Read source, render through the Class A vocabulary,
                // write to destination — but only if the destination
                // does not already exist. This is `write_if_missing`
                // semantics, NOT `fs::copy` semantics: a user who
                // edits AGENTS.md after init must not have their
                // edits clobbered by the next sync.
                //
                // Limitation (v0.16.4 / DEC-032): upstream improvements
                // to a templated file (e.g. processkit ships a better
                // AGENTS.md template in a new release) do NOT
                // auto-propagate. The three-way diff machinery in
                // content_diff currently treats templated files as
                // skipped because the templates mirror holds the
                // unrendered cache content while live holds the
                // rendered output, so SHA comparison would always
                // false-positive. v0.16.5+ will fix this by also
                // rendering templated files into the templates mirror
                // and comparing on the rendered side.
                let dest = project_root.join(&target_rel);
                if dest.exists() {
                    // Don't clobber existing user-edited (or
                    // first-install) content. Skip without counting
                    // it as a fresh install.
                    *files_skipped += 1;
                    continue;
                }
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create parent directory {}", parent.display())
                    })?;
                }
                let source = fs::read_to_string(&path).with_context(|| {
                    format!("failed to read templated source {}", path.display())
                })?;
                let rendered = context::render(&source, template_vars);
                fs::write(&dest, rendered).with_context(|| {
                    format!(
                        "failed to write rendered template {} -> {}",
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── build_effective_skill_set / [skills] activation (BACK-118) ─────

    fn write_synth_packages_dir(
        project_root: &Path,
        version: &str,
        packages: &[(&str, &[&str], &[&str])],
    ) {
        // (package_name, extends, skills) — v0.8.0 layout: .processkit/packages/
        let dir = project_root
            .join(TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(processkit_vocab::src::DOTPROCESSKIT)
            .join(processkit_vocab::src::PACKAGES);
        fs::create_dir_all(&dir).unwrap();
        for (name, extends, skills) in packages {
            let extends_yaml = if extends.is_empty() {
                String::new()
            } else {
                format!("  extends: [{}]\n", extends.join(", "))
            };
            let skills_yaml = if skills.is_empty() {
                "    skills: []\n".to_string()
            } else {
                let lines: Vec<String> = skills.iter().map(|s| format!("      - {}", s)).collect();
                format!("    skills:\n{}\n", lines.join("\n"))
            };
            let body = format!(
                "apiVersion: processkit.projectious.work/v1\n\
                 kind: Package\n\
                 metadata:\n  name: {}\n\
                 spec:\n  description: test\n{}  includes:\n{}",
                name, extends_yaml, skills_yaml
            );
            fs::write(dir.join(format!("{}.yaml", name)), body).unwrap();
        }
    }

    fn config_with_packages_and_skills(
        version: &str,
        packages: &[&str],
        include: &[&str],
        exclude: &[&str],
    ) -> AiboxConfig {
        let mut config = crate::config::test_config();
        config.processkit.version = version.to_string();
        config.context.packages = packages.iter().map(|s| s.to_string()).collect();
        config.skills.include = include.iter().map(|s| s.to_string()).collect();
        config.skills.exclude = exclude.iter().map(|s| s.to_string()).collect();
        config
    }

    #[test]
    fn effective_skill_set_returns_none_when_version_unset() {
        let tmp = TempDir::new().unwrap();
        let config = config_with_packages_and_skills(
            crate::config::PROCESSKIT_VERSION_UNSET,
            &["managed"],
            &[],
            &[],
        );
        let result = build_effective_skill_set(tmp.path(), &config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn effective_skill_set_returns_none_when_mirror_missing() {
        let tmp = TempDir::new().unwrap();
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["managed"],
            &[],
            &[],
        );
        let result = build_effective_skill_set(tmp.path(), &config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn effective_skill_set_collects_single_package() {
        let tmp = TempDir::new().unwrap();
        write_synth_packages_dir(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &[("minimal", &[], &["workitem-management", "decision-record"])],
        );
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["minimal"],
            &[],
            &[],
        );
        let set = build_effective_skill_set(tmp.path(), &config)
            .unwrap()
            .unwrap();
        assert_eq!(set.len(), 2);
        assert!(set.contains("workitem-management"));
        assert!(set.contains("decision-record"));
    }

    #[test]
    fn effective_skill_set_expands_extends_chain() {
        let tmp = TempDir::new().unwrap();
        write_synth_packages_dir(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &[
                ("minimal", &[], &["workitem-management"]),
                (
                    "managed",
                    &["minimal"],
                    &["decision-record", "scope-management"],
                ),
                ("software", &["managed"], &["code-review"]),
            ],
        );
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["software"],
            &[],
            &[],
        );
        let set = build_effective_skill_set(tmp.path(), &config)
            .unwrap()
            .unwrap();
        assert_eq!(set.len(), 4);
        assert!(set.contains("workitem-management")); // from minimal
        assert!(set.contains("decision-record")); // from managed
        assert!(set.contains("scope-management")); // from managed
        assert!(set.contains("code-review")); // from software
    }

    #[test]
    fn effective_skill_set_user_include_adds() {
        let tmp = TempDir::new().unwrap();
        write_synth_packages_dir(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &[("minimal", &[], &["workitem-management"])],
        );
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["minimal"],
            &["latex-authoring"],
            &[],
        );
        let set = build_effective_skill_set(tmp.path(), &config)
            .unwrap()
            .unwrap();
        assert_eq!(set.len(), 2);
        assert!(set.contains("workitem-management"));
        assert!(set.contains("latex-authoring"));
    }

    #[test]
    fn effective_skill_set_user_exclude_removes() {
        let tmp = TempDir::new().unwrap();
        write_synth_packages_dir(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &[("minimal", &[], &["workitem-management", "decision-record"])],
        );
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["minimal"],
            &[],
            &["decision-record"],
        );
        let set = build_effective_skill_set(tmp.path(), &config)
            .unwrap()
            .unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.contains("workitem-management"));
        assert!(!set.contains("decision-record"));
    }

    #[test]
    fn effective_skill_set_handles_diamond_extends() {
        // package D extends both B and C; B and C both extend A.
        // A's skills should appear once, not twice (no double-include).
        let tmp = TempDir::new().unwrap();
        write_synth_packages_dir(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &[
                ("a", &[], &["from-a"]),
                ("b", &["a"], &["from-b"]),
                ("c", &["a"], &["from-c"]),
                ("d", &["b", "c"], &["from-d"]),
            ],
        );
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["d"],
            &[],
            &[],
        );
        let set = build_effective_skill_set(tmp.path(), &config)
            .unwrap()
            .unwrap();
        assert_eq!(set.len(), 4);
        for skill in &["from-a", "from-b", "from-c", "from-d"] {
            assert!(set.contains(*skill), "missing: {}", skill);
        }
    }

    #[test]
    fn effective_skill_set_errors_on_unknown_package() {
        let tmp = TempDir::new().unwrap();
        write_synth_packages_dir(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &[("minimal", &[], &["x"])],
        );
        let config = config_with_packages_and_skills(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            &["nonexistent"],
            &[],
            &[],
        );
        let err = build_effective_skill_set(tmp.path(), &config).unwrap_err();
        assert!(format!("{}", err).contains("nonexistent"));
    }

    #[test]
    fn core_skill_survives_user_exclude() {
        // A skill with `metadata.processkit.core: true` must land even when the
        // user puts it in [skills].exclude. This is the enforcement point for
        // the core: true convention (processkit/aibox#36).
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        write_synth_packages_dir(
            tmp.path(),
            version,
            &[("minimal", &[], &["workitem-management", "skill-finder"])],
        );
        // Write a minimal SKILL.md with core: true for skill-finder.
        let skills_dir = tmp
            .path()
            .join(TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join("skills/skill-finder");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(
            skills_dir.join("SKILL.md"),
            "---\nname: skill-finder\nmetadata:\n  processkit:\n    core: true\n---\n",
        )
        .unwrap();

        let config = config_with_packages_and_skills(version, &["minimal"], &[], &["skill-finder"]);
        let set = build_effective_skill_set(tmp.path(), &config)
            .unwrap()
            .unwrap();
        // skill-finder is excluded by the user but must be re-added because core: true.
        assert!(
            set.contains("skill-finder"),
            "core skill must survive exclude"
        );
        assert!(set.contains("workitem-management"));
    }

    #[test]
    fn validate_skill_overrides_warns_on_excluded_core_skill() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        write_synth_packages_dir(
            tmp.path(),
            version,
            &[("minimal", &[], &["workitem-management", "skill-finder"])],
        );
        // Materialise the templates mirror skills dir so validate can read it
        // (v0.8.0 layout: context/skills/).
        let mirror_skills = tmp
            .path()
            .join(TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(processkit_vocab::src::CONTEXT_DIR)
            .join(processkit_vocab::src::SKILLS);
        let sf_dir = mirror_skills.join("skill-finder");
        let wm_dir = mirror_skills.join("workitem-management");
        fs::create_dir_all(&sf_dir).unwrap();
        fs::create_dir_all(&wm_dir).unwrap();
        fs::write(
            sf_dir.join("SKILL.md"),
            "---\nname: skill-finder\nmetadata:\n  processkit:\n    core: true\n---\n",
        )
        .unwrap();
        fs::write(
            wm_dir.join("SKILL.md"),
            "---\nname: workitem-management\n---\n",
        )
        .unwrap();

        let config = config_with_packages_and_skills(version, &["minimal"], &[], &["skill-finder"]);
        let warnings = validate_skill_overrides(tmp.path(), &config).unwrap();
        assert!(
            warnings.iter().any(|w| w.contains("core skill")),
            "expected a core-skill warning, got: {:?}",
            warnings
        );
    }

    /// Build a synthetic processkit-shaped src tree under `root`. Includes
    /// extras to exercise skip rules (INDEX.md, skills/FORMAT.md, packages/).
    fn synth_cache(root: &Path) {
        fs::create_dir_all(root.join("skills/event-log/templates")).unwrap();
        fs::create_dir_all(root.join("skills/workitem-management")).unwrap();
        fs::create_dir_all(root.join("primitives/schemas")).unwrap();
        fs::create_dir_all(root.join("primitives/state-machines")).unwrap();
        fs::create_dir_all(root.join("lib/processkit")).unwrap();
        fs::create_dir_all(root.join("processes")).unwrap();
        fs::create_dir_all(root.join("packages")).unwrap();

        // Files that map to Install.
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
        fs::write(root.join("lib/processkit/entity.py"), "print('x')\n").unwrap();
        fs::write(root.join("processes/release.md"), "# release\n").unwrap();

        // Files that map to Skip.
        fs::write(root.join("PROVENANCE.toml"), "version = \"v0.4.0\"\n").unwrap();
        fs::write(root.join("primitives/FORMAT.md"), "# primitives format\n").unwrap();
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
            AddonsSection, AiSection, AiboxConfig, AiboxSection, AudioSection, ContainerSection,
            ContextSection, CustomizationSection, ProcessKitSection, SkillsSection,
        };
        AiboxConfig {
            aibox: AiboxSection {
                version: "0.15.0".to_string(),
                base: crate::config::BaseImage::Debian,
            },
            container: ContainerSection {
                name: "t".to_string(),
                hostname: "t".to_string(),
                user: "aibox".to_string(),
                post_create_command: None,
                keepalive: false,
                environment: std::collections::HashMap::new(),
                extra_volumes: vec![],
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
            agents: crate::config::AgentsSection::default(),
            customization: CustomizationSection::default(),
            audio: AudioSection::default(),
            mcp: crate::config::McpSection::default(),
            local_env: std::collections::HashMap::new(),
            local_mcp_servers: vec![],
        }
    }

    // -- sentinel skip ------------------------------------------------------

    #[test]
    fn install_content_source_skips_on_unset_sentinel() {
        let tmp = TempDir::new().unwrap();
        let cfg = config_with_version(PROCESSKIT_VERSION_UNSET);
        let report = install_content_source(tmp.path(), &cfg).unwrap();
        assert!(report.skipped_due_to_unset);
        assert_eq!(report.files_installed, 0);
        assert_eq!(report.files_skipped, 0);
        assert_eq!(report.fetched_version, PROCESSKIT_VERSION_UNSET);
        // No lock, no templates dir — we did no I/O.
        assert!(!lock::lock_path(tmp.path()).exists());
        assert!(!tmp.path().join(TEMPLATES_PROCESSKIT_DIR).exists());
    }

    // -- install_files_from_cache -------------------------------------------

    #[test]
    fn install_copies_skill_files_under_context_skills() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(proj.join("context/skills/event-log/SKILL.md").exists());
        assert!(
            proj.join("context/skills/event-log/templates/entry.yaml")
                .exists()
        );
        assert!(
            proj.join("context/skills/workitem-management/SKILL.md")
                .exists()
        );
    }

    #[test]
    fn install_copies_lib_files_under_context_skills_lib() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(
            proj.join("context/skills/_lib/processkit/entity.py")
                .exists(),
            "lib file should land under context/skills/_lib/processkit/"
        );
    }

    #[test]
    fn install_copies_primitive_schemas_under_context_schemas() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(proj.join("context/schemas/workitem.yaml").exists());
        assert!(proj.join("context/state-machines/workflow.yaml").exists());
    }

    #[test]
    fn install_copies_processes_under_context_processes() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        assert!(proj.join("context/processes/release.md").exists());
    }

    #[test]
    fn install_does_not_copy_provenance_or_format_to_live_tree() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        // PROVENANCE.toml and primitives/FORMAT.md are processkit-internal —
        // they live in the templates dir but never in the live tree.
        assert!(!proj.join("PROVENANCE.toml").exists());
        assert!(!proj.join("context/PROVENANCE.toml").exists());
        assert!(!proj.join("context/primitives-FORMAT.md").exists());
        assert!(!proj.join("context/schemas/FORMAT.md").exists());
    }

    #[test]
    fn install_routes_index_md_to_per_directory_destinations() {
        // Since v0.16.4 (BACK-116), INDEX.md files install where they
        // belong: top-level INDEX.md → context/INDEX.md;
        // skills/INDEX.md → context/skills/INDEX.md;
        // processes/INDEX.md → context/processes/INDEX.md; the schemas
        // and state-machines INDEX files likewise. The three INDEX.md
        // files without a sensible destination (primitives/INDEX.md,
        // scaffolding/INDEX.md, packages/INDEX.md) remain skipped.
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        // Installed:
        assert!(
            proj.join("context/INDEX.md").exists(),
            "top-level INDEX.md should land at context/INDEX.md"
        );
        assert!(
            proj.join("context/skills/INDEX.md").exists(),
            "skills/INDEX.md should land under context/skills/"
        );

        // Still skipped (no destination):
        assert!(
            !proj.join("context/primitives/INDEX.md").exists(),
            "primitives/INDEX.md has no destination — aibox splits primitives \
             into flat schemas/ and state-machines/"
        );
        assert!(
            !proj.join("context/INDEX.md").is_dir(),
            "context/INDEX.md must be a file, not a directory"
        );
    }

    #[test]
    fn install_skips_packages() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        // packages/* files should not exist anywhere under the live tree.
        assert!(!proj.join("packages").exists());
        assert!(!proj.join("context/skills/packages").exists());
        assert!(!proj.join("context/packages").exists());
    }

    #[test]
    fn install_counts_groups() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        let (installed, skipped, groups) = install_files_from_cache(&cache, &proj).unwrap();

        // 9 installed: 2 event-log files, 1 workitem-mgmt SKILL, 1 schema,
        // 1 state-machine, 1 lib/entity.py, 1 release.md, plus
        // 2 INDEX.md files (top-level + skills/) since v0.16.4 (BACK-116).
        assert_eq!(installed, 9, "unexpected installed count");
        // 6 skipped: PROVENANCE.toml, primitives/FORMAT.md, skills/FORMAT.md,
        // primitives/INDEX.md (no destination), and 2 packages.
        // NOT skipped any more: top-level INDEX.md and skills/INDEX.md
        // (BACK-116 routed them to per-directory destinations).
        assert_eq!(skipped, 6, "unexpected skipped count");
        // 7 groups: skills/event-log, skills/workitem-management,
        // primitives/schemas/workitem, primitives/state-machines/workflow,
        // lib, processes/release, plus skills/INDEX.md (the new
        // skills-level INDEX install added by BACK-116). Top-level
        // INDEX.md installs but contributes no group (top-level loose
        // file → group_for_path returns None).
        assert_eq!(groups, 7, "unexpected group count");
    }

    #[test]
    fn install_creates_parent_directories() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        // Build a minimal cache with one deeply-nested file.
        fs::create_dir_all(cache.join("skills/deep/nested/path")).unwrap();
        fs::write(cache.join("skills/deep/nested/path/SKILL.md"), "# deep\n").unwrap();
        fs::create_dir_all(&proj).unwrap();

        install_files_from_cache(&cache, &proj).unwrap();

        let dest = proj.join("context/skills/deep/nested/path/SKILL.md");
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
        let dest = proj.join("context/skills/event-log/SKILL.md");
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
        assert!(!proj.join("context/skills/event-log/__pycache__").exists());
    }

    // -- copy_templates_from_cache ------------------------------------------

    #[test]
    fn templates_dir_path_is_under_context_templates_processkit_version() {
        let tmp = TempDir::new().unwrap();
        let dir = templates_dir_for_version(tmp.path(), "v0.4.0");
        assert_eq!(dir, tmp.path().join("context/templates/processkit/v0.4.0"));
    }

    #[test]
    fn copy_templates_mirrors_full_cache_minus_skip_rules() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        // Add some noise the copy must skip.
        fs::create_dir_all(cache.join(".git/objects")).unwrap();
        fs::write(cache.join(".git/objects/foo"), b"git internals").unwrap();
        fs::create_dir_all(cache.join("skills/event-log/__pycache__")).unwrap();
        fs::write(cache.join("skills/event-log/__pycache__/x.pyc"), b"junk").unwrap();
        fs::create_dir_all(&proj).unwrap();

        copy_templates_from_cache(&cache, &proj, "v0.4.0").unwrap();

        let base = proj.join("context/templates/processkit/v0.4.0");
        // Files that the live install would skip are still in templates —
        // the templates dir is the *full* upstream reference.
        assert!(base.join("PROVENANCE.toml").exists());
        assert!(base.join("primitives/FORMAT.md").exists());
        assert!(base.join("skills/FORMAT.md").exists());
        assert!(base.join("INDEX.md").exists());
        assert!(base.join("skills/INDEX.md").exists());
        assert!(base.join("packages/minimal.yaml").exists());
        // Files that the live install would copy are also in templates.
        assert!(base.join("skills/event-log/SKILL.md").exists());
        assert!(base.join("primitives/schemas/workitem.yaml").exists());
        assert!(base.join("lib/processkit/entity.py").exists());
        assert!(base.join("processes/release.md").exists());
        // .git, __pycache__, .pyc are skipped by should_skip_entry.
        assert!(!base.join(".git").exists());
        assert!(!base.join("skills/event-log/__pycache__").exists());
    }

    #[test]
    fn copy_templates_clears_stale_dir_on_reinstall() {
        let tmp = TempDir::new().unwrap();
        let cache = tmp.path().join("cache");
        let proj = tmp.path().join("proj");
        synth_cache(&cache);
        fs::create_dir_all(&proj).unwrap();

        copy_templates_from_cache(&cache, &proj, "v0.4.0").unwrap();
        // Pollute the templates dir with a stale file.
        let stale = proj.join("context/templates/processkit/v0.4.0/stale.md");
        fs::write(&stale, b"x").unwrap();
        assert!(stale.exists());

        // Re-running should wipe the stale file.
        copy_templates_from_cache(&cache, &proj, "v0.4.0").unwrap();
        assert!(!stale.exists(), "re-copy should clear stale templates dir");
    }
}
