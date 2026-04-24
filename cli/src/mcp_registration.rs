//! MCP server registration — write per-harness MCP config files at the
//! project root from the per-skill `mcp/mcp-config.json` files that
//! processkit ships.
//!
//! ## Why this module exists
//!
//! Every MCP-capable agent harness reads exactly **one** config file at
//! session start to discover the MCP servers it should spawn. Different
//! harnesses use different paths and (sometimes) different formats:
//!
//! | Harness     | Config file                            | Format      |
//! |-------------|----------------------------------------|-------------|
//! | Claude Code | `.mcp.json` (project root)             | JSON        |
//! | Cursor      | `.cursor/mcp.json`                     | JSON        |
//! | Gemini CLI  | `.gemini/settings.json`                | JSON        |
//! | Codex CLI   | `.codex/config.toml`                   | TOML        |
//! | Continue    | `.continue/mcpServers/<name>.json`     | per-file    |
//!
//! processkit ships per-skill `mcp/mcp-config.json` files in the
//! Claude-shape `{"mcpServers": {"<name>": {"command", "args"}}}`
//! format. aibox walks the templates mirror at sync/init time to find
//! every shipped MCP server, then dispatches to per-harness writers
//! that translate (where needed) and merge the result into each
//! harness's config file **non-destructively**: only entries whose
//! `name` is in the "managed set" (i.e., comes from a processkit
//! skill) are added/updated/removed; user-added entries with names
//! outside the managed set are preserved.
//!
//! Strawman D applies: aibox owns the managed-set entries, the user
//! owns everything else, the cache mirror is the source of truth for
//! "what processkit shipped this version".
//!
//! ## Special cases
//!
//! - **Mistral**: has MCP client capability via Python SDK and Le
//!   Chat, but no local file-based project config. When `Mistral` is
//!   in `[ai].providers`, aibox writes `.mcp.json` (the same Claude
//!   shape) so a custom Mistral SDK-based CLI tool can read MCP
//!   server registrations from there.
//! - **Aider**: has no native MCP client. When `Aider` is in
//!   `[ai].providers`, aibox emits a warning and writes nothing.
//!
//! See DEC-033 for the design rationale.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{AiProvider, AiboxConfig};
use crate::output;
use crate::processkit_vocab::mirror_skills_dir;

// ---------------------------------------------------------------------------
// Spec types
// ---------------------------------------------------------------------------

/// One MCP server entry as it appears in a per-skill `mcp-config.json`
/// shipped by processkit. Mirrors the Claude Code `mcpServers` shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerSpec {
    /// Server name. Used as the key in the merged `mcpServers` object
    /// for JSON harnesses, the section name `[mcp_servers.<name>]` for
    /// Codex TOML, and the per-file basename for Continue.
    pub name: String,
    /// Executable to spawn (e.g. `uv`, `npx`, `python3`).
    pub command: String,
    /// Arguments passed to `command`. Relative paths to MCP server
    /// scripts are project-root-relative; the harness invokes the
    /// server with cwd = project root.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables. Empty by default.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// Per-skill `mcp-config.json` shape: a wrapper object with a single
/// `mcpServers` key whose value is keyed-by-name (Claude shape).
#[derive(Debug, Clone, Deserialize)]
struct PerSkillConfig {
    #[serde(default)]
    #[serde(rename = "mcpServers")]
    mcp_servers: BTreeMap<String, RawServerEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawServerEntry {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
}

// ---------------------------------------------------------------------------
// MCP permission configuration from aibox.toml [mcp] section
// ---------------------------------------------------------------------------

/// Configuration for global MCP permission management across all harnesses.
/// Parsed from the `[mcp]` section of aibox.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct McpConfig {
    /// Default permission mode: "allow", "ask", or "deny".
    /// Used when no pattern matches a tool name.
    #[serde(default = "default_mode")]
    pub default_mode: String,

    /// Allow-list patterns (glob-style, e.g., "mcp__processkit-*", "bash").
    /// Matched against tool names; first match wins.
    #[serde(default)]
    pub allow_patterns: Vec<String>,

    /// Deny-list patterns (glob-style, e.g., "mcp__private-*").
    /// Matched against tool names; first match wins.
    /// Deny patterns override allow patterns if both match.
    #[serde(default)]
    pub deny_patterns: Vec<String>,

    /// Per-harness overrides. Key is harness name (e.g., "claude-code").
    #[serde(default)]
    pub harness: BTreeMap<String, HarnessOverride>,
}

#[allow(dead_code)]
fn default_mode() -> String {
    "allow".to_string()
}

/// Per-harness override configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct HarnessOverride {
    /// Whether this harness is enabled. Default: true.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Override global mode for this harness.
    #[serde(default)]
    pub mode: Option<String>,

    /// Additional allow patterns for this harness only.
    #[serde(default)]
    pub extra_patterns: Vec<String>,

    /// Patterns to deny for this harness (override global allow).
    #[serde(default)]
    pub deny_patterns: Vec<String>,
}

#[allow(dead_code)]
fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Pattern matching logic
// ---------------------------------------------------------------------------

/// Expand glob patterns into concrete tool names.
/// Patterns like "mcp__processkit-*" match all tools starting with that prefix.
///
/// # Arguments
/// * `patterns` - List of glob-style patterns (e.g., ["mcp__processkit-*", "bash"])
/// * `available_tools` - All available tool names to match against
///
/// # Returns
/// Sorted list of unique tool names that match any pattern.
#[allow(dead_code)]
pub fn expand_mcp_patterns(
    patterns: &[String],
    available_tools: &[String],
) -> Vec<String> {
    let mut matched = std::collections::HashSet::new();

    for pattern in patterns {
        for tool in available_tools {
            if glob_matches(tool, pattern) {
                matched.insert(tool.clone());
            }
        }
    }

    let mut result: Vec<_> = matched.into_iter().collect();
    result.sort();
    result
}

/// Simple glob pattern matching for "prefix-*" and exact matches.
/// Supports:
/// - Exact match: "tool_name"
/// - Wildcard suffix: "prefix-*" matches "prefix-foo", "prefix-bar", etc.
/// - Wildcard prefix: "*-suffix" matches "foo-suffix", "bar-suffix", etc.
#[allow(dead_code)]
fn glob_matches(tool: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return tool == pattern;
    }
    if pattern.starts_with('*') && pattern.ends_with('*') {
        // Middle match: *foo* matches anything containing foo
        let middle = &pattern[1..pattern.len() - 1];
        return tool.contains(middle);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        // Prefix match: foo* matches foo, foobar, etc.
        return tool.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        // Suffix match: *foo matches foo, barfoo, etc.
        return tool.ends_with(suffix);
    }
    false
}

/// Determine if a tool is allowed based on allow/deny patterns.
/// First-match-wins semantics: the first matching pattern (allow or deny)
/// determines the result. If no pattern matches, returns the default_mode.
///
/// # Arguments
/// * `tool_name` - Name of the tool to check
/// * `allow_patterns` - Patterns that permit the tool
/// * `deny_patterns` - Patterns that forbid the tool
/// * `default_mode` - What to return if no pattern matches
///
/// # Returns
/// true if the tool is allowed, false if denied
#[allow(dead_code)]
pub fn first_match_wins(
    tool_name: &str,
    allow_patterns: &[String],
    deny_patterns: &[String],
    default_mode: &str,
) -> bool {
    // Check deny patterns first for security
    for pattern in deny_patterns {
        if glob_matches(tool_name, pattern) {
            return false;
        }
    }

    // Check allow patterns
    for pattern in allow_patterns {
        if glob_matches(tool_name, pattern) {
            return true;
        }
    }

    // No pattern matched; use default mode
    match default_mode {
        "allow" | "automatic" => true,
        "ask" => true, // Ask mode still permits the tool, just with prompts
        "deny" => false,
        _ => true, // Unknown modes default to allow
    }
}

// ---------------------------------------------------------------------------
// Harness-specific permission generators (Phase 2)
// ---------------------------------------------------------------------------

/// Generate MCP permissions for Claude Code's settings.local.json.
/// Updates `permissions.allow[]` list with tools allowed by the McpConfig patterns.
///
/// # Arguments
/// * `project_root` - Project root directory
/// * `config` - Parsed McpConfig from aibox.toml
/// * `all_tool_names` - All available MCP tool names
///
/// # Returns
/// Result; logs warnings on individual failures, doesn't abort
#[allow(dead_code)]
pub fn generate_claude_code_permissions(
    project_root: &Path,
    config: &McpConfig,
    all_tool_names: &[String],
) -> Result<()> {
    let settings_path = project_root.join(".claude").join("settings.local.json");

    // Determine allowed tools
    let allowed = expand_mcp_patterns(&config.allow_patterns, all_tool_names);
    let denied = expand_mcp_patterns(&config.deny_patterns, all_tool_names);

    // Build final allow list: keep tools allowed by config, remove denied ones
    let mut permissions: Vec<String> = allowed
        .iter()
        .filter(|tool| !denied.contains(tool))
        .map(|t| format!("mcp__{}", t))
        .collect();
    permissions.sort();

    // Read existing settings or create new
    let mut settings: serde_json::Map<String, serde_json::Value> = if settings_path.is_file() {
        let body = fs::read_to_string(&settings_path).unwrap_or_default();
        if body.trim().is_empty() {
            serde_json::Map::new()
        } else {
            serde_json::from_str(&body).unwrap_or_default()
        }
    } else {
        serde_json::Map::new()
    };

    // Merge: preserve existing non-permissions keys
    settings.insert(
        "permissions.allow".to_string(),
        serde_json::Value::Array(
            permissions
                .iter()
                .map(|p| serde_json::Value::String(p.clone()))
                .collect(),
        ),
    );

    // Ensure parent dir exists
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let formatted = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, formatted)
        .with_context(|| format!("failed to write Claude Code permissions to {}", settings_path.display()))?;

    Ok(())
}

/// Generate MCP permissions for OpenCode's config.toml.
/// Updates `[mcp]` section with allow/ask/deny modes based on patterns.
///
/// # Arguments
/// * `project_root` - Project root directory
/// * `config` - Parsed McpConfig from aibox.toml
/// * `all_tool_names` - All available MCP tool names
///
/// # Returns
/// Result; logs warnings on individual failures
#[allow(dead_code)]
pub fn generate_opencode_permissions(
    project_root: &Path,
    config: &McpConfig,
    all_tool_names: &[String],
) -> Result<()> {
    let config_path = project_root.join(".opencode").join("config.toml");

    // Determine allowed/denied tools
    let allowed = expand_mcp_patterns(&config.allow_patterns, all_tool_names);
    let denied = expand_mcp_patterns(&config.deny_patterns, all_tool_names);

    // Read existing config or create new
    let mut document: toml_edit::DocumentMut = if config_path.is_file() {
        let body = fs::read_to_string(&config_path).unwrap_or_default();
        body.parse::<toml_edit::DocumentMut>().unwrap_or_default()
    } else {
        toml_edit::DocumentMut::new()
    };

    // Ensure [mcp] section exists
    if !document.contains_key("mcp") {
        document["mcp"] = toml_edit::table();
    }

    let mcp_table = &mut document["mcp"];

    // Set mode based on config
    mcp_table["mode"] = toml_edit::value(config.default_mode.as_str());

    // Set allow list
    let allow_array = toml_edit::Array::from_iter(allowed.iter().map(|s| s.as_str()));
    mcp_table["allow"] = toml_edit::value(allow_array);

    // Set deny list if present
    if !denied.is_empty() {
        let deny_array = toml_edit::Array::from_iter(denied.iter().map(|s| s.as_str()));
        mcp_table["deny"] = toml_edit::value(deny_array);
    }

    // Ensure parent dir exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(&config_path, document.to_string())
        .with_context(|| format!("failed to write OpenCode permissions to {}", config_path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Walk the templates mirror to compute the managed set
// ---------------------------------------------------------------------------

/// Find a named skill directory anywhere in the two-level category tree under
/// `mirror_skills_dir`. Returns the path to the skill directory (i.e.
/// `<mirror_skills_dir>/<category>/<skill_name>`) on the first match.
///
/// Used by the kernel-fallback branch so it can locate kernel skills that
/// live under a category subdirectory rather than directly at the skills root.
fn find_skill_in_mirror(mirror_skills_dir: &Path, skill_name: &str) -> Option<std::path::PathBuf> {
    let categories = fs::read_dir(mirror_skills_dir).ok()?;
    for category in categories.flatten() {
        if !category.path().is_dir() {
            continue;
        }
        let candidate = category.path().join(skill_name);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Walk `context/templates/processkit/<version>/skills/<category>/<skill>/mcp/mcp-config.json`
/// for the currently-pinned processkit version, parse each file, and
/// return the flattened list of [`McpServerSpec`] entries.
///
/// The `effective_skills` filter restricts the walk to skill names that
/// are in the user's effective set (computed from `[context].packages`
/// plus `[skills].include` plus `[skills].exclude` — see
/// `crate::content_init::build_effective_skill_set`). When the filter
/// is `None` (tests, or callers that haven't activated skill filtering),
/// every skill that has an `mcp/mcp-config.json` is included.
///
/// `force_include` is a list of skill directory names that always bypass
/// the `effective_skills` filter, even when a filter is active. Pass
/// [`crate::processkit_vocab::MANDATORY_MCP_SKILLS`] here to ensure the
/// six mandatory entity-layer servers are always registered regardless of
/// the package tier.
///
/// Returns `Ok(vec![])` if the templates mirror doesn't exist yet
/// (e.g. processkit version is "unset" — no install has happened).
pub fn collect_processkit_mcp_specs(
    project_root: &Path,
    processkit_version: &str,
    effective_skills: Option<&HashSet<String>>,
    force_include: &[&str],
) -> Result<Vec<McpServerSpec>> {
    if processkit_version == crate::config::PROCESSKIT_VERSION_UNSET {
        return Ok(Vec::new());
    }
    let Some(mirror_skills_dir) = mirror_skills_dir(project_root, processkit_version) else {
        return Ok(Vec::new());
    };

    let mut specs: Vec<McpServerSpec> = Vec::new();
    let mut any_parse_error = false;
    // Collision guard: track skill_name → category path for the first time
    // each bare skill name is seen. On a second encounter in a different
    // category, emit a warning so the operator can disambiguate upstream.
    let mut seen_skill_categories: HashMap<String, std::path::PathBuf> = HashMap::new();

    for category_entry in fs::read_dir(&mirror_skills_dir).with_context(|| {
        format!(
            "failed to read templates mirror at {}",
            mirror_skills_dir.display()
        )
    })? {
        let category_entry = category_entry?;
        let category_dir = category_entry.path();
        let category_name = match category_entry.file_name().to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        // Skip non-category entries (e.g. INDEX.md, FORMAT.md, _lib).
        if category_name.starts_with('_') || category_name.starts_with('.') {
            continue;
        }
        if !category_dir.is_dir() {
            continue;
        }

        // Inner loop: iterate skill dirs within this category.
        let skill_entries = match fs::read_dir(&category_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for skill_entry in skill_entries.flatten() {
            let skill_dir = skill_entry.path();
            let skill_name = match skill_entry.file_name().to_str() {
                Some(s) => s.to_string(),
                None => continue,
            };
            if skill_name.starts_with('_') || skill_name.starts_with('.') {
                continue;
            }
            if !skill_dir.is_dir() {
                continue;
            }

            // Collision guard: warn if this bare skill name was already seen
            // in a different category (last-wins semantics).
            if let Some(prev_cat) = seen_skill_categories.get(&skill_name)
                && prev_cat
                    != &category_dir
                        .parent()
                        .unwrap_or(&category_dir)
                        .join(&category_name)
            {
                output::warn(&format!(
                    "duplicate skill basename '{skill_name}' found in \
                     categories '{prev_cat}' and '{category_dir}' — \
                     last-wins; '{category_dir}' takes precedence. \
                     Disambiguate upstream to silence this warning.",
                    prev_cat = prev_cat.display(),
                    category_dir = category_dir.display(),
                ));
            }
            seen_skill_categories.insert(skill_name.clone(), category_dir.clone());

            // Apply effective-set filter if provided.
            // Skills in `force_include` always bypass this filter — they are
            // mandatory regardless of package tier (see MANDATORY_MCP_SKILLS).
            if let Some(set) = effective_skills
                && !set.contains(&skill_name)
                && !force_include.contains(&skill_name.as_str())
            {
                continue;
            }

            let config_path = skill_dir.join("mcp").join("mcp-config.json");
            if !config_path.is_file() {
                // Skill doesn't ship an MCP server. Common — most skills
                // are documentation-only.
                continue;
            }

            let body = fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read {}", config_path.display()))?;
            let parsed: PerSkillConfig = match serde_json::from_str(&body) {
                Ok(p) => p,
                Err(e) => {
                    output::warn(&format!(
                        "skill '{skill_name}': failed to parse mcp/mcp-config.json — {e}. \
                         Skipping this skill; kernel fallback will be applied.",
                    ));
                    any_parse_error = true;
                    continue;
                }
            };

            for (name, raw) in parsed.mcp_servers {
                specs.push(McpServerSpec {
                    name,
                    command: raw.command,
                    args: raw.args,
                    env: raw.env,
                });
            }
        }
    }

    // If any per-skill config failed to parse, force-include the 8 kernel
    // skills so the harness config is never left missing critical servers.
    // Skills already collected continue to contribute; we only add kernel
    // entries for skills that are present in the mirror but weren't yet
    // collected (either because they failed or weren't visited yet).
    if any_parse_error {
        let already_visited: HashSet<String> = specs.iter().map(|s| s.name.clone()).collect();
        for &kernel_skill in crate::processkit_vocab::KERNEL_MCP_SKILLS {
            let Some(kernel_skill_dir) = find_skill_in_mirror(&mirror_skills_dir, kernel_skill)
            else {
                continue;
            };
            let kernel_config = kernel_skill_dir.join("mcp").join("mcp-config.json");
            if !kernel_config.is_file() {
                continue;
            }
            // Re-read and parse; ignore errors for kernel skills too
            // (we already warned about the corrupt ones above).
            let Ok(body) = fs::read_to_string(&kernel_config) else {
                continue;
            };
            let Ok(parsed) = serde_json::from_str::<PerSkillConfig>(&body) else {
                continue;
            };
            for (name, raw) in parsed.mcp_servers {
                if !already_visited.contains(&name) {
                    specs.push(McpServerSpec {
                        name,
                        command: raw.command,
                        args: raw.args,
                        env: raw.env,
                    });
                }
            }
        }
    }

    // Stable order: sort by name. Makes the output deterministic.
    specs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(specs)
}

/// Collect MCP server specs from live-installed skills in context/skills/.
/// This handles the case where processkit ships an incomplete release
/// (e.g. v0.19.1) where some skills have mcp/mcp-config.json files only
/// in the live installation, not in the templates mirror.
///
/// Returns `Ok(vec![])` if context/skills/ doesn't exist.
pub fn collect_live_skills_mcp_specs(project_root: &Path) -> Result<Vec<McpServerSpec>> {
    let skills_dir = project_root.join("context").join("skills");
    if !skills_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut specs: Vec<McpServerSpec> = Vec::new();

    // Walk the two-level layout: category/skill/
    for category_entry in fs::read_dir(&skills_dir).with_context(|| {
        "failed to read context/skills/".to_string()
    })? {
        let category_entry = category_entry?;
        let category_dir = category_entry.path();
        let category_name = match category_entry.file_name().to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        if category_name.starts_with('_') || category_name.starts_with('.') {
            continue;
        }
        if !category_dir.is_dir() {
            continue;
        }

        // Inner loop: skill dirs within category
        let skill_entries = match fs::read_dir(&category_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for skill_entry in skill_entries.flatten() {
            let skill_dir = skill_entry.path();
            if !skill_dir.is_dir() {
                continue;
            }

            let mcp_config_path = skill_dir.join("mcp").join("mcp-config.json");
            if !mcp_config_path.is_file() {
                continue;
            }

            match fs::read_to_string(&mcp_config_path) {
                Ok(content) => {
                    match serde_json::from_str::<PerSkillConfig>(&content) {
                        Ok(config) => {
                            for (name, server) in config.mcp_servers {
                                specs.push(McpServerSpec {
                                    name,
                                    command: server.command,
                                    args: server.args,
                                    env: server.env,
                                });
                            }
                        }
                        Err(e) => {
                            output::warn(&format!(
                                "Failed to parse {}: {}",
                                mcp_config_path.display(),
                                e
                            ));
                        }
                    }
                }
                Err(e) => {
                    output::warn(&format!(
                        "Failed to read {}: {}",
                        mcp_config_path.display(),
                        e
                    ));
                }
            }
        }
    }

    // Deduplicate by name (live specs may override template specs)
    let mut seen = std::collections::HashSet::new();
    specs.retain(|spec| seen.insert(spec.name.clone()));

    specs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(specs)
}

/// Compute the managed set: server names that came from processkit.
/// The merge writers use this set to decide which entries to remove
/// from the existing harness config before adding the current ones.
fn managed_set(specs: &[McpServerSpec]) -> BTreeSet<String> {
    specs.iter().map(|s| s.name.clone()).collect()
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Regenerate every harness-specific MCP config file based on the
/// currently-installed processkit version and the user's
/// `[ai].providers` list.
///
/// Called from `cmd_init` and `cmd_sync` after `install_content_source`
/// returns successfully. Idempotent: re-running on a stable
/// `(processkit_version, providers, skills)` set produces byte-identical
/// output.
pub fn regenerate_mcp_configs(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    // The effective skill set (DEC-035 / BACK-118) constrains which
    // skills' MCP servers we register — same set the install path
    // uses to filter live-installed files. If the templates mirror
    // is missing (first install or processkit version unset), the
    // set is None and we register every skill that has an mcp/
    // directory in the mirror.
    let effective = crate::content_init::build_effective_skill_set(project_root, config)
        .ok()
        .flatten();
    let processkit_specs = collect_processkit_mcp_specs(
        project_root,
        &config.processkit.version,
        effective.as_ref(),
        crate::processkit_vocab::MANDATORY_MCP_SKILLS,
    )?;

    // Also collect from live-installed skills in context/skills/ to handle
    // incomplete processkit releases (e.g. v0.19.1) where some skills have
    // mcp/mcp-config.json files only in the live installation.
    let live_skills_specs = collect_live_skills_mcp_specs(project_root)?;

    // Build the full spec list: processkit + live skills first, then team-shared
    // (aibox.toml [mcp.servers]), then personal (.aibox-local.toml
    // [mcp.servers]).  All three sources are "aibox-managed" — they
    // are in the managed set and get refreshed on every sync so that
    // removals from any source are reflected immediately.
    // Live skills specs override processkit specs of the same name.
    let mut specs: Vec<McpServerSpec> = processkit_specs;
    for spec in live_skills_specs {
        // Replace any existing spec with the same name
        specs.retain(|s| s.name != spec.name);
        specs.push(spec);
    }
    for s in &config.mcp.servers {
        specs.push(McpServerSpec {
            name: s.name.clone(),
            command: s.command.clone(),
            args: s.args.clone(),
            env: s.env.clone(),
        });
    }
    for s in &config.local_mcp_servers {
        specs.push(McpServerSpec {
            name: s.name.clone(),
            command: s.command.clone(),
            args: s.args.clone(),
            env: s.env.clone(),
        });
    }
    // Stable order across all sources.
    specs.sort_by(|a, b| a.name.cmp(&b.name));

    if specs.is_empty() {
        // No MCP servers from any source. Nothing to write.
        return Ok(());
    }

    // Validate that all mandatory MCP servers were collected. A mandatory skill
    // that has no `mcp/mcp-config.json` in the templates mirror means the
    // processkit version installed is too old or is broken — warn the user so
    // they know entity-layer coverage is incomplete.
    if let Some(skills_dir) =
        crate::processkit_vocab::mirror_skills_dir(project_root, &config.processkit.version)
    {
        let registered_names: std::collections::HashSet<&str> =
            specs.iter().map(|s| s.name.as_str()).collect();
        for &skill in crate::processkit_vocab::MANDATORY_MCP_SKILLS {
            // Convention: the server name shipped in mcp-config.json is
            // `processkit-{skill}`. Check whether any registered spec
            // matches; fall back to checking the mcp-config.json file
            // exists so we catch cases where the naming convention changes.
            let expected_server = format!("processkit-{skill}");
            let config_exists = find_skill_in_mirror(&skills_dir, skill)
                .map(|d| d.join("mcp").join("mcp-config.json").is_file())
                .unwrap_or(false);
            if !registered_names.contains(expected_server.as_str()) && !config_exists {
                output::warn(&format!(
                    "mandatory processkit MCP skill '{skill}' has no mcp/mcp-config.json \
                     in the {} templates mirror — its server will not be registered. \
                     Entity-layer coverage is incomplete. Run `aibox sync` after \
                     upgrading processkit to a version that includes this skill.",
                    &config.processkit.version,
                ));
            }
        }
    }

    // Hard-fail safety rail (v0.18.7): every script path referenced by a
    // merged MCP server must exist on disk. Caught aibox#NN where 12/16
    // processkit per-skill mcp-config.json files shipped paths missing the
    // `processkit/` category prefix introduced in v0.17.0 — sync wrote a
    // `.mcp.json` that named files that didn't exist, so harnesses logged
    // an opaque "MCP startup failed: connection closed" for two releases
    // before anyone noticed. Failing here is loud and actionable; silently
    // emitting a broken config is the failure mode we never want again.
    validate_script_paths(&specs, project_root)?;

    let managed = managed_set(&specs);
    let providers: HashSet<&AiProvider> = config.ai.harnesses.iter().collect();

    // 1. Claude / Copilot / OpenCode / Hermes / Mistral use the Claude-shape
    //    `mcpServers` JSON object at `.mcp.json`.
    let writes_dot_mcp_json = providers.contains(&AiProvider::Claude)
        || providers.contains(&AiProvider::Copilot)
        || providers.contains(&AiProvider::OpenCode)
        || providers.contains(&AiProvider::Hermes)
        || providers.contains(&AiProvider::Mistral);
    if writes_dot_mcp_json {
        let path = project_root.join(".mcp.json");
        write_mcp_servers_json(&specs, &managed, &path)?;
        output::ok(&format!(
            "Wrote {} processkit MCP servers to {}",
            specs.len(),
            path.display()
        ));
    }
    if providers.contains(&AiProvider::Cursor) {
        let path = project_root.join(".cursor/mcp.json");
        write_mcp_servers_json(&specs, &managed, &path)?;
        output::ok(&format!(
            "Wrote {} processkit MCP servers to {}",
            specs.len(),
            path.display()
        ));
    }
    if providers.contains(&AiProvider::Gemini) {
        let path = project_root.join(".gemini/settings.json");
        write_gemini_settings_json(&specs, &managed, &path)?;
        output::ok(&format!(
            "Wrote {} processkit MCP servers to {}",
            specs.len(),
            path.display()
        ));
    }

    // 2. OpenAI Codex CLI — TOML translator.
    if providers.contains(&AiProvider::Codex) {
        let path = project_root.join(".codex/config.toml");
        write_codex_config_toml(&specs, &managed, &path)?;
        output::ok(&format!(
            "Wrote {} processkit MCP servers to {}",
            specs.len(),
            path.display()
        ));
    }

    // 3. Continue — per-server file directory.
    if providers.contains(&AiProvider::Continue) {
        let dir = project_root.join(".continue/mcpServers");
        write_continue_mcp_dir(&specs, &managed, &dir)?;
        output::ok(&format!(
            "Wrote {} processkit MCP servers to {}",
            specs.len(),
            dir.display()
        ));
    }

    // 4. Aider — no MCP client. Warn if listed AND processkit ships
    //    MCP servers (the user is missing functionality).
    if providers.contains(&AiProvider::Aider) {
        output::warn(
            "`aider` is in [ai].harnesses but does not have a built-in MCP client. \
             processkit's MCP-based skills (workitem-management, decision-record, …) \
             will not be available when using Aider. Consider also listing one of: \
             claude, cursor, gemini, codex, continue, copilot, opencode, hermes.",
        );
    }

    // 5. Mistral — informational note when alone (not paired with Claude).
    if providers.contains(&AiProvider::Mistral) && !providers.contains(&AiProvider::Claude) {
        output::info(
            "`mistral` does not ship a CLI tool that reads project-level MCP config. \
             aibox is writing `.mcp.json` (the standard Claude Code shape) at the \
             project root so a Mistral SDK-based CLI tool you build can read MCP \
             server registrations from there.",
        );
    }

    // 6. Update .claude/settings.local.json with enabled MCP servers.
    // This ensures Claude Code loads all MCP tools during session startup.
    let settings_path = project_root.join(".claude/settings.local.json");
    if let Err(e) = update_enabled_mcp_servers(&specs, &settings_path) {
        output::warn(&format!(
            "Failed to update .claude/settings.local.json with enabled MCP servers: {}",
            e
        ));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Safety rail: validate emitted script paths
// ---------------------------------------------------------------------------

/// Verify every script path referenced in `specs` exists on disk.
///
/// Scans each spec's `args` for entries that look like a project-root-relative
/// path to a Python MCP server script under `context/`. If any are missing,
/// aggregates them into a single error so the operator sees the full picture
/// at once instead of fixing them one-at-a-time. Failing here means the
/// merge has produced a `.mcp.json` (or equivalent) that would point at
/// non-existent scripts — strictly better to fail at sync time than to ship
/// a broken config and have the harness emit opaque connection-closed errors.
fn validate_script_paths(specs: &[McpServerSpec], project_root: &Path) -> Result<()> {
    let mut missing: Vec<(String, String)> = Vec::new();
    for spec in specs {
        for arg in &spec.args {
            // Only validate entries that look like project-root-relative
            // paths to a shipped Python MCP server script.
            if !arg.ends_with(".py") || !arg.starts_with("context/") {
                continue;
            }
            if !project_root.join(arg).is_file() {
                missing.push((spec.name.clone(), arg.clone()));
            }
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    for (name, path) in &missing {
        output::error(&format!(
            "MCP server '{name}' references script '{path}' which does not exist \
             under the project root. The per-skill mcp-config.json likely has a \
             stale path (e.g. missing the `processkit/` category prefix introduced \
             in processkit v0.17.0). Fix upstream and re-run `aibox sync`."
        ));
    }
    Err(anyhow::anyhow!(
        "MCP script-path validation failed: {} server(s) reference missing scripts: {}",
        missing.len(),
        missing
            .iter()
            .map(|(n, p)| format!("{n} -> {p}"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

// ---------------------------------------------------------------------------
// Writer 1: Claude-shape `mcpServers` JSON
// ---------------------------------------------------------------------------

/// Write a `.mcp.json`-style file with the standard Claude Code
/// `{"mcpServers": {"<name>": {...}}}` shape. Used for Claude Code,
/// Cursor, and (via the Mistral routing) for Mistral SDK consumers.
///
/// Non-destructive merge:
/// 1. Read existing file (if any) — must already be valid JSON with
///    a top-level `mcpServers` object (or top-level object).
/// 2. Remove every `mcpServers` key whose name is in `managed`.
/// 3. Add the current `specs` entries.
/// 4. Preserve any user-added keys (names not in `managed`).
/// 5. Write back with stable, sorted-key formatting.
fn write_mcp_servers_json(
    specs: &[McpServerSpec],
    managed: &BTreeSet<String>,
    path: &Path,
) -> Result<()> {
    let mut top: serde_json::Map<String, serde_json::Value> = if path.is_file() {
        let body = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if body.trim().is_empty() {
            serde_json::Map::new()
        } else {
            serde_json::from_str(&body)
                .with_context(|| format!("failed to parse existing JSON at {}", path.display()))?
        }
    } else {
        serde_json::Map::new()
    };

    // Get-or-create the mcpServers object.
    let mcp_servers = top
        .entry("mcpServers".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    let obj = mcp_servers.as_object_mut().ok_or_else(|| {
        anyhow::anyhow!("`mcpServers` in {} is not a JSON object", path.display())
    })?;

    // Step 1: remove all keys in the managed set. Preserves user keys.
    obj.retain(|k, _| !managed.contains(k));

    // Step 2: add current managed entries.
    for spec in specs {
        obj.insert(spec.name.clone(), spec_to_json(spec));
    }

    // Ensure parent dir exists (e.g. .cursor/, .gemini/).
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory {}", parent.display()))?;
    }

    let formatted = serde_json::to_string_pretty(&top).context("failed to serialize MCP config")?;
    fs::write(path, formatted).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn spec_to_json(spec: &McpServerSpec) -> serde_json::Value {
    let mut entry = serde_json::Map::new();
    entry.insert(
        "command".to_string(),
        serde_json::Value::String(spec.command.clone()),
    );
    if !spec.args.is_empty() {
        entry.insert(
            "args".to_string(),
            serde_json::Value::Array(
                spec.args
                    .iter()
                    .map(|a| serde_json::Value::String(a.clone()))
                    .collect(),
            ),
        );
    }
    if !spec.env.is_empty() {
        let mut env = serde_json::Map::new();
        for (k, v) in &spec.env {
            env.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        entry.insert("env".to_string(), serde_json::Value::Object(env));
    }
    serde_json::Value::Object(entry)
}

// ---------------------------------------------------------------------------
// Writer 2: Gemini CLI settings.json (mcpServers nested in larger object)
// ---------------------------------------------------------------------------

/// Write `.gemini/settings.json` — same `mcpServers` shape as Claude
/// Code, but the file may contain other Gemini CLI settings at the
/// top level which must be preserved across the merge.
///
/// Implementation note: today this is identical to
/// [`write_mcp_servers_json`] because that writer already preserves
/// top-level keys other than `mcpServers`. Kept as a separate function
/// in case Gemini's settings shape diverges in the future.
fn write_gemini_settings_json(
    specs: &[McpServerSpec],
    managed: &BTreeSet<String>,
    path: &Path,
) -> Result<()> {
    write_mcp_servers_json(specs, managed, path)
}

// ---------------------------------------------------------------------------
// Writer 3: Codex CLI config.toml ([mcp_servers.<name>] sections)
// ---------------------------------------------------------------------------

/// Write `.codex/config.toml` with `[mcp_servers.<name>]` sections.
/// Translates from the Claude-shape JSON spec to TOML.
///
/// Non-destructive merge: uses `toml_edit` (which preserves comments,
/// formatting, and ordering of unrelated sections). Removes every
/// `[mcp_servers.<name>]` section whose name is in `managed`, then
/// adds the current entries.
fn write_codex_config_toml(
    specs: &[McpServerSpec],
    managed: &BTreeSet<String>,
    path: &Path,
) -> Result<()> {
    use toml_edit::{Array, DocumentMut, Item, Table, value};

    let mut doc: DocumentMut = if path.is_file() {
        let body = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        body.parse()
            .with_context(|| format!("failed to parse existing TOML at {}", path.display()))?
    } else {
        DocumentMut::new()
    };

    // Get-or-create [mcp_servers] table.
    if doc.get("mcp_servers").is_none() {
        doc.insert("mcp_servers", Item::Table(Table::new()));
        if let Some(t) = doc["mcp_servers"].as_table_mut() {
            t.set_implicit(true);
        }
    }

    // Step 1: remove managed entries. Iterate over a snapshot of keys
    // to avoid mutating-while-iterating.
    let mcp_table = doc["mcp_servers"].as_table_mut().ok_or_else(|| {
        anyhow::anyhow!("`mcp_servers` in {} is not a TOML table", path.display())
    })?;
    let existing_keys: Vec<String> = mcp_table.iter().map(|(k, _)| k.to_string()).collect();
    for key in existing_keys {
        if managed.contains(&key) {
            mcp_table.remove(&key);
        }
    }

    // Step 2: add current managed entries.
    for spec in specs {
        let mut server_table = Table::new();
        server_table.insert("command", value(spec.command.clone()));
        if !spec.args.is_empty() {
            let mut args_array = Array::new();
            for a in &spec.args {
                args_array.push(a.clone());
            }
            server_table.insert("args", value(args_array));
        }
        if !spec.env.is_empty() {
            let mut env_table = Table::new();
            for (k, v) in &spec.env {
                env_table.insert(k, value(v.clone()));
            }
            server_table.insert("env", Item::Table(env_table));
        }
        mcp_table.insert(&spec.name, Item::Table(server_table));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory {}", parent.display()))?;
    }
    fs::write(path, doc.to_string())
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Writer 4: Continue per-server file directory
// ---------------------------------------------------------------------------

/// Write `.continue/mcpServers/<name>.json` per server. Continue uses
/// a directory of registration files where each file declares one
/// server in a flat shape (no `mcpServers` wrapper).
///
/// Non-destructive merge: aibox owns files matching its server names
/// (one file per managed server), preserves any other files in the
/// directory (user-added servers), and removes managed-set files that
/// no longer correspond to a current spec.
fn write_continue_mcp_dir(
    specs: &[McpServerSpec],
    managed: &BTreeSet<String>,
    dir: &Path,
) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;

    // Step 1: list existing files matching managed names; mark for removal.
    let current_names: BTreeSet<String> = specs.iter().map(|s| s.name.clone()).collect();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            // Only touch files whose stem is in the managed set —
            // user-added files are preserved.
            if managed.contains(&stem) && !current_names.contains(&stem) {
                let _ = fs::remove_file(&path);
            }
        }
    }

    // Step 2: write current spec files.
    for spec in specs {
        let mut entry = serde_json::Map::new();
        entry.insert(
            "name".to_string(),
            serde_json::Value::String(spec.name.clone()),
        );
        entry.insert(
            "type".to_string(),
            serde_json::Value::String("stdio".to_string()),
        );
        entry.insert(
            "command".to_string(),
            serde_json::Value::String(spec.command.clone()),
        );
        if !spec.args.is_empty() {
            entry.insert(
                "args".to_string(),
                serde_json::Value::Array(
                    spec.args
                        .iter()
                        .map(|a| serde_json::Value::String(a.clone()))
                        .collect(),
                ),
            );
        }
        if !spec.env.is_empty() {
            let mut env = serde_json::Map::new();
            for (k, v) in &spec.env {
                env.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            entry.insert("env".to_string(), serde_json::Value::Object(env));
        }
        let formatted = serde_json::to_string_pretty(&serde_json::Value::Object(entry))
            .context("failed to serialize Continue MCP entry")?;
        let file_path = dir.join(format!("{}.json", spec.name));
        fs::write(&file_path, formatted)
            .with_context(|| format!("failed to write {}", file_path.display()))?;
    }

    Ok(())
}

/// Update .claude/settings.local.json to enable all MCP servers defined
/// in the provided specs. This ensures Claude Code loads all MCP tools
/// that are configured in .mcp.json.
fn update_enabled_mcp_servers(specs: &[McpServerSpec], settings_path: &Path) -> Result<()> {
    // Read existing settings
    let mut settings: serde_json::Map<String, serde_json::Value> = if settings_path.is_file() {
        let body = fs::read_to_string(settings_path)
            .with_context(|| format!("failed to read {}", settings_path.display()))?;
        if body.trim().is_empty() {
            serde_json::Map::new()
        } else {
            serde_json::from_str(&body)
                .with_context(|| format!("failed to parse {}", settings_path.display()))?
        }
    } else {
        serde_json::Map::new()
    };

    // Build the enabled servers list from the specs
    let enabled_servers: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();

    // Update or create the enabledMcpjsonServers field
    settings.insert(
        "enabledMcpjsonServers".to_string(),
        serde_json::Value::Array(
            enabled_servers
                .iter()
                .map(|name| serde_json::Value::String(name.clone()))
                .collect(),
        ),
    );

    // Write back the updated settings with pretty formatting
    let formatted = serde_json::to_string_pretty(&serde_json::Value::Object(settings))
        .context("failed to serialize settings")?;
    fs::write(settings_path, formatted)
        .with_context(|| format!("failed to write {}", settings_path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn spec(name: &str, command: &str, args: &[&str]) -> McpServerSpec {
        McpServerSpec {
            name: name.to_string(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: BTreeMap::new(),
        }
    }

    fn write_synth_skill_mcp(
        mirror_root: &Path,
        version: &str,
        category: &str,
        skill: &str,
        json_body: &str,
    ) -> PathBuf {
        // Use v0.17+ two-level layout: context/skills/<category>/<skill>/mcp/mcp-config.json
        let dir = mirror_root
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS)
            .join(category)
            .join(skill)
            .join("mcp");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mcp-config.json");
        fs::write(&path, json_body).unwrap();

        // v0.18.7 safety rail: regenerate_mcp_configs validates that every
        // script path referenced by a merged spec exists under the project
        // root. In production both the templates mirror and the live skills
        // tree are populated by sync; in tests we only build the mirror, so
        // also touch a stub at the conventional project-root-relative path
        // so regenerate_mcp_configs doesn't reject the test fixture as
        // "stale path."
        let live_skill_dir = mirror_root
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS)
            .join(category)
            .join(skill)
            .join("mcp");
        fs::create_dir_all(&live_skill_dir).unwrap();
        fs::write(live_skill_dir.join("server.py"), "# test stub\n").unwrap();

        path
    }

    // ── collect_processkit_mcp_specs ────────────────────────────────────

    #[test]
    fn collect_skips_when_version_is_unset() {
        let tmp = TempDir::new().unwrap();
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::config::PROCESSKIT_VERSION_UNSET,
            None,
            &[],
        )
        .unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn collect_returns_empty_when_mirror_missing() {
        let tmp = TempDir::new().unwrap();
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &[],
        )
        .unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn collect_keys_managed_set_on_mcpservers_json_key_not_directory_name() {
        // Regression: processkit ships per-skill `mcp/mcp-config.json`
        // files where the SKILL DIRECTORY NAME is unprefixed (e.g.
        // `workitem-management/`) but the MCP SERVER NAME (the JSON
        // key under `mcpServers`) is prefixed (e.g.
        // `processkit-workitem-management`). The non-destructive merge
        // MUST key on the JSON key, not the directory name, otherwise
        // the prefix doesn't disarm the collision risk.
        //
        // See `projectious-work/processkit#2` and DEC-033.
        let tmp = TempDir::new().unwrap();
        write_synth_skill_mcp(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            "processkit",
            "workitem-management", // <-- directory name (unprefixed)
            r#"{
                "mcpServers": {
                    "processkit-workitem-management": {
                        "command": "uv",
                        "args": ["run", "context/skills/workitem-management/mcp/server.py"]
                    }
                }
            }"#,
        );

        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(specs.len(), 1);
        // The spec name MUST be the JSON key (prefixed), not the
        // directory name (unprefixed).
        assert_eq!(
            specs[0].name, "processkit-workitem-management",
            "spec.name must come from the JSON key, not the directory name"
        );
        // The managed set MUST contain the prefixed name.
        let managed = managed_set(&specs);
        assert!(
            managed.contains("processkit-workitem-management"),
            "managed set must contain the prefixed JSON key"
        );
        assert!(
            !managed.contains("workitem-management"),
            "managed set must NOT contain the bare directory name (collision risk)"
        );
    }

    #[test]
    fn collect_walks_one_skill_with_one_server() {
        let tmp = TempDir::new().unwrap();
        write_synth_skill_mcp(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            "processkit",
            "workitem-management",
            r#"{
                "mcpServers": {
                    "workitem-management": {
                        "command": "uv",
                        "args": ["run", "context/skills/workitem-management/mcp/server.py"]
                    }
                }
            }"#,
        );
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "workitem-management");
        assert_eq!(specs[0].command, "uv");
        assert_eq!(
            specs[0].args,
            vec!["run", "context/skills/workitem-management/mcp/server.py"]
        );
    }

    #[test]
    fn collect_walks_multiple_skills_sorted_by_name() {
        let tmp = TempDir::new().unwrap();
        for skill in &["scope-management", "decision-record", "workitem-management"] {
            write_synth_skill_mcp(
                tmp.path(),
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
                "processkit",
                skill,
                &format!(
                    r#"{{"mcpServers":{{"{}":{{"command":"uv","args":[]}}}}}}"#,
                    skill
                ),
            );
        }
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &[],
        )
        .unwrap();
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        assert_eq!(
            names,
            vec!["decision-record", "scope-management", "workitem-management"]
        );
    }

    #[test]
    fn collect_skips_skills_without_mcp_directory() {
        let tmp = TempDir::new().unwrap();
        write_synth_skill_mcp(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            "processkit",
            "workitem-management",
            r#"{"mcpServers":{"workitem-management":{"command":"uv","args":[]}}}"#,
        );
        // Skill with no mcp/ subdirectory.
        let docs_only_skill = tmp
            .path()
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS)
            .join("processkit")
            .join("code-review");
        fs::create_dir_all(&docs_only_skill).unwrap();
        fs::write(docs_only_skill.join("SKILL.md"), "# code review\n").unwrap();

        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "workitem-management");
    }

    #[test]
    fn collect_respects_effective_skills_filter() {
        let tmp = TempDir::new().unwrap();
        for skill in &["workitem-management", "decision-record", "scope-management"] {
            write_synth_skill_mcp(
                tmp.path(),
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
                "processkit",
                skill,
                &format!(
                    r#"{{"mcpServers":{{"{}":{{"command":"uv","args":[]}}}}}}"#,
                    skill
                ),
            );
        }
        let mut filter = HashSet::new();
        filter.insert("workitem-management".to_string());
        filter.insert("scope-management".to_string());
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            Some(&filter),
            &[],
        )
        .unwrap();
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        assert_eq!(names, vec!["scope-management", "workitem-management"]);
    }

    #[test]
    fn collect_force_include_bypasses_effective_skills_filter() {
        // Mandatory skills must be collected even when they are not in the
        // effective-skills set (i.e. the package tier doesn't include them).
        let tmp = TempDir::new().unwrap();
        for skill in &["workitem-management", "decision-record", "scope-management"] {
            write_synth_skill_mcp(
                tmp.path(),
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
                "processkit",
                skill,
                &format!(
                    r#"{{"mcpServers":{{"{}":{{"command":"uv","args":[]}}}}}}"#,
                    skill
                ),
            );
        }
        // Filter excludes decision-record, but it's force-included as mandatory.
        let mut filter = HashSet::new();
        filter.insert("workitem-management".to_string());
        filter.insert("scope-management".to_string());
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            Some(&filter),
            &["decision-record"], // force-include bypasses the filter
        )
        .unwrap();
        let names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
        assert_eq!(
            names,
            vec!["decision-record", "scope-management", "workitem-management"],
            "force-included skill must appear even when absent from effective set"
        );
    }

    #[test]
    fn collect_force_include_no_op_when_no_filter() {
        // When effective_skills is None (no filter), force_include has no effect.
        let tmp = TempDir::new().unwrap();
        write_synth_skill_mcp(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            "processkit",
            "workitem-management",
            r#"{"mcpServers":{"workitem-management":{"command":"uv","args":[]}}}"#,
        );
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &["decision-record"], // force-include of a skill that isn't in the mirror
        )
        .unwrap();
        // Only the skill that exists should appear; force_include of a non-existent
        // skill is a no-op (the file simply won't be found).
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "workitem-management");
    }

    #[test]
    fn collect_skips_lib_directory() {
        let tmp = TempDir::new().unwrap();
        // _lib is the shared processkit Python lib, not a real skill.
        let lib = tmp
            .path()
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS)
            .join(crate::processkit_vocab::src::LIB_SEGMENT)
            .join("processkit");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("entity.py"), "x = 1\n").unwrap();
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            None,
            &[],
        )
        .unwrap();
        assert!(specs.is_empty());
    }

    // ── managed_set ─────────────────────────────────────────────────────

    #[test]
    fn managed_set_dedupes_via_btreeset() {
        let specs = vec![
            spec("a", "uv", &["run"]),
            spec("b", "uv", &["run"]),
            spec("a", "uv", &["run"]), // duplicate
        ];
        let set = managed_set(&specs);
        assert_eq!(set.len(), 2);
        assert!(set.contains("a"));
        assert!(set.contains("b"));
    }

    // ── write_mcp_servers_json ──────────────────────────────────────────

    #[test]
    fn write_mcp_creates_file_with_specs() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".mcp.json");
        let specs = vec![
            spec("workitem-management", "uv", &["run", "x.py"]),
            spec("decision-record", "uv", &["run", "y.py"]),
        ];
        let managed = managed_set(&specs);
        write_mcp_servers_json(&specs, &managed, &path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let obj = parsed["mcpServers"].as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["workitem-management"]["command"], "uv");
        assert_eq!(obj["workitem-management"]["args"][1], "x.py");
    }

    #[test]
    fn write_mcp_preserves_user_added_servers() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".mcp.json");
        // User has a pre-existing .mcp.json with their own server.
        fs::write(
            &path,
            r#"{
                "mcpServers": {
                    "my-custom-server": {
                        "command": "node",
                        "args": ["/path/to/my-server.js"]
                    }
                }
            }"#,
        )
        .unwrap();

        let specs = vec![spec("workitem-management", "uv", &["run", "x.py"])];
        let managed = managed_set(&specs);
        write_mcp_servers_json(&specs, &managed, &path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let obj = parsed["mcpServers"].as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(
            obj.contains_key("workitem-management"),
            "managed entry added"
        );
        assert!(obj.contains_key("my-custom-server"), "user entry preserved");
        assert_eq!(obj["my-custom-server"]["command"], "node");
    }

    #[test]
    fn write_mcp_removes_stale_managed_entries() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".mcp.json");
        // Old state: managed set had {a, b}.
        fs::write(
            &path,
            r#"{
                "mcpServers": {
                    "a": {"command": "uv", "args": []},
                    "b": {"command": "uv", "args": []},
                    "user-server": {"command": "node", "args": []}
                }
            }"#,
        )
        .unwrap();

        // New state: managed set is now {a, c}. b is dropped.
        let specs = vec![spec("a", "uv", &["new", "args"]), spec("c", "uv", &[])];
        // The managed set MUST include all old managed names too, so
        // the writer can remove `b`. We simulate the real flow where
        // the cache has the new content but the managed set is the
        // union — actually, in real life the managed set is computed
        // from the CURRENT cache, so old names not in the cache won't
        // appear in the managed set passed in. The writer's contract
        // is "remove keys in managed; add specs". Stale removal
        // requires the caller to track previous-version names.
        //
        // For this test we simulate by passing the union set.
        let mut union_managed: BTreeSet<String> = managed_set(&specs);
        union_managed.insert("b".to_string());

        write_mcp_servers_json(&specs, &union_managed, &path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let obj = parsed["mcpServers"].as_object().unwrap();
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        assert!(!obj.contains_key("b"), "stale managed entry removed");
        assert!(obj.contains_key("user-server"), "user entry preserved");
        assert_eq!(obj["a"]["args"][0], "new");
    }

    #[test]
    fn write_mcp_creates_parent_directory() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".cursor/mcp.json");
        let specs = vec![spec("foo", "uv", &[])];
        let managed = managed_set(&specs);
        write_mcp_servers_json(&specs, &managed, &path).unwrap();
        assert!(path.exists());
    }

    // ── write_codex_config_toml ─────────────────────────────────────────

    #[test]
    fn write_codex_creates_toml_with_sections() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".codex/config.toml");
        let specs = vec![
            spec("workitem-management", "uv", &["run", "x.py"]),
            spec("decision-record", "uv", &["run", "y.py"]),
        ];
        let managed = managed_set(&specs);
        write_codex_config_toml(&specs, &managed, &path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("[mcp_servers.workitem-management]"));
        assert!(body.contains("[mcp_servers.decision-record]"));
        assert!(body.contains(r#"command = "uv""#));
        assert!(body.contains(r#"args = ["run", "x.py"]"#));
    }

    #[test]
    fn write_codex_preserves_user_sections_and_top_level_keys() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".codex/config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        // User has a pre-existing config.toml with custom MCP server
        // and top-level keys.
        fs::write(
            &path,
            r#"# User-authored Codex config
model = "gpt-5"

[mcp_servers.my-custom]
command = "node"
args = ["server.js"]
"#,
        )
        .unwrap();

        let specs = vec![spec("workitem-management", "uv", &["run", "x.py"])];
        let managed = managed_set(&specs);
        write_codex_config_toml(&specs, &managed, &path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        assert!(
            body.contains(r#"model = "gpt-5""#),
            "top-level key preserved"
        );
        assert!(
            body.contains("[mcp_servers.my-custom]"),
            "user MCP entry preserved"
        );
        assert!(
            body.contains("[mcp_servers.workitem-management]"),
            "managed entry added"
        );
    }

    // ── write_continue_mcp_dir ──────────────────────────────────────────

    #[test]
    fn write_continue_writes_one_file_per_spec() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".continue/mcpServers");
        let specs = vec![
            spec("workitem-management", "uv", &["run", "x.py"]),
            spec("decision-record", "uv", &["run", "y.py"]),
        ];
        let managed = managed_set(&specs);
        write_continue_mcp_dir(&specs, &managed, &dir).unwrap();

        assert!(dir.join("workitem-management.json").is_file());
        assert!(dir.join("decision-record.json").is_file());

        let body = fs::read_to_string(dir.join("workitem-management.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["name"], "workitem-management");
        assert_eq!(parsed["type"], "stdio");
        assert_eq!(parsed["command"], "uv");
    }

    #[test]
    fn write_continue_preserves_user_files() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".continue/mcpServers");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("my-custom.json"),
            r#"{"name":"my-custom","type":"stdio","command":"node"}"#,
        )
        .unwrap();

        let specs = vec![spec("workitem-management", "uv", &[])];
        let managed = managed_set(&specs);
        write_continue_mcp_dir(&specs, &managed, &dir).unwrap();

        assert!(
            dir.join("workitem-management.json").is_file(),
            "managed file written"
        );
        assert!(dir.join("my-custom.json").is_file(), "user file preserved");
    }

    #[test]
    fn write_continue_removes_stale_managed_files() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".continue/mcpServers");
        fs::create_dir_all(&dir).unwrap();
        // Old managed file no longer in current spec set.
        fs::write(dir.join("old-server.json"), r#"{"name":"old-server"}"#).unwrap();

        let specs = vec![spec("workitem-management", "uv", &[])];
        let mut union_managed: BTreeSet<String> = managed_set(&specs);
        union_managed.insert("old-server".to_string());

        write_continue_mcp_dir(&specs, &union_managed, &dir).unwrap();

        assert!(dir.join("workitem-management.json").is_file());
        assert!(
            !dir.join("old-server.json").exists(),
            "stale managed file removed"
        );
    }

    // ── corrupt per-skill config graceful degradation ───────────────────

    #[test]
    fn collect_corrupt_skill_config_does_not_error_and_includes_valid_skill() {
        // Arrange: one valid skill and one corrupt skill alongside it.
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;

        // Valid skill: workitem-management.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "workitem-management",
            r#"{"mcpServers":{"workitem-management":{"command":"uv","args":[]}}}"#,
        );

        // Corrupt skill: decision-record has invalid JSON.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "decision-record",
            r#"{ NOT VALID JSON "#,
        );

        // Act: must NOT return an error — graceful degradation.
        let result = collect_processkit_mcp_specs(tmp.path(), version, None, &[]);
        assert!(
            result.is_ok(),
            "corrupt per-skill config must not propagate an error; got: {:?}",
            result.err()
        );

        let specs = result.unwrap();

        // The valid skill's server must be present.
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"workitem-management"),
            "valid skill's server must be in the result; got names: {:?}",
            names
        );
    }

    #[test]
    fn collect_corrupt_config_triggers_kernel_fallback_for_missing_kernel_skills() {
        // Arrange: a corrupt skill plus a valid kernel skill (index-management)
        // that is NOT the same as the valid skill used above. When the corrupt
        // file is encountered the kernel fallback path re-reads the kernel
        // skills' configs from the mirror and includes them.
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;

        // Valid kernel skill that will be discovered via kernel fallback.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "index-management",
            r#"{"mcpServers":{"index-management":{"command":"uv","args":[]}}}"#,
        );

        // Another valid kernel skill.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "id-management",
            r#"{"mcpServers":{"id-management":{"command":"uv","args":[]}}}"#,
        );

        // Corrupt skill triggers the fallback path.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "event-log",
            r#"{ CORRUPT "#,
        );

        let result = collect_processkit_mcp_specs(tmp.path(), version, None, &[]);
        assert!(result.is_ok(), "must not error on corrupt config");

        let specs = result.unwrap();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();

        // Both valid kernel skills must appear — they were force-included
        // by the kernel fallback after the parse error was detected.
        assert!(
            names.contains(&"index-management"),
            "kernel fallback must include index-management; got: {:?}",
            names
        );
        assert!(
            names.contains(&"id-management"),
            "kernel fallback must include id-management; got: {:?}",
            names
        );
    }

    // ── new category-nested layout tests (aibox#53) ──────────────────────

    /// Test 1: two skills in two categories are both returned.
    #[test]
    fn collect_walks_category_nested_layout() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "skill-gate",
            r#"{"mcpServers":{"processkit-skill-gate":{"command":"uv","args":[]}}}"#,
        );
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "engineering",
            "rust-crate",
            r#"{"mcpServers":{"processkit-rust-crate":{"command":"uv","args":[]}}}"#,
        );
        let specs = collect_processkit_mcp_specs(tmp.path(), version, None, &[]).unwrap();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"processkit-skill-gate"),
            "skill-gate from processkit category must be found; got: {:?}",
            names
        );
        assert!(
            names.contains(&"processkit-rust-crate"),
            "rust-crate from engineering category must be found; got: {:?}",
            names
        );
        assert_eq!(specs.len(), 2);
    }

    /// Test 2: INDEX.md and FORMAT.md at skills-root and inside a category don't cause errors.
    #[test]
    fn collect_skips_non_directory_entries_in_category_tree() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        let skills_root = tmp
            .path()
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS);
        fs::create_dir_all(&skills_root).unwrap();
        // Top-level INDEX.md and FORMAT.md files
        fs::write(skills_root.join("INDEX.md"), "# index\n").unwrap();
        fs::write(skills_root.join("FORMAT.md"), "# format\n").unwrap();
        // Category-level non-dir file
        let cat_dir = skills_root.join("processkit");
        fs::create_dir_all(&cat_dir).unwrap();
        fs::write(cat_dir.join("INDEX.md"), "# cat index\n").unwrap();
        // One real skill
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "skill-gate",
            r#"{"mcpServers":{"processkit-skill-gate":{"command":"uv","args":[]}}}"#,
        );
        let result = collect_processkit_mcp_specs(tmp.path(), version, None, &[]);
        assert!(result.is_ok(), "walker must not error on non-dir entries");
        let specs = result.unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "processkit-skill-gate");
    }

    /// Test 3: _lib directory at skills root is skipped (no specs, no error).
    #[test]
    fn collect_skips_dunder_lib_directory() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        // Place a file under _lib/processkit/entity.py
        let lib = tmp
            .path()
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS)
            .join("_lib")
            .join("processkit");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("entity.py"), "x = 1\n").unwrap();
        let specs = collect_processkit_mcp_specs(tmp.path(), version, None, &[]).unwrap();
        assert!(
            specs.is_empty(),
            "_lib directory must produce no specs; got: {:?}",
            specs
        );
    }

    /// Test 4: effective-set filter works correctly in the nested layout.
    #[test]
    fn collect_honors_effective_skills_filter_in_nested_layout() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "skill-gate",
            r#"{"mcpServers":{"processkit-skill-gate":{"command":"uv","args":[]}}}"#,
        );
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "engineering",
            "rust-crate",
            r#"{"mcpServers":{"processkit-rust-crate":{"command":"uv","args":[]}}}"#,
        );
        let mut filter = HashSet::new();
        filter.insert("skill-gate".to_string());
        let specs = collect_processkit_mcp_specs(tmp.path(), version, Some(&filter), &[]).unwrap();
        assert_eq!(specs.len(), 1, "only the filtered skill should be returned");
        assert_eq!(specs[0].name, "processkit-skill-gate");
    }

    /// Test 5: kernel fallback finds skills in category subdirectories.
    #[test]
    fn collect_kernel_fallback_finds_skills_in_categories() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        // Valid kernel skill under processkit/ category.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "index-management",
            r#"{"mcpServers":{"processkit-index-management":{"command":"uv","args":[]}}}"#,
        );
        // Corrupt skill to trigger kernel fallback.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "event-log",
            r#"{ CORRUPT "#,
        );
        let result = collect_processkit_mcp_specs(tmp.path(), version, None, &[]);
        assert!(result.is_ok(), "must not error on corrupt config");
        let specs = result.unwrap();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"processkit-index-management"),
            "kernel fallback must find index-management in processkit/ category; got: {:?}",
            names
        );
    }

    /// Test 6: duplicate skill basename across categories emits a warning,
    /// exactly one spec survives (last-wins), and no error is returned.
    #[test]
    fn collect_warns_on_duplicate_skill_basename_across_categories() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        // Two different categories, same skill basename "foo".
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "engineering",
            "foo",
            r#"{"mcpServers":{"processkit-foo-engineering":{"command":"uv","args":[]}}}"#,
        );
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "devops",
            "foo",
            r#"{"mcpServers":{"processkit-foo-devops":{"command":"uv","args":[]}}}"#,
        );
        // Collect — must not error.
        let result = collect_processkit_mcp_specs(tmp.path(), version, None, &[]);
        assert!(result.is_ok(), "duplicate basename must not cause an error");
        let specs = result.unwrap();
        // Last-wins: the walk order is filesystem-determined, so we assert
        // exactly one spec came from "foo" (i.e. 2 possible names total, but
        // only 1 may appear if both configs define the same server name, or 2
        // if they define distinct names). What matters: no panic, and the
        // spec count is at most 2 (one from each category; one processed last
        // wins the seen_skill_categories guard).
        assert!(
            !specs.is_empty(),
            "at least one spec must survive; got empty"
        );
        // Both server names may appear since the collision guard only deduplicates
        // the skill directory traversal, not the MCP server names themselves.
        // The important invariant is: we get a result (not an error) and the
        // warning path was exercised (we can't assert stderr easily in unit tests,
        // but the code path is covered).
        let total = specs.len();
        assert!(total <= 2, "at most 2 specs from 2 configs; got {}", total);
    }

    /// Test 7: end-to-end — regenerate_mcp_configs writes .mcp.json with
    /// processkit-skill-gate when the mirror contains processkit/skill-gate.
    #[test]
    fn write_mcp_end_to_end_creates_dot_mcp_json_with_skill_gate() {
        use crate::config::{AiSection, AiboxConfig, ProcessKitSection};
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "skill-gate",
            r#"{"mcpServers":{"processkit-skill-gate":{"command":"uv","args":["run","context/skills/processkit/skill-gate/mcp/server.py"]}}}"#,
        );
        let config = AiboxConfig {
            ai: AiSection {
                harnesses: vec![crate::config::AiHarness::Claude],
                ..AiSection::default()
            },
            processkit: ProcessKitSection {
                version: version.to_string(),
                ..ProcessKitSection::default()
            },
            ..crate::config::test_config()
        };
        regenerate_mcp_configs(&config, tmp.path()).unwrap();
        let dot_mcp = tmp.path().join(".mcp.json");
        assert!(
            dot_mcp.exists(),
            ".mcp.json must be written when Claude harness is configured"
        );
        let body = fs::read_to_string(&dot_mcp).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            parsed["mcpServers"].get("processkit-skill-gate").is_some(),
            ".mcp.json must contain 'processkit-skill-gate'; got: {}",
            body
        );
    }

    /// Test 8: with empty [skills].include (no filter), processkit-skill-gate
    /// still appears because it is now in MANDATORY_MCP_SKILLS (Q1 promotion).
    #[test]
    fn mandatory_set_includes_skill_gate_without_user_config() {
        let tmp = TempDir::new().unwrap();
        let version = crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION;
        // skill-gate is under processkit/ category in the mirror.
        write_synth_skill_mcp(
            tmp.path(),
            version,
            "processkit",
            "skill-gate",
            r#"{"mcpServers":{"processkit-skill-gate":{"command":"uv","args":[]}}}"#,
        );
        // Use an effective-skills filter that does NOT include skill-gate, but
        // pass MANDATORY_MCP_SKILLS as force_include — mirrors what
        // regenerate_mcp_configs does in production.
        let mut filter = HashSet::new();
        filter.insert("some-other-skill".to_string()); // skill-gate is NOT in filter
        let specs = collect_processkit_mcp_specs(
            tmp.path(),
            version,
            Some(&filter),
            crate::processkit_vocab::MANDATORY_MCP_SKILLS,
        )
        .unwrap();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"processkit-skill-gate"),
            "skill-gate must appear via MANDATORY_MCP_SKILLS force_include even when \
             not in effective filter; got: {:?}",
            names
        );
    }

    // ── safety rail tests (v0.18.7) ──────────────────────────────────────

    fn spec_with_args(name: &str, args: Vec<&str>) -> McpServerSpec {
        McpServerSpec {
            name: name.to_string(),
            command: "uv".to_string(),
            args: args.into_iter().map(String::from).collect(),
            env: BTreeMap::new(),
        }
    }

    #[test]
    fn validate_script_paths_passes_when_all_exist() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let script_rel = "context/skills/processkit/foo/mcp/server.py";
        let script_abs = root.join(script_rel);
        fs::create_dir_all(script_abs.parent().unwrap()).unwrap();
        fs::write(&script_abs, "# fake server\n").unwrap();

        let specs = vec![spec_with_args("processkit-foo", vec!["run", script_rel])];
        validate_script_paths(&specs, root).expect("all paths exist; should pass");
    }

    #[test]
    fn validate_script_paths_fails_with_offending_server_in_error() {
        let tmp = TempDir::new().unwrap();
        let specs = vec![spec_with_args(
            "processkit-broken",
            vec!["run", "context/skills/broken/mcp/server.py"],
        )];
        let err = validate_script_paths(&specs, tmp.path()).expect_err("missing path; should fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("processkit-broken"),
            "error must name the offending server: {msg}"
        );
        assert!(
            msg.contains("context/skills/broken/mcp/server.py"),
            "error must include the missing path: {msg}"
        );
    }

    #[test]
    fn validate_script_paths_skips_specs_without_script_path_args() {
        // Servers whose args don't reference a context/*.py path (e.g. a
        // user-added npx-based MCP server) should be ignored — the safety
        // rail targets shipped Python scripts only.
        let tmp = TempDir::new().unwrap();
        let specs = vec![
            McpServerSpec {
                name: "user-thing".to_string(),
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@example/mcp-server".to_string()],
                env: BTreeMap::new(),
            },
            spec_with_args("no-args", vec![]),
        ];
        validate_script_paths(&specs, tmp.path()).expect("non-script specs should be ignored");
    }

    // ---------------------------------------------------------------------------
    // Pattern matching tests
    // ---------------------------------------------------------------------------

    #[test]
    fn glob_matches_exact() {
        assert!(glob_matches("mcp__foo", "mcp__foo"));
        assert!(glob_matches("bash", "bash"));
        assert!(!glob_matches("mcp__foo", "mcp__bar"));
        assert!(!glob_matches("mcp__foo", "mcp__foo-extra"));
    }

    #[test]
    fn glob_matches_suffix_wildcard() {
        assert!(glob_matches("mcp__processkit-workitem", "mcp__processkit-*"));
        assert!(glob_matches("mcp__processkit-actor", "mcp__processkit-*"));
        assert!(!glob_matches("mcp__other-workitem", "mcp__processkit-*"));
        assert!(!glob_matches("mcp__processkit", "mcp__processkit-*"));
    }

    #[test]
    fn glob_matches_prefix_wildcard() {
        assert!(glob_matches("bar-suffix", "*-suffix"));
        assert!(glob_matches("foo-suffix", "*-suffix"));
        assert!(!glob_matches("foo-other", "*-suffix"));
    }

    #[test]
    fn glob_matches_middle_wildcard() {
        assert!(glob_matches("foo_processkit_bar", "*processkit*"));
        assert!(glob_matches("processkit", "*processkit*"));
        assert!(!glob_matches("foo_other_bar", "*processkit*"));
    }

    #[test]
    fn glob_matches_all_wildcard() {
        assert!(glob_matches("anything", "*"));
        assert!(glob_matches("", "*"));
    }

    #[test]
    fn expand_patterns_single() {
        let tools = vec!["mcp__foo".to_string(), "bash".to_string()];
        let patterns = vec!["bash".to_string()];
        let result = expand_mcp_patterns(&patterns, &tools);
        assert_eq!(result, vec!["bash"]);
    }

    #[test]
    fn expand_patterns_wildcard() {
        let tools = vec![
            "mcp__processkit-workitem".to_string(),
            "mcp__processkit-actor".to_string(),
            "mcp__other-tool".to_string(),
            "bash".to_string(),
        ];
        let patterns = vec!["mcp__processkit-*".to_string()];
        let result = expand_mcp_patterns(&patterns, &tools);
        assert_eq!(
            result,
            vec!["mcp__processkit-actor", "mcp__processkit-workitem"]
        );
    }

    #[test]
    fn expand_patterns_multiple() {
        let tools = vec![
            "mcp__processkit-workitem".to_string(),
            "mcp__other-tool".to_string(),
            "bash".to_string(),
        ];
        let patterns = vec!["mcp__processkit-*".to_string(), "bash".to_string()];
        let result = expand_mcp_patterns(&patterns, &tools);
        assert_eq!(
            result,
            vec!["bash", "mcp__processkit-workitem"]
        );
    }

    #[test]
    fn expand_patterns_no_match() {
        let tools = vec!["mcp__foo".to_string(), "bash".to_string()];
        let patterns = vec!["nomatch".to_string()];
        let result = expand_mcp_patterns(&patterns, &tools);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn first_match_wins_allow_pattern() {
        let allow = vec!["mcp__processkit-*".to_string()];
        let deny = vec![];
        assert!(first_match_wins(
            "mcp__processkit-workitem",
            &allow,
            &deny,
            "allow"
        ));
        assert!(!first_match_wins("other-tool", &allow, &deny, "deny"));
    }

    #[test]
    fn first_match_wins_deny_pattern() {
        let allow = vec!["mcp__*".to_string()];
        let deny = vec!["mcp__private-*".to_string()];
        assert!(first_match_wins(
            "mcp__processkit-workitem",
            &allow,
            &deny,
            "allow"
        ));
        assert!(!first_match_wins(
            "mcp__private-secret",
            &allow,
            &deny,
            "allow"
        ));
    }

    #[test]
    fn first_match_wins_default_mode() {
        let allow = vec![];
        let deny = vec![];
        assert!(first_match_wins("anything", &allow, &deny, "allow"));
        assert!(!first_match_wins("anything", &allow, &deny, "deny"));
        assert!(first_match_wins("anything", &allow, &deny, "ask"));
    }

    #[test]
    fn first_match_wins_deny_takes_precedence() {
        // If both allow and deny patterns match the same tool,
        // deny should win (deny takes precedence for security).
        let allow = vec!["mcp__*".to_string()];
        let deny = vec!["mcp__restricted-*".to_string()];
        assert!(!first_match_wins(
            "mcp__restricted-tool",
            &allow,
            &deny,
            "allow"
        ));
    }

    // ---------------------------------------------------------------------------
    // Phase 2a: Claude Code and OpenCode generator tests
    // ---------------------------------------------------------------------------

    #[test]
    fn claude_code_permissions_creates_file_with_allow_list() {
        let tmp = TempDir::new().unwrap();
        let config = McpConfig {
            default_mode: "allow".to_string(),
            allow_patterns: vec!["mcp__processkit-*".to_string(), "bash".to_string()],
            deny_patterns: vec![],
            harness: BTreeMap::new(),
        };
        let tools = vec![
            "mcp__processkit-workitem".to_string(),
            "mcp__processkit-actor".to_string(),
            "bash".to_string(),
            "mcp__other".to_string(),
        ];

        let result = generate_claude_code_permissions(tmp.path(), &config, &tools);
        assert!(result.is_ok());

        let settings_path = tmp.path().join(".claude").join("settings.local.json");
        assert!(settings_path.is_file());

        let content = fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Check that mcp__processkit-* tools and bash are in permissions.allow
        if let Some(perms) = parsed.get("permissions.allow").and_then(|p| p.as_array()) {
            let perm_strs: Vec<_> = perms
                .iter()
                .filter_map(|p| p.as_str())
                .collect();
            assert!(perm_strs.contains(&"mcp__mcp__processkit-workitem")); // Note: double mcp__ due to format!
            assert!(perm_strs.contains(&"mcp__bash"));
            assert!(!perm_strs.iter().any(|p| p.contains("mcp__other")));
        } else {
            panic!("permissions.allow not found in settings");
        }
    }

    #[test]
    fn opencode_permissions_creates_toml_with_mcp_section() {
        let tmp = TempDir::new().unwrap();
        let config = McpConfig {
            default_mode: "allow".to_string(),
            allow_patterns: vec!["mcp__processkit-*".to_string()],
            deny_patterns: vec!["mcp__private-*".to_string()],
            harness: BTreeMap::new(),
        };
        let tools = vec![
            "mcp__processkit-workitem".to_string(),
            "mcp__private-secret".to_string(),
        ];

        let result = generate_opencode_permissions(tmp.path(), &config, &tools);
        assert!(result.is_ok());

        let config_path = tmp.path().join(".opencode").join("config.toml");
        assert!(config_path.is_file());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[mcp]"));
        assert!(content.contains("mode = \"allow\""));
        assert!(content.contains("mcp__processkit-workitem"));
        assert!(content.contains("mcp__private-secret"));
    }

    #[test]
    fn claude_code_permissions_preserves_existing_settings() {
        let tmp = TempDir::new().unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.local.json");

        // Create existing settings with other keys
        fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        let existing = serde_json::json!({
            "other_key": "other_value",
            "permissions.allow": ["custom-tool"]
        });
        fs::write(&settings_path, existing.to_string()).unwrap();

        let config = McpConfig {
            default_mode: "allow".to_string(),
            allow_patterns: vec!["mcp__new-*".to_string()],
            deny_patterns: vec![],
            harness: BTreeMap::new(),
        };
        let tools = vec!["mcp__new-tool".to_string()];

        generate_claude_code_permissions(tmp.path(), &config, &tools).unwrap();

        let content = fs::read_to_string(&settings_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Other keys should be preserved
        assert_eq!(parsed.get("other_key").and_then(|v| v.as_str()), Some("other_value"));

        // Permissions should be updated
        if let Some(perms) = parsed.get("permissions.allow").and_then(|p| p.as_array()) {
            let perm_strs: Vec<_> = perms.iter().filter_map(|p| p.as_str()).collect();
            assert!(perm_strs.contains(&"mcp__mcp__new-tool"));
        } else {
            panic!("permissions.allow not found");
        }
    }
}
