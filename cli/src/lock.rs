//! aibox lock file + small helpers used by the full-templates 3-way diff.
//!
//! ## What this module owns
//!
//! - [`AiboxLock`] — pinned `(source, version, commit)` for the project's
//!   consumed processkit. Top-level `<project_root>/aibox.lock`, git-tracked.
//! - [`sha256_of_file`] — computes the hex digest of a file in 64 KiB chunks.
//!   Used on the fly by the 3-way diff, which compares template-vs-cache-vs-live
//!   without persisting any SHAs.
//! - [`group_for_path`] — computes a logical group name for a
//!   `src_path`-relative file, so the 3-way diff can enforce the
//!   "auto-update by group, never by individual file" rule.
//!
//! ## What this module does NOT own (anymore)
//!
//! Earlier drafts of the consumption logic stored a `processkit.manifest`
//! file mapping every installed file to its SHA. That design was dropped in
//! favour of **full upstream reference templates** under
//! `context/templates/processkit/<version>/`: the templates dir itself is
//! the "as-installed" reference, SHAs are computed on the fly by the diff,
//! and nothing needs to be persisted beyond the lock.
//!
//! See `cli/src/content_install.rs` for the cache-to-project install
//! mapping, and `cli/src/content_init.rs` / `cli/src/content_diff.rs`
//! for the consumers of this module.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::processkit_vocab::{self as pk, PROVENANCE_FILENAME};

// ---------------------------------------------------------------------------
// Lock file (v0.17.0+ sectioned shape)
// ---------------------------------------------------------------------------
//
// `<project_root>/aibox.lock` is now a TOML file with two sections:
//
//     [aibox]                          # which CLI version touched the project last
//     cli_version = "0.17.0"
//     synced_at   = "2026-04-08T16:42:00Z"
//
//     [processkit]                     # what processkit version is installed
//     source                = "https://github.com/projectious-work/processkit.git"
//     version               = "v0.6.0"
//     src_path              = "src"
//     branch                = "main"           # optional
//     resolved_commit       = "abc123def456"   # optional
//     release_asset_sha256  = "deadbeef..."    # optional
//     installed_at          = "2026-04-08T16:30:00Z"
//
// Generalized from the v0.16.x flat shape (no sections, processkit fields
// at top level) to make space for future content sources (community packs,
// company forks). The legacy flat shape is auto-migrated on first read —
// see `migrate_legacy_lock`.

/// `<project_root>/aibox.lock` contents.
///
/// Generalized in v0.17.0 (DEC-037) to absorb the legacy `.aibox-version`
/// file. The `[aibox]` section tracks the CLI version that performed
/// the last install/sync; the `[processkit]` section tracks what
/// processkit content was installed. Future content sources will land as
/// additional top-level sections in the same file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiboxLock {
    pub aibox: AiboxLockSection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processkit: Option<ProcessKitLockSection>,
}

/// `[aibox]` section of `aibox.lock`. Tracks the CLI version that last
/// touched the project — what `.aibox-version` used to record in v0.16.x.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiboxLockSection {
    /// Aibox CLI version (semver) that performed the last install/sync.
    pub cli_version: String,
    /// ISO 8601 UTC timestamp of the last sync.
    pub synced_at: String,
}

/// `[processkit]` section of `aibox.lock`. Pinned processkit
/// `(source, version, sha256)` for the project's consumed processkit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessKitLockSection {
    pub source: String,
    pub version: String,
    pub src_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_commit: Option<String>,
    /// SHA256 hex digest of the release-asset tarball, when the release-
    /// asset fetch strategy was used and a sibling `.sha256` file was
    /// available for verification. Bit-exact reproducibility marker:
    /// re-fetching the same `(source, version)` must yield a tarball with
    /// the same SHA256.
    ///
    /// `None` for fetches that took the git-tarball or git-clone path
    /// (those use `resolved_commit` for reproducibility instead).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_asset_sha256: Option<String>,
    /// ISO 8601 UTC timestamp of the install (e.g. "2026-04-06T12:34:56Z").
    pub installed_at: String,
}

// ---------------------------------------------------------------------------
// Legacy v0.16.x flat lock shape — for one-time migration
// ---------------------------------------------------------------------------

/// The v0.16.x flat shape, kept around just long enough to migrate
/// existing project files. After migration this struct is never written
/// again — only `AiboxLock` (sectioned) is.
#[derive(Debug, Clone, Deserialize)]
struct LegacyFlatLock {
    source: String,
    version: String,
    src_path: String,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    resolved_commit: Option<String>,
    #[serde(default)]
    release_asset_sha256: Option<String>,
    installed_at: String,
}

// ---------------------------------------------------------------------------
// Standard file locations
// ---------------------------------------------------------------------------

/// `<project_root>/aibox.lock` — top level, git-tracked, Cargo-style.
pub fn lock_path(project_root: &Path) -> PathBuf {
    project_root.join("aibox.lock")
}

// ---------------------------------------------------------------------------
// Lock read / write
// ---------------------------------------------------------------------------

/// Read the lock file. Returns `Ok(None)` if the file does not exist.
///
/// **Backwards-compat:** if the file exists in the v0.16.x flat shape
/// (no `[aibox]` or `[processkit]` sections, fields at top level), this
/// function transparently upgrades it in memory to the new sectioned
/// shape. The CLI version recorded in the upgraded `[aibox]` section is
/// pulled from the legacy `.aibox-version` file (sibling to the lock).
/// If `.aibox-version` is also missing, the cli_version field is set to
/// the empty string and the migration helper at the call site
/// (`migration::migrate_legacy_lock_files`) takes care of writing back
/// the upgraded form and deleting the orphan.
pub fn read_lock(project_root: &Path) -> Result<Option<AiboxLock>> {
    let path = lock_path(project_root);
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    // Try new sectioned shape first.
    if let Ok(parsed) = toml::from_str::<AiboxLock>(&body) {
        return Ok(Some(parsed));
    }

    // Fall back to legacy v0.16.x flat shape.
    let legacy: LegacyFlatLock = toml::from_str(&body)
        .with_context(|| format!("failed to parse {} as TOML (neither new nor legacy shape)", path.display()))?;

    // Read the sibling `.aibox-version` to recover the CLI version that
    // last touched this project. If absent (very rare — fresh init that
    // somehow didn't write it), fall back to "unknown" which the migration
    // helper at the call site will replace with the current binary's version.
    let aibox_version_path = project_root.join(".aibox-version");
    let cli_version = if aibox_version_path.exists() {
        fs::read_to_string(&aibox_version_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| String::new())
    } else {
        String::new()
    };

    Ok(Some(AiboxLock {
        aibox: AiboxLockSection {
            cli_version,
            // Reuse the processkit installed_at as a best-guess synced_at;
            // the next real sync will overwrite this with a fresh timestamp.
            synced_at: legacy.installed_at.clone(),
        },
        processkit: Some(ProcessKitLockSection {
            source: legacy.source,
            version: legacy.version,
            src_path: legacy.src_path,
            branch: legacy.branch,
            resolved_commit: legacy.resolved_commit,
            release_asset_sha256: legacy.release_asset_sha256,
            installed_at: legacy.installed_at,
        }),
    }))
}

/// Write the lock file, creating parent directories if needed (for a
/// top-level lock, that's a no-op, but the API stays tolerant).
pub fn write_lock(project_root: &Path, lock: &AiboxLock) -> Result<()> {
    let path = lock_path(project_root);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let body = toml::to_string_pretty(lock)
        .with_context(|| "failed to serialize AiboxLock to TOML".to_string())?;
    fs::write(&path, body).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

/// Compute the SHA256 hex digest of a file's contents. Reads the file
/// in 64 KiB chunks so very large files do not balloon memory.
pub fn sha256_of_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = fs::File::open(path)
        .with_context(|| format!("failed to open {} for hashing", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f
            .read(&mut buf)
            .with_context(|| format!("failed to read {} for hashing", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Small filesystem helpers (shared with the diff walker)
// ---------------------------------------------------------------------------

/// Returns true if a directory-entry name should be skipped during any
/// walk over a processkit source tree or a templates/cache snapshot.
///
/// Kept here so the init walker (A5) and the diff walker (A6) agree on
/// the same skip rules — an inconsistency between them would silently
/// corrupt the 3-way comparison.
pub fn should_skip_entry(name: &str) -> bool {
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

/// Convert a relative `Path` into its forward-slash string form. Used so
/// keys are stable across Windows and Unix hosts.
pub fn path_to_forward_slash(rel: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in rel.components() {
        if let Component::Normal(os) = c {
            parts.push(os.to_string_lossy().to_string());
        }
    }
    parts.join("/")
}

// ---------------------------------------------------------------------------
// Group heuristic
// ---------------------------------------------------------------------------

/// Compute a logical group name for a `src_path`-relative file path.
///
/// Groups drive the "auto-update by group, never by individual file" rule
/// in the 3-way diff: if any file in a group has been edited locally, the
/// whole group is held back for review.
///
/// Handles both the **v0.8.0+ GrandLily layout** (paths start with
/// `context/`) and the **v0.7.0 legacy layout** (bare top-level names).
/// The group name strings are identical across both layouts so that the
/// diff logic works consistently regardless of which processkit version
/// is installed.
///
/// Priority order:
/// 1. `PROVENANCE.toml` → `"PROVENANCE"`
/// 2. v0.8.0 `context/skills/_lib/...` → `"lib"`
/// 3. v0.8.0 `context/skills/<name>/...` → `"skills/<name>"`
/// 4. v0.8.0 `context/schemas/<X>...` → `"schemas/<X>"`
/// 5. v0.8.0 `context/state-machines/<X>...` → `"state-machines/<X>"`
/// 6. v0.8.0 `context/processes/<name>...` → `"processes/<name>"`
/// 7. v0.8.0 `AGENTS.md` (top-level) → `"AGENTS"`
/// 8. legacy `skills/<name>/...` → `"skills/<name>"`
/// 9. legacy `primitives/schemas/<X>...` → `"primitives/schemas/<X>"`
/// 10. legacy `primitives/state-machines/<X>...` → `"primitives/state-machines/<X>"`
/// 11. legacy `primitives/<other>` → `"primitives"`
/// 12. legacy `lib/...` → `"lib"`
/// 13. legacy `processes/<name>...` → `"processes/<name>"`
/// 14. anything else → immediate parent dir or `None`
pub fn group_for_path(rel_path: &Path) -> Option<String> {
    let parts: Vec<String> = rel_path
        .components()
        .filter_map(|c| match c {
            Component::Normal(os) => Some(os.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();

    if parts.is_empty() {
        return None;
    }

    // 1. PROVENANCE.toml at top
    if parts.len() == 1 && parts[0] == PROVENANCE_FILENAME {
        return Some("PROVENANCE".to_string());
    }

    // ── v0.8.0+ GrandLily layout ─────────────────────────────────────────────

    // 7. AGENTS.md at top-level (v0.8.0 puts it at the tarball root)
    if parts.len() == 1 && parts[0] == crate::processkit_vocab::AGENTS_FILENAME {
        return Some("AGENTS".to_string());
    }

    if parts[0] == pk::src::CONTEXT_DIR && parts.len() >= 2 {
        let sub = &parts[1];

        // 2. context/skills/_lib/...  → "lib"
        if sub == pk::src::SKILLS
            && parts.len() >= 3
            && parts[2] == pk::src::LIB_SEGMENT
        {
            return Some("lib".to_string());
        }

        // 3. context/skills/<name>/...  → "skills/<name>"
        if sub == pk::src::SKILLS && parts.len() >= 3 {
            return Some(format!("skills/{}", parts[2]));
        }

        // 4. context/schemas/<X>...  → "schemas/<X>"
        if sub == pk::src::SCHEMAS && parts.len() >= 3 {
            let leaf = strip_known_ext(&parts[2]);
            return Some(format!("schemas/{}", leaf));
        }

        // 5. context/state-machines/<X>...  → "state-machines/<X>"
        if sub == pk::src::STATE_MACHINES && parts.len() >= 3 {
            let leaf = strip_known_ext(&parts[2]);
            return Some(format!("state-machines/{}", leaf));
        }

        // 6. context/processes/<name>...  → "processes/<name>"
        if sub == pk::src::PROCESSES && parts.len() >= 3 {
            let leaf = strip_known_ext(&parts[2]);
            return Some(format!("processes/{}", leaf));
        }

        // context/<other> → parent path fallback
        return Some(parts[..parts.len() - 1].join("/"));
    }

    // ── v0.7.0 legacy layout ─────────────────────────────────────────────────

    // 8. skills/<name>/...
    if parts[0] == pk::src::LEGACY_SKILLS && parts.len() >= 2 {
        return Some(format!("skills/{}", parts[1]));
    }

    // 9, 10, 11. primitives/...
    if parts[0] == pk::src::LEGACY_PRIMITIVES {
        if parts.len() >= 3
            && (parts[1] == pk::src::LEGACY_SCHEMAS
                || parts[1] == pk::src::LEGACY_STATE_MACHINES)
        {
            let leaf = strip_known_ext(&parts[2]);
            return Some(format!("primitives/{}/{}", parts[1], leaf));
        }
        return Some("primitives".to_string());
    }

    // 12. lib/...
    if parts[0] == pk::src::LEGACY_LIB {
        return Some("lib".to_string());
    }

    // 13. processes/<name>(/...)
    if parts[0] == pk::src::LEGACY_PROCESSES && parts.len() >= 2 {
        let leaf = strip_known_ext(&parts[1]);
        return Some(format!("processes/{}", leaf));
    }

    // 14. fallback — immediate parent dir, or None for top-level loose files.
    if parts.len() >= 2 {
        return Some(parts[..parts.len() - 1].join("/"));
    }
    None
}

/// Strip a single trailing known extension from a file name.
fn strip_known_ext(name: &str) -> String {
    for ext in [".yaml", ".yml", ".toml", ".md", ".py"] {
        if let Some(stripped) = name.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    name.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_pk() -> ProcessKitLockSection {
        ProcessKitLockSection {
            source: crate::processkit_vocab::PROCESSKIT_GIT_SOURCE.to_string(),
            version: "v0.4.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: Some("deadbeefcafebabe".to_string()),
            release_asset_sha256: None,
            installed_at: "2026-04-06T12:00:00Z".to_string(),
        }
    }

    fn sample_lock() -> AiboxLock {
        AiboxLock {
            aibox: AiboxLockSection {
                cli_version: "0.16.5".to_string(),
                synced_at: "2026-04-06T12:00:00Z".to_string(),
            },
            processkit: Some(sample_pk()),
        }
    }

    #[test]
    fn lock_round_trip_with_release_asset_sha256() {
        let tmp = TempDir::new().unwrap();
        let mut pk = sample_pk();
        pk.resolved_commit = None;
        pk.release_asset_sha256 = Some(
            "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz567abc890def123".to_string(),
        );
        let lock = AiboxLock {
            aibox: AiboxLockSection {
                cli_version: "0.16.5".to_string(),
                synced_at: "2026-04-06T12:00:00Z".to_string(),
            },
            processkit: Some(pk),
        };
        write_lock(tmp.path(), &lock).unwrap();
        let back = read_lock(tmp.path()).unwrap().unwrap();
        assert_eq!(back, lock);
        let back_pk = back.processkit.as_ref().unwrap();
        assert!(back_pk.release_asset_sha256.is_some());
        assert!(back_pk.resolved_commit.is_none());
    }

    // -- Lock round trip ---------------------------------------------------

    #[test]
    fn lock_round_trip() {
        let tmp = TempDir::new().unwrap();
        let lock = sample_lock();
        write_lock(tmp.path(), &lock).unwrap();
        let back = read_lock(tmp.path()).unwrap().unwrap();
        assert_eq!(back, lock);
    }

    #[test]
    fn lock_returns_none_if_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(read_lock(tmp.path()).unwrap().is_none());
    }

    #[test]
    fn lock_path_is_top_level_aibox_lock() {
        let tmp = TempDir::new().unwrap();
        write_lock(tmp.path(), &sample_lock()).unwrap();
        assert!(tmp.path().join("aibox.lock").exists());
        // Explicitly NOT under context/.aibox/ anymore.
        assert!(!tmp.path().join("context/.aibox/processkit.lock").exists());
        assert!(!tmp.path().join("context/.aibox/aibox.lock").exists());
    }

    // -- Hashing ------------------------------------------------------------

    #[test]
    fn sha256_of_file_known_value() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("hello.txt");
        fs::write(&p, b"hello\n").unwrap();
        let sha = sha256_of_file(&p).unwrap();
        // sha256("hello\n")
        assert_eq!(
            sha,
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
    }

    #[test]
    fn sha256_of_file_handles_empty_file() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("empty");
        fs::write(&p, b"").unwrap();
        let sha = sha256_of_file(&p).unwrap();
        // sha256("")
        assert_eq!(
            sha,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // -- Skip rules ---------------------------------------------------------

    #[test]
    fn should_skip_entry_matches_expected_names() {
        assert!(should_skip_entry(".git"));
        assert!(should_skip_entry("__pycache__"));
        assert!(should_skip_entry(".fetch-complete"));
        assert!(should_skip_entry(".DS_Store"));
        assert!(should_skip_entry("foo.pyc"));
        assert!(!should_skip_entry("SKILL.md"));
        assert!(!should_skip_entry("workitem.yaml"));
        assert!(!should_skip_entry("event-log"));
    }

    #[test]
    fn path_to_forward_slash_handles_nested() {
        assert_eq!(
            path_to_forward_slash(Path::new("skills/event-log/SKILL.md")),
            "skills/event-log/SKILL.md"
        );
        assert_eq!(
            path_to_forward_slash(Path::new("a")),
            "a"
        );
    }

    // -- Group heuristic — shared --------------------------------------------

    #[test]
    fn group_provenance() {
        assert_eq!(
            group_for_path(Path::new("PROVENANCE.toml")),
            Some("PROVENANCE".to_string())
        );
    }

    #[test]
    fn group_unknown_top_level_file_returns_none() {
        assert_eq!(group_for_path(Path::new("CHANGELOG.md")), None);
    }

    // -- Group heuristic — v0.8.0+ GrandLily layout -------------------------

    #[test]
    fn v8_group_agents_md() {
        assert_eq!(
            group_for_path(Path::new("AGENTS.md")),
            Some("AGENTS".to_string())
        );
    }

    #[test]
    fn v8_group_skill_subfile() {
        assert_eq!(
            group_for_path(Path::new("context/skills/event-log/SKILL.md")),
            Some("skills/event-log".to_string())
        );
        assert_eq!(
            group_for_path(Path::new("context/skills/event-log/mcp/server.py")),
            Some("skills/event-log".to_string())
        );
    }

    #[test]
    fn v8_group_lib() {
        assert_eq!(
            group_for_path(Path::new("context/skills/_lib/processkit/entity.py")),
            Some("lib".to_string())
        );
    }

    #[test]
    fn v8_group_schema() {
        assert_eq!(
            group_for_path(Path::new("context/schemas/workitem.yaml")),
            Some("schemas/workitem".to_string())
        );
    }

    #[test]
    fn v8_group_state_machine() {
        assert_eq!(
            group_for_path(Path::new("context/state-machines/migration.yaml")),
            Some("state-machines/migration".to_string())
        );
    }

    #[test]
    fn v8_group_process() {
        assert_eq!(
            group_for_path(Path::new("context/processes/release.md")),
            Some("processes/release".to_string())
        );
        assert_eq!(
            group_for_path(Path::new("context/processes/release/steps.yaml")),
            Some("processes/release".to_string())
        );
    }

    // -- Group heuristic — v0.7.0 legacy layout ------------------------------

    #[test]
    fn legacy_group_skill_subfile() {
        assert_eq!(
            group_for_path(Path::new("skills/event-log/SKILL.md")),
            Some("skills/event-log".to_string())
        );
        assert_eq!(
            group_for_path(Path::new("skills/event-log/templates/entry.yaml")),
            Some("skills/event-log".to_string())
        );
        assert_eq!(
            group_for_path(Path::new("skills/event-log/mcp/server.py")),
            Some("skills/event-log".to_string())
        );
    }

    #[test]
    fn legacy_group_primitive_schema() {
        assert_eq!(
            group_for_path(Path::new("primitives/schemas/workitem.yaml")),
            Some("primitives/schemas/workitem".to_string())
        );
    }

    #[test]
    fn legacy_group_primitive_state_machine() {
        assert_eq!(
            group_for_path(Path::new("primitives/state-machines/migration.yaml")),
            Some("primitives/state-machines/migration".to_string())
        );
    }

    #[test]
    fn legacy_group_lib() {
        assert_eq!(
            group_for_path(Path::new("lib/processkit/entity.py")),
            Some("lib".to_string())
        );
    }

    #[test]
    fn legacy_group_process_subfile() {
        assert_eq!(
            group_for_path(Path::new("processes/release.md")),
            Some("processes/release".to_string())
        );
        assert_eq!(
            group_for_path(Path::new("processes/release/steps.yaml")),
            Some("processes/release".to_string())
        );
    }

    #[test]
    fn legacy_group_primitive_format_doc() {
        assert_eq!(
            group_for_path(Path::new("primitives/FORMAT.md")),
            Some("primitives".to_string())
        );
    }
}
