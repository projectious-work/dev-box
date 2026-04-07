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
//! ## Install layout
//!
//! Cache file path (relative to `<cache>/<src_path>/`) → project install path
//! (relative to project root).
//!
//! | Cache path                          | Project install path                       |
//! |-------------------------------------|--------------------------------------------|
//! | `skills/<name>/...`                 | `context/skills/<name>/...`                |
//! | `lib/processkit/...`                | `context/skills/_lib/processkit/...`       |
//! | `primitives/schemas/<f>.yaml`       | `context/schemas/<f>.yaml`                 |
//! | `primitives/state-machines/<f>.yaml`| `context/state-machines/<f>.yaml`          |
//! | `processes/<f>.md`                  | `context/processes/<f>.md`                 |
//! | `primitives/FORMAT.md`              | skipped (internal reference)               |
//! | `skills/FORMAT.md`                  | skipped (internal reference)               |
//! | `PROVENANCE.toml` (top of src/)     | skipped (aibox reads from cache)           |
//! | `INDEX.md` (any level)              | skipped (processkit-internal docs)         |
//! | `packages/...`                      | skipped (consumed by init logic)           |
//! | anything unrecognized               | skipped                                    |
//!
//! ## Why every path is under `context/`
//!
//! `context/skills/`, `context/schemas/`, `context/state-machines/`, and
//! `context/processes/` are **visible, editable, top-level** locations. No
//! hidden directories. Users and agents can navigate to them via any file
//! browser or `ls context/`. Following Strawman D, consumers edit installed
//! files in place — there is no separate override location. The 3-way diff
//! at `aibox sync` time uses `context/templates/processkit/<version>/` as
//! the immutable upstream reference (written by `aibox init`), and computes
//! SHAs on the fly to classify user edits vs upstream changes.
//!
//! The shared lib lands at `context/skills/_lib/processkit/` because MCP
//! servers' `_find_lib()` boilerplate walks up from
//! `<server>/mcp/server.py` looking for `_lib/processkit/`. With the server
//! at `context/skills/<name>/mcp/server.py`, walk-up finds
//! `context/skills/_lib/processkit/`.
//!
//! ## What is NOT installed
//!
//! - `primitives/FORMAT.md` and `skills/FORMAT.md`: internal reference docs
//!   that agents don't need at runtime. The entity file format is
//!   self-evident from the entity files themselves plus the installed JSON
//!   schemas.
//! - `PROVENANCE.toml`: aibox sync reads it directly from the fetched
//!   cache. No project-side copy needed.
//! - `INDEX.md` files at every level: processkit-internal documentation.
//! - `packages/*.yaml`: consumed by init-time skill selection logic, not
//!   installed into the project.

use std::path::{Path, PathBuf};

/// What to do with a single file from the processkit cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallAction {
    /// Install at this project-root-relative path.
    Install(PathBuf),
    /// Skip this file (processkit-internal, not user-facing).
    Skip,
}

/// Map a cache file (path relative to `<cache>/<src_path>/`) to its install
/// action. Pure function — no I/O.
///
/// The input is a relative path using forward slashes. Backslashes are
/// normalized for Windows-friendliness (though aibox does not target
/// Windows hosts at v0.14.x; the normalization is defensive).
pub fn install_action_for(rel_path: &Path) -> InstallAction {
    let s = rel_path.to_string_lossy().replace('\\', "/");
    if s.is_empty() || s == "." {
        return InstallAction::Skip;
    }

    let parts: Vec<&str> = s.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return InstallAction::Skip;
    }

    // INDEX.md files are processkit-internal docs at every level.
    if parts.last().copied() == Some("INDEX.md") {
        return InstallAction::Skip;
    }

    // FORMAT.md files (primitives/FORMAT.md, skills/FORMAT.md) are
    // processkit-internal reference docs and are not installed.
    if parts.last().copied() == Some("FORMAT.md") {
        return InstallAction::Skip;
    }

    // Top-level files. None are currently installed (PROVENANCE.toml is
    // skipped because aibox reads it from the cache directly).
    if parts.len() == 1 {
        return InstallAction::Skip;
    }

    match parts[0] {
        // skills/<name>/...  →  context/skills/<name>/...
        // Provider-neutral location. Any agent discovers skills via CLAUDE.md /
        // AGENTS.md pointing at context/skills/.
        "skills" if parts.len() >= 2 => {
            let mut p = PathBuf::from("context/skills");
            for part in &parts[1..] {
                p.push(part);
            }
            InstallAction::Install(p)
        }

        // lib/processkit/...  →  context/skills/_lib/processkit/...
        // Matches MCP server _find_lib() walk-up: from
        // context/skills/<name>/mcp/server.py it finds context/skills/_lib/.
        "lib" if parts.len() >= 2 => {
            let mut p = PathBuf::from("context/skills/_lib");
            for part in &parts[1..] {
                p.push(part);
            }
            InstallAction::Install(p)
        }

        // primitives/schemas/X.yaml       →  context/schemas/X.yaml
        // primitives/state-machines/X.yaml →  context/state-machines/X.yaml
        // primitives/<anything-else>      →  skipped (including FORMAT.md
        //                                    handled above and INDEX.md)
        "primitives" if parts.len() >= 3 => match parts[1] {
            "schemas" => {
                let mut p = PathBuf::from("context/schemas");
                for part in &parts[2..] {
                    p.push(part);
                }
                InstallAction::Install(p)
            }
            "state-machines" => {
                let mut p = PathBuf::from("context/state-machines");
                for part in &parts[2..] {
                    p.push(part);
                }
                InstallAction::Install(p)
            }
            _ => InstallAction::Skip,
        },

        // processes/<f>.md  →  context/processes/<f>.md
        // Top-level under context/, existing convention. May coexist with
        // user-authored processes — Strawman D says edit in place.
        "processes" if parts.len() >= 2 => {
            let mut p = PathBuf::from("context/processes");
            for part in &parts[1..] {
                p.push(part);
            }
            InstallAction::Install(p)
        }

        // packages/*  →  skipped (consumed at init time to select skills,
        // not installed into the project).
        "packages" => InstallAction::Skip,

        // Anything else is unknown — skip rather than guess.
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

    fn assert_skipped(input: &str) {
        assert_eq!(install(input), InstallAction::Skip);
    }

    #[test]
    fn skill_files_install_under_context_skills() {
        assert_installs_to(
            "skills/event-log/SKILL.md",
            "context/skills/event-log/SKILL.md",
        );
        assert_installs_to(
            "skills/workitem-management/templates/workitem.yaml",
            "context/skills/workitem-management/templates/workitem.yaml",
        );
        assert_installs_to(
            "skills/event-log/mcp/server.py",
            "context/skills/event-log/mcp/server.py",
        );
    }

    #[test]
    fn lib_installs_under_context_skills_lib() {
        // Matches MCP server _find_lib() walk-up: from
        // context/skills/<name>/mcp/server.py it finds context/skills/_lib/.
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
    fn primitive_schemas_install_under_context_schemas() {
        assert_installs_to(
            "primitives/schemas/workitem.yaml",
            "context/schemas/workitem.yaml",
        );
        assert_installs_to(
            "primitives/schemas/logentry.yaml",
            "context/schemas/logentry.yaml",
        );
    }

    #[test]
    fn primitive_state_machines_install_under_context_state_machines() {
        assert_installs_to(
            "primitives/state-machines/workitem.yaml",
            "context/state-machines/workitem.yaml",
        );
        assert_installs_to(
            "primitives/state-machines/migration.yaml",
            "context/state-machines/migration.yaml",
        );
    }

    #[test]
    fn processes_install_under_context_processes() {
        assert_installs_to(
            "processes/release.md",
            "context/processes/release.md",
        );
        assert_installs_to(
            "processes/code-review/template.yaml",
            "context/processes/code-review/template.yaml",
        );
    }

    #[test]
    fn primitive_format_doc_is_skipped() {
        // FORMAT.md is processkit-internal reference, not installed.
        assert_skipped("primitives/FORMAT.md");
    }

    #[test]
    fn skills_format_doc_is_skipped() {
        // skills/FORMAT.md is processkit-internal reference, not installed.
        assert_skipped("skills/FORMAT.md");
    }

    #[test]
    fn provenance_toml_is_skipped() {
        // aibox reads PROVENANCE.toml from the cache directly; no project-side
        // copy is installed.
        assert_skipped("PROVENANCE.toml");
    }

    #[test]
    fn index_md_at_every_level_is_skipped() {
        assert_skipped("INDEX.md");
        assert_skipped("primitives/INDEX.md");
        assert_skipped("primitives/schemas/INDEX.md");
        assert_skipped("primitives/state-machines/INDEX.md");
        assert_skipped("skills/INDEX.md");
        assert_skipped("packages/INDEX.md");
        assert_skipped("processes/INDEX.md");
    }

    #[test]
    fn packages_are_skipped() {
        assert_skipped("packages/minimal.yaml");
        assert_skipped("packages/managed.yaml");
        assert_skipped("packages/software.yaml");
    }

    #[test]
    fn primitive_subdirectories_other_than_schemas_and_state_machines_are_skipped() {
        // Defensive — if processkit grows a new primitives/<foo>/ directory,
        // the install mapping should skip it rather than guess.
        assert_skipped("primitives/examples/workitem-example.yaml");
    }

    #[test]
    fn unknown_top_level_is_skipped() {
        assert_skipped("CHANGELOG.md");
        assert_skipped("LICENSE");
        assert_skipped("random/file.md");
    }

    #[test]
    fn empty_path_is_skipped() {
        assert_skipped("");
        assert_skipped(".");
    }

    #[test]
    fn windows_backslashes_are_normalized() {
        assert_installs_to(
            "skills\\event-log\\SKILL.md",
            "context/skills/event-log/SKILL.md",
        );
    }
}
