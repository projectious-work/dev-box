//! Table-driven tests for comprehensive aibox.toml settings coverage.
//!
//! Each test case:
//! 1. Runs `aibox init` with a base config
//! 2. Patches `aibox.toml` with specific settings
//! 3. Runs `aibox sync` (alias: generate)
//! 4. Asserts the generated files contain/don't contain expected strings
//!
//! These are Tier 1 tests — no running container needed, fast.

use std::fs;
use std::process::Command;

/// Get the path to the aibox binary.
fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

/// Get the path to addon definitions.
fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run aibox in a directory.
fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox")
}

/// Initialize a project in a temp dir with default settings.
fn init_project(dir: &std::path::Path, name: &str) {
    let output = run_in(
        dir,
        &[
            "init",
            "--name",
            name,
            "--base",
            "debian",
            "--process",
            "managed",
        ],
    );
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Patch the aibox.toml in a directory by appending TOML content.
/// For replacing sections, read the file, do string replacement, write back.
fn patch_toml(dir: &std::path::Path, patch: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read aibox.toml");
    fs::write(&toml_path, format!("{}\n{}", content, patch)).expect("failed to write aibox.toml");
}

/// Replace a section in aibox.toml.
fn replace_toml_section(dir: &std::path::Path, section: &str, replacement: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read aibox.toml");

    // Find the section header at the start of a line (not in comments).
    // We search for "\n[section]" to avoid matching comment lines like "# [section]".
    let section_header = format!("[{}]", section);
    let needle = format!("\n{}", section_header);
    if let Some(needle_pos) = content.find(&needle) {
        // start = position of the "[" in the section header (after the \n)
        let start = needle_pos + 1;
        let rest = &content[start + section_header.len()..];
        // Find next top-level section (line starting with `[` but not `[[`)
        let end = rest
            .find("\n[")
            .map(|i| start + section_header.len() + i)
            .unwrap_or(content.len());
        let new_content = format!(
            "{}{}\n{}{}",
            &content[..start],
            &section_header,
            replacement,
            &content[end..]
        );
        fs::write(&toml_path, new_content).expect("failed to write aibox.toml");
    } else {
        // Section doesn't exist, append it
        patch_toml(dir, &format!("[{}]\n{}", section, replacement));
    }
}

/// Sync (regenerate) the project files.
fn sync_project(dir: &std::path::Path) {
    let output = run_in(dir, &["sync", "--no-build"]);
    assert!(
        output.status.success(),
        "sync failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Read a generated file relative to the project directory.
fn read_generated(dir: &std::path::Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

// ─── Container Section Tests ─────────────────────────────────────────────────

#[test]
fn container_name_in_compose() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "my-project");
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains("container_name: my-project"),
        "compose should contain container_name: my-project"
    );
}

#[test]
fn container_hostname_in_compose() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "my-project");
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    // Default hostname is "aibox" unless overridden
    assert!(
        compose.contains("hostname:"),
        "compose should contain hostname"
    );
}

// ─── AI Section Tests ────────────────────────────────────────────────────────

#[test]
fn ai_claude_provider_volume_mount() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ai-claude");
    // Default is claude, so compose should already have .claude mount
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".claude"),
        "compose should mount .claude for claude provider"
    );
}

#[test]
fn ai_aider_provider_volume_mount() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ai-aider");
    replace_toml_section(
        dir.path(),
        "ai",
        r#"
harnesses = ["aider"]
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".aider"),
        "compose should mount .aider for aider provider"
    );
}

#[test]
fn ai_multiple_providers_volume_mounts() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ai-multi");
    replace_toml_section(
        dir.path(),
        "ai",
        r#"
harnesses = ["claude", "aider", "gemini"]
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(compose.contains(".claude"), "compose should mount .claude");
    assert!(compose.contains(".aider"), "compose should mount .aider");
    assert!(compose.contains(".gemini"), "compose should mount .gemini");
}

// ─── Audio Section Tests ─────────────────────────────────────────────────────

#[test]
fn audio_enabled_adds_mounts() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "audio-on");
    replace_toml_section(
        dir.path(),
        "audio",
        r#"
enabled = true
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".asoundrc"),
        "compose should mount .asoundrc when audio enabled"
    );
}

#[test]
fn audio_disabled_no_mounts() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "audio-off");
    replace_toml_section(
        dir.path(),
        "audio",
        r#"
enabled = false
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        !compose.contains(".asoundrc"),
        "compose should not mount .asoundrc when audio disabled"
    );
}

// ─── Addon Section Tests ─────────────────────────────────────────────────────

#[test]
fn addon_python_in_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "addon-py");
    patch_toml(
        dir.path(),
        r#"
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
"#,
    );
    sync_project(dir.path());
    let dockerfile = read_generated(dir.path(), ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python"),
        "Dockerfile should contain python addon commands"
    );
}

#[test]
fn addon_rust_in_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "addon-rs");
    patch_toml(
        dir.path(),
        r#"
[addons.rust.tools]
rust = {}
"#,
    );
    sync_project(dir.path());
    let dockerfile = read_generated(dir.path(), ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("rustup") || dockerfile.contains("rust"),
        "Dockerfile should contain rust addon commands"
    );
}

#[test]
fn addon_multiple_in_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "addon-multi");
    patch_toml(
        dir.path(),
        r#"
[addons.python.tools]
python = { version = "3.13" }

[addons.node.tools]
node = { version = "22" }
"#,
    );
    sync_project(dir.path());
    let dockerfile = read_generated(dir.path(), ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python"),
        "Dockerfile should contain python addon"
    );
    assert!(
        dockerfile.contains("node") || dockerfile.contains("Node"),
        "Dockerfile should contain node addon"
    );
}

// ─── processkit package selection tests ──────────────────────────────────────
//
// Since v0.16.0 aibox no longer scaffolds context-doc files (BACKLOG.md,
// DECISIONS.md, PRD.md, PROJECTS.md, …). Those are created lazily by the
// single-file processkit skills (backlog-context, decisions-adr, …) when
// an agent first uses them, OR by the entity-sharded processkit skills
// (workitem-management, decision-record, …) which write per-entity YAML
// files under context/workitems/ etc.
//
// What aibox owns at init time is the slim project skeleton: context/
// directory, aibox.lock, .gitignore, CLAUDE.md thin pointer, and
// (when [processkit] version != "unset") the processkit content
// installed by content_init. The tests below verify that contract for
// each of the five processkit package presets.

fn assert_post_init_skeleton(dir: &std::path::Path) {
    assert!(dir.join("context").exists(), "context/ should exist");
    assert!(
        dir.join("CLAUDE.md").exists(),
        "CLAUDE.md thin pointer should exist"
    );
    assert!(
        !dir.join(".aibox-version").exists(),
        ".aibox-version must NOT be created (absorbed into aibox.lock)"
    );
    assert!(dir.join(".gitignore").exists(), ".gitignore should exist");
    let claude = fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
    assert!(
        claude.contains("Pointer file") && claude.contains("AGENTS.md"),
        "CLAUDE.md should be a thin pointer to AGENTS.md, got: {claude}"
    );
}

#[test]
fn process_minimal_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "--name",
            "proc-min",
            "--base",
            "debian",
            "--process",
            "minimal",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
}

#[test]
fn process_managed_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "--name",
            "proc-mgd",
            "--base",
            "debian",
            "--process",
            "managed",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
    // Context-doc files are NOT created by aibox post-v0.16.0 — they
    // are owned by processkit single-file skills.
    assert!(
        !dir.path().join("context/BACKLOG.md").exists(),
        "aibox v0.16.0 must not scaffold context/BACKLOG.md (owned by processkit backlog-context skill)"
    );
}

#[test]
fn process_product_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "--name",
            "proc-prod",
            "--base",
            "debian",
            "--process",
            "product",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
    assert!(
        !dir.path().join("context/PRD.md").exists(),
        "aibox v0.16.0 must not scaffold context/PRD.md"
    );
}

#[test]
fn process_research_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "--name",
            "proc-res",
            "--base",
            "debian",
            "--process",
            "research",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
}
