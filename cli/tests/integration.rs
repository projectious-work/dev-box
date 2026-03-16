use std::process::Command;

/// Get the path to the built binary.
fn dev_box_bin() -> String {
    // Use the debug binary built by cargo test
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/dev-box", manifest_dir)
}

/// Run the dev-box binary with the given args and return the output.
fn run(args: &[&str]) -> std::process::Output {
    Command::new(dev_box_bin())
        .args(args)
        .output()
        .expect("failed to execute dev-box binary")
}

/// Run the dev-box binary in a specific directory.
fn run_in_dir(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(dev_box_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute dev-box binary")
}

#[test]
fn help_exits_zero() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "dev-box --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dev-box") || stdout.contains("development container"),
        "help output should mention dev-box"
    );
}

#[test]
fn init_help_exits_zero() {
    let output = run(&["init", "--help"]);
    assert!(output.status.success(), "dev-box init --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--name") || stdout.contains("name"), "init help should mention --name");
}

#[test]
fn generate_without_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["generate"]);
    assert!(
        !output.status.success(),
        "dev-box generate without dev-box.toml should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("dev-box.toml") || stderr.contains("No dev-box.toml"),
        "error should mention missing config file"
    );
}

#[test]
fn status_without_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["status"]);
    assert!(
        !output.status.success(),
        "dev-box status without dev-box.toml should fail"
    );
}

#[test]
fn init_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &["init", "--name", "test-project", "--image", "python", "--process", "minimal"],
    );
    assert!(
        output.status.success(),
        "init should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(dir.path().join("dev-box.toml").exists(), "dev-box.toml should be created");
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
    assert!(dir.path().join("CLAUDE.md").exists(), "CLAUDE.md should be created");
    assert!(
        dir.path().join(".dev-box-version").exists(),
        ".dev-box-version should be created"
    );
}

#[test]
fn init_existing_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    // First init
    run_in_dir(
        dir.path(),
        &["init", "--name", "test", "--image", "base", "--process", "minimal"],
    );
    // Second init should fail
    let output = run_in_dir(
        dir.path(),
        &["init", "--name", "test", "--image", "base", "--process", "minimal"],
    );
    assert!(
        !output.status.success(),
        "init with existing dev-box.toml should fail"
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
        &["init", "--name", "gen-test", "--image", "base", "--process", "minimal"],
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
fn init_invalid_image_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &["init", "--name", "test", "--image", "invalid-flavor", "--process", "minimal"],
    );
    assert!(
        !output.status.success(),
        "init with invalid image should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown image flavor") || stderr.contains("invalid"),
        "error should mention invalid image: {}",
        stderr
    );
}

#[test]
fn init_invalid_process_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &["init", "--name", "test", "--image", "base", "--process", "invalid-process"],
    );
    assert!(
        !output.status.success(),
        "init with invalid process should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown process flavor") || stderr.contains("invalid"),
        "error should mention invalid process: {}",
        stderr
    );
}

#[test]
fn init_with_all_image_flavors() {
    for flavor in &["base", "python", "latex", "rust", "python-latex", "rust-latex"] {
        let dir = tempfile::tempdir().unwrap();
        let output = run_in_dir(
            dir.path(),
            &["init", "--name", "test", "--image", flavor, "--process", "minimal"],
        );
        assert!(
            output.status.success(),
            "init with image '{}' should succeed: {}",
            flavor,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn init_with_all_process_flavors() {
    for flavor in &["minimal", "managed", "research", "product"] {
        let dir = tempfile::tempdir().unwrap();
        let output = run_in_dir(
            dir.path(),
            &["init", "--name", "test", "--image", "base", "--process", flavor],
        );
        assert!(
            output.status.success(),
            "init with process '{}' should succeed: {}",
            flavor,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn init_generated_toml_is_parseable() {
    let dir = tempfile::tempdir().unwrap();
    run_in_dir(
        dir.path(),
        &["init", "--name", "parse-test", "--image", "python", "--process", "managed"],
    );
    let content = std::fs::read_to_string(dir.path().join("dev-box.toml")).unwrap();
    // Should be valid TOML
    let _: toml::Value = toml::from_str(&content).expect("generated dev-box.toml should be valid TOML");
}

#[test]
fn doctor_without_config_reports_errors() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["doctor"]);
    // Doctor exits 0 even when reporting errors (it's a diagnostic tool)
    assert!(output.status.success(), "doctor should always exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("dev-box.toml") || stderr.contains("Config"),
        "doctor should report missing config"
    );
}
