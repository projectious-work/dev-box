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

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::config::{AiboxConfig, PROCESSKIT_VERSION_UNSET};
use crate::output;
use crate::processkit_vocab::{mirror_skills_dir, parse_skill_frontmatter};

/// Generate missing command adapter files from SKILL.md declarations.
///
/// When a skill declares commands in its SKILL.md `metadata.processkit.commands`
/// but the corresponding `commands/<name>.md` file is absent, this function
/// generates the file with the standard adapter template. Used to handle
/// incomplete skill distributions (e.g., processkit v0.19.1 pk-doctor).
///
/// Best-effort: logs warnings for individual failures but does not abort.
fn generate_missing_command_files(live_skills_dir: &Path) {
    if !live_skills_dir.is_dir() {
        return;
    }

    // Walk the two-level <category>/<skill>/ layout.
    let Ok(categories) = fs::read_dir(live_skills_dir) else {
        return;
    };

    for category in categories.flatten() {
        if !category.path().is_dir() {
            continue;
        }
        let Ok(skills) = fs::read_dir(category.path()) else {
            continue;
        };

        for skill in skills.flatten() {
            let skill_path = skill.path();
            if !skill_path.is_dir() {
                continue;
            }

            let skill_md = skill_path.join("SKILL.md");
            let Ok(fm) = parse_skill_frontmatter(&skill_md) else {
                continue;
            };

            let Some(meta) = fm.processkit_meta() else {
                continue;
            };

            for cmd in &meta.commands {
                let cmd_filename = format!("{}.md", cmd.name);
                let cmd_file = skill_path.join("commands").join(&cmd_filename);

                if cmd_file.exists() {
                    continue;
                }

                // Generate adapter file with standard template
                let content = format!(
                    "---\nargument-hint: \"{}\"\nallowed-tools: []\n---\n\n{}\n",
                    cmd.args, cmd.description
                );

                if let Some(parent) = cmd_file.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                match fs::write(&cmd_file, content) {
                    Ok(_) => {
                        output::warn(&format!(
                            "Generated missing command file {} from SKILL.md declaration",
                            cmd_file.display()
                        ));
                    }
                    Err(e) => {
                        output::warn(&format!(
                            "Failed to generate command file {}: {}",
                            cmd_file.display(),
                            e
                        ));
                    }
                }
            }
        }
    }
}

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

    // Generate any missing command files from SKILL.md declarations before collecting.
    generate_missing_command_files(&live_skills_dir);

    // Step 1: build the universe of all known processkit command filenames by
    // scanning the templates mirror. Anything in this set that appears in
    // .claude/commands/ is considered aibox-managed.
    let empty_dir = std::path::PathBuf::new();
    let mirror_dir_ref = mirror_skills_dir.as_deref().unwrap_or(&empty_dir);
    let universe = collect_command_filenames(mirror_dir_ref);

    // Step 2: build the wanted set from the live installed skills. Source
    // path is stored so we can copy the content verbatim. Returns Err on
    // slash-command name collision between two skills.
    let wanted = collect_live_commands(&live_skills_dir)?;

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

/// Walk `skills_dir/<category>/<skill>/commands/*.md` and return a set of all
/// command filenames (basenames only). Used to build the universe from the
/// templates mirror.
///
/// The layout is two levels deep: `skills_dir/<category>/<skill>/commands/`.
/// Top-level non-directory entries (e.g. `INDEX.md`) are skipped gracefully.
///
/// Emits a warning (last-wins) when the same command filename appears in two
/// different skill directories across categories.
fn collect_command_filenames(skills_dir: &Path) -> HashSet<String> {
    let mut set = HashSet::new();
    // Collision guard: filename → category/skill path of first occurrence.
    let mut seen: HashMap<String, std::path::PathBuf> = HashMap::new();
    let Ok(categories) = fs::read_dir(skills_dir) else {
        return set;
    };
    for category in categories.flatten() {
        if !category.path().is_dir() {
            continue;
        }
        let Ok(skills) = fs::read_dir(category.path()) else {
            continue;
        };
        for skill in skills.flatten() {
            let commands_dir = skill.path().join("commands");
            let Ok(cmd_entries) = fs::read_dir(&commands_dir) else {
                continue;
            };
            for cmd in cmd_entries.flatten() {
                let name = cmd.file_name();
                let Some(s) = name.to_str() else { continue };
                if s.ends_with(".md") {
                    if let Some(prev) = seen.get(s)
                        && prev != &skill.path()
                    {
                        crate::output::warn(&format!(
                            "duplicate command filename '{s}' found in \
                             '{prev}' and '{cur}' — last-wins; \
                             '{cur}' takes precedence. \
                             Disambiguate upstream to silence this warning.",
                            prev = prev.display(),
                            cur = skill.path().display(),
                        ));
                    }
                    seen.insert(s.to_string(), skill.path());
                    set.insert(s.to_string());
                }
            }
        }
    }
    set
}

/// Walk `skills_dir/<category>/<skill>/commands/*.md` and return a map of
/// filename → source path. Used to build the wanted set from the live
/// installed skills.
///
/// The layout is two levels deep: `skills_dir/<category>/<skill>/commands/`.
/// Top-level non-directory entries (e.g. `INDEX.md`) are skipped gracefully.
///
/// Returns `Err` when the same command filename would deploy from two
/// different skill directories. Iteration order is deterministic:
/// `(category, skill_name)` lexicographic, so collision error messages
/// are stable across runs.
fn collect_live_commands(skills_dir: &Path) -> Result<HashMap<String, std::path::PathBuf>> {
    let mut map: HashMap<String, std::path::PathBuf> = HashMap::new();
    // Collision guard: filename → skill path of first occurrence.
    let mut seen_skill: HashMap<String, std::path::PathBuf> = HashMap::new();

    let Ok(category_entries) = fs::read_dir(skills_dir) else {
        return Ok(map);
    };

    // Deterministic ordering: sort categories alphabetically.
    let mut categories: Vec<std::path::PathBuf> = category_entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    categories.sort();

    for category in categories {
        let Ok(skill_entries) = fs::read_dir(&category) else {
            continue;
        };

        // Deterministic ordering: sort skills alphabetically within each category.
        let mut skills: Vec<std::path::PathBuf> = skill_entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        skills.sort();

        for skill_path in skills {
            let commands_dir = skill_path.join("commands");
            let Ok(cmd_entries) = fs::read_dir(&commands_dir) else {
                continue;
            };

            // Deterministic ordering: sort command files alphabetically.
            let mut cmds: Vec<std::path::PathBuf> =
                cmd_entries.flatten().map(|e| e.path()).collect();
            cmds.sort();

            for cmd_path in cmds {
                let Some(s) = cmd_path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !s.ends_with(".md") {
                    continue;
                }

                if let Some(prev) = seen_skill.get(s)
                    && prev != &skill_path
                {
                    let prev_cmd = prev.join("commands").join(s);
                    return Err(anyhow!(
                        "Slash command name collision: {name} is shipped by both\n  \
                         - {prev}\n  - {cur}\n\
                         This blocks .claude/commands/{name} deployment.\n\
                         Resolution: file an upstream issue with the offending skill, \
                         or set [skills].exclude in aibox.toml to drop one of the \
                         conflicting skills.",
                        name = s,
                        prev = prev_cmd.display(),
                        cur = cmd_path.display(),
                    ));
                }
                seen_skill.insert(s.to_string(), skill_path.clone());
                map.insert(s.to_string(), cmd_path);
            }
        }
    }
    Ok(map)
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

    /// Create `skills_dir/<category>/<skill>/commands/<name>` entries.
    /// The two-level layout matches the real processkit skills tree.
    fn make_skill_commands(
        skills_dir: &Path,
        category: &str,
        skill: &str,
        commands: &[&str],
        content: &str,
    ) {
        let cmd_dir = skills_dir.join(category).join(skill).join("commands");
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
            "processkit",
            "session-handover",
            &["session-handover-write.md", "session-handover-read.md"],
            "body",
        );
        make_skill_commands(
            &skills,
            "processkit",
            "morning-briefing",
            &["morning-briefing-run.md"],
            "body",
        );
        // Non-.md file should be ignored
        fs::write(
            skills
                .join("processkit")
                .join("session-handover")
                .join("commands")
                .join("ignore.txt"),
            "x",
        )
        .unwrap();
        // Top-level non-directory file should be ignored gracefully
        fs::write(skills.join("INDEX.md"), "index").unwrap();

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
            "processkit",
            "note-management",
            &["note-management-capture.md"],
            "content",
        );

        let map = collect_live_commands(&skills).unwrap();
        assert!(map.contains_key("note-management-capture.md"));
        assert!(map["note-management-capture.md"].ends_with("note-management-capture.md"));
    }

    #[test]
    fn sync_copies_wanted_and_removes_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        // Mirror: knows about two commands from two skills (two-level layout)
        let mirror = project.join("context/templates/processkit/v0.8.0/context/skills");
        make_skill_commands(&mirror, "cat-a", "skill-a", &["skill-a-run.md"], "body");
        make_skill_commands(&mirror, "cat-b", "skill-b", &["skill-b-run.md"], "body");

        // Live: only skill-a is installed (skill-b was removed from effective set)
        let live = project.join("context/skills");
        make_skill_commands(&live, "cat-a", "skill-a", &["skill-a-run.md"], "# command");

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

        let mirror = project.join("context/templates/processkit/v0.8.0/context/skills");
        make_skill_commands(&mirror, "cat-a", "skill-a", &["skill-a-run.md"], "body");

        let live = project.join("context/skills");
        make_skill_commands(&live, "cat-a", "skill-a", &["skill-a-run.md"], "body");

        let claude_cmds = project.join(".claude/commands");
        fs::create_dir_all(&claude_cmds).unwrap();
        fs::write(claude_cmds.join("skill-a-run.md"), "body").unwrap();

        let mtime_before = fs::metadata(claude_cmds.join("skill-a-run.md"))
            .unwrap()
            .modified()
            .unwrap();

        let config = config_with_pk_version("v0.8.0");

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

    /// Universe-side: duplicate command basenames across categories are
    /// tolerated by `collect_command_filenames` (the universe is a set, not
    /// a deployment plan). The hard-fail lives in `collect_live_commands`.
    #[test]
    fn collect_command_filenames_tolerates_duplicate_basename_across_categories() {
        let tmp = tempfile::tempdir().unwrap();
        let skills = tmp.path().join("skills");
        // Two skills in different categories both ship pk-foo.md.
        make_skill_commands(
            &skills,
            "engineering",
            "bar",
            &["pk-foo.md"],
            "# engineering",
        );
        make_skill_commands(&skills, "devops", "baz", &["pk-foo.md"], "# devops");

        // collect_command_filenames: filename set must contain pk-foo.md (once).
        let set = collect_command_filenames(&skills);
        assert!(
            set.contains("pk-foo.md"),
            "pk-foo.md must be in the universe set; got: {:?}",
            set
        );
        assert_eq!(set.len(), 1, "only one entry for the duplicate filename");
    }

    /// WS-4: duplicate command basename across two skills must hard-fail
    /// `collect_live_commands` (and therefore `sync_claude_commands`). The
    /// error message must include both source skill paths so the user can
    /// disambiguate.
    #[test]
    fn collect_live_commands_hard_fails_on_collision() {
        let tmp = tempfile::tempdir().unwrap();
        let skills = tmp.path().join("skills");
        make_skill_commands(
            &skills,
            "engineering",
            "bar",
            &["pk-foo.md"],
            "# engineering",
        );
        make_skill_commands(&skills, "devops", "baz", &["pk-foo.md"], "# devops");

        let result = collect_live_commands(&skills);
        assert!(result.is_err(), "expected collision to return Err");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("pk-foo.md"),
            "error must name the colliding command; got: {err}"
        );
        assert!(
            err.contains("engineering") && err.contains("bar"),
            "error must name the first colliding skill path; got: {err}"
        );
        assert!(
            err.contains("devops") && err.contains("baz"),
            "error must name the second colliding skill path; got: {err}"
        );
        assert!(
            err.contains("[skills].exclude"),
            "error should mention the [skills].exclude resolution path; got: {err}"
        );
    }

    /// WS-4: deterministic ordering — given the same on-disk layout, the
    /// collision error names the same "first" path on every run. We verify
    /// by sorting `(category, skill)` lexicographically: `devops/baz`
    /// sorts before `engineering/bar`, so `devops/baz` must be the
    /// "previous" path in the error message.
    #[test]
    fn collect_live_commands_collision_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let skills = tmp.path().join("skills");
        make_skill_commands(
            &skills,
            "engineering",
            "bar",
            &["pk-foo.md"],
            "# engineering",
        );
        make_skill_commands(&skills, "devops", "baz", &["pk-foo.md"], "# devops");

        let err = collect_live_commands(&skills).unwrap_err().to_string();
        // The error format is "shipped by both\n  - <first>\n  - <second>".
        // With (category, skill) sort, devops/baz precedes engineering/bar.
        let first_idx = err.find("devops").expect("devops must appear");
        let second_idx = err.find("engineering").expect("engineering must appear");
        assert!(
            first_idx < second_idx,
            "deterministic order: devops/baz must precede engineering/bar in error; got: {err}"
        );
    }

    /// WS-4: two skills with non-colliding command names → sync deploys
    /// both files successfully.
    #[test]
    fn sync_succeeds_when_two_skills_have_non_colliding_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        let mirror = project.join("context/templates/processkit/v0.21.0/context/skills");
        make_skill_commands(
            &mirror,
            "processkit",
            "morning-briefing",
            &["morning-briefing-generate.md"],
            "# mb",
        );
        make_skill_commands(
            &mirror,
            "processkit",
            "status-briefing",
            &["status-briefing-generate.md"],
            "# sb",
        );

        let live = project.join("context/skills");
        make_skill_commands(
            &live,
            "processkit",
            "morning-briefing",
            &["morning-briefing-generate.md"],
            "# mb body",
        );
        make_skill_commands(
            &live,
            "processkit",
            "status-briefing",
            &["status-briefing-generate.md"],
            "# sb body",
        );

        let config = config_with_pk_version("v0.21.0");
        sync_claude_commands(project, &config).expect("non-colliding sync must succeed");

        let claude_cmds = project.join(".claude/commands");
        assert!(claude_cmds.join("morning-briefing-generate.md").exists());
        assert!(claude_cmds.join("status-briefing-generate.md").exists());
        assert_eq!(
            fs::read_to_string(claude_cmds.join("morning-briefing-generate.md")).unwrap(),
            "# mb body"
        );
        assert_eq!(
            fs::read_to_string(claude_cmds.join("status-briefing-generate.md")).unwrap(),
            "# sb body"
        );
    }

    /// WS-4: end-to-end — `sync_claude_commands` returns `Err` when two
    /// skills' `commands/<same-name>.md` would both deploy.
    #[test]
    fn sync_claude_commands_hard_fails_on_collision() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        let mirror = project.join("context/templates/processkit/v0.21.0/context/skills");
        make_skill_commands(
            &mirror,
            "processkit",
            "morning-briefing",
            &["pk-resume.md"],
            "# mb",
        );
        make_skill_commands(
            &mirror,
            "processkit",
            "status-briefing",
            &["pk-resume.md"],
            "# sb",
        );

        let live = project.join("context/skills");
        make_skill_commands(
            &live,
            "processkit",
            "morning-briefing",
            &["pk-resume.md"],
            "# mb body",
        );
        make_skill_commands(
            &live,
            "processkit",
            "status-briefing",
            &["pk-resume.md"],
            "# sb body",
        );

        let config = config_with_pk_version("v0.21.0");
        let result = sync_claude_commands(project, &config);
        assert!(result.is_err(), "expected collision to abort sync");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("pk-resume.md"));
        assert!(err.contains("morning-briefing"));
        assert!(err.contains("status-briefing"));
    }

    #[test]
    fn sync_two_level_layout_copies_pk_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        // Mirror: two-level layout matching real processkit tree
        let mirror = project.join("context/templates/processkit/v0.18.1/context/skills");
        make_skill_commands(
            &mirror,
            "processkit",
            "note-management",
            &["pk-note.md"],
            "# pk-note mirror",
        );
        make_skill_commands(
            &mirror,
            "devops",
            "release-semver",
            &["pk-release.md"],
            "# pk-release mirror",
        );

        // Live: same two-level layout
        let live = project.join("context/skills");
        make_skill_commands(
            &live,
            "processkit",
            "note-management",
            &["pk-note.md"],
            "# pk-note content",
        );
        make_skill_commands(
            &live,
            "devops",
            "release-semver",
            &["pk-release.md"],
            "# pk-release content",
        );

        let config = config_with_pk_version("v0.18.1");
        sync_claude_commands(project, &config).unwrap();

        let claude_cmds = project.join(".claude/commands");
        assert!(
            claude_cmds.join("pk-note.md").exists(),
            ".claude/commands/pk-note.md should have been created"
        );
        assert!(
            claude_cmds.join("pk-release.md").exists(),
            ".claude/commands/pk-release.md should have been created"
        );
        assert_eq!(
            fs::read_to_string(claude_cmds.join("pk-note.md")).unwrap(),
            "# pk-note content"
        );
        assert_eq!(
            fs::read_to_string(claude_cmds.join("pk-release.md")).unwrap(),
            "# pk-release content"
        );
    }

    #[test]
    fn generate_missing_command_files_from_skill_md() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        // Create a skill with SKILL.md that declares commands but no commands/ dir exists yet.
        let skill_dir = project.join("context/skills/processkit/test-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        // Write a SKILL.md with commands declaration
        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            r#"---
name: test-skill
metadata:
  processkit:
    commands:
      - name: test-skill-run
        args: "[--verbose]"
        description: "Execute the test skill with optional verbosity"
---
# Test Skill
"#,
        )
        .unwrap();

        // Do NOT create commands/ dir — test that generation works
        assert!(!skill_dir.join("commands").exists());

        // Call generate_missing_command_files
        generate_missing_command_files(&project.join("context/skills"));

        // Assert the command file was created with correct format
        let cmd_file = skill_dir.join("commands/test-skill-run.md");
        assert!(cmd_file.exists(), "command file should have been generated");

        let content = fs::read_to_string(&cmd_file).unwrap();
        assert!(content.contains("argument-hint: \"[--verbose]\""));
        assert!(content.contains("allowed-tools: []"));
        assert!(content.contains("Execute the test skill with optional verbosity"));
    }

    #[test]
    fn generate_missing_command_files_skips_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        // Create a skill where commands/ dir already exists with a file
        let skill_dir = project.join("context/skills/processkit/existing-skill");
        let commands_dir = skill_dir.join("commands");
        fs::create_dir_all(&commands_dir).unwrap();

        // Write existing command file
        let existing_cmd = commands_dir.join("existing-skill-run.md");
        fs::write(&existing_cmd, "# existing content").unwrap();

        // Write SKILL.md that declares the same command
        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            r#"---
name: existing-skill
metadata:
  processkit:
    commands:
      - name: existing-skill-run
        args: ""
        description: "Run existing skill"
---
# Existing Skill
"#,
        )
        .unwrap();

        let mtime_before = fs::metadata(&existing_cmd).unwrap().modified().unwrap();

        // Call generate_missing_command_files
        generate_missing_command_files(&project.join("context/skills"));

        let mtime_after = fs::metadata(&existing_cmd).unwrap().modified().unwrap();
        // File should not have been overwritten
        assert_eq!(mtime_before, mtime_after);
        assert_eq!(
            fs::read_to_string(&existing_cmd).unwrap(),
            "# existing content"
        );
    }
}
