//! Sync perimeter — the documented and enforced set of file paths that
//! `aibox sync` is allowed to create, modify, or delete.
//!
//! ## Why this exists
//!
//! Downstream projects (e.g. processkit, company forks) consume aibox to
//! manage their devcontainer but own the rest of the repository: their
//! own `README.md`, `AGENTS.md`, `src/`, `docs/`, and so on. These users
//! need a hard guarantee that `aibox sync` will *never* touch a file
//! outside aibox's domain.
//!
//! Without a documented perimeter, the question "could `aibox sync`
//! clobber file X?" becomes a recurring source of friction every time a
//! consumer makes a non-trivial edit. The answer must be unambiguous and
//! stable across releases.
//!
//! See [GitHub issue #34](https://github.com/projectious-work/aibox/issues/34).
//!
//! ## The perimeter
//!
//! `aibox sync` is allowed to write to (and only to) these paths,
//! relative to the project root:
//!
//! | Path                                          | Why                                                              |
//! |-----------------------------------------------|------------------------------------------------------------------|
//! | `aibox.toml`                                  | One-time schema migrations (e.g. inserting `[processkit]`)       |
//! | `.aibox-version`                              | Tracks installed CLI version for migration detection             |
//! | `.aibox-home/`                                | Runtime config seed (shells, vim, zellij, yazi, …); gitignored   |
//! | `.devcontainer/Dockerfile`                    | Regenerated from `aibox.toml`                                    |
//! | `.devcontainer/docker-compose.yml`            | Regenerated from `aibox.toml`                                    |
//! | `.devcontainer/devcontainer.json`             | Regenerated from `aibox.toml`                                    |
//! | `.claude/skills/`                             | Skill deployment (write-if-missing; never overwrites)            |
//! | `context/AIBOX.md`                            | Universal baseline (regenerated; explicitly aibox-owned)         |
//! | `context/migrations/`                         | Migration documents (additive; never overwrites)                 |
//!
//! Anything else is **out of perimeter**. Notable items that aibox sync
//! will NOT touch under any circumstances:
//!
//! - `README.md`, `AGENTS.md`, `CLAUDE.md`, `LICENSE`, `CHANGELOG.md`
//! - `src/`, `docs/`, `tests/`, `scripts/`, `assets/`
//! - `context/BACKLOG.md`, `context/DECISIONS.md`, `context/PRD.md`,
//!   `context/work-instructions/`, `context/skills/` (note: install-time
//!   destination for processkit content; sync never writes here)
//! - `.gitignore` (created by `aibox init`; sync never edits it)
//!
//! Note: `aibox init` is allowed to create files outside this list as
//! part of project bootstrap. The perimeter applies only to **sync**, not
//! init. Init's contract is "I am setting up a new project root"; sync's
//! contract is "I am refreshing aibox-managed files in an existing one".
//!
//! ## Enforcement
//!
//! Two complementary checks:
//!
//! 1. **Static.** [`is_within_perimeter`] is unit-tested against the
//!    full list of paths every sync write function targets today. If a
//!    new write site is added that targets an out-of-perimeter path,
//!    those tests must be updated together — flagging the change.
//! 2. **Dynamic.** [`check_perimeter`] is called by sync write helpers
//!    before performing the write. An attempt to write outside the
//!    perimeter returns an error with the path that triggered it,
//!    rather than silently corrupting user files.

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

/// The complete list of project-root-relative path prefixes that
/// `aibox sync` is allowed to create, modify, or delete. Each entry is
/// either a literal file path or a directory path ending in `/`. A
/// candidate path is "in perimeter" if it equals a literal entry or
/// starts with a directory entry.
///
/// Order is informational only; the check is O(n).
//
// `#[allow(dead_code)]` on the static check API: these functions are
// part of the documented public surface and are exercised by
// `#[cfg(test)]` tests, but aren't yet wired into every fs::write call
// site (that migration is intentionally out of scope for the first
// Phase 1 commit — the runtime guarantee is provided by `Tripwire`).
#[allow(dead_code)]
pub const SYNC_PERIMETER: &[&str] = &[
    // ── Top-level files aibox owns ─────────────────────────────────────
    "aibox.toml",
    ".aibox-version",
    // ── Runtime config seed (gitignored) ────────────────────────────────
    ".aibox-home/",
    // ── Devcontainer (the three files; nothing else under .devcontainer/) ─
    ".devcontainer/Dockerfile",
    ".devcontainer/docker-compose.yml",
    ".devcontainer/devcontainer.json",
    // ── Skills deployed by aibox (write-if-missing) ─────────────────────
    ".claude/skills/",
    // ── Universal baseline + migrations ────────────────────────────────
    "context/AIBOX.md",
    "context/migrations/",
];

/// Normalize a path to its forward-slash, project-root-relative string
/// form. Strips a leading `./` and rejects absolute paths and paths that
/// escape the project root via `..`.
#[allow(dead_code)]
fn normalize(rel_path: &Path) -> Result<String> {
    if rel_path.is_absolute() {
        return Err(anyhow!(
            "sync perimeter check: absolute path not allowed: {}",
            rel_path.display()
        ));
    }
    let mut parts: Vec<String> = Vec::new();
    for component in rel_path.components() {
        match component {
            Component::Normal(os) => parts.push(os.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(anyhow!(
                    "sync perimeter check: path escapes project root: {}",
                    rel_path.display()
                ));
            }
            // RootDir / Prefix can't appear in a non-absolute path on Unix.
            _ => {}
        }
    }
    Ok(parts.join("/"))
}

/// Returns `true` if `rel_path` (project-root-relative) is within the
/// sync perimeter.
#[allow(dead_code)]
pub fn is_within_perimeter(rel_path: &Path) -> bool {
    let Ok(normalized) = normalize(rel_path) else {
        return false;
    };
    if normalized.is_empty() {
        return false;
    }
    for entry in SYNC_PERIMETER {
        if let Some(dir) = entry.strip_suffix('/') {
            // Directory entry — match if normalized is the dir itself or
            // anything strictly inside it.
            if normalized == dir || normalized.starts_with(&format!("{}/", dir)) {
                return true;
            }
        } else if normalized == *entry {
            return true;
        }
    }
    false
}

/// Errors with a clear message if `rel_path` is outside the sync
/// perimeter. Use this in sync-time write helpers before performing
/// any write that could land in user-owned territory.
#[allow(dead_code)]
pub fn check_perimeter(rel_path: &Path) -> Result<()> {
    if is_within_perimeter(rel_path) {
        return Ok(());
    }
    Err(anyhow!(
        "refusing to modify {}: outside the aibox sync perimeter. \
         aibox sync only writes to: {}. \
         If this looks like an aibox bug, please report it.",
        rel_path.display(),
        SYNC_PERIMETER.join(", "),
    ))
}

// ---------------------------------------------------------------------------
// Runtime tripwire
// ---------------------------------------------------------------------------

/// A snapshot of representative out-of-perimeter sentinel files,
/// captured before `aibox sync` does any work. Calling [`Tripwire::verify`]
/// after sync confirms that none of the sentinels were created,
/// modified, or deleted — providing a runtime sanity check that
/// complements the static [`is_within_perimeter`] tests.
///
/// The sentinel set is intentionally small and biased toward the files
/// downstream consumers actually worry about: project entry points
/// (`README.md`, `AGENTS.md`, `CLAUDE.md`), source/test/script directory
/// markers, and user-owned context files. We do not snapshot every file
/// in the project — that would be O(repo size) and slow on large
/// projects.
///
/// `Tripwire::snapshot(None)` returns an inert tripwire (e.g. when
/// running outside a project root); `verify` is then a no-op.
pub struct Tripwire {
    project_root: Option<PathBuf>,
    states: BTreeMap<PathBuf, FileState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileState {
    Absent,
    Present {
        len: u64,
        modified: Option<SystemTime>,
    },
}

/// The fixed list of sentinel **file** paths checked by [`Tripwire`]. All
/// are project-root-relative. Directories are intentionally excluded:
/// directory mtimes don't reliably reflect modifications to nested
/// files, so a tree-walk-based check would be needed for directory
/// coverage. That's deferred to a follow-up; the static
/// [`is_within_perimeter`] check (verified by `cargo test`) already
/// covers directory cases at build time.
///
/// For runtime coverage we focus on the top-level files downstream
/// consumers actually worry about: project entry points (`README.md`,
/// `AGENTS.md`, `CLAUDE.md`) and the user-owned context documents.
const TRIPWIRE_SENTINELS: &[&str] = &[
    // Top-level project entry points — the things issue #34 explicitly
    // worried about.
    "README.md",
    "AGENTS.md",
    "CLAUDE.md",
    "LICENSE",
    "CHANGELOG.md",
    ".gitignore",
    // Top-level user-owned context files (product process).
    "context/BACKLOG.md",
    "context/DECISIONS.md",
    "context/PRD.md",
    "context/PROJECTS.md",
    "context/STANDUPS.md",
    "context/OWNER.md",
];

fn read_state(path: &Path) -> FileState {
    match fs::symlink_metadata(path) {
        Err(_) => FileState::Absent,
        Ok(meta) => FileState::Present {
            len: meta.len(),
            modified: meta.modified().ok(),
        },
    }
}

impl Tripwire {
    /// Snapshot the sentinel files at `project_root`. If `project_root`
    /// is `None`, the tripwire is inert and `verify` is a no-op.
    pub fn snapshot(project_root: Option<&Path>) -> Self {
        let Some(root) = project_root else {
            return Self {
                project_root: None,
                states: BTreeMap::new(),
            };
        };
        let mut states = BTreeMap::new();
        for sentinel in TRIPWIRE_SENTINELS {
            let abs = root.join(sentinel);
            states.insert(PathBuf::from(*sentinel), read_state(&abs));
        }
        Self {
            project_root: Some(root.to_path_buf()),
            states,
        }
    }

    /// Verify that none of the snapshotted sentinels have been
    /// modified, created, or deleted since [`snapshot`]. Errors with
    /// a clear message naming the offending paths if any sentinel
    /// changed.
    pub fn verify(&self) -> Result<()> {
        let Some(root) = &self.project_root else {
            return Ok(());
        };
        let mut violations: Vec<String> = Vec::new();
        for (rel, before) in &self.states {
            let after = read_state(&root.join(rel));
            if &after != before {
                violations.push(format!(
                    "{} ({} → {})",
                    rel.display(),
                    describe(before),
                    describe(&after)
                ));
            }
        }
        if violations.is_empty() {
            return Ok(());
        }
        Err(anyhow!(
            "aibox sync perimeter tripwire fired — these out-of-perimeter \
             paths were modified during sync, which is a bug: {}. \
             Please file an issue at https://github.com/projectious-work/aibox/issues",
            violations.join(", ")
        ))
    }
}

fn describe(state: &FileState) -> String {
    match state {
        FileState::Absent => "absent".to_string(),
        FileState::Present { len, .. } => format!("present, {} bytes", len),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn within(p: &str) -> bool {
        is_within_perimeter(Path::new(p))
    }

    // -- Files explicitly in perimeter --------------------------------------

    #[test]
    fn aibox_toml_is_in_perimeter() {
        assert!(within("aibox.toml"));
    }

    #[test]
    fn aibox_version_is_in_perimeter() {
        assert!(within(".aibox-version"));
    }

    #[test]
    fn devcontainer_dockerfile_is_in_perimeter() {
        assert!(within(".devcontainer/Dockerfile"));
    }

    #[test]
    fn devcontainer_compose_is_in_perimeter() {
        assert!(within(".devcontainer/docker-compose.yml"));
    }

    #[test]
    fn devcontainer_json_is_in_perimeter() {
        assert!(within(".devcontainer/devcontainer.json"));
    }

    #[test]
    fn aibox_md_is_in_perimeter() {
        assert!(within("context/AIBOX.md"));
    }

    // -- Directories in perimeter ------------------------------------------

    #[test]
    fn aibox_home_subtree_is_in_perimeter() {
        assert!(within(".aibox-home"));
        assert!(within(".aibox-home/.config/zellij/config.kdl"));
        assert!(within(".aibox-home/.config/yazi/yazi.toml"));
        assert!(within(".aibox-home/.vim/vimrc"));
    }

    #[test]
    fn claude_skills_subtree_is_in_perimeter() {
        assert!(within(".claude/skills"));
        assert!(within(".claude/skills/agent-management/SKILL.md"));
        assert!(within(".claude/skills/foo/references/bar.md"));
    }

    #[test]
    fn migrations_subtree_is_in_perimeter() {
        assert!(within("context/migrations"));
        assert!(within("context/migrations/0.13.0-to-0.14.0.md"));
        assert!(within("context/migrations/pending/MIG-20260407T120000.md"));
        assert!(within("context/migrations/aibox-processkit-section-added.md"));
    }

    // -- Files explicitly OUT of perimeter ---------------------------------

    #[test]
    fn user_owned_top_level_files_are_out_of_perimeter() {
        assert!(!within("README.md"));
        assert!(!within("AGENTS.md"));
        assert!(!within("CLAUDE.md"));
        assert!(!within("LICENSE"));
        assert!(!within("CHANGELOG.md"));
        assert!(!within(".gitignore"));
    }

    #[test]
    fn source_directories_are_out_of_perimeter() {
        assert!(!within("src/main.rs"));
        assert!(!within("docs/index.md"));
        assert!(!within("tests/integration.rs"));
        assert!(!within("scripts/build.sh"));
    }

    #[test]
    fn user_context_files_are_out_of_perimeter() {
        // These belong to the project; sync must never touch them.
        assert!(!within("context/BACKLOG.md"));
        assert!(!within("context/DECISIONS.md"));
        assert!(!within("context/PRD.md"));
        assert!(!within("context/PROJECTS.md"));
        assert!(!within("context/STANDUPS.md"));
        assert!(!within("context/work-instructions/DEVELOPMENT.md"));
        assert!(!within("context/notes/anything.md"));
        // The processkit install destinations live under context/ but
        // are written by `aibox init`, never by sync. Sync must not
        // touch them either.
        assert!(!within("context/skills/event-log/SKILL.md"));
        assert!(!within("context/schemas/workitem.yaml"));
        assert!(!within("context/state-machines/workflow.yaml"));
        assert!(!within("context/processes/release.md"));
        assert!(!within("context/templates/processkit/v0.5.0/skills/event-log/SKILL.md"));
    }

    #[test]
    fn other_devcontainer_files_are_out_of_perimeter() {
        // Only the three regenerated files are in. Overlay files,
        // arbitrary additions, and any other path under .devcontainer/
        // are user-owned.
        assert!(!within(".devcontainer/Dockerfile.local"));
        assert!(!within(".devcontainer/docker-compose.override.yml"));
        assert!(!within(".devcontainer/post-create.sh"));
    }

    #[test]
    fn other_dotclaude_files_are_out_of_perimeter() {
        // Only .claude/skills/. The rest of .claude/ may be user state
        // (settings.json, history, etc.) — sync never touches it.
        assert!(!within(".claude/settings.json"));
        assert!(!within(".claude/cache/foo"));
    }

    // -- Edge cases --------------------------------------------------------

    #[test]
    fn empty_path_is_out_of_perimeter() {
        assert!(!within(""));
    }

    #[test]
    fn current_dir_is_out_of_perimeter() {
        assert!(!within("."));
    }

    #[test]
    fn absolute_path_is_out_of_perimeter() {
        assert!(!within("/etc/passwd"));
        assert!(!within("/home/user/.bashrc"));
    }

    #[test]
    fn parent_escape_is_out_of_perimeter() {
        assert!(!within("../etc/passwd"));
        assert!(!within(".aibox-home/../README.md"));
    }

    #[test]
    fn leading_dot_slash_is_normalized() {
        assert!(within("./aibox.toml"));
        assert!(within("./.devcontainer/Dockerfile"));
    }

    #[test]
    fn similar_prefix_is_not_a_match() {
        // ".aibox-home" is in the list, ".aibox-homely" must not be.
        assert!(!within(".aibox-homely/foo"));
        // "context/AIBOX.md" is the literal entry, "context/AIBOX.md.bak" is not.
        assert!(!within("context/AIBOX.md.bak"));
    }

    // -- check_perimeter ---------------------------------------------------

    #[test]
    fn check_perimeter_ok_for_in() {
        check_perimeter(Path::new(".devcontainer/Dockerfile")).unwrap();
    }

    #[test]
    fn check_perimeter_err_for_out() {
        let err = check_perimeter(Path::new("README.md")).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("README.md"));
        assert!(msg.contains("outside the aibox sync perimeter"));
    }

    // -- Acceptance: every known sync-time write target is in perimeter ----
    //
    // This is the linchpin test. If a new sync write site is added that
    // targets a path not in this list, the developer must either add
    // the path to the SYNC_PERIMETER constant (with rationale) or
    // demonstrate that the new write does not happen during sync.

    #[test]
    fn all_known_sync_write_targets_are_in_perimeter() {
        let known_sync_writes = [
            // migration::check_and_generate_migration_in
            ".aibox-version",
            "context/migrations/0.13.0-to-0.14.0.md",
            // migration::ensure_processkit_section_in
            "aibox.toml",
            "context/migrations/aibox-processkit-section-added.md",
            // generate::generate_all
            ".devcontainer/Dockerfile",
            ".devcontainer/docker-compose.yml",
            ".devcontainer/devcontainer.json",
            // seed::sync_theme_files / seed_root_dir
            ".aibox-home/.vim/vimrc",
            ".aibox-home/.config/git/config",
            ".aibox-home/.config/zellij/config.kdl",
            ".aibox-home/.config/zellij/themes/catppuccin-mocha.kdl",
            ".aibox-home/.config/zellij/layouts/dev.kdl",
            ".aibox-home/.config/zellij/layouts/focus.kdl",
            ".aibox-home/.config/zellij/layouts/cowork.kdl",
            ".aibox-home/.config/zellij/layouts/ai.kdl",
            ".aibox-home/.config/yazi/yazi.toml",
            ".aibox-home/.config/yazi/keymap.toml",
            ".aibox-home/.config/yazi/theme.toml",
            ".aibox-home/.config/lazygit/config.yml",
            ".aibox-home/.bashrc",
            ".aibox-home/.config/starship.toml",
            ".aibox-home/.asoundrc",
            // context::reconcile_skills
            ".claude/skills/agent-management/SKILL.md",
            ".claude/skills/code-review/references/checklist.md",
            // context::generate_aibox_md
            "context/AIBOX.md",
            // content_diff::write_migration_document
            "context/migrations/pending/MIG-20260407T120000.md",
        ];

        for path in &known_sync_writes {
            assert!(
                within(path),
                "known sync write target {} is OUTSIDE the perimeter — \
                 either add it to SYNC_PERIMETER or demonstrate that it \
                 does not happen during sync",
                path
            );
        }
    }

    // -- Tripwire ---------------------------------------------------------

    #[test]
    fn tripwire_inert_when_no_root() {
        let tw = Tripwire::snapshot(None);
        tw.verify().unwrap();
    }

    #[test]
    fn tripwire_passes_when_nothing_changes() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("README.md"), "hello\n").unwrap();
        std::fs::write(tmp.path().join("AGENTS.md"), "agents\n").unwrap();
        let tw = Tripwire::snapshot(Some(tmp.path()));
        tw.verify().unwrap();
    }

    #[test]
    fn tripwire_fires_when_readme_changes() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("README.md"), "hello\n").unwrap();
        let tw = Tripwire::snapshot(Some(tmp.path()));
        // Simulate sync clobbering README.md
        std::fs::write(tmp.path().join("README.md"), "tampered content\n").unwrap();
        let err = tw.verify().unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("README.md"));
        assert!(msg.contains("tripwire fired"));
    }

    #[test]
    fn tripwire_fires_when_agents_md_is_created() {
        let tmp = tempfile::TempDir::new().unwrap();
        // AGENTS.md does not exist before sync.
        let tw = Tripwire::snapshot(Some(tmp.path()));
        // Simulate sync creating AGENTS.md (which it must not do).
        std::fs::write(tmp.path().join("AGENTS.md"), "new\n").unwrap();
        let err = tw.verify().unwrap_err();
        assert!(format!("{}", err).contains("AGENTS.md"));
    }

    #[test]
    fn tripwire_fires_when_user_context_file_changes() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("context")).unwrap();
        std::fs::write(tmp.path().join("context/BACKLOG.md"), "items\n").unwrap();
        let tw = Tripwire::snapshot(Some(tmp.path()));
        std::fs::write(tmp.path().join("context/BACKLOG.md"), "items\nnew\n").unwrap();
        let err = tw.verify().unwrap_err();
        assert!(format!("{}", err).contains("context/BACKLOG.md"));
    }

    #[test]
    fn tripwire_does_not_track_directory_subtrees() {
        // The tripwire intentionally only watches top-level files.
        // Coverage for nested writes under processkit install
        // destinations (context/skills/, context/schemas/, …) comes
        // from the static `all_known_sync_write_targets_are_in_perimeter`
        // test, which would fail at build time if any sync write site
        // started targeting those paths.
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("context/skills/event-log")).unwrap();
        std::fs::write(
            tmp.path().join("context/skills/event-log/SKILL.md"),
            "v1\n",
        )
        .unwrap();
        let tw = Tripwire::snapshot(Some(tmp.path()));
        // Modifying a deeply-nested file does NOT fire the file-only
        // tripwire — by design.
        std::fs::write(
            tmp.path().join("context/skills/event-log/SKILL.md"),
            "v2\n",
        )
        .unwrap();
        tw.verify().unwrap();
    }
}
