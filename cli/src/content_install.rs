//! The mapping from processkit cache files to their installation paths
//! in a consuming project. **Single source of truth** used by:
//!
//! - `aibox init` (A5) — installs files via this mapping
//! - `aibox sync` (A6) — uses this mapping for the live-vs-template 3-way comparison
//! - `aibox migrate` (A7) — references files via this mapping when working
//!   through Migration documents
//!
//! ## Design principle — provider neutrality
//!
//! processkit content must be reachable by any MCP-capable agent, not just
//! Claude Code. That means no `.claude/` path in the install mapping. All
//! installed content lives under `context/`, which every provider can be
//! pointed at via its own config (usually via CLAUDE.md / AGENTS.md / guide
//! files at the project root).
//!
//! ## Install layout — v0.8.0+ (GrandLily)
//!
//! Since v0.8.0, processkit's `src/` directory is a **literal mirror of the
//! consumer project root**. The install path is therefore identical to the
//! cache path for all `context/` content; only a handful of special cases
//! need explicit handling.
//!
//! Cache path (relative to `<cache>/<src_path>/`) → project install path:
//!
//! | Cache path                        | Project install path              |
//! |-----------------------------------|-----------------------------------|
//! | `AGENTS.md`                       | `AGENTS.md` (rendered, root)      |
//! | `INDEX.md`                        | `context/INDEX.md`                |
//! | `context/skills/<name>/...`       | `context/skills/<name>/...`       |
//! | `context/skills/_lib/...`         | `context/skills/_lib/...`         |
//! | `context/schemas/<f>.yaml`        | `context/schemas/<f>.yaml`        |
//! | `context/state-machines/<f>.yaml` | `context/state-machines/<f>.yaml` |
//! | `context/processes/<f>.md`        | `context/processes/<f>.md`        |
//! | `.processkit/...`                 | skipped (catalog, not installed)  |
//! | `PROVENANCE.toml`                 | skipped (aibox reads from cache)  |
//! | `FORMAT.md` (anywhere)            | skipped (internal reference)      |
//! | anything else                     | skipped                           |
//!
//! ## Install layout — v0.7.0 and earlier (legacy)
//!
//! Older tarballs use bare top-level segment names. `install_action_for`
//! handles both layouts in one function — the prefix sets are disjoint
//! (`context/` vs. bare names) so no version check is needed. Legacy
//! projects pinned to v0.7.0 continue to install correctly when run
//! with a new aibox binary.
//!
//! | Cache path                          | Project install path                 |
//! |-------------------------------------|--------------------------------------|
//! | `INDEX.md`                          | `context/INDEX.md`                   |
//! | `skills/<name>/...`                 | `context/skills/<name>/...`          |
//! | `lib/processkit/...`                | `context/skills/_lib/processkit/...` |
//! | `primitives/schemas/<f>.yaml`       | `context/schemas/<f>.yaml`           |
//! | `primitives/state-machines/<f>.yaml`| `context/state-machines/<f>.yaml`    |
//! | `processes/<f>.md`                  | `context/processes/<f>.md`           |
//! | `scaffolding/AGENTS.md`             | `AGENTS.md` (rendered, root)         |
//! | `packages/...`, `scaffolding/*`,    | skipped                              |
//! |   `primitives/INDEX.md`, etc.       |                                      |
//!
//! ## Why INDEX.md installs (since v0.16.4 — BACK-116)
//!
//! processkit ships INDEX.md files as navigation documents at every
//! content level. They are tracked in `PROVENANCE.toml` and are part of
//! the shipping contract — agents browsing `context/` expect to find them.
//! v0.16.0 skipped them blanket; v0.16.4 routes each INDEX.md to its
//! parent directory's live destination.
//!
//! ## Why every path is under `context/`
//!
//! `context/skills/`, `context/schemas/`, `context/state-machines/`, and
//! `context/processes/` are **visible, editable, top-level** locations.
//! The 3-way diff at `aibox sync` time uses
//! `context/templates/processkit/<version>/` as the immutable upstream
//! reference, and computes SHAs on the fly.
//!
//! The shared lib lands at `context/skills/_lib/processkit/` because MCP
//! servers' `_find_lib()` boilerplate walks up from
//! `<server>/mcp/server.py` to find `_lib/processkit/`.
//!
//! ## What is NOT installed
//!
//! - `.processkit/FORMAT.md` (v0.8.0+) / `skills/FORMAT.md` (legacy):
//!   internal reference docs; the entity format is self-evident from the
//!   installed files and JSON schemas.
//! - `PROVENANCE.toml`: aibox sync reads it directly from the cache.
//! - `.processkit/packages/` (v0.8.0+) / `packages/` (legacy): consumed
//!   by init-time skill selection, not installed into the project.

use std::path::{Path, PathBuf};

use crate::processkit_vocab::{
    self as pk, AGENTS_FILENAME, FORMAT_FILENAME, INDEX_FILENAME,
    LIVE_LIB_DIR, LIVE_PROCESSES_DIR, LIVE_SCHEMAS_DIR, LIVE_SKILLS_DIR,
    LIVE_STATE_MACHINES_DIR,
};

/// What to do with a single file from the processkit cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallAction {
    /// Install at this project-root-relative path. The file is copied
    /// verbatim from the cache to the destination.
    Install(PathBuf),
    /// Install at this project-root-relative path, but FIRST run the
    /// content through `crate::context::render` with the Class A
    /// substitution vocabulary. Used for files that processkit ships
    /// with `{{PLACEHOLDER}}` markers (e.g. `scaffolding/AGENTS.md`).
    /// See DEC-032 for the vocabulary contract.
    InstallTemplated(PathBuf),
    /// Skip this file (processkit-internal, not user-facing).
    Skip,
}

/// Map a cache file (path relative to `<cache>/<src_path>/`) to its install
/// action. Pure function — no I/O.
///
/// Handles both the **v0.8.0+ GrandLily layout** (paths start with `context/`
/// or are bare top-level files) and the **v0.7.0 legacy layout** (bare
/// top-level segment names: `skills/`, `lib/`, `primitives/`, etc.).
/// The two prefix sets are disjoint, so no version check is needed.
///
/// The input is a relative path using forward slashes. Backslashes are
/// normalised for Windows-friendliness.
pub fn install_action_for(rel_path: &Path) -> InstallAction {
    let s = rel_path.to_string_lossy().replace('\\', "/");
    if s.is_empty() || s == "." {
        return InstallAction::Skip;
    }

    let parts: Vec<&str> = s.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return InstallAction::Skip;
    }

    // FORMAT.md anywhere → skip (internal reference in any layout version).
    if parts.last().copied() == Some(FORMAT_FILENAME) {
        return InstallAction::Skip;
    }

    // ── Top-level bare files (both layout versions) ──────────────────────────
    if parts.len() == 1 {
        return match parts[0] {
            // v0.8.0+: AGENTS.md is at the tarball root, rendered into project root.
            f if f == AGENTS_FILENAME => {
                InstallAction::InstallTemplated(PathBuf::from(AGENTS_FILENAME))
            }
            // Both: top-level INDEX.md → context/INDEX.md.
            f if f == INDEX_FILENAME => {
                InstallAction::Install(PathBuf::from("context").join(INDEX_FILENAME))
            }
            // PROVENANCE.toml: aibox reads from cache directly; skip live install.
            _ => InstallAction::Skip,
        };
    }

    // ── v0.8.0+ layout — first segment is "context" or ".processkit" ─────────

    // .processkit/... → catalog only, never installed.
    if parts[0] == pk::src::DOTPROCESSKIT {
        return InstallAction::Skip;
    }

    // context/... → install at the same path (src mirrors consumer root).
    if parts[0] == pk::src::CONTEXT_DIR && parts.len() >= 2 {
        // context/INDEX.md at any level installs verbatim (no remapping needed
        // since the path already starts with context/).
        let install_path: PathBuf = parts.iter().collect();
        return InstallAction::Install(install_path);
    }

    // ── v0.7.0 legacy layout — bare top-level segment names ─────────────────

    match parts[0] {
        // skills/<name>/...  →  context/skills/<name>/...
        s if s == pk::src::LEGACY_SKILLS && parts.len() >= 2 => {
            let mut p = PathBuf::from(LIVE_SKILLS_DIR);
            for part in &parts[1..] {
                p.push(part);
            }
            InstallAction::Install(p)
        }

        // lib/processkit/...  →  context/skills/_lib/processkit/...
        s if s == pk::src::LEGACY_LIB && parts.len() >= 2 => {
            let mut p = PathBuf::from(LIVE_LIB_DIR);
            for part in &parts[1..] {
                p.push(part);
            }
            InstallAction::Install(p)
        }

        // primitives/schemas/...       →  context/schemas/...
        // primitives/state-machines/...→  context/state-machines/...
        // primitives/<other>           →  skipped
        s if s == pk::src::LEGACY_PRIMITIVES && parts.len() >= 3 => {
            if parts[1] == pk::src::LEGACY_SCHEMAS {
                let mut p = PathBuf::from(LIVE_SCHEMAS_DIR);
                for part in &parts[2..] {
                    p.push(part);
                }
                InstallAction::Install(p)
            } else if parts[1] == pk::src::LEGACY_STATE_MACHINES {
                let mut p = PathBuf::from(LIVE_STATE_MACHINES_DIR);
                for part in &parts[2..] {
                    p.push(part);
                }
                InstallAction::Install(p)
            } else {
                InstallAction::Skip
            }
        }

        // processes/<f>.md  →  context/processes/<f>.md
        s if s == pk::src::LEGACY_PROCESSES && parts.len() >= 2 => {
            let mut p = PathBuf::from(LIVE_PROCESSES_DIR);
            for part in &parts[1..] {
                p.push(part);
            }
            InstallAction::Install(p)
        }

        // packages/*  →  skipped (declarative, read from templates mirror).
        s if s == pk::src::LEGACY_PACKAGES => InstallAction::Skip,

        // scaffolding/AGENTS.md  →  AGENTS.md (project root, rendered).
        // scaffolding/INDEX.md and any other scaffolding files → skipped.
        s if s == pk::src::LEGACY_SCAFFOLDING
            && parts.len() == 2
            && parts[1] == AGENTS_FILENAME =>
        {
            InstallAction::InstallTemplated(PathBuf::from(AGENTS_FILENAME))
        }
        s if s == pk::src::LEGACY_SCAFFOLDING => InstallAction::Skip,

        // Unknown → skip.
        _ => InstallAction::Skip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install(s: &str) -> InstallAction {
        install_action_for(Path::new(s))
    }

    fn assert_installs_to(input: &str, expected: &str) {
        match install(input) {
            InstallAction::Install(p) => assert_eq!(p, PathBuf::from(expected)),
            other => panic!("expected Install({}), got {:?}", expected, other),
        }
    }

    fn assert_installs_templated_to(input: &str, expected: &str) {
        match install(input) {
            InstallAction::InstallTemplated(p) => assert_eq!(p, PathBuf::from(expected)),
            other => panic!("expected InstallTemplated({}), got {:?}", expected, other),
        }
    }

    fn assert_skipped(input: &str) {
        assert_eq!(install(input), InstallAction::Skip);
    }

    // ── v0.8.0+ GrandLily layout ─────────────────────────────────────────────

    #[test]
    fn v8_agents_md_installs_templated_at_project_root() {
        assert_installs_templated_to("AGENTS.md", "AGENTS.md");
    }

    #[test]
    fn v8_top_level_index_md_installs_at_context_index_md() {
        assert_installs_to("INDEX.md", "context/INDEX.md");
    }

    #[test]
    fn v8_skill_files_install_verbatim() {
        assert_installs_to(
            "context/skills/event-log/SKILL.md",
            "context/skills/event-log/SKILL.md",
        );
        assert_installs_to(
            "context/skills/workitem-management/templates/workitem.yaml",
            "context/skills/workitem-management/templates/workitem.yaml",
        );
        assert_installs_to(
            "context/skills/event-log/mcp/server.py",
            "context/skills/event-log/mcp/server.py",
        );
    }

    #[test]
    fn v8_lib_installs_verbatim() {
        assert_installs_to(
            "context/skills/_lib/processkit/__init__.py",
            "context/skills/_lib/processkit/__init__.py",
        );
    }

    #[test]
    fn v8_schemas_install_verbatim() {
        assert_installs_to(
            "context/schemas/workitem.yaml",
            "context/schemas/workitem.yaml",
        );
    }

    #[test]
    fn v8_state_machines_install_verbatim() {
        assert_installs_to(
            "context/state-machines/workitem.yaml",
            "context/state-machines/workitem.yaml",
        );
    }

    #[test]
    fn v8_processes_install_verbatim() {
        assert_installs_to(
            "context/processes/release.md",
            "context/processes/release.md",
        );
    }

    #[test]
    fn v8_context_index_mds_install_verbatim() {
        assert_installs_to(
            "context/skills/INDEX.md",
            "context/skills/INDEX.md",
        );
        assert_installs_to(
            "context/schemas/INDEX.md",
            "context/schemas/INDEX.md",
        );
        assert_installs_to(
            "context/state-machines/INDEX.md",
            "context/state-machines/INDEX.md",
        );
        assert_installs_to(
            "context/processes/INDEX.md",
            "context/processes/INDEX.md",
        );
        assert_installs_to(
            "context/INDEX.md",
            "context/INDEX.md",
        );
    }

    #[test]
    fn v8_dotprocesskit_is_skipped() {
        assert_skipped(".processkit/packages/minimal.yaml");
        assert_skipped(".processkit/FORMAT.md");
        assert_skipped(".processkit/packages/INDEX.md");
    }

    #[test]
    fn v8_provenance_toml_is_skipped() {
        assert_skipped("PROVENANCE.toml");
    }

    #[test]
    fn v8_format_md_anywhere_is_skipped() {
        assert_skipped("context/skills/FORMAT.md");
        assert_skipped("FORMAT.md");
    }

    // ── v0.7.0 legacy layout ─────────────────────────────────────────────────

    #[test]
    fn legacy_skill_files_install_under_context_skills() {
        assert_installs_to(
            "skills/event-log/SKILL.md",
            "context/skills/event-log/SKILL.md",
        );
        assert_installs_to(
            "skills/workitem-management/templates/workitem.yaml",
            "context/skills/workitem-management/templates/workitem.yaml",
        );
    }

    #[test]
    fn legacy_lib_installs_under_context_skills_lib() {
        assert_installs_to(
            "lib/processkit/__init__.py",
            "context/skills/_lib/processkit/__init__.py",
        );
        assert_installs_to(
            "lib/processkit/entity.py",
            "context/skills/_lib/processkit/entity.py",
        );
    }

    #[test]
    fn legacy_primitive_schemas_install_under_context_schemas() {
        assert_installs_to(
            "primitives/schemas/workitem.yaml",
            "context/schemas/workitem.yaml",
        );
    }

    #[test]
    fn legacy_primitive_state_machines_install_under_context_state_machines() {
        assert_installs_to(
            "primitives/state-machines/workitem.yaml",
            "context/state-machines/workitem.yaml",
        );
    }

    #[test]
    fn legacy_processes_install_under_context_processes() {
        assert_installs_to("processes/release.md", "context/processes/release.md");
    }

    #[test]
    fn legacy_skills_index_md_installs_under_context_skills() {
        assert_installs_to("skills/INDEX.md", "context/skills/INDEX.md");
    }

    #[test]
    fn legacy_primitive_schemas_index_md_installs() {
        assert_installs_to("primitives/schemas/INDEX.md", "context/schemas/INDEX.md");
    }

    #[test]
    fn legacy_primitive_state_machines_index_md_installs() {
        assert_installs_to(
            "primitives/state-machines/INDEX.md",
            "context/state-machines/INDEX.md",
        );
    }

    #[test]
    fn legacy_primitives_top_level_index_md_is_skipped() {
        assert_skipped("primitives/INDEX.md");
    }

    #[test]
    fn legacy_packages_are_skipped() {
        assert_skipped("packages/minimal.yaml");
        assert_skipped("packages/INDEX.md");
    }

    #[test]
    fn legacy_scaffolding_agents_md_installs_templated() {
        assert_installs_templated_to("scaffolding/AGENTS.md", "AGENTS.md");
    }

    #[test]
    fn legacy_scaffolding_other_files_are_skipped() {
        assert_skipped("scaffolding/INDEX.md");
        assert_skipped("scaffolding/foo.md");
    }

    #[test]
    fn legacy_format_docs_are_skipped() {
        assert_skipped("primitives/FORMAT.md");
        assert_skipped("skills/FORMAT.md");
    }

    #[test]
    fn legacy_primitives_unknown_subdir_is_skipped() {
        assert_skipped("primitives/examples/workitem-example.yaml");
    }

    // ── Shared / defensive ───────────────────────────────────────────────────

    #[test]
    fn empty_path_is_skipped() {
        assert_skipped("");
        assert_skipped(".");
    }

    #[test]
    fn unknown_top_level_is_skipped() {
        assert_skipped("CHANGELOG.md");
        assert_skipped("LICENSE");
        assert_skipped("random/file.md");
    }

    #[test]
    fn windows_backslashes_are_normalized() {
        // Legacy path with backslashes
        assert_installs_to(
            "skills\\event-log\\SKILL.md",
            "context/skills/event-log/SKILL.md",
        );
        // v0.8.0 path with backslashes
        assert_installs_to(
            "context\\skills\\event-log\\SKILL.md",
            "context/skills/event-log/SKILL.md",
        );
    }
}
