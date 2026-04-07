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

// ---------------------------------------------------------------------------
// Lock file
// ---------------------------------------------------------------------------

/// `<project_root>/aibox.lock` contents.
///
/// Named `aibox.lock` (generic) rather than `processkit.lock` so that once
/// aibox gains community-package consumption, additional sources can be
/// recorded in the same file without another name collision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiboxLock {
    pub source: String,
    pub version: String,
    pub src_path: String,
    pub branch: Option<String>,
    pub resolved_commit: Option<String>,
    /// ISO 8601 UTC timestamp of the install (e.g. "2026-04-06T12:34:56Z").
    pub installed_at: String,
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
pub fn read_lock(project_root: &Path) -> Result<Option<AiboxLock>> {
    let path = lock_path(project_root);
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: AiboxLock = toml::from_str(&body)
        .with_context(|| format!("failed to parse {} as TOML", path.display()))?;
    Ok(Some(parsed))
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
/// whole group is held back for review. Heuristic, in priority order:
///
/// 1. `PROVENANCE.toml` at the top → `"PROVENANCE"`
/// 2. `skills/<name>/...` → `"skills/<name>"` (everything under a skill dir)
/// 3. `primitives/schemas/<X>.yaml` → `"primitives/schemas/<X>"`
/// 4. `primitives/state-machines/<X>.yaml` → `"primitives/state-machines/<X>"`
/// 5. `primitives/<other>` → `"primitives"`
/// 6. `lib/...` → `"lib"`
/// 7. `processes/<name>/...` or `processes/<name>.md` → `"processes/<name>"`
/// 8. anything else → the immediate parent directory or `None`
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
    if parts.len() == 1 && parts[0] == "PROVENANCE.toml" {
        return Some("PROVENANCE".to_string());
    }

    // 2. skills/<name>/...
    if parts[0] == "skills" && parts.len() >= 2 {
        return Some(format!("skills/{}", parts[1]));
    }

    // 3, 4, 5. primitives/...
    if parts[0] == "primitives" {
        if parts.len() >= 3 && (parts[1] == "schemas" || parts[1] == "state-machines") {
            // Strip a trailing extension from the leaf so all files
            // describing the same primitive share a group.
            let leaf = strip_known_ext(&parts[2]);
            return Some(format!("primitives/{}/{}", parts[1], leaf));
        }
        return Some("primitives".to_string());
    }

    // 6. lib/...
    if parts[0] == "lib" {
        return Some("lib".to_string());
    }

    // 7. processes/<name>(/...)
    if parts[0] == "processes" && parts.len() >= 2 {
        let leaf = strip_known_ext(&parts[1]);
        return Some(format!("processes/{}", leaf));
    }

    // 8. fallback — immediate parent dir, or None for top-level loose files.
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

    fn sample_lock() -> AiboxLock {
        AiboxLock {
            source: "https://github.com/projectious-work/processkit.git".to_string(),
            version: "v0.4.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: Some("deadbeefcafebabe".to_string()),
            installed_at: "2026-04-06T12:00:00Z".to_string(),
        }
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

    // -- Group heuristic ----------------------------------------------------

    #[test]
    fn group_provenance() {
        assert_eq!(
            group_for_path(Path::new("PROVENANCE.toml")),
            Some("PROVENANCE".to_string())
        );
    }

    #[test]
    fn group_skill_subfile() {
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
    fn group_primitive_schema() {
        assert_eq!(
            group_for_path(Path::new("primitives/schemas/workitem.yaml")),
            Some("primitives/schemas/workitem".to_string())
        );
    }

    #[test]
    fn group_primitive_state_machine() {
        assert_eq!(
            group_for_path(Path::new("primitives/state-machines/migration.yaml")),
            Some("primitives/state-machines/migration".to_string())
        );
    }

    #[test]
    fn group_lib() {
        assert_eq!(
            group_for_path(Path::new("lib/processkit/entity.py")),
            Some("lib".to_string())
        );
    }

    #[test]
    fn group_process_subfile() {
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
    fn group_primitive_format_doc() {
        // primitives/FORMAT.md is not schemas or state-machines so it falls
        // under the general "primitives" group.
        assert_eq!(
            group_for_path(Path::new("primitives/FORMAT.md")),
            Some("primitives".to_string())
        );
    }

    #[test]
    fn group_unknown_top_level_file_returns_none() {
        assert_eq!(group_for_path(Path::new("CHANGELOG.md")), None);
    }
}
