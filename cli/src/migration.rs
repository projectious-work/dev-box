//! Migration system for aibox version changes.
//!
//! On `aibox sync`, compares `aibox.lock [aibox].cli_version` against the
//! running CLI version. If they differ, generates a migration document at
//! `context/migrations/{from}-to-{to}.md`. Also handles the one-time hard-cut
//! migration that absorbs the legacy `.aibox-version` file into `aibox.lock`.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::output;

/// Check for version mismatch and generate migration document if needed.
/// Called during `aibox sync`. Operates in the current working directory.
pub fn check_and_generate_migration() -> Result<()> {
    // Hard-cut: absorb legacy .aibox-version into aibox.lock (one-time, idempotent).
    migrate_legacy_lock_files(Path::new("."))?;
    check_and_generate_migration_in(Path::new("."))?;
    ensure_processkit_section_in(Path::new("."))?;
    Ok(())
}

/// One-time hard-cut migration: if a legacy `.aibox-version` file still exists,
/// read `aibox.lock` (which upgrades the flat shape automatically), write back
/// the sectioned format, and delete `.aibox-version`. Idempotent once the file
/// is gone.
pub fn migrate_legacy_lock_files(root: &Path) -> Result<()> {
    let version_file = root.join(".aibox-version");
    if !version_file.exists() {
        return Ok(());
    }

    // read_lock already upgrades the legacy flat shape in memory, recovering
    // cli_version from the sibling .aibox-version.
    match crate::lock::read_lock(root)? {
        Some(lock) => {
            // Write back the sectioned shape to persist the upgrade.
            crate::lock::write_lock(root, &lock)
                .context("Failed to write upgraded aibox.lock during legacy migration")?;
        }
        None => {
            // Lock is absent — nothing to upgrade; just drop the orphan file.
        }
    }

    fs::remove_file(&version_file).context("Failed to remove legacy .aibox-version")?;
    output::ok("Migrated: .aibox-version absorbed into aibox.lock");

    Ok(())
}

/// Check for version mismatch and generate migration document if needed.
/// Operates relative to the given `root` directory.
fn check_and_generate_migration_in(root: &Path) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Read aibox.lock — if absent this is a fresh project, no migration needed.
    let lock = match crate::lock::read_lock(root)? {
        Some(l) => l,
        None => return Ok(()),
    };

    let stored_version = lock.aibox.cli_version.clone();

    // Empty means the lock was just promoted from legacy without a recoverable
    // cli_version — treat as "no known version", skip migration this cycle.
    if stored_version.is_empty() || stored_version == current_version {
        return Ok(());
    }

    output::info(&format!(
        "Version change detected: {} \u{2192} {}",
        stored_version, current_version
    ));

    // Generate migration document.
    generate_migration_doc(root, &stored_version, current_version)?;

    // Update lock with new cli_version. synced_at is left unchanged here;
    // cmd_sync updates it when it writes the full lock after install.
    let mut updated_lock = lock;
    updated_lock.aibox.cli_version = current_version.to_string();
    crate::lock::write_lock(root, &updated_lock)
        .context("Failed to update aibox.lock after migration check")?;

    Ok(())
}

/// Generate a migration document at `{root}/context/migrations/{from}-to-{to}.md`.
fn generate_migration_doc(root: &Path, from: &str, to: &str) -> Result<()> {
    let migrations_dir = root.join("context").join("migrations");
    fs::create_dir_all(&migrations_dir).context("Failed to create context/migrations/")?;

    let filename = format!("{}-to-{}.md", from, to);
    let filepath = migrations_dir.join(&filename);

    // Don't overwrite existing migration docs (user may have edited status)
    if filepath.exists() {
        output::info(&format!(
            "Migration document {} already exists, skipping",
            filename
        ));
        return Ok(());
    }

    let date = chrono_free_date();
    let content = format_migration_doc(from, to, &date);

    fs::write(&filepath, content)
        .with_context(|| format!("Failed to write migration document {}", filename))?;

    output::ok(&format!(
        "Generated migration document: context/migrations/{}",
        filename
    ));
    output::warn("Review the migration document with your AI agent before proceeding");

    Ok(())
}

/// Get the current date without requiring the chrono crate.
fn chrono_free_date() -> String {
    std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// ---------------------------------------------------------------------------
// Migration registry — version-specific migration knowledge
// ---------------------------------------------------------------------------

/// A known migration entry with specific breaking changes and action items.
struct MigrationEntry {
    from: &'static str,
    to: &'static str,
    breaking_changes: &'static [&'static str],
    action_items: &'static [&'static str],
}

/// Registry of known migrations with version-specific details.
static KNOWN_MIGRATIONS: &[MigrationEntry] = &[
    // Will be populated as we release versions
    // Example:
    // MigrationEntry {
    //     from: "0.8.0",
    //     to: "0.9.0",
    //     breaking_changes: &[
    //         "`[aibox] image` renamed to `[aibox] base` — only \"debian\" is valid",
    //         "Process packages replace monolithic process levels",
    //     ],
    //     action_items: &[
    //         "Update aibox.toml: change `image = \"python\"` to `base = \"debian\"` and add `[addons.python.tools]`",
    //     ],
    // },
];

/// Find a known migration entry for the given version pair.
fn find_known_migration(from: &str, to: &str) -> Option<&'static MigrationEntry> {
    KNOWN_MIGRATIONS
        .iter()
        .find(|m| m.from == from && m.to == to)
}

// ---------------------------------------------------------------------------
// Document formatting
// ---------------------------------------------------------------------------

/// Format the full migration document content.
fn format_migration_doc(from: &str, to: &str, date: &str) -> String {
    let known = find_known_migration(from, to);

    let breaking_changes = if let Some(entry) = known {
        entry
            .breaking_changes
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        format!(
            "- Review the [changelog](https://github.com/projectious-work/aibox/releases) \
             for breaking changes between v{} and v{}.",
            from, to
        )
    };

    let action_items = if let Some(entry) = known {
        let mut items: Vec<String> = entry
            .action_items
            .iter()
            .map(|a| format!("- [ ] {}", a))
            .collect();
        // Always include standard items
        items.push("- [ ] Review this migration document with the project owner".to_string());
        items.push("- [ ] Run `aibox sync` to regenerate container files".to_string());
        items.push("- [ ] Rebuild the container: `aibox build`".to_string());
        items.push("- [ ] Verify all context files are intact".to_string());
        items.push(
            "- [ ] Mark this migration as completed (change Status to \"completed\")".to_string(),
        );
        items.join("\n")
    } else {
        "\
- [ ] Review this migration document with the project owner
- [ ] Run `aibox sync` to regenerate container files
- [ ] Rebuild the container: `aibox build`
- [ ] Verify all context files are intact
- [ ] Mark this migration as completed (change Status to \"completed\")"
            .to_string()
    };

    format!(
        "\
# Migration: v{from} \u{2192} v{to}

> **SAFETY: Do not execute any actions in this document automatically.**
> **Discuss each item with the project owner before proceeding.**
> **Do not modify aibox.toml without explicit user confirmation.**

**Generated:** {date}
**Status:** pending
**aibox CLI version:** v{to}

## Summary

aibox has been updated from v{from} to v{to}. Review the changes below
and discuss each action item with the project owner.

## Breaking Changes

{breaking_changes}

## Action Items

{action_items}

## processkit Content Changes

Skills, processes, and primitives are installed from processkit (not bundled
in aibox). To pick up new or changed content, pin a newer processkit version
in `[processkit].version` and run `aibox sync`. The three-way diff will show
which installed files changed and let you review conflicts before overwriting.

Use `[skills].include` / `[skills].exclude` in aibox.toml to control which
skills are installed.

## Verification Checklist

- [ ] `aibox sync` completes without errors
- [ ] Container builds successfully (`aibox build`)
- [ ] Context files are intact (`AGENTS.md`, `context/skills/`, `context/processes/`)
- [ ] `AGENTS.md` is present and points to `context/` correctly

## Rollback

To revert this migration:
```
git checkout HEAD -- aibox.lock context/ .devcontainer/
aibox sync
```

## Known Issues

Check https://github.com/projectious-work/aibox/issues for known issues
with v{to}.
"
    )
}

// ---------------------------------------------------------------------------
// One-shot migration: auto-insert [processkit] section into legacy aibox.toml
// ---------------------------------------------------------------------------

/// Default `[processkit]` block written into legacy aibox.toml files that
/// don't yet have one. The content is intentionally a string literal (not
/// generated from `ProcessKitSection::default()`) so this migration is stable
/// and reviewable as a fixture.
const DEFAULT_PROCESSKIT_BLOCK: &str = "\
# =============================================================================
# [processkit] — content layer source (skills, primitives, processes)
# =============================================================================
# processkit ships the skills and primitives that aibox installs into the
# project. The default upstream is the canonical projectious-work/processkit
# repo. Companies can fork processkit and have their projects consume the fork
# by changing `source` to point at their fork.
#
# `version` is the git tag of the processkit source to consume. The sentinel
# value \"unset\" means \"no version pinned yet\" — the project doesn't yet
# consume processkit content. Edit this once a real version is available.
[processkit]
source   = \"https://github.com/projectious-work/processkit.git\"
version  = \"unset\"
src_path = \"src\"
# branch = \"main\"   # optional — for tracking a moving branch (discouraged)
";

/// If `aibox.toml` exists in `root` and lacks a `[processkit]` section,
/// surgically insert the default block and write a migration note. This
/// runs at most once per project — once the section is present, this is a
/// no-op.
fn ensure_processkit_section_in(root: &Path) -> Result<()> {
    let toml_path = root.join("aibox.toml");
    if !toml_path.exists() {
        return Ok(());
    }

    let original = fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;

    if has_processkit_section(&original) {
        return Ok(());
    }

    let updated = insert_processkit_section(&original);
    fs::write(&toml_path, &updated)
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;

    output::ok("Added [processkit] section to aibox.toml (one-time migration)");

    // Write a migration note describing what was done.
    let note_path = root
        .join("context")
        .join("migrations")
        .join("aibox-processkit-section-added.md");
    write_processkit_migration_note(&note_path)?;

    Ok(())
}

/// Detect whether the TOML source already contains a `[processkit]` section
/// header. Looks for a line that, after trimming whitespace, equals the
/// literal `[processkit]`. This avoids matching e.g. `[processkit.foo]` or
/// commented-out lines.
fn has_processkit_section(toml_src: &str) -> bool {
    for line in toml_src.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.trim_end() == "[processkit]" {
            return true;
        }
    }
    false
}

/// Insert the default `[processkit]` block into `toml_src`. Insertion point:
///
/// 1. If a `[customization]` (or legacy `[appearance]`) section exists,
///    insert directly above its header (and any comments preceding the
///    header). This puts processkit in a sensible place: after content
///    (`[ai]`, `[skills]`, `[addons]`) and before presentation.
/// 2. Otherwise, append the block to the end of the file with a leading
///    blank line so it's visually separated from whatever section came last.
fn insert_processkit_section(toml_src: &str) -> String {
    let lines: Vec<&str> = toml_src.lines().collect();

    // Locate the [customization] / [appearance] header line, if present.
    let mut header_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        let head = trimmed.trim_end();
        if head == "[customization]" || head == "[appearance]" {
            header_idx = Some(i);
            break;
        }
    }

    let block = format!("{}\n", DEFAULT_PROCESSKIT_BLOCK);

    match header_idx {
        Some(idx) => {
            // Walk backwards over the comment-band immediately preceding the
            // header so we insert above the comments that belong to it (e.g.
            // the `# =====` separator and the `# [customization] — ...` line).
            let mut insert_at = idx;
            while insert_at > 0 {
                let prev = lines[insert_at - 1];
                let trimmed = prev.trim_start();
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    insert_at -= 1;
                } else {
                    break;
                }
            }

            let mut out = String::with_capacity(toml_src.len() + block.len() + 2);
            for (i, line) in lines.iter().enumerate() {
                if i == insert_at {
                    out.push_str(&block);
                    out.push('\n');
                }
                out.push_str(line);
                out.push('\n');
            }
            // If the original file did not end with a newline, the loop above
            // still added one. Trim it to preserve original trailing-newline
            // semantics: if the original ended without a newline, drop ours.
            if !toml_src.ends_with('\n') {
                out.pop();
            }
            out
        }
        None => {
            // Append to end. Ensure we have a clean blank-line separator.
            let mut out = String::with_capacity(toml_src.len() + block.len() + 2);
            out.push_str(toml_src);
            if !toml_src.ends_with('\n') {
                out.push('\n');
            }
            if !toml_src.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str(&block);
            out
        }
    }
}

/// Write the migration note for the processkit-section addition. Idempotent:
/// does not overwrite an existing note.
fn write_processkit_migration_note(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create migration directory {}", parent.display())
        })?;
    }
    let date = chrono_free_date();
    let body = format!(
        "\
# Migration: [processkit] section added to aibox.toml

> **SAFETY: Do not execute any actions in this document automatically.**
> **Discuss each item with the project owner before proceeding.**

**Generated:** {date}
**Status:** pending
**Type:** schema migration (additive, one-time)

## Summary

aibox now reads a `[processkit]` section from `aibox.toml` to determine where
project content (skills, primitives, processes) should come from. Existing
projects pre-date this section, so on the first `aibox sync` after upgrading,
aibox surgically inserts a default `[processkit]` block into your
`aibox.toml`:

```toml
[processkit]
source   = \"https://github.com/projectious-work/processkit.git\"
version  = \"unset\"
src_path = \"src\"
# branch = \"main\"
```

The sentinel `version = \"unset\"` means \"no processkit version pinned yet\".
Until you set a real version, no processkit content will be fetched. This
migration is purely plumbing — your existing project files are untouched.

## What you should do

- [ ] Decide whether you want to consume processkit content for this project.
- [ ] If yes: replace `\"unset\"` with a real released version of processkit
      (e.g. `\"v0.4.0\"`). Run `aibox sync` again to pull content.
- [ ] If you maintain a fork of processkit, change `source` to point at it.
- [ ] If you want to track a moving branch instead of a tag, set `branch`
      and leave `version` as the empty sentinel — discouraged but supported.
- [ ] Mark this migration as completed (change Status above to \"completed\").

## Rollback

To revert: `git checkout HEAD -- aibox.toml` and delete this file. The
migration will re-run on the next `aibox sync`.
"
    );
    fs::write(path, body)
        .with_context(|| format!("Failed to write migration note {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_sample_lock(root: &std::path::Path, cli_version: &str) {
        let lock = crate::lock::AiboxLock {
            aibox: crate::lock::AiboxLockSection {
                cli_version: cli_version.to_string(),
                synced_at: "2026-04-01T00:00:00Z".to_string(),
            },
            processkit: None,
        };
        crate::lock::write_lock(root, &lock).unwrap();
    }

    #[test]
    fn test_no_migration_when_versions_match() {
        let tmp = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");
        write_sample_lock(tmp.path(), current);
        fs::create_dir_all(tmp.path().join("context/migrations")).unwrap();

        check_and_generate_migration_in(tmp.path()).unwrap();

        // No migration document should be created
        let entries: Vec<_> = fs::read_dir(tmp.path().join("context/migrations"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(entries.is_empty(), "no migration doc when versions match");
    }

    #[test]
    fn test_no_migration_when_no_lock() {
        let tmp = TempDir::new().unwrap();

        // No aibox.lock — fresh project
        check_and_generate_migration_in(tmp.path()).unwrap();

        // context/migrations/ should not even be created
        assert!(
            !tmp.path().join("context/migrations").exists(),
            "no migrations dir for fresh project"
        );
    }

    #[test]
    fn test_migration_doc_generated_on_version_change() {
        let tmp = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");
        write_sample_lock(tmp.path(), "0.0.1");

        check_and_generate_migration_in(tmp.path()).unwrap();

        let expected_path = tmp
            .path()
            .join("context/migrations")
            .join(format!("0.0.1-to-{}.md", current));
        assert!(expected_path.exists(), "migration doc should be created");

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains(&format!("v0.0.1 \u{2192} v{}", current)));
        assert!(content.contains("**Status:** pending"));

        // aibox.lock cli_version should be updated
        let updated_lock = crate::lock::read_lock(tmp.path()).unwrap().unwrap();
        assert_eq!(updated_lock.aibox.cli_version, current);
    }

    #[test]
    fn test_migration_doc_not_overwritten() {
        let tmp = TempDir::new().unwrap();
        let current = env!("CARGO_PKG_VERSION");
        let migrations_dir = tmp.path().join("context/migrations");
        fs::create_dir_all(&migrations_dir).unwrap();

        let filename = format!("0.0.1-to-{}.md", current);
        let filepath = migrations_dir.join(&filename);
        let existing_content = "# User-edited migration doc\nStatus: in-progress\n";
        fs::write(&filepath, existing_content).unwrap();

        write_sample_lock(tmp.path(), "0.0.1");

        check_and_generate_migration_in(tmp.path()).unwrap();

        // File should not be overwritten
        let content = fs::read_to_string(&filepath).unwrap();
        assert_eq!(
            content, existing_content,
            "existing migration doc should not be overwritten"
        );
    }

    #[test]
    fn migrate_legacy_lock_files_absorbs_aibox_version() {
        let tmp = TempDir::new().unwrap();
        // Write a legacy flat aibox.lock (no sections).
        let flat_lock = "\
source = \"https://github.com/example/processkit.git\"
version = \"v0.4.0\"
src_path = \"src\"
installed_at = \"2026-04-01T00:00:00Z\"
";
        fs::write(tmp.path().join("aibox.lock"), flat_lock).unwrap();
        fs::write(tmp.path().join(".aibox-version"), "0.16.5").unwrap();

        migrate_legacy_lock_files(tmp.path()).unwrap();

        // .aibox-version must be deleted.
        assert!(
            !tmp.path().join(".aibox-version").exists(),
            ".aibox-version should be deleted"
        );
        // aibox.lock must now be in sectioned shape.
        let lock = crate::lock::read_lock(tmp.path()).unwrap().unwrap();
        assert_eq!(lock.aibox.cli_version, "0.16.5");
        assert!(lock.processkit.is_some());
    }

    #[test]
    fn migrate_legacy_lock_files_noop_when_no_aibox_version() {
        let tmp = TempDir::new().unwrap();
        // No .aibox-version, no lock — nothing to do.
        migrate_legacy_lock_files(tmp.path()).unwrap();
        assert!(!tmp.path().join(".aibox-version").exists());
    }

    #[test]
    fn test_migration_doc_contains_required_sections() {
        let doc = format_migration_doc("0.7.0", "0.8.0", "2026-03-23");

        // Safety header
        assert!(doc.contains("SAFETY: Do not execute any actions in this document automatically."));
        assert!(doc.contains("Discuss each item with the project owner before proceeding."));
        assert!(doc.contains("Do not modify aibox.toml without explicit user confirmation."));

        // Status
        assert!(doc.contains("**Status:** pending"));

        // Action items with checkboxes
        assert!(doc.contains("- [ ] Review this migration document with the project owner"));
        assert!(doc.contains("- [ ] Run `aibox sync` to regenerate container files"));
        assert!(doc.contains("- [ ] Rebuild the container: `aibox build`"));

        // Verification checklist
        assert!(doc.contains("## Verification Checklist"));
        assert!(doc.contains("- [ ] `aibox sync` completes without errors"));
        assert!(doc.contains("- [ ] Container builds successfully"));
        assert!(doc.contains("- [ ] `AGENTS.md` is present"));

        // Rollback section
        assert!(doc.contains("## Rollback"));
        assert!(doc.contains("git checkout HEAD -- aibox.lock context/ .devcontainer/"));

        // Other required sections
        assert!(doc.contains("## Breaking Changes"));
        assert!(doc.contains("## processkit Content Changes"));
        assert!(doc.contains("## Known Issues"));
    }

    #[test]
    fn test_format_migration_doc_versions_and_date() {
        let doc = format_migration_doc("1.2.3", "2.0.0", "2026-01-15");

        assert!(doc.contains("# Migration: v1.2.3 \u{2192} v2.0.0"));
        assert!(doc.contains("**Generated:** 2026-01-15"));
        assert!(doc.contains("**aibox CLI version:** v2.0.0"));
        assert!(doc.contains("from v1.2.3 to v2.0.0"));
    }

    // -- ProcessKit section auto-migration ----------------------------------

    const SAMPLE_LEGACY_TOML: &str = r#"[aibox]
version = "0.14.4"
base = "debian"

[container]
name = "demo"
hostname = "demo"
user = "aibox"

[context]
schema_version = "1.0.0"
packages = ["core"]

# =============================================================================
# [ai] — AI coding assistant providers
# =============================================================================
[ai]
providers = ["claude"]

# =============================================================================
# [customization] — color theme, shell prompt, and zellij layout
# =============================================================================
[customization]
theme  = "gruvbox-dark"
prompt = "default"
layout = "dev"

[audio]
enabled = false
"#;

    #[test]
    fn migration_adds_processkit_section_when_missing() {
        let updated = insert_processkit_section(SAMPLE_LEGACY_TOML);
        assert!(
            has_processkit_section(&updated),
            "after insertion, [processkit] should be present"
        );
        assert!(
            updated.contains("https://github.com/projectious-work/processkit.git"),
            "default source should be present"
        );
        assert!(
            updated.contains("version  = \"unset\""),
            "default version sentinel should be present"
        );
        assert!(
            updated.contains("src_path = \"src\""),
            "default src_path should be present"
        );
    }

    #[test]
    fn migration_preserves_existing_processkit_section() {
        let already_has = format!(
            "{}\n[processkit]\nsource = \"https://forks.example/pk.git\"\nversion = \"v0.5.0\"\n",
            SAMPLE_LEGACY_TOML
        );
        assert!(has_processkit_section(&already_has));

        // The ensure helper would no-op; verify the detector itself is correct.
        let tmp = TempDir::new().unwrap();
        let toml_path = tmp.path().join("aibox.toml");
        fs::write(&toml_path, &already_has).unwrap();

        ensure_processkit_section_in(tmp.path()).unwrap();

        let after = fs::read_to_string(&toml_path).unwrap();
        assert_eq!(after, already_has, "file should be unchanged");
        assert!(
            !tmp.path()
                .join("context/migrations/aibox-processkit-section-added.md")
                .exists(),
            "no migration note should be written when section already exists"
        );
    }

    #[test]
    fn migration_preserves_comments_and_ordering_in_other_sections() {
        let updated = insert_processkit_section(SAMPLE_LEGACY_TOML);

        // [aibox], [container], [context], [ai], [audio] are all still present
        // and in the original order.
        let aibox_pos = updated.find("[aibox]").unwrap();
        let container_pos = updated.find("[container]").unwrap();
        let context_pos = updated.find("[context]").unwrap();
        let ai_pos = updated.find("[ai]").unwrap();
        let processkit_pos = updated.find("[processkit]").unwrap();
        let custom_pos = updated.find("[customization]").unwrap();
        let audio_pos = updated.find("[audio]").unwrap();

        assert!(aibox_pos < container_pos);
        assert!(container_pos < context_pos);
        assert!(context_pos < ai_pos);
        assert!(ai_pos < processkit_pos);
        assert!(processkit_pos < custom_pos);
        assert!(custom_pos < audio_pos);

        // The original [ai] header comment band is intact.
        assert!(updated.contains("# [ai] — AI coding assistant providers"));
        assert!(updated.contains("# [customization] — color theme, shell prompt, and zellij layout"));
        // Original concrete values are intact.
        assert!(updated.contains("name = \"demo\""));
        assert!(updated.contains("theme  = \"gruvbox-dark\""));
        assert!(updated.contains("layout = \"dev\""));
    }

    #[test]
    fn migration_inserts_above_customization_when_present() {
        let updated = insert_processkit_section(SAMPLE_LEGACY_TOML);
        let processkit_pos = updated.find("[processkit]").unwrap();
        let custom_pos = updated.find("[customization]").unwrap();
        assert!(
            processkit_pos < custom_pos,
            "[processkit] must appear above [customization]"
        );

        // The customization comment band must follow processkit, not precede it.
        let custom_comment_pos = updated
            .find("# [customization] — color theme")
            .expect("customization comment should exist");
        assert!(
            processkit_pos < custom_comment_pos,
            "processkit block should be inserted ABOVE the [customization] comment band"
        );
    }

    #[test]
    fn migration_appends_to_end_when_no_customization() {
        let no_custom = r#"[aibox]
version = "0.14.4"
base = "debian"

[container]
name = "demo"

[ai]
providers = ["claude"]
"#;
        let updated = insert_processkit_section(no_custom);
        assert!(has_processkit_section(&updated));
        // Processkit block should come after [ai].
        let ai_pos = updated.find("[ai]").unwrap();
        let processkit_pos = updated.find("[processkit]").unwrap();
        assert!(processkit_pos > ai_pos);
    }

    #[test]
    fn migration_appends_when_legacy_appearance_section_present() {
        // [appearance] is the legacy alias for [customization]; insertion
        // should still target it.
        let with_appearance = r#"[aibox]
version = "0.14.4"

[container]
name = "demo"

[appearance]
theme = "dracula"
"#;
        let updated = insert_processkit_section(with_appearance);
        let processkit_pos = updated.find("[processkit]").unwrap();
        let appearance_pos = updated.find("[appearance]").unwrap();
        assert!(
            processkit_pos < appearance_pos,
            "processkit must precede legacy [appearance]"
        );
    }

    #[test]
    fn migration_end_to_end_writes_file_and_note() {
        let tmp = TempDir::new().unwrap();
        let toml_path = tmp.path().join("aibox.toml");
        fs::write(&toml_path, SAMPLE_LEGACY_TOML).unwrap();

        ensure_processkit_section_in(tmp.path()).unwrap();

        let after = fs::read_to_string(&toml_path).unwrap();
        assert!(has_processkit_section(&after));

        // The result must still be a parseable AiboxConfig.
        crate::config::AiboxConfig::from_str(&after)
            .expect("post-migration aibox.toml must remain valid");

        let note = tmp
            .path()
            .join("context/migrations/aibox-processkit-section-added.md");
        assert!(note.exists(), "migration note should be created");
        let body = fs::read_to_string(&note).unwrap();
        assert!(body.contains("[processkit]"));
        assert!(body.contains("**Status:** pending"));
    }

    #[test]
    fn migration_end_to_end_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let toml_path = tmp.path().join("aibox.toml");
        fs::write(&toml_path, SAMPLE_LEGACY_TOML).unwrap();

        ensure_processkit_section_in(tmp.path()).unwrap();
        let first = fs::read_to_string(&toml_path).unwrap();

        ensure_processkit_section_in(tmp.path()).unwrap();
        let second = fs::read_to_string(&toml_path).unwrap();

        assert_eq!(first, second, "second run must be a no-op");
    }

    #[test]
    fn migration_no_op_when_no_aibox_toml() {
        let tmp = TempDir::new().unwrap();
        // No aibox.toml at all
        ensure_processkit_section_in(tmp.path()).unwrap();
        assert!(
            !tmp.path()
                .join("context/migrations/aibox-processkit-section-added.md")
                .exists()
        );
    }

    #[test]
    fn has_processkit_section_ignores_commented_lines() {
        let src = "# [processkit] this is just a comment\n[ai]\n";
        assert!(!has_processkit_section(src));
    }

    #[test]
    fn has_processkit_section_ignores_subsection_keys() {
        let src = "[processkit.tools]\nfoo = 1\n";
        assert!(!has_processkit_section(src));
    }

    #[test]
    fn test_chrono_free_date_returns_valid_format() {
        let date = chrono_free_date();
        // Should be YYYY-MM-DD or "unknown"
        if date != "unknown" {
            assert_eq!(date.len(), 10, "date should be 10 chars: {}", date);
            assert_eq!(&date[4..5], "-", "should have dash at pos 4");
            assert_eq!(&date[7..8], "-", "should have dash at pos 7");
        }
    }
}
