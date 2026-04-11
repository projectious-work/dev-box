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

use std::collections::{BTreeMap, BTreeSet, HashSet};
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
// Walk the templates mirror to compute the managed set
// ---------------------------------------------------------------------------

/// Walk `context/templates/processkit/<version>/skills/*/mcp/mcp-config.json`
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

    for entry in fs::read_dir(&mirror_skills_dir).with_context(|| {
        format!(
            "failed to read templates mirror at {}",
            mirror_skills_dir.display()
        )
    })? {
        let entry = entry?;
        let skill_dir = entry.path();
        let skill_name = match entry.file_name().to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        // Skip non-skill entries (e.g. _lib, INDEX.md, FORMAT.md).
        if skill_name.starts_with('_') || skill_name.starts_with('.') {
            continue;
        }
        if !skill_dir.is_dir() {
            continue;
        }
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
        let parsed: PerSkillConfig = serde_json::from_str(&body)
            .with_context(|| format!("failed to parse JSON from {}", config_path.display()))?;

        for (name, raw) in parsed.mcp_servers {
            specs.push(McpServerSpec {
                name,
                command: raw.command,
                args: raw.args,
                env: raw.env,
            });
        }
    }

    // Stable order: sort by name. Makes the output deterministic.
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

    // Build the full spec list: processkit first, then team-shared
    // (aibox.toml [mcp.servers]), then personal (.aibox-local.toml
    // [mcp.servers]).  All three sources are "aibox-managed" — they
    // are in the managed set and get refreshed on every sync so that
    // removals from any source are reflected immediately.
    let mut specs: Vec<McpServerSpec> = processkit_specs;
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
    if let Some(skills_dir) = crate::processkit_vocab::mirror_skills_dir(project_root, &config.processkit.version) {
        let registered_names: std::collections::HashSet<&str> =
            specs.iter().map(|s| s.name.as_str()).collect();
        for &skill in crate::processkit_vocab::MANDATORY_MCP_SKILLS {
            // Convention: the server name shipped in mcp-config.json is
            // `processkit-{skill}`. Check whether any registered spec
            // matches; fall back to checking the mcp-config.json file
            // exists so we catch cases where the naming convention changes.
            let expected_server = format!("processkit-{skill}");
            let config_path = skills_dir.join(skill).join("mcp").join("mcp-config.json");
            if !registered_names.contains(expected_server.as_str()) && !config_path.is_file() {
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

    let managed = managed_set(&specs);
    let providers: HashSet<&AiProvider> = config.ai.providers.iter().collect();

    // 1. Claude / Mistral / Copilot use the Claude-shape `mcpServers` JSON
    //    object at `.mcp.json`. Mistral routes there so SDK consumers can
    //    read it; Copilot CLI reads `.mcp.json` natively.
    let writes_dot_mcp_json = providers.contains(&AiProvider::Claude)
        || providers.contains(&AiProvider::Mistral)
        || providers.contains(&AiProvider::Copilot);
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

    // 2. Codex — TOML translator.
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
            "`aider` is in [ai].providers but does not have a built-in MCP client. \
             processkit's MCP-based skills (workitem-management, decision-record, …) \
             will not be available when using Aider. Consider also listing one of: \
             claude, cursor, gemini, codex, continue, copilot, mistral.",
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

    Ok(())
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
        skill: &str,
        json_body: &str,
    ) -> PathBuf {
        // Use v0.8.0 layout: context/skills/<name>/mcp/mcp-config.json
        let dir = mirror_root
            .join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join(crate::processkit_vocab::src::CONTEXT_DIR)
            .join(crate::processkit_vocab::src::SKILLS)
            .join(skill)
            .join("mcp");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mcp-config.json");
        fs::write(&path, json_body).unwrap();
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
}
