//! Claude Code command registration — sync `commands/` adapter files from
//! installed processkit skills to `.claude/commands/` so that Claude Code
//! can tab-complete them as slash commands.
//!
//! ## What this does
//!
//! processkit v0.7.0 introduced a `commands/` convention: skills that expose
//! user-invocable workflows ship thin adapter files at
//! `commands/<skill>-<workflow>.md` containing Claude Code-specific
//! frontmatter (`argument-hint`, `allowed-tools`) and a one-line invocation
//! body. Claude Code discovers these files from `.claude/commands/` in the
//! project root.
//!
//! This module handles the sync between the live installed skills and
//! `.claude/commands/`:
//!
//! 1. **Universe** — scan the templates mirror
//!    (`context/templates/processkit/<version>/skills/*/commands/*.md`) to
//!    discover every command filename that processkit knows about. Files in
//!    `.claude/commands/` whose names are in this universe are "aibox-managed"
//!    and may be updated or removed on subsequent syncs. Files whose names are
//!    NOT in the universe are assumed to be user-created and are never touched.
//!
//! 2. **Wanted** — walk the live installed skills
//!    (`context/skills/*/commands/*.md`). Because the live install already
//!    applies the effective-skill filter, only commands from skills in the
//!    user's effective set are present here.
//!
//! 3. **Write** — copy each wanted command to `.claude/commands/<name>`. Skips
//!    files that are already byte-identical to avoid unnecessary I/O.
//!
//! 4. **Cleanup** — remove any file from `.claude/commands/` that is in the
//!    universe (aibox-managed) but not in the wanted set (skill was removed).
//!
//! See projectious-work/aibox#37 for the full spec.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::{AiboxConfig, PROCESSKIT_VERSION_UNSET};
use crate::output;
use crate::processkit_vocab::mirror_skills_dir;

/// Sync processkit command adapter files to `.claude/commands/`.
///
/// Idempotent — re-running on a stable (version, skills) set produces
/// byte-identical output. Best-effort callers should warn-and-continue on
/// error rather than aborting the rest of sync.
pub fn sync_claude_commands(project_root: &Path, config: &AiboxConfig) -> Result<()> {
    let pk_version = &config.processkit.version;
    if pk_version == PROCESSKIT_VERSION_UNSET {
        return Ok(());
    }

    let mirror_skills_dir = mirror_skills_dir(project_root, pk_version);
    let live_skills_dir = project_root.join("context").join("skills");

    if mirror_skills_dir.is_none() && !live_skills_dir.is_dir() {
        return Ok(());
    }

    // Step 1: build the universe of all known processkit command filenames by
    // scanning the templates mirror. Anything in this set that appears in
    // .claude/commands/ is considered aibox-managed.
    let empty_dir = std::path::PathBuf::new();
    let mirror_dir_ref = mirror_skills_dir.as_deref().unwrap_or(&empty_dir);
    let universe = collect_command_filenames(mirror_dir_ref);

    // Step 2: build the wanted set from the live installed skills. Source
    // path is stored so we can copy the content verbatim.
    let wanted = collect_live_commands(&live_skills_dir);

    if universe.is_empty() && wanted.is_empty() {
        return Ok(());
    }

    // Step 3: ensure .claude/commands/ exists.
    let claude_commands_dir = project_root.join(".claude").join("commands");
    fs::create_dir_all(&claude_commands_dir)
        .with_context(|| format!("failed to create {}", claude_commands_dir.display()))?;

    let mut added = 0usize;
    let mut removed = 0usize;

    // Step 4: write wanted commands (skip if byte-identical).
    for (filename, source_path) in &wanted {
        let dest = claude_commands_dir.join(filename);
        let new_content = fs::read(source_path)
            .with_context(|| format!("failed to read {}", source_path.display()))?;
        if dest.exists() && fs::read(&dest).ok().as_deref() == Some(&new_content) {
            continue; // already up-to-date
        }
        fs::write(&dest, &new_content)
            .with_context(|| format!("failed to write {}", dest.display()))?;
        added += 1;
    }

    // Step 5: remove stale managed commands (in universe but not in wanted).
    if claude_commands_dir.is_dir() {
        for entry in fs::read_dir(&claude_commands_dir)
            .with_context(|| format!("failed to read {}", claude_commands_dir.display()))?
        {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name_str) = name.to_str() else {
                continue;
            };
            if !name_str.ends_with(".md") {
                continue;
            }
            if universe.contains(name_str) && !wanted.contains_key(name_str) {
                fs::remove_file(entry.path()).with_context(|| {
                    format!("failed to remove stale command {}", entry.path().display())
                })?;
                removed += 1;
            }
        }
    }

    if added > 0 || removed > 0 {
        output::ok(&format!(
            "Claude commands: {} added/updated, {} removed → .claude/commands/",
            added, removed
        ));
    }

    Ok(())
}

/// Walk `skills_dir/*/commands/*.md` and return a set of all command
/// filenames (basenames only). Used to build the universe from the
/// templates mirror.
fn collect_command_filenames(skills_dir: &Path) -> HashSet<String> {
    let mut set = HashSet::new();
    let Ok(entries) = fs::read_dir(skills_dir) else {
        return set;
    };
    for entry in entries.flatten() {
        let commands_dir = entry.path().join("commands");
        let Ok(cmd_entries) = fs::read_dir(&commands_dir) else {
            continue;
        };
        for cmd in cmd_entries.flatten() {
            let name = cmd.file_name();
            let Some(s) = name.to_str() else { continue };
            if s.ends_with(".md") {
                set.insert(s.to_string());
            }
        }
    }
    set
}

/// Walk `skills_dir/*/commands/*.md` and return a map of filename → source
/// path. Used to build the wanted set from the live installed skills.
fn collect_live_commands(skills_dir: &Path) -> HashMap<String, std::path::PathBuf> {
    let mut map = HashMap::new();
    let Ok(entries) = fs::read_dir(skills_dir) else {
        return map;
    };
    for entry in entries.flatten() {
        let commands_dir = entry.path().join("commands");
        let Ok(cmd_entries) = fs::read_dir(&commands_dir) else {
            continue;
        };
        for cmd in cmd_entries.flatten() {
            let name = cmd.file_name();
            let Some(s) = name.to_str() else { continue };
            if s.ends_with(".md") {
                map.insert(s.to_string(), cmd.path());
            }
        }
    }
    map
}

/// Remove only the processkit-managed command files from `.claude/commands/`,
/// then remove the directory itself if it is empty afterwards.
///
/// Called by `aibox reset` so user-authored commands in `.claude/commands/`
/// are preserved. The "managed set" is derived from the templates mirror (the
/// same source used by `sync_claude_commands`), so any file whose name appears
/// in the mirror is considered aibox-managed and is eligible for removal.
///
/// If the templates mirror is absent (e.g. processkit was never installed or
/// the context/ directory was already deleted), the function is a no-op —
/// the caller is responsible for removing the rest of context/ in that case.
pub fn remove_managed_commands(project_root: &Path, config: &AiboxConfig) -> Result<()> {
    let pk_version = &config.processkit.version;
    if pk_version == PROCESSKIT_VERSION_UNSET {
        return Ok(());
    }

    let mirror_dir = mirror_skills_dir(project_root, pk_version);
    let empty_dir = std::path::PathBuf::new();
    let mirror_dir_ref = mirror_dir.as_deref().unwrap_or(&empty_dir);
    let universe = collect_command_filenames(mirror_dir_ref);
    if universe.is_empty() {
        return Ok(());
    }

    let claude_commands_dir = project_root.join(".claude").join("commands");
    if !claude_commands_dir.is_dir() {
        return Ok(());
    }

    let mut removed = 0usize;
    for filename in &universe {
        let path = claude_commands_dir.join(filename);
        if path.is_file() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            removed += 1;
        }
    }

    // Remove the directory only if it is now empty (no user files remain).
    let is_empty = std::fs::read_dir(&claude_commands_dir)
        .map(|mut d| d.next().is_none())
        .unwrap_or(false);
    if is_empty {
        std::fs::remove_dir(&claude_commands_dir)
            .with_context(|| format!("failed to remove {}", claude_commands_dir.display()))?;
    }

    if removed > 0 {
        output::ok(&format!(
            "Removed {} managed command file(s) from .claude/commands/",
            removed
        ));
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

    fn make_skill_commands(skills_dir: &Path, skill: &str, commands: &[&str], content: &str) {
        let cmd_dir = skills_dir.join(skill).join("commands");
        fs::create_dir_all(&cmd_dir).unwrap();
        for name in commands {
            fs::write(cmd_dir.join(name), content).unwrap();
        }
    }

    fn config_with_pk_version(version: &str) -> AiboxConfig {
        use crate::config::{
            AddonsSection, AiSection, AiboxConfig, AiboxSection, AudioSection, ContainerSection,
            ContextSection, CustomizationSection, ProcessKitSection, SkillsSection,
        };
        AiboxConfig {
            aibox: AiboxSection {
                version: "0.17.3".to_string(),
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
            ai: AiSection::default(),
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

    #[test]
    fn collect_command_filenames_returns_all_md_files() {
        let tmp = tempfile::tempdir().unwrap();
        let skills = tmp.path().join("skills");
        make_skill_commands(
            &skills,
            "session-handover",
            &["session-handover-write.md", "session-handover-read.md"],
            "body",
        );
        make_skill_commands(
            &skills,
            "morning-briefing",
            &["morning-briefing-run.md"],
            "body",
        );
        // Non-.md file should be ignored
        fs::write(
            skills
                .join("session-handover")
                .join("commands")
                .join("ignore.txt"),
            "x",
        )
        .unwrap();

        let set = collect_command_filenames(&skills);
        assert_eq!(set.len(), 3);
        assert!(set.contains("session-handover-write.md"));
        assert!(set.contains("session-handover-read.md"));
        assert!(set.contains("morning-briefing-run.md"));
        assert!(!set.contains("ignore.txt"));
    }

    #[test]
    fn collect_live_commands_maps_filename_to_path() {
        let tmp = tempfile::tempdir().unwrap();
        let skills = tmp.path().join("skills");
        make_skill_commands(
            &skills,
            "note-management",
            &["note-management-capture.md"],
            "content",
        );

        let map = collect_live_commands(&skills);
        assert!(map.contains_key("note-management-capture.md"));
        assert!(map["note-management-capture.md"].ends_with("note-management-capture.md"));
    }

    #[test]
    fn sync_copies_wanted_and_removes_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        // Mirror: knows about two commands from two skills (v0.8.0 layout)
        let mirror = project.join("context/templates/processkit/v0.8.0/context/skills");
        make_skill_commands(&mirror, "skill-a", &["skill-a-run.md"], "body");
        make_skill_commands(&mirror, "skill-b", &["skill-b-run.md"], "body");

        // Live: only skill-a is installed (skill-b was removed from effective set)
        let live = project.join("context/skills");
        make_skill_commands(&live, "skill-a", &["skill-a-run.md"], "# command");

        // Pre-place a stale skill-b command that was previously installed
        let claude_cmds = project.join(".claude/commands");
        fs::create_dir_all(&claude_cmds).unwrap();
        fs::write(claude_cmds.join("skill-b-run.md"), "old").unwrap();
        // Also a user-created file that should be left alone
        fs::write(claude_cmds.join("my-custom.md"), "mine").unwrap();

        let config = config_with_pk_version("v0.8.0");

        sync_claude_commands(project, &config).unwrap();

        // skill-a-run.md was added
        assert!(claude_cmds.join("skill-a-run.md").exists());
        assert_eq!(
            fs::read_to_string(claude_cmds.join("skill-a-run.md")).unwrap(),
            "# command"
        );

        // skill-b-run.md was removed (in universe, not in wanted)
        assert!(!claude_cmds.join("skill-b-run.md").exists());

        // my-custom.md was left alone (not in universe)
        assert_eq!(
            fs::read_to_string(claude_cmds.join("my-custom.md")).unwrap(),
            "mine"
        );
    }

    #[test]
    fn sync_skips_identical_files() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        let mirror = project.join("context/templates/processkit/v0.7.0/skills");
        make_skill_commands(&mirror, "skill-a", &["skill-a-run.md"], "body");

        let live = project.join("context/skills");
        make_skill_commands(&live, "skill-a", &["skill-a-run.md"], "body");

        let claude_cmds = project.join(".claude/commands");
        fs::create_dir_all(&claude_cmds).unwrap();
        fs::write(claude_cmds.join("skill-a-run.md"), "body").unwrap();

        let mtime_before = fs::metadata(claude_cmds.join("skill-a-run.md"))
            .unwrap()
            .modified()
            .unwrap();

        let config = config_with_pk_version("v0.7.0");

        sync_claude_commands(project, &config).unwrap();

        let mtime_after = fs::metadata(claude_cmds.join("skill-a-run.md"))
            .unwrap()
            .modified()
            .unwrap();

        // File was not rewritten — mtime unchanged
        assert_eq!(mtime_before, mtime_after);
    }

    #[test]
    fn sync_no_ops_when_version_unset() {
        let tmp = tempfile::tempdir().unwrap();
        let config = config_with_pk_version(crate::config::PROCESSKIT_VERSION_UNSET);
        // Should not create .claude/commands/ or touch anything
        sync_claude_commands(tmp.path(), &config).unwrap();
        assert!(!tmp.path().join(".claude/commands").exists());
    }
}
