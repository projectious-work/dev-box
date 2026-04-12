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
    // Migrate old processkit runtime settings out of [context] (processkit v0.8.0+).
    migrate_processkit_context_settings(Path::new("."))?;
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

    // Read the desired processkit version from aibox.toml (what sync will install).
    // The lock holds the old installed version; the config holds the target.
    // Falls back to None if aibox.toml can't be read.
    let config_pk_version: Option<String> =
        crate::config::AiboxConfig::load(&root.join("aibox.toml"))
            .ok()
            .and_then(|c| {
                let v = c.processkit.version;
                if v.is_empty() || v == "unset" {
                    None
                } else {
                    Some(v)
                }
            });

    // Generate migration document, passing processkit info for context.
    generate_migration_doc(
        root,
        &stored_version,
        current_version,
        lock.processkit.as_ref(),
        config_pk_version.as_deref(),
    )?;

    // Update lock with new cli_version. synced_at is left unchanged here;
    // cmd_sync updates it when it writes the full lock after install.
    let mut updated_lock = lock;
    updated_lock.aibox.cli_version = current_version.to_string();
    crate::lock::write_lock(root, &updated_lock)
        .context("Failed to update aibox.lock after migration check")?;

    Ok(())
}

/// Generate a migration document at
/// `{root}/context/migrations/YYYYMMDD_HHMM_{from}-to-{to}.md`.
fn generate_migration_doc(
    root: &Path,
    from: &str,
    to: &str,
    pk: Option<&crate::lock::ProcessKitLockSection>,
    config_pk_version: Option<&str>,
) -> Result<()> {
    let migrations_dir = root.join("context").join("migrations");
    fs::create_dir_all(&migrations_dir).context("Failed to create context/migrations/")?;

    let datetime_slug = chrono_free_datetime_slug();
    let filename = format!("{}_{}-to-{}.md", datetime_slug, from, to);
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
    let content = format_migration_doc(from, to, &date, pk, config_pk_version);

    fs::write(&filepath, content)
        .with_context(|| format!("Failed to write migration document {}", filename))?;

    output::ok(&format!(
        "Generated migration document: context/migrations/{}",
        filename
    ));
    output::warn("Review the migration document with your AI agent before proceeding");

    Ok(())
}

/// Get the current date for display in document bodies (`YYYY-MM-DD`).
fn chrono_free_date() -> String {
    std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get a sortable datetime slug for migration filenames (`YYYYMMDD_HHMM`).
fn chrono_free_datetime_slug() -> String {
    std::process::Command::new("date")
        .arg("+%Y%m%d_%H%M")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "00000000_0000".to_string())
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
static KNOWN_MIGRATIONS: &[MigrationEntry] = &[MigrationEntry {
    from: "0.17.4",
    to: "0.17.5",
    breaking_changes: &[
        "processkit v0.8.0 restructured its `src/` directory (GrandLily layout). \
             The live install destinations are unchanged — existing project `context/` \
             directories are unaffected. Only the aibox installer needed updating.",
    ],
    action_items: &[
        "No `aibox.toml` changes required — the installer handles both v0.7.0 and v0.8.0 layouts transparently",
        "If pinning processkit in `[processkit].version`, update to `v0.8.0` for the latest content",
    ],
}];

/// Find a known migration entry for the given version pair.
fn find_known_migration(from: &str, to: &str) -> Option<&'static MigrationEntry> {
    KNOWN_MIGRATIONS
        .iter()
        .find(|m| m.from == from && m.to == to)
}

// ---------------------------------------------------------------------------
// Document formatting
// ---------------------------------------------------------------------------

/// Enumerate every semver version strictly between `from` (exclusive) and
/// `to` (inclusive), in ascending order. Parses `major.minor.patch` integers.
/// If parsing fails or `from >= to`, returns `vec![to.to_string()]`.
fn intermediate_versions(from: &str, to: &str) -> Vec<String> {
    fn parse_semver(s: &str) -> Option<(u64, u64, u64)> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let mut parts = s.splitn(3, '.');
        let major = parts.next()?.parse::<u64>().ok()?;
        let minor = parts.next()?.parse::<u64>().ok()?;
        let patch = parts.next()?.parse::<u64>().ok()?;
        Some((major, minor, patch))
    }

    let (Some(from_v), Some(to_v)) = (parse_semver(from), parse_semver(to)) else {
        return vec![to.to_string()];
    };

    if from_v >= to_v {
        return vec![to.to_string()];
    }

    let (fmaj, fmin, fpatch) = from_v;
    let (tmaj, tmin, tpatch) = to_v;

    // Only enumerate patch-level increments within the same major.minor range.
    // For cross-minor or cross-major jumps, just return the target version to
    // keep the list manageable.
    if fmaj == tmaj && fmin == tmin {
        ((fpatch + 1)..=tpatch)
            .map(|patch| format!("{}.{}.{}", fmaj, fmin, patch))
            .collect()
    } else {
        vec![to.to_string()]
    }
}

/// Format the full migration document content.
fn format_migration_doc(
    from: &str,
    to: &str,
    date: &str,
    pk: Option<&crate::lock::ProcessKitLockSection>,
    config_pk_version: Option<&str>,
) -> String {
    let known = find_known_migration(from, to);

    // Build breaking changes block.
    let breaking_changes = if let Some(entry) = known {
        let mut lines: Vec<String> = entry
            .breaking_changes
            .iter()
            .map(|c| format!("- {}", c))
            .collect();
        // Append version-specific action items into the breaking changes section.
        if !entry.action_items.is_empty() {
            lines.push(String::new());
            lines.push("**Required actions for this upgrade:**".to_string());
            for item in entry.action_items {
                lines.push(format!("- {}", item));
            }
        }
        lines.join("\n")
    } else {
        let versions = intermediate_versions(from, to);
        let links: Vec<String> = versions
            .iter()
            .map(|v| {
                let tag = if v.starts_with('v') {
                    v.clone()
                } else {
                    format!("v{}", v)
                };
                format!(
                    "- https://github.com/projectious-work/aibox/releases/tag/{}",
                    tag
                )
            })
            .collect();
        format!(
            "Review the release notes for each version in this upgrade:\n{}",
            links.join("\n")
        )
    };

    // Determine the effective processkit version to display.
    // When aibox.toml says "latest", show the actually installed version from the
    // lock (the concrete tag that was last synced) rather than the sentinel string.
    let config_is_latest = config_pk_version
        .map(|v| v == crate::config::PROCESSKIT_VERSION_LATEST)
        .unwrap_or(false);
    let effective_pk_version = if config_is_latest {
        // Show the installed (concrete) version from the lock when config uses "latest".
        pk.map(|p| p.version.as_str())
            .unwrap_or("not yet installed")
    } else {
        config_pk_version
            .or_else(|| pk.map(|p| p.version.as_str()))
            .unwrap_or("not configured")
    };
    let effective_pk_version = effective_pk_version.trim_start_matches('v');

    // Previous processkit version (from lock) — used to guide template diffing.
    let prev_pk_version = pk
        .map(|p| p.version.trim_start_matches('v').to_string())
        .unwrap_or_default();

    // Build processkit header line.
    let pk_line = if effective_pk_version == "not configured" {
        "not configured".to_string()
    } else {
        format!("v{}", effective_pk_version)
    };

    // Note if lock and config versions differ (user upgraded processkit alongside aibox).
    let pk_version_note = match (pk, config_pk_version) {
        (Some(p), Some(cfg))
            if p.version.trim_start_matches('v') != cfg.trim_start_matches('v') =>
        {
            format!(
                " (upgraded from `v{}` — aibox.toml now targets `v{}`)",
                p.version.trim_start_matches('v'),
                cfg.trim_start_matches('v')
            )
        }
        _ => String::new(),
    };

    // Build processkit state section.
    let processkit_state_section = match pk {
        Some(p) => {
            let source = &p.source;
            let version_line = if config_is_latest {
                format!(
                    "processkit is tracking `version = \"latest\"` — installed: \
                     `v{effective_pk_version}`{pk_version_note} (source: `{source}`).\n\
                     \n\
                     **Upgrade policy for `version = \"latest\"`:**\n\
                     `aibox sync` resolves `latest` at run time using a semver-aware policy:\n\
                     - **Patch / minor upgrades** (same major): applied automatically.\n\
                     - **Major upgrades**: blocked — a warning is shown and sync stays on the\n\
                       latest release within the current major. To cross a major boundary, pin\n\
                       an explicit version in `aibox.toml` (e.g. `version = \"v2.0.0\"`)."
                )
            } else {
                format!(
                    "processkit is pinned to `v{effective_pk_version}`{pk_version_note} \
                     (source: `{source}`)."
                )
            };
            format!(
                "{version_line}\n\
                 \n\
                 **Check for pending processkit content migrations:**\n\
                 Look in `/workspace/context/migrations/pending/` — `aibox sync` deposits\n\
                 content migration documents there during the 3-way diff. Do NOT skip this step.\n\
                 \n\
                 If files exist, do NOT handle them manually. Instead:\n\
                 \n\
                 1. Use `skill-finder` to locate the processkit migration management skill\n\
                    (search for \"migration\" or \"content update\").\n\
                 2. Invoke that skill — it knows the correct workflow, state machine, and document\n\
                    format for reviewing and applying processkit content migrations.\n\
                 3. Work through each pending migration with the owner before marking it applied.\n\
                 \n\
                 processkit owns the migration format and workflow; defer entirely to its skill."
            )
        }
        None => {
            // No processkit info from lock, but we may have it from config.
            if config_is_latest {
                "processkit is set to `version = \"latest\"` in aibox.toml \
                 (lock not yet written — fresh install).\n\
                 `aibox sync` will resolve `latest` to the newest available release \
                 and install it.\n\
                 Check `/workspace/context/migrations/pending/` after sync for any \
                 content migration files."
                    .to_string()
            } else if let Some(cfg_v) = config_pk_version {
                let v = cfg_v.trim_start_matches('v');
                format!(
                    "processkit is pinned to `v{v}` (source: aibox.toml; lock not yet written).\n\
                     Check `/workspace/context/migrations/pending/` for any pending content migrations\n\
                     and use `skill-finder` to locate the processkit migration skill if any exist."
                )
            } else {
                "processkit is not yet configured in this project. Run `aibox sync` on the host\n\
                 to initialize processkit content."
                    .to_string()
            }
        }
    };

    // Snapshot dir label for template diffs: "v0.8.0" if previous version known,
    // otherwise a generic placeholder.
    let prev_pk_snapshot_dir = if prev_pk_version.is_empty() {
        "v<previous-version>".to_string()
    } else {
        format!("v{}", prev_pk_version)
    };

    format!(
        "\
# Migration: v{from} \u{2192} v{to}

> **SAFETY: Do not execute any actions automatically.**
> **Discuss each item with the project owner before proceeding.**
> **Do not modify aibox.toml without explicit user confirmation.**
> **`aibox` commands run on the HOST, outside the container — you cannot run them.**

**Generated:** {date}
**Status:** pending
**aibox CLI:** v{from} \u{2192} v{to}
**processkit:** {pk_line}

## Summary

aibox has been updated from v{from} to v{to}. Review each section below
and discuss action items with the project owner.

## Breaking Changes

{breaking_changes}

## Action Items

### Host actions (owner runs these outside the container)

- [ ] Owner: verify `aibox sync` was run for v{to} — check `aibox.lock` at
      `/workspace/aibox.lock`: `[aibox].cli_version` should equal `{to}` and
      `synced_at` should be a recent timestamp
- [ ] Owner: if sync has NOT been run, run `aibox sync` on the host, then `aibox build`
- [ ] Owner: if the container was not rebuilt after sync, run `aibox build` then `aibox start`

### Agent verification (you can do these now)

- [ ] Read `/workspace/aibox.lock` — confirm `[aibox].cli_version = \"{to}\"`
- [ ] Read `/workspace/aibox.lock` — confirm `[processkit].version` matches
      `/workspace/aibox.toml [processkit].version`
- [ ] Verify `/workspace/AGENTS.md` exists and is non-empty
- [ ] Verify `/workspace/context/skills/` directory exists
- [ ] Verify `/workspace/context/skills/skill-finder/` exists (core skill)
- [ ] Verify `/workspace/context/processes/` directory exists
- [ ] Verify `/workspace/context/schemas/` directory exists
- [ ] Verify `/workspace/context/templates/processkit/v{effective_pk_version}/` snapshot directory exists
- [ ] Check `/workspace/context/migrations/` (this directory) for **other unreviewed CLI migration
      documents** (files named `YYYYMMDD_HHMM_X.Y.Z-to-A.B.C.md`):
      - Files with the **same** `from→to` range as this one (e.g. two `0.17.5-to-0.17.6.md`
        files) are retries — the most recent is authoritative; mark older ones as cancelled
      - Files with a **different** range (e.g. `0.17.5-to-0.17.6.md` alongside this
        `0.17.6-to-0.17.7.md`) are **sequential migrations** — both must be reviewed in
        chronological order; do NOT discard them
- [ ] Check `/workspace/context/migrations/pending/` for processkit content migration files:
      - Use `skill-finder` to locate the processkit migration skill, then work through each
        file with the owner in chronological order — do NOT handle migrations manually
      - **When reviewing template files like `AGENTS.md`:** do NOT diff the installed
        (customized) file against the new template — that creates noise from project
        customizations, hiding real upstream changes. Instead, diff the two template
        snapshots to see only what changed in the upstream:
        `diff context/templates/processkit/{prev_pk_snapshot_dir}/AGENTS.md \
context/templates/processkit/v{effective_pk_version}/AGENTS.md`
        Then apply only those delta changes on top of the customized installed file.
- [ ] Mark this migration as completed (change Status to \"completed\")

## processkit State

{processkit_state_section}

## AGENTS.md Review

After the upgrade, verify `AGENTS.md` is current:
- [ ] processkit version reference matches `aibox.lock` (`v{effective_pk_version}`)
- [ ] Configured AI harnesses/providers match the `[ai]` section in `aibox.toml`
- [ ] Build / test / lint commands are still accurate
- [ ] Project-specific notes and operational gotchas are up to date

## Verification Summary

After all host actions are confirmed and agent verifications pass, mark Status as \"completed\".

## Rollback

To revert this migration (owner runs on host):
```
git checkout HEAD~1 -- aibox.lock context/ .devcontainer/
aibox sync
```

## Known Issues

Check https://github.com/projectious-work/aibox/issues for known issues with v{to}.
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
///
/// NOTE: the source URL here must match `crate::processkit_vocab::PROCESSKIT_GIT_SOURCE`.
/// Rust's `concat!` macro does not accept non-literal `const` values, so we
/// cannot embed the constant directly. Update both places together.
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
- [ ] After running `aibox sync`, check `context/migrations/pending/` for any
      processkit content migration documents — these describe content-level
      changes that may require your review.
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

// ---------------------------------------------------------------------------
// processkit v0.8.0 context settings migration
// ---------------------------------------------------------------------------

/// The aibox-owned keys that belong permanently in `[context]`.
/// Everything else in `[context]` is a processkit runtime setting that
/// should live in per-skill `config/settings.toml` files.
const AIBOX_CONTEXT_KEYS: &[&str] = &["schema_version", "packages"];

/// Migrate old processkit runtime settings out of `aibox.toml [context]`.
///
/// processkit v0.8.0 moved runtime configuration from `aibox.toml [context]`
/// to per-skill config files (`context/skills/<name>/config/settings.toml`).
/// This migration detects any old keys still present in `[context]` and moves
/// them to the correct location, then removes them from `aibox.toml`.
///
/// **Mappings:**
/// - `id_format`, `id_slug` → `context/skills/id-management/config/settings.toml` under `[ids]`
/// - `directories`, `sharding`, `index` → `context/skills/index-management/config/settings.toml`
/// - `budget`, `grooming`, and any other unrecognised keys → warning, left in place
///
/// **Idempotent:** if the target skill config file already exists (the agent
/// already set it up), the old keys are removed from `aibox.toml` without
/// overwriting the skill config.
pub fn migrate_processkit_context_settings(root: &Path) -> Result<()> {
    let aibox_toml_path = root.join("aibox.toml");
    if !aibox_toml_path.exists() {
        return Ok(());
    }

    // Phase 1 — read raw TOML to extract values (owned data, no borrow conflicts).
    let raw = fs::read_to_string(&aibox_toml_path)
        .with_context(|| format!("Failed to read {}", aibox_toml_path.display()))?;
    let raw_value: toml::Value =
        toml::from_str(&raw).with_context(|| "Failed to parse aibox.toml as TOML")?;

    let context_table = match raw_value.get("context").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => return Ok(()),
    };

    // Collect keys that don't belong to aibox.
    let old_keys: Vec<&str> = context_table
        .keys()
        .filter(|k| !AIBOX_CONTEXT_KEYS.contains(&k.as_str()))
        .map(|k| k.as_str())
        .collect();

    if old_keys.is_empty() {
        return Ok(());
    }

    output::info(&format!(
        "Found old processkit settings in aibox.toml [context]: {}",
        old_keys.join(", ")
    ));

    let date = chrono_free_date();

    // Phase 2 — migrate known keys to per-skill config files.
    // Keys successfully migrated (to be removed from aibox.toml).
    let mut remove_keys: Vec<&str> = Vec::new();

    // --- id-management: id_format, id_slug → [ids] -------------------------
    let id_format = context_table
        .get("id_format")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let id_slug = context_table.get("id_slug").and_then(|v| v.as_bool());

    if id_format.is_some() || id_slug.is_some() {
        let skill_dir = root.join("context").join("skills").join("id-management");
        let config_dir = skill_dir.join("config");
        let settings_path = config_dir.join("settings.toml");

        if settings_path.exists() {
            output::info(
                "context/skills/id-management/config/settings.toml already exists \
                 — removing old keys from aibox.toml without overwriting",
            );
        } else if skill_dir.is_dir() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create {}", config_dir.display()))?;
            let content = build_id_management_settings(id_format.as_deref(), id_slug, &date);
            fs::write(&settings_path, &content)
                .with_context(|| format!("Failed to write {}", settings_path.display()))?;
            output::ok("Migrated id settings → context/skills/id-management/config/settings.toml");
        } else {
            output::warn(
                "id-management skill is not installed — id_format/id_slug will be \
                 removed from aibox.toml. Re-run `aibox sync` after installing the skill.",
            );
        }
        if id_format.is_some() {
            remove_keys.push("id_format");
        }
        if id_slug.is_some() {
            remove_keys.push("id_slug");
        }
    }

    // --- index-management: directories, sharding, index → settings.toml ----
    let has_index_keys = ["directories", "sharding", "index"]
        .iter()
        .any(|k| context_table.contains_key(*k));

    if has_index_keys {
        let skill_dir = root.join("context").join("skills").join("index-management");
        let config_dir = skill_dir.join("config");
        let settings_path = config_dir.join("settings.toml");

        if settings_path.exists() {
            output::info(
                "context/skills/index-management/config/settings.toml already exists \
                 — removing old keys from aibox.toml without overwriting",
            );
        } else if skill_dir.is_dir() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create {}", config_dir.display()))?;
            let content = build_index_management_settings(context_table, &date)?;
            fs::write(&settings_path, &content)
                .with_context(|| format!("Failed to write {}", settings_path.display()))?;
            output::ok(
                "Migrated index settings → context/skills/index-management/config/settings.toml",
            );
        } else {
            output::warn(
                "index-management skill is not installed — directories/sharding/index will be \
                 removed from aibox.toml. Re-run `aibox sync` after installing the skill.",
            );
        }
        for key in &["directories", "sharding", "index"] {
            if context_table.contains_key(*key) {
                remove_keys.push(key);
            }
        }
    }

    // --- Unknown keys -------------------------------------------------------
    let known_old_keys = ["id_format", "id_slug", "directories", "sharding", "index"];
    for key in &old_keys {
        if !known_old_keys.contains(key) {
            output::warn(&format!(
                "aibox.toml [context] has unrecognised processkit key '{}' — \
                 cannot determine migration target, leaving in place",
                key
            ));
        }
    }

    // Phase 3 — remove migrated keys from aibox.toml using toml_edit
    // (preserves comments and formatting of the rest of the file).
    if remove_keys.is_empty() {
        return Ok(());
    }

    let mut doc: toml_edit::DocumentMut = raw
        .parse()
        .with_context(|| "Failed to parse aibox.toml with toml_edit")?;

    if let Some(context) = doc["context"].as_table_mut() {
        for key in &remove_keys {
            context.remove(key);
        }
    }

    fs::write(&aibox_toml_path, doc.to_string())
        .with_context(|| format!("Failed to write {}", aibox_toml_path.display()))?;

    output::ok(&format!(
        "Removed {} old processkit key(s) from aibox.toml [context]: {}",
        remove_keys.len(),
        remove_keys.join(", ")
    ));

    Ok(())
}

/// Build `settings.toml` content for the `id-management` skill.
fn build_id_management_settings(
    id_format: Option<&str>,
    id_slug: Option<bool>,
    date: &str,
) -> String {
    let format_line = id_format
        .map(|f| format!("format = {:?}  # word | uuid", f))
        .unwrap_or_else(|| "# format = \"word\"  # word | uuid".to_string());
    let slug_line = id_slug
        .map(|s| format!("slug   = {}", s))
        .unwrap_or_else(|| "# slug = false".to_string());

    format!(
        "# processkit — id-management settings\n\
         # Migrated from aibox.toml [context] by aibox sync on {date}\n\
         \n\
         [ids]\n\
         {format_line}\n\
         {slug_line}\n"
    )
}

/// Build `settings.toml` content for the `index-management` skill.
/// Serialises `directories`, `sharding`, and `index` sub-tables from the
/// old `[context]` table into the new flat format.
fn build_index_management_settings(
    context: &toml::map::Map<String, toml::Value>,
    date: &str,
) -> Result<String> {
    let mut out = format!(
        "# processkit — index-management settings\n\
         # Migrated from aibox.toml [context] by aibox sync on {date}\n"
    );

    for section in &["directories", "index", "sharding"] {
        let Some(value) = context.get(*section) else {
            continue;
        };
        // Wrap the value under the section key and serialize.
        let mut wrapper = toml::map::Map::new();
        wrapper.insert((*section).to_string(), value.clone());
        let serialized = toml::to_string(&toml::Value::Table(wrapper))
            .with_context(|| format!("Failed to serialize context.{} to TOML", section))?;
        out.push('\n');
        out.push_str(&serialized);
    }

    Ok(out)
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
            addons: None,
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

        // File now has a datetime prefix — find it by scanning the directory.
        let migrations_dir = tmp.path().join("context/migrations");
        let suffix = format!("0.0.1-to-{}.md", current);
        let entries: Vec<_> = fs::read_dir(&migrations_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let migration_file = entries
            .iter()
            .find(|e| e.file_name().to_string_lossy().ends_with(&suffix))
            .expect("migration doc should be created");

        let content = fs::read_to_string(migration_file.path()).unwrap();
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

        // Use a datetime-prefixed filename matching what the generator produces.
        let filename = format!("20260101_0000_0.0.1-to-{}.md", current);
        let filepath = migrations_dir.join(&filename);
        let existing_content = "# User-edited migration doc\nStatus: in-progress\n";
        fs::write(&filepath, existing_content).unwrap();

        write_sample_lock(tmp.path(), "0.0.1");

        check_and_generate_migration_in(tmp.path()).unwrap();

        // The pre-existing file must not be overwritten (only one file in dir).
        let entries: Vec<_> = fs::read_dir(&migrations_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        // There may now be a second file with the real datetime slug — the old
        // file should be unchanged regardless.
        let old_content = fs::read_to_string(&filepath).unwrap();
        assert_eq!(
            old_content, existing_content,
            "pre-existing migration doc should not be overwritten"
        );
        // Either only the original file exists (if datetime slug matched), or a
        // second file was created — both are acceptable since the guard only
        // checks the exact filename.
        let _ = entries;
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
        let doc = format_migration_doc("0.7.0", "0.8.0", "2026-03-23", None, None);

        // Safety header
        assert!(doc.contains("SAFETY: Do not execute any actions automatically."));
        assert!(doc.contains("Discuss each item with the project owner before proceeding."));
        assert!(doc.contains("Do not modify aibox.toml without explicit user confirmation."));
        assert!(doc.contains("aibox` commands run on the HOST, outside the container"));

        // Status and CLI header
        assert!(doc.contains("**Status:** pending"));
        assert!(doc.contains("**aibox CLI:** v0.7.0"));
        assert!(doc.contains("**processkit:** not configured"));

        // Action items — host section
        assert!(doc.contains("### Host actions (owner runs these outside the container)"));
        assert!(doc.contains("- [ ] Owner: verify `aibox sync` was run for v0.8.0"));

        // Action items — agent verification
        assert!(doc.contains("### Agent verification (you can do these now)"));
        assert!(doc.contains("- [ ] Read `/workspace/aibox.lock`"));
        assert!(doc.contains("- [ ] Verify `/workspace/AGENTS.md` exists"));
        assert!(doc.contains("- [ ] Verify `/workspace/context/skills/skill-finder/`"));
        assert!(doc.contains("- [ ] Check `/workspace/context/migrations/pending/`"));
        assert!(doc.contains("- [ ] Mark this migration as completed"));

        // processkit state section
        assert!(doc.contains("## processkit State"));
        assert!(doc.contains("processkit is not yet configured in this project"));

        // Verification summary
        assert!(doc.contains("## Verification Summary"));

        // Rollback section
        assert!(doc.contains("## Rollback"));
        assert!(doc.contains("git checkout HEAD~1 -- aibox.lock context/ .devcontainer/"));

        // Other required sections
        assert!(doc.contains("## Breaking Changes"));
        assert!(doc.contains("## Known Issues"));
    }

    #[test]
    fn test_format_migration_doc_versions_and_date() {
        let doc = format_migration_doc("1.2.3", "2.0.0", "2026-01-15", None, None);

        assert!(doc.contains("# Migration: v1.2.3 \u{2192} v2.0.0"));
        assert!(doc.contains("**Generated:** 2026-01-15"));
        assert!(doc.contains("**aibox CLI:** v1.2.3 \u{2192} v2.0.0"));
        assert!(doc.contains("from v1.2.3 to v2.0.0"));
    }

    #[test]
    fn test_format_migration_doc_with_processkit() {
        let pk = crate::lock::ProcessKitLockSection {
            source: "https://github.com/projectious-work/processkit.git".to_string(),
            version: "v0.8.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: None,
            release_asset_sha256: None,
            installed_at: "2026-04-01T00:00:00Z".to_string(),
        };
        let doc = format_migration_doc("0.17.4", "0.17.5", "2026-04-10", Some(&pk), None);

        assert!(doc.contains("**processkit:** v0.8.0"));
        assert!(doc.contains("processkit is pinned to `v0.8.0`"));
        assert!(doc.contains("context/migrations/pending/"));
    }

    #[test]
    fn test_intermediate_versions_basic() {
        let v = intermediate_versions("0.17.3", "0.17.5");
        assert_eq!(v, vec!["0.17.4", "0.17.5"]);
    }

    #[test]
    fn test_intermediate_versions_single_step() {
        let v = intermediate_versions("0.17.4", "0.17.5");
        assert_eq!(v, vec!["0.17.5"]);
    }

    #[test]
    fn test_intermediate_versions_same() {
        let v = intermediate_versions("0.17.5", "0.17.5");
        assert_eq!(v, vec!["0.17.5"]);
    }

    #[test]
    fn test_intermediate_versions_cross_minor() {
        // Cross-minor: just return the target.
        let v = intermediate_versions("0.16.9", "0.17.5");
        assert_eq!(v, vec!["0.17.5"]);
    }

    #[test]
    fn test_intermediate_versions_bad_input() {
        let v = intermediate_versions("bad", "0.17.5");
        assert_eq!(v, vec!["0.17.5"]);
    }

    #[test]
    fn test_known_migration_0_17_4_to_0_17_5() {
        let doc = format_migration_doc("0.17.4", "0.17.5", "2026-04-10", None, None);
        assert!(doc.contains("processkit v0.8.0 restructured its `src/` directory"));
        assert!(doc.contains("No `aibox.toml` changes required"));
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
        assert!(
            updated.contains("# [customization] — color theme, shell prompt, and zellij layout")
        );
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

    // -- processkit context settings migration --------------------------------

    fn write_aibox_toml(dir: &std::path::Path, context_extra: &str) {
        let content = format!(
            r#"[aibox]
version = "0.17.3"
[container]
name = "test"
[context]
schema_version = "1.0.0"
packages = ["managed"]
{context_extra}
"#
        );
        fs::write(dir.join("aibox.toml"), content).unwrap();
    }

    #[test]
    fn context_migration_noop_when_no_old_keys() {
        let tmp = TempDir::new().unwrap();
        write_aibox_toml(tmp.path(), "");
        let before = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();
        migrate_processkit_context_settings(tmp.path()).unwrap();
        let after = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();
        assert_eq!(before, after, "file should be unchanged when no old keys");
    }

    #[test]
    fn context_migration_noop_when_no_aibox_toml() {
        let tmp = TempDir::new().unwrap();
        // No aibox.toml — should be a silent no-op.
        migrate_processkit_context_settings(tmp.path()).unwrap();
    }

    #[test]
    fn context_migration_removes_id_keys_from_aibox_toml() {
        let tmp = TempDir::new().unwrap();
        write_aibox_toml(tmp.path(), "id_format = \"word\"\nid_slug = false\n");

        // Create the skill directory (but not settings.toml).
        let skill_config = tmp.path().join("context/skills/id-management/config");
        fs::create_dir_all(&skill_config).unwrap();

        migrate_processkit_context_settings(tmp.path()).unwrap();

        let after = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();
        assert!(!after.contains("id_format"), "id_format should be removed");
        assert!(!after.contains("id_slug"), "id_slug should be removed");
        assert!(
            after.contains("schema_version"),
            "schema_version should remain"
        );
        assert!(after.contains("packages"), "packages should remain");

        // settings.toml should have been written.
        let settings = fs::read_to_string(skill_config.join("settings.toml")).unwrap();
        assert!(settings.contains("[ids]"));
        assert!(settings.contains("format = \"word\""));
        assert!(settings.contains("slug   = false"));
    }

    #[test]
    fn context_migration_skips_write_if_settings_toml_exists() {
        let tmp = TempDir::new().unwrap();
        write_aibox_toml(tmp.path(), "id_format = \"uuid\"\n");

        let skill_config = tmp.path().join("context/skills/id-management/config");
        fs::create_dir_all(&skill_config).unwrap();
        let existing = "# already set up by agent\n[ids]\nformat = \"word\"\n";
        fs::write(skill_config.join("settings.toml"), existing).unwrap();

        migrate_processkit_context_settings(tmp.path()).unwrap();

        // Old key removed from aibox.toml.
        let after = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();
        assert!(!after.contains("id_format"));

        // settings.toml NOT overwritten.
        let settings = fs::read_to_string(skill_config.join("settings.toml")).unwrap();
        assert_eq!(
            settings, existing,
            "existing settings.toml should not be overwritten"
        );
    }

    #[test]
    fn context_migration_handles_directories_sub_table() {
        let tmp = TempDir::new().unwrap();
        let extra = "[context.directories]\nWorkItem = \"workitems\"\nLogEntry = \"logs\"\n";
        // Write as raw file since write_aibox_toml uses format! which won't nest tables cleanly.
        let content = format!(
            "[aibox]\nversion = \"0.17.3\"\n[container]\nname = \"t\"\n\
             [context]\nschema_version = \"1.0.0\"\npackages = [\"managed\"]\n\n{extra}"
        );
        fs::write(tmp.path().join("aibox.toml"), content).unwrap();

        let skill_config = tmp.path().join("context/skills/index-management/config");
        fs::create_dir_all(&skill_config).unwrap();

        migrate_processkit_context_settings(tmp.path()).unwrap();

        let after = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();
        assert!(
            !after.contains("[context.directories]"),
            "directories sub-table should be removed"
        );

        let settings = fs::read_to_string(skill_config.join("settings.toml")).unwrap();
        assert!(settings.contains("[directories]"));
        assert!(settings.contains("WorkItem"));
    }

    #[test]
    fn context_migration_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        write_aibox_toml(tmp.path(), "id_format = \"word\"\n");

        let skill_config = tmp.path().join("context/skills/id-management/config");
        fs::create_dir_all(&skill_config).unwrap();

        migrate_processkit_context_settings(tmp.path()).unwrap();
        let after_first = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();

        migrate_processkit_context_settings(tmp.path()).unwrap();
        let after_second = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();

        assert_eq!(after_first, after_second, "second run must be a no-op");
    }

    #[test]
    fn context_migration_removed_aibox_toml_remains_valid() {
        let tmp = TempDir::new().unwrap();
        write_aibox_toml(tmp.path(), "id_format = \"word\"\nid_slug = true\n");

        let skill_config = tmp.path().join("context/skills/id-management/config");
        fs::create_dir_all(&skill_config).unwrap();

        migrate_processkit_context_settings(tmp.path()).unwrap();

        let after = fs::read_to_string(tmp.path().join("aibox.toml")).unwrap();
        crate::config::AiboxConfig::from_str(&after)
            .expect("aibox.toml must remain valid after migration");
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
