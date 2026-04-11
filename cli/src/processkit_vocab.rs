//! Compile-time vocabulary that mirrors processkit's published contracts.
//!
//! These constants and types are derived from processkit's canonical definitions
//! (primarily `src/.processkit/FORMAT.md`) and should be updated here when processkit
//! publishes a new vocabulary change. For runtime information — which skills are
//! installed, exact frontmatter values — read from the live `context/` directory
//! or the templates mirror rather than consulting these constants.
//!
//! ## Update checklist (when processkit releases)
//!
//! 1. Bump `PROCESSKIT_DEFAULT_VERSION` to the new tag.
//! 2. Check `CATEGORY_ORDER` against `src/.processkit/FORMAT.md`; add/remove entries
//!    if the vocabulary changed.
//! 3. Run `cargo test` — the vocabulary tests below will catch order drift.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Version constant
// ---------------------------------------------------------------------------

/// Canonical Git source URL for the upstream processkit repository.
/// Used as the default in `[processkit].source` and referenced in test fixtures.
/// Update if the repository is ever moved.
pub const PROCESSKIT_GIT_SOURCE: &str = "https://github.com/projectious-work/processkit.git";

// ---------------------------------------------------------------------------
// Mandatory MCP server sets (processkit v0.11.1+)
// ---------------------------------------------------------------------------

/// Skills whose MCP servers are mandatory in every processkit install, regardless
/// of package tier. These skills back entity-layer invariants — schema validation,
/// state-machine enforcement, and index sync are tool-side guarantees that cannot
/// be replicated by hand-editing files. A minimal-tier install that omits any of
/// these breaks entity-layer correctness silently.
///
/// Values are skill *directory names* (e.g. `"index-management"`) as they appear
/// under `context/skills/` in the templates mirror. The MCP server name shipped
/// in each skill's `mcp/mcp-config.json` follows the `processkit-{skill}` convention.
///
/// Mirrors the mandatory set formalised in processkit v0.11.1. Update here when
/// processkit adds or removes a mandatory server.
pub const MANDATORY_MCP_SKILLS: &[&str] = &[
    "decision-record",
    "discussion-management",
    "event-log",
    "id-management",
    "index-management",
    "workitem-management",
];

/// Skills whose MCP servers are registered only when the matching tier package is
/// installed (present in `[context].packages`). Unlike the mandatory set, these
/// are not required for entity-layer correctness on every install — they extend
/// the entity layer for teams that opt into governance or advisory features.
///
/// Values are skill directory names under `context/skills/`.
pub const TIER_SPECIFIC_MCP_SKILLS: &[&str] = &[
    "actor-profile",
    "binding-management",
    "gate-management",
    "model-recommender",
    "role-management",
    "scope-management",
];

/// The processkit release recommended for new projects created by `aibox init`.
/// Updated here on each processkit release; `resolve_processkit_section` in
/// `container.rs` queries GitHub at init-time and falls back to "unset" — this
/// constant is the single source of truth for documentation, test fixtures, and
/// any code that needs a concrete default tag.
///
/// In production builds the live version is queried from the GitHub API; this
/// constant serves as the canonical reference for tests and documentation.
// Used in #[cfg(test)] blocks across multiple modules and in documentation.
#[allow(dead_code)]
pub const PROCESSKIT_DEFAULT_VERSION: &str = "v0.13.0";

// ---------------------------------------------------------------------------
// Processkit source-tree directory segments
// (the layout inside the processkit `src/` directory)
// ---------------------------------------------------------------------------

/// Path segments describing processkit's `src/` tree layout.
///
/// **v0.8.0+ (GrandLily) layout** — `src/` is now a literal mirror of the
/// consumer project root. Content lives under `src/context/`; catalog-only
/// files (packages, FORMAT.md) live under `src/.processkit/`.
///
/// **v0.7.0 legacy layout** — kept as `LEGACY_*` constants so that
/// `content_install` and `lock` can route and group both old and new tarballs
/// without a version check. The two prefix sets are disjoint (`context/` vs.
/// bare top-level names), so a single match expression handles both.
pub mod src {
    // ── v0.8.0+ segments ────────────────────────────────────────────────────
    /// Top-level directory in the v0.8.0 tarball (`src/context/`).
    pub const CONTEXT_DIR: &str = "context";
    /// Skills live at `context/skills/` (second segment).
    pub const SKILLS: &str = "skills";
    /// Schemas live at `context/schemas/` (second segment).
    pub const SCHEMAS: &str = "schemas";
    /// State machines live at `context/state-machines/` (second segment).
    pub const STATE_MACHINES: &str = "state-machines";
    /// Processes live at `context/processes/` (second segment).
    pub const PROCESSES: &str = "processes";
    /// Shared lib segment inside `context/skills/_lib/`.
    pub const LIB_SEGMENT: &str = "_lib";
    /// Hidden catalog directory — never installed (`src/.processkit/`).
    pub const DOTPROCESSKIT: &str = ".processkit";
    /// Package YAML files live at `.processkit/packages/`.
    pub const PACKAGES: &str = "packages";

    // ── v0.7.0 legacy segments (kept for backward-compat routing) ───────────
    /// Legacy: skills were at `src/skills/` (top-level, pre-v0.8.0).
    pub const LEGACY_SKILLS: &str = "skills";
    /// Legacy: shared lib was at `src/lib/` (top-level, pre-v0.8.0).
    pub const LEGACY_LIB: &str = "lib";
    /// Legacy: primitive schemas/state-machines were under `src/primitives/`.
    pub const LEGACY_PRIMITIVES: &str = "primitives";
    /// Legacy: schemas sub-segment inside `primitives/schemas/`.
    pub const LEGACY_SCHEMAS: &str = "schemas";
    /// Legacy: state-machine sub-segment inside `primitives/state-machines/`.
    pub const LEGACY_STATE_MACHINES: &str = "state-machines";
    /// Legacy: processes were at `src/processes/` (top-level, pre-v0.8.0).
    pub const LEGACY_PROCESSES: &str = "processes";
    /// Legacy: AGENTS.md was under `src/scaffolding/` (pre-v0.8.0).
    pub const LEGACY_SCAFFOLDING: &str = "scaffolding";
    /// Legacy: packages were at `src/packages/` (top-level, pre-v0.8.0).
    pub const LEGACY_PACKAGES: &str = "packages";
}

// ---------------------------------------------------------------------------
// Well-known filenames from the processkit contract
// ---------------------------------------------------------------------------

/// `SKILL.md` — the agent-readable instruction file inside every skill dir.
pub const SKILL_FILENAME: &str = "SKILL.md";
/// `PROVENANCE.toml` — shipping manifest written into every processkit release.
pub const PROVENANCE_FILENAME: &str = "PROVENANCE.toml";
/// `FORMAT.md` — processkit-internal reference docs (never installed).
pub const FORMAT_FILENAME: &str = "FORMAT.md";
/// `INDEX.md` — per-directory navigation documents (selectively installed).
pub const INDEX_FILENAME: &str = "INDEX.md";
/// `AGENTS.md` — the canonical agent entry point, installed from scaffolding.
pub const AGENTS_FILENAME: &str = "AGENTS.md";

// ---------------------------------------------------------------------------
// Live install path prefixes (project-root-relative)
// ---------------------------------------------------------------------------

/// Path segment holding the version-stamped upstream snapshot used by
/// `aibox sync`'s three-way diff. The full path is
/// `TEMPLATES_PROCESSKIT_DIR/<version>/`.
pub const TEMPLATES_PROCESSKIT_DIR: &str = "context/templates/processkit";

/// Live editable destination for installed skills.
pub const LIVE_SKILLS_DIR: &str = "context/skills";

/// Live editable destination for installed primitive schemas.
pub const LIVE_SCHEMAS_DIR: &str = "context/schemas";

/// Live editable destination for installed state machines.
pub const LIVE_STATE_MACHINES_DIR: &str = "context/state-machines";

/// Live editable destination for installed processes.
pub const LIVE_PROCESSES_DIR: &str = "context/processes";

/// Shared MCP lib directory (under the live skills tree, not the src tree).
pub const LIVE_LIB_DIR: &str = "context/skills/_lib";

// ---------------------------------------------------------------------------
// Templates mirror path helpers
// ---------------------------------------------------------------------------

/// Resolve the skills directory inside the templates mirror for a given
/// processkit version, handling both layout versions transparently.
///
/// - v0.8.0+: `<mirror>/<version>/context/skills/`
/// - v0.7.0 legacy: `<mirror>/<version>/skills/`
///
/// Returns the first path that exists on disk, trying v0.8.0 first.
/// Returns `None` if neither exists (processkit not yet installed).
pub fn mirror_skills_dir(
    project_root: &std::path::Path,
    version: &str,
) -> Option<std::path::PathBuf> {
    // v0.8.0+ path
    let new = project_root
        .join(TEMPLATES_PROCESSKIT_DIR)
        .join(version)
        .join(src::CONTEXT_DIR)
        .join(src::SKILLS);
    if new.is_dir() {
        return Some(new);
    }
    // v0.7.0 legacy path
    let legacy = project_root
        .join(TEMPLATES_PROCESSKIT_DIR)
        .join(version)
        .join(src::LEGACY_SKILLS);
    if legacy.is_dir() {
        return Some(legacy);
    }
    None
}

/// Resolve the processes directory inside the templates mirror for a given
/// processkit version, handling both layout versions transparently.
///
/// - v0.8.0+: `<mirror>/<version>/context/processes/`
/// - v0.7.0 legacy: `<mirror>/<version>/processes/`
pub fn mirror_processes_dir(
    project_root: &std::path::Path,
    version: &str,
) -> Option<std::path::PathBuf> {
    let new = project_root
        .join(TEMPLATES_PROCESSKIT_DIR)
        .join(version)
        .join(src::CONTEXT_DIR)
        .join(src::PROCESSES);
    if new.is_dir() {
        return Some(new);
    }
    let legacy = project_root
        .join(TEMPLATES_PROCESSKIT_DIR)
        .join(version)
        .join(src::LEGACY_PROCESSES);
    if legacy.is_dir() {
        return Some(legacy);
    }
    None
}

/// Resolve the packages directory inside the templates mirror for a given
/// processkit version, handling both layout versions transparently.
///
/// - v0.8.0+: `<mirror>/<version>/.processkit/packages/`
/// - v0.7.0 legacy: `<mirror>/<version>/packages/`
pub fn mirror_packages_dir(
    project_root: &std::path::Path,
    version: &str,
) -> Option<std::path::PathBuf> {
    let new = project_root
        .join(TEMPLATES_PROCESSKIT_DIR)
        .join(version)
        .join(src::DOTPROCESSKIT)
        .join(src::PACKAGES);
    if new.is_dir() {
        return Some(new);
    }
    let legacy = project_root
        .join(TEMPLATES_PROCESSKIT_DIR)
        .join(version)
        .join(src::LEGACY_PACKAGES);
    if legacy.is_dir() {
        return Some(legacy);
    }
    None
}

// ---------------------------------------------------------------------------
// Display constants
// ---------------------------------------------------------------------------

/// Maximum number of characters shown for a skill or process description
/// in `aibox kit` table output before the description is truncated with `…`.
pub const DESCRIPTION_DISPLAY_MAX: usize = 60;

// ---------------------------------------------------------------------------
// Category vocabulary
// ---------------------------------------------------------------------------

/// Canonical 14-value category vocabulary defined in processkit's
/// `src/skills/FORMAT.md`. Used by `aibox kit skill list` for display grouping.
///
/// **Order matters** — it controls the display sort order within `aibox kit`.
/// Update if processkit reorders or adds/removes categories.
pub const CATEGORY_ORDER: &[&str] = &[
    "process",
    "meta",
    "architecture",
    "language",
    "framework",
    "ai",
    "data",
    "infrastructure",
    "database",
    "api",
    "security",
    "observability",
    "design",
    "performance",
];

/// Return the display sort index for a category string.
/// Unknown / uncategorized values sort after all known categories.
pub fn category_sort_index(cat: &str) -> usize {
    CATEGORY_ORDER
        .iter()
        .position(|&c| c == cat)
        .unwrap_or(usize::MAX)
}

// ---------------------------------------------------------------------------
// Shared frontmatter types
// ---------------------------------------------------------------------------

/// Parsed YAML frontmatter from a processkit `SKILL.md` file.
///
/// Shared by `kit` (display) and `content_init` (core-skill enforcement).
/// Only fields that aibox consumes are declared; unknown fields are ignored
/// by serde.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SkillFrontmatter {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub metadata: Option<SkillMetadata>,
}

impl SkillFrontmatter {
    /// Convenience accessor — returns the `metadata.processkit` sub-struct, if present.
    pub fn processkit_meta(&self) -> Option<&SkillProcesskitMeta> {
        self.metadata.as_ref()?.processkit.as_ref()
    }

    /// Category string, falling back to `"uncategorized"` when absent.
    pub fn category(&self) -> &str {
        self.processkit_meta()
            .map(|m| m.category.as_str())
            .filter(|c| !c.is_empty())
            .unwrap_or("uncategorized")
    }

    /// Whether this skill carries `metadata.processkit.core: true`.
    pub fn is_core(&self) -> bool {
        self.processkit_meta().map(|m| m.core).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillMetadata {
    pub processkit: Option<SkillProcesskitMeta>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SkillProcesskitMeta {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub version: String,
    /// When `true`, aibox installs this skill regardless of `[skills].include`
    /// or `[skills].exclude` configuration. `aibox doctor` warns when a core
    /// skill appears in `[skills].exclude`. Proposed convention in processkit
    /// v0.6.0; see processkit/aibox#36.
    #[serde(default)]
    pub core: bool,
    /// User-invocable command names declared by this skill. Each entry names a
    /// `commands/<skill>-<workflow>.md` adapter file that ships alongside the
    /// skill. aibox copies these to `.claude/commands/` during sync so Claude
    /// Code can tab-complete them. Introduced in processkit v0.7.0; see
    /// projectious-work/aibox#37.
    ///
    /// The actual file discovery uses the filesystem (walking
    /// `context/skills/*/commands/*.md`) rather than this list, so the field
    /// is currently read-only metadata. Future uses: `aibox kit skill info`
    /// command listing, other-harness registration.
    #[serde(default)]
    #[allow(dead_code)]
    pub commands: Vec<String>,
}

// ---------------------------------------------------------------------------
// Frontmatter parser
// ---------------------------------------------------------------------------

/// Parse YAML frontmatter from a processkit `SKILL.md` (between the first two
/// `---` fences). Returns a zeroed `SkillFrontmatter` when no frontmatter is
/// present; serde errors are silently swallowed (unknown fields, type mismatches)
/// so that a new processkit field never breaks an older aibox binary.
pub fn parse_skill_frontmatter(path: &Path) -> Result<SkillFrontmatter> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let mut lines = content.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Ok(SkillFrontmatter::default());
    }

    let yaml_block: String = lines
        .take_while(|l| l.trim() != "---")
        .collect::<Vec<_>>()
        .join("\n");

    // Swallow parse errors — an unknown frontmatter shape should not crash
    // `aibox kit` or `aibox sync`. The skill will appear with empty fields.
    let fm: SkillFrontmatter = serde_yaml::from_str(&yaml_block).unwrap_or_default();
    Ok(fm)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_order_has_exactly_14_entries() {
        assert_eq!(CATEGORY_ORDER.len(), 14);
    }

    #[test]
    fn category_order_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for cat in CATEGORY_ORDER {
            assert!(seen.insert(*cat), "duplicate category: {}", cat);
        }
    }

    #[test]
    fn category_sort_index_known() {
        assert_eq!(category_sort_index("process"), 0);
        assert_eq!(category_sort_index("performance"), 13);
    }

    #[test]
    fn category_sort_index_unknown_sorts_last() {
        assert_eq!(category_sort_index("unknown"), usize::MAX);
        assert_eq!(category_sort_index(""), usize::MAX);
    }

    #[test]
    fn default_version_is_semver_with_v_prefix() {
        assert!(PROCESSKIT_DEFAULT_VERSION.starts_with('v'));
        let bare = PROCESSKIT_DEFAULT_VERSION.trim_start_matches('v');
        semver::Version::parse(bare).expect("PROCESSKIT_DEFAULT_VERSION must be valid semver");
    }

    #[test]
    fn frontmatter_is_core_false_by_default() {
        let fm = SkillFrontmatter::default();
        assert!(!fm.is_core());
    }

    #[test]
    fn parse_skill_frontmatter_reads_core_flag() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmp,
            "---\nname: skill-finder\nmetadata:\n  processkit:\n    core: true\n---\n"
        )
        .unwrap();
        let fm = parse_skill_frontmatter(tmp.path()).unwrap();
        assert!(fm.is_core());
        assert_eq!(fm.name, "skill-finder");
    }

    #[test]
    fn parse_skill_frontmatter_no_frontmatter_returns_default() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "# No frontmatter here").unwrap();
        let fm = parse_skill_frontmatter(tmp.path()).unwrap();
        assert!(!fm.is_core());
        assert!(fm.name.is_empty());
    }
}
