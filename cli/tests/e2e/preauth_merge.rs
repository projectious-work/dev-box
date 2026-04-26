//! Tier 1 E2E tests for WS-5 preauth merge.
//!
//! These tests drive `aibox init` + `aibox sync` against a fresh
//! tempdir with `AIBOX_NO_CONTAINER=1` so no container runtime or
//! network is required. After init we manually drop a synthetic
//! `preauth.json` fixture into
//! `context/skills/processkit/skill-gate/assets/preauth.json` (the
//! processkit install in the test env may not run network fetch, so
//! we inject the file ourselves) and then re-run sync to exercise the
//! merge path.
//!
//! Tests:
//!
//! 11. `preauth_merge_e2e_basic` — happy path; assert
//!     `permissions.allow[]` and `enabledMcpjsonServers[]` shape via
//!     parsed JSON (not substring match).
//! 12. `preauth_merge_e2e_user_entries_survive_double_sync` — two
//!     consecutive syncs preserve a user-added `permissions.allow`
//!     entry and produce byte-identical output.
//! 13. `preauth_merge_e2e_unknown_version_skips` — `version: 99`
//!     fixture; merge soft-warns and leaves `_processkit_managed_keys`
//!     out of settings.json (or leaves it untouched if present).
//! 14. `preauth_merge_e2e_absent_spec_is_soft_warn` — no preauth.json;
//!     sync exits 0; warning appears in stderr; no
//!     `_processkit_managed_keys` sidecar appears in settings.json.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run aibox in `dir` with `AIBOX_NO_CONTAINER=1` set so the
/// `--no-container` flag is supplied via the env-var mirror.
fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .output()
        .expect("failed to execute aibox")
}

fn fmt_output(label: &str, out: &Output) -> String {
    format!(
        "{label}: status={}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

/// Inject a synthetic preauth.json fixture at the path the merge
/// helper looks for. Mirrors the fact that `aibox init` can't always
/// fetch processkit (no network in CI) — we materialize the file
/// ourselves so the merge path is exercised deterministically.
fn write_preauth_fixture(dir: &Path, body: &str) {
    let asset_dir = dir.join("context/skills/processkit/skill-gate/assets");
    fs::create_dir_all(&asset_dir).expect("create asset dir");
    fs::write(asset_dir.join("preauth.json"), body).expect("write preauth.json");
}

/// 18-pattern + 18-server v1 fixture that mimics the v0.22.0 spec.
fn v0_22_0_preauth_body() -> String {
    let allow: Vec<String> = (0..18)
        .map(|i| format!("\"mcp__processkit-skill-{i:02}__*\""))
        .collect();
    let servers: Vec<String> = (0..18)
        .map(|i| format!("\"processkit-skill-{i:02}\""))
        .collect();
    format!(
        r#"{{
          "version": 1,
          "description": "synthetic v0.22.0",
          "permissions": {{ "allow": [{}] }},
          "enabledMcpjsonServers": [{}]
        }}"#,
        allow.join(", "),
        servers.join(", ")
    )
}

/// Init with `--processkit-version unset` so the `[processkit]`
/// install path is short-circuited (decide_sync returns Skip on
/// "unset"). This keeps the test deterministic: it does not require
/// network access to GitHub, and our synthetic preauth.json fixture
/// won't be overwritten by a re-fetch on `aibox sync`.
fn init_project(dir: &Path) {
    let init_out = run_in(
        dir,
        &[
            "init",
            "--name",
            "preauth-fixture",
            "--base",
            "debian",
            "--process",
            "managed",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );
}

fn read_settings_json(dir: &Path) -> Value {
    let body =
        fs::read_to_string(dir.join(".claude/settings.json")).expect("read .claude/settings.json");
    serde_json::from_str(&body).expect("parse settings.json")
}

// ─── Test 11 ────────────────────────────────────────────────────────────
#[test]
fn preauth_merge_e2e_basic() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    init_project(dir);
    write_preauth_fixture(dir, &v0_22_0_preauth_body());

    let sync_out = run_in(dir, &["sync"]);
    assert!(
        sync_out.status.success(),
        "sync failed.\n{}",
        fmt_output("sync", &sync_out)
    );

    let settings_path = dir.join(".claude/settings.json");
    assert!(
        settings_path.exists(),
        "expected .claude/settings.json to exist after sync.\n{}",
        fmt_output("sync", &sync_out)
    );

    let settings = read_settings_json(dir);

    // Parsed key-path lookups (NOT substring matching).
    let allow = settings["permissions"]["allow"]
        .as_array()
        .expect("permissions.allow must be a JSON array");
    assert!(
        allow.len() >= 18,
        "expected at least 18 entries in permissions.allow, got {}",
        allow.len()
    );
    assert!(
        allow
            .iter()
            .any(|v| v.as_str() == Some("mcp__processkit-skill-00__*")),
        "expected 'mcp__processkit-skill-00__*' in permissions.allow"
    );

    let servers = settings["enabledMcpjsonServers"]
        .as_array()
        .expect("enabledMcpjsonServers must be a JSON array");
    assert!(
        servers.len() >= 18,
        "expected at least 18 entries in enabledMcpjsonServers, got {}",
        servers.len()
    );
    assert!(
        servers
            .iter()
            .any(|v| v.as_str() == Some("processkit-skill-00")),
        "expected 'processkit-skill-00' in enabledMcpjsonServers"
    );

    // Sidecar must be present and mirror the spec.
    let snap = settings
        .get("_processkit_managed_keys")
        .expect("_processkit_managed_keys must be present after merge");
    assert_eq!(snap["allow"].as_array().unwrap().len(), 18);
    assert_eq!(snap["enabled_servers"].as_array().unwrap().len(), 18);
}

// ─── Test 12 ────────────────────────────────────────────────────────────
#[test]
fn preauth_merge_e2e_user_entries_survive_double_sync() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    init_project(dir);
    write_preauth_fixture(dir, &v0_22_0_preauth_body());

    // First sync: produces an initial settings.json.
    let sync1 = run_in(dir, &["sync"]);
    assert!(sync1.status.success(), "{}", fmt_output("sync1", &sync1));

    // Inject a user-added permissions.allow entry.
    let settings_path = dir.join(".claude/settings.json");
    let mut settings = read_settings_json(dir);
    {
        let allow = settings["permissions"]["allow"]
            .as_array_mut()
            .expect("permissions.allow array");
        allow.push(Value::String("Bash(custom)".to_string()));
    }
    let mutated = serde_json::to_string_pretty(&settings).unwrap();
    fs::write(&settings_path, mutated).unwrap();

    // Second sync: must preserve the user entry.
    let sync2 = run_in(dir, &["sync"]);
    assert!(sync2.status.success(), "{}", fmt_output("sync2", &sync2));
    let after_second = fs::read(&settings_path).unwrap();

    let after_settings: Value = serde_json::from_slice(&after_second).unwrap();
    let allow_after: Vec<String> = after_settings["permissions"]["allow"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        allow_after.contains(&"Bash(custom)".to_string()),
        "user entry must survive sync.\n{}",
        fmt_output("sync2", &sync2)
    );

    // Third sync (stable spec) must produce byte-identical output.
    let sync3 = run_in(dir, &["sync"]);
    assert!(sync3.status.success(), "{}", fmt_output("sync3", &sync3));
    let after_third = fs::read(&settings_path).unwrap();
    assert_eq!(
        after_second, after_third,
        "two consecutive syncs on a stable spec must be byte-identical"
    );
}

// ─── Test 13 ────────────────────────────────────────────────────────────
#[test]
fn preauth_merge_e2e_unknown_version_skips() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    init_project(dir);
    // Future-version fixture; merge must soft-warn + skip.
    write_preauth_fixture(
        dir,
        r#"{
          "version": 99,
          "permissions": { "allow": ["mcp__future__*"] },
          "enabledMcpjsonServers": ["future-server"]
        }"#,
    );

    let sync_out = run_in(dir, &["sync"]);
    assert!(
        sync_out.status.success(),
        "{}",
        fmt_output("sync", &sync_out)
    );

    // Hooks should still have been merged (regenerate_hook_configs ran).
    let settings_path = dir.join(".claude/settings.json");
    assert!(settings_path.exists());
    let settings = read_settings_json(dir);
    assert!(
        settings.get("hooks").is_some(),
        "hooks block must be present (regenerate_hook_configs ran)"
    );

    // No future-version preauth content should have leaked through.
    if let Some(allow) = settings.get("permissions").and_then(|p| p.get("allow")) {
        let strs: Vec<&str> = allow
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert!(
            !strs.contains(&"mcp__future__*"),
            "v99 spec entry must NOT appear in permissions.allow"
        );
    }
    if let Some(servers) = settings.get("enabledMcpjsonServers") {
        let strs: Vec<&str> = servers
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert!(
            !strs.contains(&"future-server"),
            "v99 server must NOT appear in enabledMcpjsonServers"
        );
    }
}

// ─── Test 14 ────────────────────────────────────────────────────────────
#[test]
fn preauth_merge_e2e_absent_spec_is_soft_warn() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    init_project(dir);
    // Deliberately do NOT write preauth.json.
    let asset_path = dir.join("context/skills/processkit/skill-gate/assets/preauth.json");
    assert!(
        !asset_path.exists(),
        "preauth.json must be absent for this test"
    );

    let sync_out = run_in(dir, &["sync"]);
    assert!(
        sync_out.status.success(),
        "sync must succeed (soft-warn) when preauth.json is absent.\n{}",
        fmt_output("sync", &sync_out)
    );

    // Warn message expected somewhere in stdout/stderr (output::warn
    // routes via tracing/println; either stream is acceptable).
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&sync_out.stdout),
        String::from_utf8_lossy(&sync_out.stderr)
    );
    assert!(
        combined.contains("preauth.json not found"),
        "expected soft-warn for missing preauth.json.\n{}",
        fmt_output("sync", &sync_out)
    );

    // Sidecar must NOT appear when no merge happened.
    let settings_path = dir.join(".claude/settings.json");
    if settings_path.exists() {
        let settings = read_settings_json(dir);
        assert!(
            settings.get("_processkit_managed_keys").is_none(),
            "_processkit_managed_keys must NOT appear when preauth.json is absent.\n{}",
            fmt_output("sync", &sync_out)
        );
    }
}
