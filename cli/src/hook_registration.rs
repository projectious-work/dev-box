//! Hook registration — write per-harness hook config files so that
//! processkit's enforcement scripts are wired into each AI harness on
//! `aibox sync` / `aibox init`.
//!
//! ## What this module does
//!
//! processkit ships two Python hook scripts under
//! `context/skills/processkit/skill-gate/scripts/`:
//!
//! - `emit_compliance_contract.py` — injects the compliance contract into
//!   every new context window (`SessionStart`, `UserPromptSubmit`).
//! - `check_route_task_called.py` — blocks `Write|Edit|MultiEdit` and the
//!   processkit write-side tools from touching `context/` until
//!   `acknowledge_contract(version='v1')` has been called (`PreToolUse`).
//!
//! | Harness     | Hook config file             | Hooks wired                          |
//! |-------------|------------------------------|--------------------------------------|
//! | Claude Code | `.claude/settings.json`      | SessionStart, UserPromptSubmit, PreToolUse |
//! | Codex CLI   | `.codex/hooks.json`          | session_start, user_prompt_submit (PreToolUse skipped — upstream limitation) |
//! | Cursor      | `.cursor/hooks.json`         | preToolUse, beforeMCPExecution (sessionStart skipped — Cursor bug) |
//!
//! The merge is non-destructive: only the processkit-managed hook entries
//! (identified by a `_processkit_managed` marker key for Claude/Codex, or
//! a command path marker for Cursor) are added/replaced; any user-added
//! entries in other positions are preserved.
//!
//! ## Cursor known limitations (as of 2026-04-17)
//!
//! - **sessionStart is buggy**: Cursor has a known platform bug where
//!   `sessionStart` hook output does not reach the agent window. For this
//!   reason aibox does NOT wire `emit_compliance_contract.py` as a
//!   `sessionStart` hook for Cursor. When the bug is fixed upstream, add a
//!   `sessionStart` entry that calls the compliance contract script.
//!   Reference: Cursor forum bug (tracked internally as CUR-BUG-001).
//!
//! - **beforeSubmitPrompt cannot inject context**: Cursor's
//!   `beforeSubmitPrompt` hook cannot inject additional context into the
//!   turn. The `preToolUse` gate is the primary enforcement mechanism.
//!
//! - **Subagent tool calls**: Plugin hooks (preToolUse, beforeMCPExecution)
//!   do not cover tool calls made by subagents spawned via Cursor's
//!   background agent feature. The gate script will not fire for subagent
//!   writes to `context/`. This is an upstream limitation of Cursor's hook
//!   architecture.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::{AiProvider, AiboxConfig};
use crate::output;

// ---------------------------------------------------------------------------
// Script paths (project-root-relative, as the harness invokes with cwd = project root)
// ---------------------------------------------------------------------------

const COMPLIANCE_SCRIPT: &str =
    "python3 context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py";
const ROUTE_GUARD_SCRIPT: &str =
    "python3 context/skills/processkit/skill-gate/scripts/check_route_task_called.py";

/// Marker key injected into each processkit-managed hook entry so we can
/// identify and replace/remove them on subsequent runs without touching
/// user-added entries. Used for Claude Code and Codex.
const MANAGED_MARKER: &str = "_processkit_managed";

/// Substring present in any processkit-managed Cursor hook entry's `command`
/// field. Cursor entries use a different merge strategy (command-path marker)
/// because Cursor's hook schema is a flat object without room for extra keys.
const CURSOR_MANAGED_MARKER: &str = "processkit/skill-gate/scripts/";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Regenerate every harness-specific hook config file from the processkit
/// hook scripts for the harnesses listed in `[ai].harnesses`.
///
/// Called from `cmd_init` and `cmd_sync` right after
/// [`crate::mcp_registration::regenerate_mcp_configs`]. Idempotent:
/// re-running on a stable config produces byte-identical output.
pub fn regenerate_hook_configs(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    use std::collections::HashSet;
    let harnesses: HashSet<&AiProvider> = config.ai.harnesses.iter().collect();

    // 1. Claude Code → .claude/settings.json
    let writes_claude = harnesses.contains(&AiProvider::Claude);
    if writes_claude {
        let path = project_root.join(".claude/settings.json");
        write_claude_settings_hooks(&path)?;
        output::ok(&format!(
            "Wrote processkit hook entries to {}",
            path.display()
        ));
    }

    // 2. Codex CLI → .codex/hooks.json
    if harnesses.contains(&AiProvider::Codex) {
        let path = project_root.join(".codex/hooks.json");
        write_codex_hooks_json(&path)?;
        output::ok(&format!(
            "Wrote processkit hook entries to {}",
            path.display()
        ));
        output::info(
            "Note: Codex CLI PreToolUse currently only intercepts `Bash` calls \
             (upstream openai/codex#16732). `check_route_task_called.py` is NOT \
             wired for PreToolUse on Codex — the script will be activated once \
             Codex lifts that restriction.",
        );
    }

    // 3. Cursor → .cursor/hooks.json
    //    Note: sessionStart is intentionally omitted — Cursor has a known
    //    platform bug (CUR-BUG-001) where sessionStart hook output does not
    //    reach the agent window. See module-level docs for full list of
    //    Cursor hook limitations.
    if harnesses.contains(&AiProvider::Cursor) {
        let path = project_root.join(".cursor/hooks.json");
        write_cursor_hooks_json(&path)?;
        output::ok(&format!(
            "Wrote processkit hook entries to {}",
            path.display()
        ));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers — build managed hook entries as serde_json Values
// ---------------------------------------------------------------------------

/// Build one processkit-managed hook entry for the Claude settings array.
///
/// Shape:
/// ```json
/// {
///   "_processkit_managed": true,
///   "matcher": "<matcher>",
///   "hooks": [{"type": "command", "command": "<cmd>"}]
/// }
/// ```
fn claude_hook_entry(matcher: &str, command: &str) -> serde_json::Value {
    serde_json::json!({
        MANAGED_MARKER: true,
        "matcher": matcher,
        "hooks": [{"type": "command", "command": command}]
    })
}

// ---------------------------------------------------------------------------
// Writer 1: Claude Code .claude/settings.json
// ---------------------------------------------------------------------------

/// Merge processkit hook entries into `.claude/settings.json`.
///
/// Non-destructive merge strategy:
/// 1. Read existing file (if any).
/// 2. For each hook event key (`SessionStart`, `UserPromptSubmit`,
///    `PreToolUse`), retain array entries that do **not** have the
///    `_processkit_managed` marker — these are user-added and must be
///    preserved.
/// 3. Append the current processkit-managed entries.
/// 4. Write back with stable formatting.
fn write_claude_settings_hooks(path: &Path) -> Result<()> {
    // Load or create the top-level settings object.
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

    // Get-or-create the "hooks" object.
    let hooks_val = top
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let hooks = hooks_val
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("`hooks` in {} is not a JSON object", path.display()))?;

    // For each event key, strip managed entries then append the current ones.
    let event_keys = ["SessionStart", "UserPromptSubmit", "PreToolUse"];
    for key in event_keys {
        let arr_val = hooks
            .entry(key.to_string())
            .or_insert_with(|| serde_json::Value::Array(vec![]));
        let arr = arr_val.as_array_mut().ok_or_else(|| {
            anyhow::anyhow!("`hooks.{key}` in {} is not a JSON array", path.display())
        })?;
        // Remove all previously-managed entries.
        arr.retain(|entry| {
            entry
                .get(MANAGED_MARKER)
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                .not()
        });
    }

    // Now append the processkit-managed entries for each event.
    //
    // SessionStart — emit compliance contract on session start/resume.
    {
        let arr = hooks["SessionStart"].as_array_mut().unwrap();
        arr.push(claude_hook_entry("", COMPLIANCE_SCRIPT));
    }

    // UserPromptSubmit — inject compliance contract into every turn.
    {
        let arr = hooks["UserPromptSubmit"].as_array_mut().unwrap();
        arr.push(claude_hook_entry("", COMPLIANCE_SCRIPT));
    }

    // PreToolUse — gate writes under context/ until contract acknowledged.
    {
        let arr = hooks["PreToolUse"].as_array_mut().unwrap();
        arr.push(claude_hook_entry(
            "Write|Edit|MultiEdit|create_workitem|transition_workitem|record_decision|\
             link_entities|open_discussion|create_artifact|log_event|create_note",
            ROUTE_GUARD_SCRIPT,
        ));
    }

    // Ensure parent dir exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let formatted =
        serde_json::to_string_pretty(&top).context("failed to serialize settings JSON")?;
    fs::write(path, formatted).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Writer 2: Codex CLI .codex/hooks.json
// ---------------------------------------------------------------------------

/// Merge processkit hook entries into `.codex/hooks.json`.
///
/// Codex uses a flat `{"hooks": {"session_start": {"command": "..."},
/// "user_prompt_submit": {"command": "..."}}}` shape (single command per
/// event, not an array).  Because the value is a single object rather
/// than an array, there is no meaningful way to preserve multiple
/// "user" entries alongside managed ones — but in practice users rarely
/// set Codex hooks manually, so we overwrite the managed event keys and
/// leave any other top-level keys untouched.
///
/// The managed event keys (`session_start`, `user_prompt_submit`) are
/// always overwritten with the processkit values; unknown sibling keys
/// inside `hooks` are preserved.
fn write_codex_hooks_json(path: &Path) -> Result<()> {
    // Load or create the top-level object.
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

    let hooks_val = top
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let hooks = hooks_val
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("`hooks` in {} is not a JSON object", path.display()))?;

    // Overwrite the managed event keys.
    hooks.insert(
        "session_start".to_string(),
        serde_json::json!({"command": COMPLIANCE_SCRIPT}),
    );
    hooks.insert(
        "user_prompt_submit".to_string(),
        serde_json::json!({"command": COMPLIANCE_SCRIPT}),
    );

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let formatted = serde_json::to_string_pretty(&top).context("failed to serialize hooks JSON")?;
    fs::write(path, formatted).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Writer 3: Cursor .cursor/hooks.json
// ---------------------------------------------------------------------------

/// Merge processkit enforcement hooks into `.cursor/hooks.json`.
///
/// Cursor uses camelCase event names and a flat list of hook objects per event.
/// Each entry has `command`, `description`, and `alwaysApprove` fields.
/// Setting `alwaysApprove: false` means Cursor blocks the action when the
/// hook exits with code 2.
///
/// Hooks written:
///
/// 1. **preToolUse** — runs `check_route_task_called.py` for Write/Edit/MultiEdit
///    tool calls against files under `context/`. Exit code 2 blocks the tool.
///
/// 2. **beforeMCPExecution** — runs `check_route_task_called.py` before any
///    processkit MCP server call. Exit code 2 blocks the MCP execution.
///
/// **sessionStart is intentionally omitted** — Cursor has a known platform bug
/// (CUR-BUG-001) where sessionStart hook output does not reach the agent
/// window. The compliance contract cannot be injected via sessionStart on
/// Cursor until this bug is fixed upstream.
///
/// Non-destructive merge:
/// - Entries whose `command` contains `processkit/skill-gate/scripts/` are
///   managed; they are removed and replaced on every sync.
/// - All other entries (user-added) are preserved in their original positions.
fn write_cursor_hooks_json(path: &Path) -> Result<()> {
    // The managed entries to write.
    let pre_tool_use_entry = serde_json::json!({
        "command": ROUTE_GUARD_SCRIPT,
        "description": "processkit: block context/ writes without route_task",
        "alwaysApprove": false
    });
    let before_mcp_entry = serde_json::json!({
        "command": ROUTE_GUARD_SCRIPT,
        "description": "processkit: gate MCP execution",
        "alwaysApprove": false
    });

    // Read existing file (if any). Work at the raw serde_json::Value level
    // so unknown top-level keys and unknown event names are preserved.
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

    // Get-or-create the "hooks" object.
    let hooks_val = top
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    let hooks = hooks_val
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("`hooks` in {} is not a JSON object", path.display()))?;

    // Merge one event's array: strip managed entries, append new managed entry.
    let merge = |hooks: &mut serde_json::Map<String, serde_json::Value>,
                 event: &str,
                 entry: &serde_json::Value|
     -> Result<()> {
        let existing: Vec<serde_json::Value> = hooks
            .get(event)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        // Keep non-managed (user-added) entries.
        let mut kept: Vec<serde_json::Value> = existing
            .into_iter()
            .filter(|v| {
                let cmd = v.get("command").and_then(|c| c.as_str()).unwrap_or("");
                !cmd.contains(CURSOR_MANAGED_MARKER)
            })
            .collect();
        kept.push(entry.clone());
        hooks.insert(event.to_string(), serde_json::Value::Array(kept));
        Ok(())
    };

    merge(hooks, "preToolUse", &pre_tool_use_entry)?;
    merge(hooks, "beforeMCPExecution", &before_mcp_entry)?;

    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let formatted =
        serde_json::to_string_pretty(&top).context("failed to serialize Cursor hooks JSON")?;
    fs::write(path, formatted).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Trait extension — `bool::not()` for readability in `.retain` closures
// ---------------------------------------------------------------------------

trait BoolExt {
    fn not(self) -> bool;
}

impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_config(harnesses: &[&str]) -> AiboxConfig {
        let harness_list = harnesses
            .iter()
            .map(|s| format!(r#""{s}""#))
            .collect::<Vec<_>>()
            .join(", ");
        let toml = format!(
            r#"
[aibox]
version = "0.18.0"

[container]
name = "test"

[ai]
harnesses = [{harness_list}]

[processkit]
version = "unset"
"#
        );
        AiboxConfig::from_str(&toml).expect("valid test config")
    }

    // -----------------------------------------------------------------------
    // Test 1: Claude Code hooks file is written with correct structure
    // -----------------------------------------------------------------------
    #[test]
    fn test_claude_hooks_written_correctly() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude/settings.json");

        write_claude_settings_hooks(&path).expect("write should succeed");

        assert!(path.is_file(), "settings.json should exist");
        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        let hooks = parsed["hooks"]
            .as_object()
            .expect("hooks must be an object");

        // SessionStart entry present and contains emit_compliance_contract.py
        let ss = hooks["SessionStart"]
            .as_array()
            .expect("SessionStart is array");
        assert!(!ss.is_empty(), "SessionStart must have at least one entry");
        let ss_cmd = ss[0]["hooks"][0]["command"]
            .as_str()
            .expect("command string");
        assert!(
            ss_cmd.contains("emit_compliance_contract.py"),
            "SessionStart command should call emit_compliance_contract.py"
        );
        assert_eq!(
            ss[0][MANAGED_MARKER].as_bool(),
            Some(true),
            "SessionStart entry must carry managed marker"
        );

        // UserPromptSubmit
        let ups = hooks["UserPromptSubmit"]
            .as_array()
            .expect("UserPromptSubmit is array");
        assert!(!ups.is_empty());
        let ups_cmd = ups[0]["hooks"][0]["command"].as_str().unwrap();
        assert!(ups_cmd.contains("emit_compliance_contract.py"));

        // PreToolUse
        let ptu = hooks["PreToolUse"].as_array().expect("PreToolUse is array");
        assert!(!ptu.is_empty());
        let ptu_cmd = ptu[0]["hooks"][0]["command"].as_str().unwrap();
        assert!(
            ptu_cmd.contains("check_route_task_called.py"),
            "PreToolUse command should call check_route_task_called.py"
        );
        let matcher = ptu[0]["matcher"].as_str().unwrap();
        assert!(matcher.contains("Write"), "matcher should include Write");
        assert!(matcher.contains("Edit"), "matcher should include Edit");
    }

    // -----------------------------------------------------------------------
    // Test 2: Existing user hook entries are preserved during merge
    // -----------------------------------------------------------------------
    #[test]
    fn test_user_hooks_preserved_during_merge() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude/settings.json");

        // Pre-populate with a user-added hook (no managed marker).
        let initial = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [{"type": "command", "command": "echo hello"}]
                    }
                ]
            },
            "model": "claude-opus-4-5"
        });
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        // Run the hook writer.
        write_claude_settings_hooks(&path).expect("write should succeed");

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        // The user-added "model" key at the top level must be preserved.
        assert_eq!(
            parsed["model"].as_str(),
            Some("claude-opus-4-5"),
            "top-level user key 'model' must survive the merge"
        );

        // The user-added PreToolUse entry (no managed marker) must survive.
        let ptu = parsed["hooks"]["PreToolUse"]
            .as_array()
            .expect("PreToolUse is array");
        let user_entry = ptu
            .iter()
            .find(|e| e["matcher"].as_str() == Some("Bash") && e.get(MANAGED_MARKER).is_none());
        assert!(
            user_entry.is_some(),
            "user-added PreToolUse Bash entry must be preserved"
        );

        // processkit-managed entry must also be present.
        let managed_entry = ptu
            .iter()
            .find(|e| e[MANAGED_MARKER].as_bool() == Some(true));
        assert!(
            managed_entry.is_some(),
            "processkit-managed PreToolUse entry must be present after merge"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: Codex hooks file has correct structure
    // -----------------------------------------------------------------------
    #[test]
    fn test_codex_hooks_written_correctly() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".codex/hooks.json");

        write_codex_hooks_json(&path).expect("write should succeed");

        assert!(path.is_file());
        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        let hooks = parsed["hooks"].as_object().expect("hooks is object");
        let ss = hooks["session_start"]["command"].as_str().unwrap();
        assert!(ss.contains("emit_compliance_contract.py"));
        let ups = hooks["user_prompt_submit"]["command"].as_str().unwrap();
        assert!(ups.contains("emit_compliance_contract.py"));
        // PreToolUse must NOT be wired for Codex.
        assert!(
            hooks.get("pre_tool_use").is_none(),
            "PreToolUse must not be wired for Codex"
        );
    }

    // -----------------------------------------------------------------------
    // Test 4: regenerate_hook_configs writes Claude file when claude in harnesses
    // -----------------------------------------------------------------------
    #[test]
    fn test_regenerate_writes_for_claude_harness() {
        let dir = TempDir::new().unwrap();
        let config = make_config(&["claude"]);
        regenerate_hook_configs(&config, dir.path()).expect("should succeed");

        let claude_path = dir.path().join(".claude/settings.json");
        assert!(
            claude_path.is_file(),
            ".claude/settings.json must be written"
        );
        // Codex file must NOT be written.
        let codex_path = dir.path().join(".codex/hooks.json");
        assert!(
            !codex_path.is_file(),
            ".codex/hooks.json must not be written when codex is not a harness"
        );
    }

    // -----------------------------------------------------------------------
    // Test 5: Re-running the writer is idempotent (managed entries not duplicated)
    // -----------------------------------------------------------------------
    #[test]
    fn test_idempotent_merge() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude/settings.json");

        write_claude_settings_hooks(&path).unwrap();
        write_claude_settings_hooks(&path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        // Each event key should have exactly one managed entry (not duplicated).
        for event in ["SessionStart", "UserPromptSubmit", "PreToolUse"] {
            let arr = parsed["hooks"][event].as_array().unwrap();
            let managed_count = arr
                .iter()
                .filter(|e| e[MANAGED_MARKER].as_bool() == Some(true))
                .count();
            assert_eq!(
                managed_count, 1,
                "{event}: expected exactly 1 managed entry after two runs, got {managed_count}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Cursor tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_cursor_hooks_written_correctly_fresh() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".cursor/hooks.json");

        write_cursor_hooks_json(&path).expect("write should succeed");

        assert!(path.is_file(), ".cursor/hooks.json should exist");
        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        let hooks = parsed["hooks"].as_object().expect("hooks must be object");

        // preToolUse entry — camelCase, gate script, alwaysApprove: false.
        let pre_tool = hooks["preToolUse"].as_array().expect("preToolUse is array");
        assert_eq!(pre_tool.len(), 1, "one preToolUse entry on fresh write");
        assert_eq!(
            pre_tool[0]["command"].as_str().unwrap(),
            ROUTE_GUARD_SCRIPT,
            "preToolUse command must be the gate script"
        );
        assert_eq!(
            pre_tool[0]["description"].as_str().unwrap(),
            "processkit: block context/ writes without route_task"
        );
        assert_eq!(
            pre_tool[0]["alwaysApprove"].as_bool().unwrap(),
            false,
            "alwaysApprove must be false so exit code 2 blocks the tool"
        );

        // beforeMCPExecution entry.
        let before_mcp = hooks["beforeMCPExecution"]
            .as_array()
            .expect("beforeMCPExecution is array");
        assert_eq!(
            before_mcp.len(),
            1,
            "one beforeMCPExecution entry on fresh write"
        );
        assert_eq!(
            before_mcp[0]["command"].as_str().unwrap(),
            ROUTE_GUARD_SCRIPT
        );
        assert_eq!(
            before_mcp[0]["description"].as_str().unwrap(),
            "processkit: gate MCP execution"
        );
        assert_eq!(before_mcp[0]["alwaysApprove"].as_bool().unwrap(), false);

        // sessionStart must NOT be written (Cursor bug CUR-BUG-001).
        assert!(
            hooks.get("sessionStart").is_none(),
            "sessionStart must not be wired for Cursor (known bug)"
        );
    }

    #[test]
    fn test_cursor_hooks_idempotent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".cursor/hooks.json");

        write_cursor_hooks_json(&path).unwrap();
        let first = fs::read_to_string(&path).unwrap();

        write_cursor_hooks_json(&path).unwrap();
        let second = fs::read_to_string(&path).unwrap();

        assert_eq!(first, second, "write_cursor_hooks_json must be idempotent");
    }

    #[test]
    fn test_cursor_hooks_preserves_user_entries() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".cursor/hooks.json");

        // Pre-populate with a user-added preToolUse entry and a custom event.
        let existing = serde_json::json!({
            "hooks": {
                "preToolUse": [
                    {
                        "command": "python3 my-team-scripts/custom-check.py",
                        "description": "our custom check",
                        "alwaysApprove": false
                    }
                ],
                "onError": [
                    {
                        "command": "python3 notify-team.py",
                        "description": "team notification",
                        "alwaysApprove": true
                    }
                ]
            }
        });
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        write_cursor_hooks_json(&path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let hooks = parsed["hooks"].as_object().unwrap();

        // preToolUse: user entry preserved first, managed entry appended.
        let pre_tool = hooks["preToolUse"].as_array().unwrap();
        assert_eq!(pre_tool.len(), 2, "user entry + managed entry");
        assert_eq!(
            pre_tool[0]["command"].as_str().unwrap(),
            "python3 my-team-scripts/custom-check.py",
            "user entry must be preserved first"
        );
        assert_eq!(
            pre_tool[1]["command"].as_str().unwrap(),
            ROUTE_GUARD_SCRIPT,
            "managed entry appended after user entry"
        );

        // onError (user-added custom event) must be preserved untouched.
        assert!(
            hooks.contains_key("onError"),
            "user-added onError event must be preserved"
        );
        let on_error = hooks["onError"].as_array().unwrap();
        assert_eq!(on_error.len(), 1);
        assert_eq!(
            on_error[0]["command"].as_str().unwrap(),
            "python3 notify-team.py"
        );
    }

    #[test]
    fn test_cursor_hooks_replaces_stale_managed_entries() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".cursor/hooks.json");

        // Simulate a stale managed entry (same marker path, old description).
        let existing = serde_json::json!({
            "hooks": {
                "preToolUse": [
                    {
                        "command": "python3 context/skills/processkit/skill-gate/scripts/check_route_task_called.py",
                        "description": "old description that should be replaced",
                        "alwaysApprove": true
                    }
                ]
            }
        });
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        write_cursor_hooks_json(&path).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        let pre_tool = parsed["hooks"]["preToolUse"].as_array().unwrap();

        // Stale entry removed; fresh entry written. No duplication.
        assert_eq!(
            pre_tool.len(),
            1,
            "stale managed entry replaced, not accumulated"
        );
        assert_eq!(
            pre_tool[0]["description"].as_str().unwrap(),
            "processkit: block context/ writes without route_task",
            "fresh description present"
        );
        assert_eq!(
            pre_tool[0]["alwaysApprove"].as_bool().unwrap(),
            false,
            "alwaysApprove reset to false"
        );
    }

    #[test]
    fn test_cursor_not_written_when_not_in_harnesses() {
        let dir = TempDir::new().unwrap();
        let config = make_config(&["claude"]);
        let path = dir.path().join(".cursor/hooks.json");

        regenerate_hook_configs(&config, dir.path()).expect("should succeed");

        assert!(
            !path.exists(),
            ".cursor/hooks.json must not be written when cursor is not in harnesses"
        );
    }

    #[test]
    fn test_cursor_written_when_in_harnesses() {
        let dir = TempDir::new().unwrap();
        let config = make_config(&["cursor"]);
        let path = dir.path().join(".cursor/hooks.json");

        regenerate_hook_configs(&config, dir.path()).expect("should succeed");

        assert!(
            path.exists(),
            ".cursor/hooks.json must be written when cursor is in harnesses"
        );
        let body = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(parsed["hooks"]["preToolUse"].is_array());
        assert!(parsed["hooks"]["beforeMCPExecution"].is_array());
    }

    #[test]
    fn test_cursor_creates_directory_if_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".cursor/hooks.json");

        assert!(!dir.path().join(".cursor").exists());
        write_cursor_hooks_json(&path).unwrap();
        assert!(
            path.exists(),
            ".cursor/hooks.json created even if .cursor/ was absent"
        );
    }
}
