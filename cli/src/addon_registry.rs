use std::collections::HashMap;

use crate::addon_loader;

// ---------------------------------------------------------------------------
// Data structures (unchanged — used throughout the codebase)
// ---------------------------------------------------------------------------

/// Internal recipe definition for an add-on.
/// `addon_version` tracks how *we* install the add-on, not the upstream
/// tool version — that lives in [`ToolDef::supported_versions`].
pub struct AddonDef {
    pub name: &'static str,
    pub addon_version: &'static str,
    pub tools: &'static [ToolDef],
}

/// A single tool inside an add-on with curated version choices.
pub struct ToolDef {
    pub name: &'static str,
    pub default_enabled: bool,
    /// Curated version strings the user can choose from.
    /// Empty slice means no version selection (e.g. clippy, texlive-core).
    pub supported_versions: &'static [&'static str],
    /// Default version string.  `""` when `supported_versions` is empty.
    pub default_version: &'static str,
}

/// Per-tool configuration coming from the parsed `aibox.toml`.
pub struct ToolConfig {
    pub enabled: bool,
    pub version: String,
}

// ---------------------------------------------------------------------------
// Lookup functions — delegate to addon_loader
// ---------------------------------------------------------------------------

/// Returns all loaded addon definitions.
/// Panics if addon_loader has not been initialized.
pub fn all_addons() -> Vec<AddonDef> {
    addon_loader::all_addons()
        .iter()
        .map(|a| {
            let tool_defs = a.to_tool_defs();
            AddonDef {
                name: addon_loader::leak_str(&a.name),
                addon_version: addon_loader::leak_str(&a.addon_version),
                tools: Box::leak(tool_defs.into_boxed_slice()),
            }
        })
        .collect()
}

/// Look up a single add-on by name.
pub fn get_addon(name: &str) -> Option<AddonDef> {
    addon_loader::get_addon(name).map(|a| {
        let tool_defs = a.to_tool_defs();
        AddonDef {
            name: addon_loader::leak_str(&a.name),
            addon_version: addon_loader::leak_str(&a.addon_version),
            tools: Box::leak(tool_defs.into_boxed_slice()),
        }
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check whether a tool is enabled in the user config, falling back to false
/// when the tool is absent from the map.
pub fn is_enabled(tools: &HashMap<String, ToolConfig>, name: &str) -> bool {
    tools.get(name).is_some_and(|t| t.enabled)
}

/// Retrieve the version string for a tool from user config, falling back to
/// the registry default when missing or empty.
pub fn version_or_default(
    tools: &HashMap<String, ToolConfig>,
    name: &str,
    default: &str,
) -> String {
    tools
        .get(name)
        .and_then(|t| {
            if t.version.is_empty() {
                None
            } else {
                Some(t.version.clone())
            }
        })
        .unwrap_or_else(|| default.to_string())
}

// ---------------------------------------------------------------------------
// Builder-stage and runtime-command generation — delegate to addon_loader
// ---------------------------------------------------------------------------

/// Returns a Dockerfile builder stage block for add-ons that need one.
pub fn generate_builder_stage(
    addon_name: &str,
    tools: &HashMap<String, ToolConfig>,
) -> Option<String> {
    let addon = addon_loader::get_addon(addon_name)?;
    addon_loader::render_builder(addon, tools)
        .ok()
        .flatten()
}

/// Returns Dockerfile `RUN` commands for the runtime stage of a given add-on.
pub fn generate_runtime_commands(
    addon_name: &str,
    tools: &HashMap<String, ToolConfig>,
) -> String {
    match addon_loader::get_addon(addon_name) {
        Some(addon) => addon_loader::render_runtime(addon, tools)
            .unwrap_or_default(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a tool config map from (name, enabled, version) tuples.
    fn tc(entries: &[(&str, bool, &str)]) -> HashMap<String, ToolConfig> {
        entries
            .iter()
            .map(|(n, e, v)| {
                (
                    n.to_string(),
                    ToolConfig {
                        enabled: *e,
                        version: v.to_string(),
                    },
                )
            })
            .collect()
    }

    /// Initialize the addon loader from the repo's addons/ directory for tests.
    fn ensure_loaded() {
        // Try to init; ignore if already initialized
        let addons_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("addons");
        let _ = addon_loader::init_from_dir(&addons_dir);
    }

    // ── Registry lookup ─────────────────────────────────────────────────

    #[test]
    fn all_addons_returns_all_entries() {
        ensure_loaded();
        let addons = all_addons();
        assert!(addons.len() >= 21, "expected at least 21 add-ons, got {}", addons.len());
    }

    #[test]
    fn get_addon_finds_known() {
        ensure_loaded();
        assert!(get_addon("python").is_some());
        assert!(get_addon("rust").is_some());
        assert!(get_addon("latex").is_some());
        assert!(get_addon("kubernetes").is_some());
        assert!(get_addon("cloud-aws").is_some());
        assert!(get_addon("docs-zensical").is_some());
    }

    #[test]
    fn get_addon_returns_none_for_unknown() {
        ensure_loaded();
        assert!(get_addon("doesnotexist").is_none());
    }

    // ── Version validation ──────────────────────────────────────────────

    #[test]
    fn python_tool_versions_are_curated() {
        ensure_loaded();
        let addon = get_addon("python").unwrap();
        let py = addon.tools.iter().find(|t| t.name == "python").unwrap();
        assert!(py.supported_versions.contains(&"3.13"));
        assert_eq!(py.default_version, "3.13");
    }

    #[test]
    fn rust_tool_versions_are_curated() {
        ensure_loaded();
        let addon = get_addon("rust").unwrap();
        let rustc = addon.tools.iter().find(|t| t.name == "rustc").unwrap();
        assert!(rustc.supported_versions.contains(&"1.87"));
        assert_eq!(rustc.default_version, "1.87");
    }

    #[test]
    fn versionless_tools_have_empty_defaults() {
        ensure_loaded();
        let addon = get_addon("rust").unwrap();
        let clippy = addon.tools.iter().find(|t| t.name == "clippy").unwrap();
        assert!(clippy.supported_versions.is_empty());
        assert_eq!(clippy.default_version, "");
    }

    // ── Builder-stage generation ────────────────────────────────────────

    #[test]
    fn rust_has_builder_stage() {
        ensure_loaded();
        let tools = tc(&[
            ("rustc", true, "1.87"),
            ("clippy", true, ""),
            ("rustfmt", true, ""),
        ]);
        let stage = generate_builder_stage("rust", &tools);
        assert!(stage.is_some());
        let stage = stage.unwrap();
        assert!(stage.contains("rust-builder"), "missing rust-builder in:\n{stage}");
        assert!(stage.contains("1.87"), "missing version 1.87 in:\n{stage}");
    }

    #[test]
    fn python_has_no_builder_stage() {
        ensure_loaded();
        let tools = tc(&[("python", true, "3.13"), ("uv", true, "0.7")]);
        assert!(generate_builder_stage("python", &tools).is_none());
    }

    // ── Runtime-command generation ──────────────────────────────────────

    #[test]
    fn python_runtime_contains_uv() {
        ensure_loaded();
        let tools = tc(&[("python", true, "3.13"), ("uv", true, "0.7")]);
        let cmds = generate_runtime_commands("python", &tools);
        assert!(cmds.contains("python3.13") || cmds.contains("python3"), "missing python in:\n{cmds}");
        assert!(cmds.contains("uv:0.7") || cmds.contains("uv"), "missing uv in:\n{cmds}");
    }

    #[test]
    fn unknown_addon_returns_empty_runtime() {
        ensure_loaded();
        let tools = HashMap::new();
        let cmds = generate_runtime_commands("nonexistent", &tools);
        assert!(cmds.is_empty());
    }

    #[test]
    fn rust_runtime_copies_from_builder() {
        ensure_loaded();
        let tools = tc(&[("rustc", true, "1.87")]);
        let cmds = generate_runtime_commands("rust", &tools);
        assert!(cmds.contains("COPY --from=rust-builder"), "missing COPY in:\n{cmds}");
    }

    // ── Default-enabled / default-disabled ──────────────────────────────

    #[test]
    fn default_enabled_flags_match_spec() {
        ensure_loaded();
        let python = get_addon("python").unwrap();
        let poetry = python.tools.iter().find(|t| t.name == "poetry").unwrap();
        assert!(!poetry.default_enabled, "poetry should be off by default");

        let node = get_addon("node").unwrap();
        let bun = node.tools.iter().find(|t| t.name == "bun").unwrap();
        assert!(!bun.default_enabled, "bun should be off by default");
    }

    // ── AI provider addons ──────────────────────────────────────────────

    #[test]
    fn ai_claude_addon_exists() {
        ensure_loaded();
        assert!(get_addon("ai-claude").is_some());
    }

    #[test]
    fn ai_claude_runtime_installs_claude() {
        ensure_loaded();
        let tools = tc(&[("claude", true, "")]);
        let cmds = generate_runtime_commands("ai-claude", &tools);
        assert!(cmds.contains("claude-code"), "should install Claude: {cmds}");
    }
}
