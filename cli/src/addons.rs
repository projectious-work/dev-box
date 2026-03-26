use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};

use crate::addon_loader;
use crate::addon_registry::{self, ToolConfig};
use crate::config::AddonsSection;

/// Result of processing all addons for Dockerfile generation.
pub struct DockerfileAddonOutput {
    /// Builder stages to insert before the runtime FROM (e.g., texlive-builder, rust-builder).
    pub builder_stages: Vec<String>,
    /// RUN commands to insert in the runtime stage.
    pub runtime_commands: Vec<String>,
}

/// Return a sort key for addon builder-stage ordering based on builder_weight
/// from the YAML definition. heavy=0, medium=1, light=2, unknown=3.
fn builder_order_key(name: &str) -> usize {
    addon_loader::get_addon(name)
        .map(|a| a.builder_order_key())
        .unwrap_or(3)
}

/// Topologically sort addon names so dependencies come first.
/// Falls back to alphabetical order for addons at the same depth.
fn topological_sort(
    addon_names: &[String],
    get_requires: impl Fn(&str) -> Vec<String>,
) -> Result<Vec<String>> {
    let name_set: HashSet<&str> = addon_names.iter().map(|s| s.as_str()).collect();

    // Validate: all required addons must be present
    for name in addon_names {
        for req in get_requires(name) {
            if !name_set.contains(req.as_str()) {
                bail!(
                    "Addon '{}' requires '{}' addon. Add [addons.{}] to your aibox.toml.",
                    name, req, req
                );
            }
        }
    }

    // Kahn's algorithm
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for name in addon_names {
        in_degree.entry(name.as_str()).or_insert(0);
        for req in get_requires(name) {
            if name_set.contains(req.as_str()) {
                *in_degree.entry(name.as_str()).or_insert(0) += 1;
                dependents
                    .entry(addon_names.iter().find(|n| n.as_str() == req).unwrap().as_str())
                    .or_default()
                    .push(name.as_str());
            }
        }
    }

    let mut queue: std::collections::VecDeque<&str> = in_degree
        .iter()
        .filter(|&(_, &deg)| deg == 0)
        .map(|(&name, _)| name)
        .collect::<Vec<_>>()
        .into_iter()
        .collect();
    queue.make_contiguous().sort();

    let mut sorted = Vec::new();
    while let Some(name) = queue.pop_front() {
        sorted.push(name.to_string());
        if let Some(deps) = dependents.get(name) {
            let mut newly_free: Vec<&str> = Vec::new();
            for &dep in deps {
                let deg = in_degree.get_mut(dep).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    newly_free.push(dep);
                }
            }
            newly_free.sort();
            for f in newly_free {
                queue.push_back(f);
            }
            // Re-sort to maintain alphabetical order among same-depth items
            queue.make_contiguous().sort();
        }
    }

    if sorted.len() != addon_names.len() {
        bail!("Circular dependency detected among addons");
    }

    Ok(sorted)
}

/// Process all enabled addons and generate Dockerfile content.
///
/// For each addon listed in `addons.addons`:
///   1. Topologically sort addons by `requires` dependencies.
///   2. Look up the addon definition in the registry.
///   3. Merge user tool entries with registry defaults via [`to_tool_configs`].
///   4. Call the registry's builder-stage generator (if the addon has one).
///   5. Call the registry's runtime-commands generator.
///   6. Collect results, ordering builder stages heavy-first.
pub fn generate_dockerfile_content(addons: &AddonsSection) -> Result<DockerfileAddonOutput> {
    let addon_names: Vec<String> = addons.addons.keys().cloned().collect();

    let sorted_names = topological_sort(&addon_names, |name| {
        addon_loader::get_addon(name)
            .map(|a| a.requires.clone())
            .unwrap_or_default()
    })?;

    let mut builder_entries: Vec<(usize, String)> = Vec::new();
    let mut runtime_commands: Vec<String> = Vec::new();

    for addon_name in &sorted_names {
        let addon_tools_section = &addons.addons[addon_name];
        let addon_def = match addon_registry::get_addon(addon_name) {
            Some(def) => def,
            None => {
                // Unknown addon — emit a Dockerfile comment warning and skip.
                runtime_commands.push(format!(
                    "# WARNING: unknown addon '{}' — skipped",
                    addon_name
                ));
                continue;
            }
        };

        let tool_configs = to_tool_configs(addon_name, &addon_tools_section.tools, &addon_def);

        // Builder stage (if this addon defines one)
        if let Some(stage) = addon_registry::generate_builder_stage(addon_name, &tool_configs) {
            let order = builder_order_key(addon_name);
            builder_entries.push((order, stage));
        }

        // Runtime commands (RUN + COPY --from=builder, etc.)
        let cmds = addon_registry::generate_runtime_commands(addon_name, &tool_configs);
        if !cmds.is_empty() {
            runtime_commands.push(cmds);
        }
    }

    // Sort builder stages: heavy builds first (latex, rust), then lighter ones.
    builder_entries.sort_by_key(|(order, _)| *order);

    Ok(DockerfileAddonOutput {
        builder_stages: builder_entries.into_iter().map(|(_, s)| s).collect(),
        runtime_commands,
    })
}

/// Convert TOML tool entries to the [`ToolConfig`] format the registry expects.
///
/// Merging strategy ("working set + user overrides"):
///   - If the tool appears in the user's config: use their version (or the
///     registry default if no version was specified).
///   - If the tool is *not* in the user's config but is `default_enabled` in
///     the registry: include it with registry defaults.
///   - If the tool is not in config and not `default_enabled`: skip (disabled).
fn to_tool_configs(
    _addon_name: &str,
    user_tools: &HashMap<String, crate::config::ToolEntry>,
    addon_def: &addon_registry::AddonDef,
) -> HashMap<String, ToolConfig> {
    let mut configs: HashMap<String, ToolConfig> = HashMap::new();

    for tool_def in addon_def.tools {
        if let Some(user_entry) = user_tools.get(tool_def.name) {
            // User explicitly listed this tool — use their version or fall back
            // to the registry default.
            let version = user_entry
                .version
                .as_deref()
                .unwrap_or(tool_def.default_version)
                .to_string();
            configs.insert(
                tool_def.name.to_string(),
                ToolConfig {
                    version,
                    enabled: true,
                },
            );
        } else if tool_def.default_enabled {
            // Not mentioned by user but on by default — include with defaults.
            configs.insert(
                tool_def.name.to_string(),
                ToolConfig {
                    version: tool_def.default_version.to_string(),
                    enabled: true,
                },
            );
        }
        // Otherwise: tool is not in user config and not default-enabled → skip.
    }

    configs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addon_registry::ToolDef;
    use crate::config::ToolEntry;

    fn ensure_loaded() {
        let addons_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("addons");
        let _ = crate::addon_loader::init_from_dir(&addons_dir);
    }

    // ── to_tool_configs tests ───────────────────────────────────────────

    fn sample_addon_def() -> addon_registry::AddonDef {
        // Leak the Vec so we get a &'static [ToolDef].  Fine in tests.
        let tools: &'static [ToolDef] = Box::leak(Box::new([
            ToolDef {
                name: "alpha",
                default_enabled: true,
                supported_versions: &["1.0", "2.0"],
                default_version: "2.0",
            },
            ToolDef {
                name: "beta",
                default_enabled: false,
                supported_versions: &["3.0"],
                default_version: "3.0",
            },
            ToolDef {
                name: "gamma",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
        ]));

        addon_registry::AddonDef {
            name: "test-addon",
            addon_version: "0.1.0",
            tools,
        }
    }

    #[test]
    fn default_enabled_tools_included_without_user_config() {
        let user_tools: HashMap<String, ToolEntry> = HashMap::new();
        let def = sample_addon_def();
        let configs = to_tool_configs("test-addon", &user_tools, &def);

        assert!(configs.contains_key("alpha"), "default_enabled alpha should be present");
        assert!(configs.contains_key("gamma"), "default_enabled gamma should be present");
        assert!(
            !configs.contains_key("beta"),
            "non-default beta should be absent"
        );
    }

    #[test]
    fn user_version_overrides_default() {
        let mut user_tools: HashMap<String, ToolEntry> = HashMap::new();
        user_tools.insert(
            "alpha".to_string(),
            ToolEntry {
                version: Some("1.0".to_string()),
            },
        );
        let def = sample_addon_def();
        let configs = to_tool_configs("test-addon", &user_tools, &def);

        assert_eq!(configs["alpha"].version, "1.0");
    }

    #[test]
    fn user_tool_without_version_gets_default() {
        let mut user_tools: HashMap<String, ToolEntry> = HashMap::new();
        user_tools.insert("alpha".to_string(), ToolEntry { version: None });
        let def = sample_addon_def();
        let configs = to_tool_configs("test-addon", &user_tools, &def);

        assert_eq!(configs["alpha"].version, "2.0");
    }

    #[test]
    fn non_default_tool_included_when_user_enables_it() {
        let mut user_tools: HashMap<String, ToolEntry> = HashMap::new();
        user_tools.insert("beta".to_string(), ToolEntry { version: None });
        let def = sample_addon_def();
        let configs = to_tool_configs("test-addon", &user_tools, &def);

        assert!(configs.contains_key("beta"), "beta should be present when user enables it");
        assert_eq!(configs["beta"].version, "3.0");
    }

    // ── builder_order_key tests ─────────────────────────────────────────

    #[test]
    fn known_addons_sort_in_canonical_order() {
        ensure_loaded();
        // heavy < medium < no-builder
        assert!(builder_order_key("latex") < builder_order_key("rust"));
        // rust, infrastructure, kubernetes are all "medium" — same weight is fine
        assert!(builder_order_key("rust") <= builder_order_key("infrastructure"));
        assert!(builder_order_key("infrastructure") <= builder_order_key("kubernetes"));
        // all medium < no-builder (python has no builder stage)
        assert!(builder_order_key("kubernetes") < builder_order_key("python"));
    }

    #[test]
    fn unknown_addon_sorts_last() {
        let unknown = builder_order_key("some-future-addon");
        assert_eq!(unknown, 3, "unknown addons should have weight 3 (no builder)");
    }

    // ── generate_dockerfile_content tests ───────────────────────────────

    #[test]
    fn empty_addons_produce_empty_output() {
        let addons = AddonsSection {
            addons: HashMap::new(),
        };
        let output = generate_dockerfile_content(&addons).unwrap();
        assert!(output.builder_stages.is_empty());
        assert!(output.runtime_commands.is_empty());
    }

    #[test]
    fn unknown_addon_emits_warning_comment() {
        let mut addons_map = HashMap::new();
        addons_map.insert(
            "nonexistent-addon".to_string(),
            crate::config::AddonToolsSection {
                tools: HashMap::new(),
            },
        );
        let addons = AddonsSection { addons: addons_map };
        let output = generate_dockerfile_content(&addons).unwrap();

        assert!(output.builder_stages.is_empty());
        assert_eq!(output.runtime_commands.len(), 1);
        assert!(output.runtime_commands[0].contains("WARNING"));
        assert!(output.runtime_commands[0].contains("nonexistent-addon"));
    }

    // ── topological_sort tests ──────────────────────────────────────────

    #[test]
    fn topo_sort_no_deps_is_alphabetical() {
        let names = vec!["c".to_string(), "a".to_string(), "b".to_string()];
        let sorted = topological_sort(&names, |_| vec![]).unwrap();
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn topo_sort_respects_dependencies() {
        let names = vec![
            "docs-docusaurus".to_string(),
            "node".to_string(),
            "latex".to_string(),
        ];
        let sorted = topological_sort(&names, |name| match name {
            "docs-docusaurus" => vec!["node".to_string()],
            _ => vec![],
        })
        .unwrap();

        let node_pos = sorted.iter().position(|n| n == "node").unwrap();
        let docu_pos = sorted.iter().position(|n| n == "docs-docusaurus").unwrap();
        assert!(
            node_pos < docu_pos,
            "node ({}) must come before docs-docusaurus ({})",
            node_pos,
            docu_pos
        );
    }

    #[test]
    fn topo_sort_missing_dependency_errors() {
        let names = vec!["docs-docusaurus".to_string()];
        let result = topological_sort(&names, |name| match name {
            "docs-docusaurus" => vec!["node".to_string()],
            _ => vec![],
        });
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("requires 'node'"), "error: {}", err);
        assert!(err.contains("[addons.node]"), "error: {}", err);
    }

    #[test]
    fn topo_sort_circular_dependency_errors() {
        let names = vec!["a".to_string(), "b".to_string()];
        let result = topological_sort(&names, |name| match name {
            "a" => vec!["b".to_string()],
            "b" => vec!["a".to_string()],
            _ => vec![],
        });
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Circular"), "error: {}", err);
    }

    #[test]
    fn topo_sort_self_referential_dependency_errors() {
        // A requires A — the simplest possible cycle.
        let names = vec!["a".to_string()];
        let result = topological_sort(&names, |name| match name {
            "a" => vec!["a".to_string()],
            _ => vec![],
        });
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // A self-loop means in_degree["a"] stays > 0 and the sorted list never
        // reaches length 1, so Kahn's algorithm reports "Circular dependency".
        assert!(err.contains("Circular"), "error: {}", err);
    }

    #[test]
    fn topo_sort_three_node_cycle_errors() {
        // A → B → C → A
        let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = topological_sort(&names, |name| match name {
            "a" => vec!["b".to_string()],
            "b" => vec!["c".to_string()],
            "c" => vec!["a".to_string()],
            _ => vec![],
        });
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Circular"), "error: {}", err);
    }
}
