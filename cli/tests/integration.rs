use std::process::Command;

/// Get the path to the built binary.
fn aibox_bin() -> String {
    // Use the debug binary built by cargo test
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

/// Get the path to the addon YAML definitions in the repo.
fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run the aibox binary with the given args and return the output.
fn run(args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox binary")
}

/// Run the aibox binary in a specific directory.
fn run_in_dir(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox binary")
}

#[test]
fn help_exits_zero() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "aibox --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aibox") || stdout.contains("development container"),
        "help output should mention aibox"
    );
}

#[test]
fn init_help_exits_zero() {
    let output = run(&["init", "--help"]);
    assert!(output.status.success(), "aibox init --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--name") || stdout.contains("name"),
        "init help should mention --name"
    );
}

#[test]
fn generate_without_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["generate"]);
    assert!(
        !output.status.success(),
        "aibox generate without aibox.toml should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("aibox.toml") || stderr.contains("No aibox.toml"),
        "error should mention missing config file"
    );
}

#[test]
fn status_without_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["status"]);
    assert!(
        !output.status.success(),
        "aibox status without aibox.toml should fail"
    );
}

#[test]
fn init_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "test-project",
            "--base",
            "debian",
            "--process",
            "managed",
            "--processkit-version",
            "unset", // avoid network fetch in tests
        ],
    );
    assert!(
        output.status.success(),
        "init should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        dir.path().join("aibox.toml").exists(),
        "aibox.toml should be created"
    );
    assert!(
        dir.path().join(".devcontainer/Dockerfile").exists(),
        "Dockerfile should be created"
    );
    assert!(
        dir.path().join(".devcontainer/docker-compose.yml").exists(),
        "docker-compose.yml should be created"
    );
    assert!(
        dir.path().join(".devcontainer/devcontainer.json").exists(),
        "devcontainer.json should be created"
    );
    // AGENTS.md is owned by processkit since v0.16.0 and lands only
    // when [processkit].version is pinned. The default `aibox init`
    // writes "unset", so AGENTS.md is intentionally absent here.
    assert!(
        !dir.path().join("AGENTS.md").exists(),
        "AGENTS.md should NOT be created when processkit version is unset"
    );
    assert!(
        dir.path().join("CLAUDE.md").exists(),
        "CLAUDE.md (thin pointer) should be created"
    );
    let claude_body = std::fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
    assert!(
        claude_body.contains("AGENTS.md") && claude_body.contains("Pointer file"),
        "thin-pointer CLAUDE.md should reference AGENTS.md"
    );
    assert!(
        !dir.path().join(".aibox-version").exists(),
        ".aibox-version must NOT be created (absorbed into aibox.lock since v0.17.0)"
    );
}

#[test]
fn init_existing_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    // First init
    run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "test",
            "--base",
            "debian",
            "--process",
            "managed",
        ],
    );
    // Second init should fail
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "test",
            "--base",
            "debian",
            "--process",
            "managed",
        ],
    );
    assert!(
        !output.status.success(),
        "init with existing aibox.toml should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists"),
        "error should mention config already exists"
    );
}

#[test]
fn generate_after_init_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    // Init first
    let init_output = run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "gen-test",
            "--base",
            "debian",
            "--process",
            "managed",
        ],
    );
    assert!(init_output.status.success(), "init should succeed");

    // Generate should work
    let gen_output = run_in_dir(dir.path(), &["generate"]);
    assert!(
        gen_output.status.success(),
        "generate after init should succeed: {}",
        String::from_utf8_lossy(&gen_output.stderr)
    );
}

#[test]
fn init_invalid_base_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "test",
            "--base",
            "invalid-base",
            "--process",
            "managed",
        ],
    );
    assert!(
        !output.status.success(),
        "init with invalid base should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("Invalid") || stderr.contains("error"),
        "error should mention invalid base: {}",
        stderr
    );
}

#[test]
fn init_invalid_process_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "test",
            "--base",
            "debian",
            "--process",
            "invalid-process!",
        ],
    );
    assert!(
        !output.status.success(),
        "init with invalid process should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("Invalid") || stderr.contains("error"),
        "error should mention invalid process: {}",
        stderr
    );
}

#[test]
fn init_with_all_base_images() {
    // Currently only "debian" is supported; add more entries when new bases land
    let bases = ["debian"];
    for base in &bases {
        let dir = tempfile::tempdir().unwrap();
        let output = run_in_dir(
            dir.path(),
            &[
                "init",
                "--name",
                "test",
                "--base",
                base,
                "--process",
                "managed",
            ],
        );
        assert!(
            output.status.success(),
            "init with base '{}' should succeed: {}",
            base,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn init_with_all_process_packages() {
    for pkg in &["minimal", "managed", "software", "research", "product"] {
        let dir = tempfile::tempdir().unwrap();
        let output = run_in_dir(
            dir.path(),
            &[
                "init",
                "--name",
                "test",
                "--base",
                "debian",
                "--process",
                pkg,
            ],
        );
        assert!(
            output.status.success(),
            "init with process '{}' should succeed: {}",
            pkg,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn init_generated_toml_is_parseable() {
    let dir = tempfile::tempdir().unwrap();
    run_in_dir(
        dir.path(),
        &[
            "init",
            "--name",
            "parse-test",
            "--base",
            "debian",
            "--process",
            "managed",
        ],
    );
    let content = std::fs::read_to_string(dir.path().join("aibox.toml")).unwrap();
    // Should be valid TOML
    let _: toml::Value =
        toml::from_str(&content).expect("generated aibox.toml should be valid TOML");
}

#[test]
fn completions_bash_exits_zero() {
    let output = run(&["completions", "bash"]);
    assert!(
        output.status.success(),
        "aibox completions bash should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aibox"),
        "bash completions should contain aibox"
    );
}

#[test]
fn completions_zsh_exits_zero() {
    let output = run(&["completions", "zsh"]);
    assert!(
        output.status.success(),
        "aibox completions zsh should exit 0"
    );
}

#[test]
fn completions_invalid_shell_exits_nonzero() {
    let output = run(&["completions", "tcsh"]);
    assert!(
        !output.status.success(),
        "aibox completions tcsh should fail"
    );
}

#[test]
fn doctor_without_config_reports_errors() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["doctor"]);
    // Doctor exits 0 even when reporting errors (it's a diagnostic tool)
    assert!(output.status.success(), "doctor should always exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("aibox.toml") || stderr.contains("Config"),
        "doctor should report missing config"
    );
}
