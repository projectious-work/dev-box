//! Version upgrade flow tests (BACK-060).
//!
//! Covers:
//!   1. Generated Dockerfile contains `LABEL aibox.version=`
//!   2. Generated Dockerfile writes `/etc/aibox-version` inside the image
//!   3. `aibox start` hard-errors when container image label mismatches config version
//!   4. `aibox start` does NOT error when container and config versions match (happy path)
//!   5. `aibox update -y` exits 0 (global_yes is correctly wired, no interactive hang)
//!   6. `aibox update --dry-run` no longer mentions `.aibox-version` (removed in BACK-060)
//!   7. `aibox doctor` warns when running container label mismatches config version
//!   8. `aibox doctor` warns when `.aibox-version` does not match the current CLI version
//!
//! All tests are Tier 1 — no SSH companion container required.
//! Tests 3, 4, 7 use MockRuntime to intercept docker/podman calls.

use std::fs;
use std::process::Command;

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run aibox without any runtime mock.
fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox")
}

/// Run aibox with a MockRuntime intercepting docker/podman calls.
///
/// `mock_state`   — value returned by `docker inspect` for container status queries
/// `mock_version` — value returned by `docker inspect` for `aibox.version` label queries
///                  (empty string = no label, i.e. pre-BACK-060 image)
fn run_in_with_mock(
    dir: &std::path::Path,
    args: &[&str],
    mock: &super::mock_runtime::MockRuntime,
    mock_state: &str,
    mock_version: &str,
) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("PATH", mock.path_env())
        .env("MOCK_LOG_FILE", mock.log_file_str())
        .env("MOCK_CONTAINER_STATE", mock_state)
        .env("MOCK_CONTAINER_VERSION", mock_version)
        .output()
        .expect("failed to execute aibox")
}

fn init_project(dir: &std::path::Path, name: &str) {
    let output = run_in(
        dir,
        &["init", "--name", name, "--base", "debian", "--process", "core"],
    );
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ─── Tests 1 & 2: Dockerfile template renders version metadata ───────────────

/// Generated Dockerfile must embed `LABEL aibox.version=<version>` so the
/// built image carries a machine-readable version label.
#[test]
fn dockerfile_contains_aibox_version_label() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ver-label");

    let dockerfile = fs::read_to_string(dir.path().join(".devcontainer/Dockerfile"))
        .expect("failed to read generated Dockerfile");

    assert!(
        dockerfile.contains("LABEL aibox.version"),
        "generated Dockerfile must contain `LABEL aibox.version=...`\n---\n{dockerfile}"
    );
}

/// Generated Dockerfile must write the image version to `/etc/aibox-version`
/// so the running container can report its build version at runtime.
#[test]
fn dockerfile_contains_etc_aibox_version_write() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ver-etc");

    let dockerfile = fs::read_to_string(dir.path().join(".devcontainer/Dockerfile"))
        .expect("failed to read generated Dockerfile");

    assert!(
        dockerfile.contains("/etc/aibox-version"),
        "generated Dockerfile must write to /etc/aibox-version\n---\n{dockerfile}"
    );
}

// ─── Test 3: cmd_start version mismatch hard-error ───────────────────────────

/// `aibox start` must exit non-zero with a clear error message when the running
/// container was built from an older image than the version pinned in aibox.toml.
/// Running with a stale image risks subtle behaviour differences — BACK-060
/// makes this a hard failure rather than a silent continue.
#[test]
fn start_fails_on_image_version_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "start-mismatch");

    let mock = super::mock_runtime::MockRuntime::new();

    // Simulate: container is running but was built from an old image (v0.0.1)
    let output = run_in_with_mock(
        dir.path(),
        &["start"],
        &mock,
        "running", // MOCK_CONTAINER_STATE
        "0.0.1",   // MOCK_CONTAINER_VERSION — old, mismatches config
    );

    assert!(
        !output.status.success(),
        "aibox start should exit non-zero on image version mismatch"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("mismatch"),
        "error output should mention 'mismatch', got:\n{stderr}"
    );
    assert!(
        stderr.contains("aibox sync"),
        "error should suggest running `aibox sync` to resolve, got:\n{stderr}"
    );
}

// ─── Test 4: cmd_start happy path — versions match ───────────────────────────

/// `aibox start` must NOT error when the container's image label matches the
/// version pinned in aibox.toml. This is the normal day-to-day path.
#[test]
fn start_does_not_error_when_versions_match() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "start-match");

    let mock = super::mock_runtime::MockRuntime::new();

    // aibox init writes CARGO_PKG_VERSION to aibox.toml — use the same version
    // as the mock label so the check passes.
    let current_version = env!("CARGO_PKG_VERSION");

    let output = run_in_with_mock(
        dir.path(),
        &["start"],
        &mock,
        "running",       // MOCK_CONTAINER_STATE
        current_version, // MOCK_CONTAINER_VERSION — matches config
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.to_lowercase().contains("mismatch"),
        "aibox start should not report version mismatch when versions match, got:\n{stderr}"
    );
}

// ─── Test 5: update -y exits zero (global_yes wired correctly) ───────────────

/// `aibox update -y` must exit 0. This verifies the global `--yes` flag is
/// correctly threaded through to `cmd_update` so it doesn't hang waiting for
/// a confirmation prompt (even when the registry is unreachable, the command
/// exits early cleanly).
#[test]
fn update_yes_flag_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "update-yes");

    let output = run_in(dir.path(), &["update", "-y"]);

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "aibox update -y should exit 0:\n{combined}"
    );
}

// ─── Test 6: update --dry-run no longer mentions .aibox-version ──────────────

/// `aibox update --dry-run` must NOT contain the old "[dry-run] Would update
/// .aibox-version" message. That write was removed in BACK-060: the image
/// version is tracked in aibox.toml only; `.aibox-version` holds the CLI version.
///
/// If the registry is unreachable the command exits early before the dry-run
/// messages — both outcomes are acceptable for this assertion.
#[test]
fn update_dry_run_does_not_mention_aibox_version_file() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "update-dryrun");

    let output = run_in(dir.path(), &["update", "--dry-run"]);

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output.status.success(),
        "aibox update --dry-run should exit 0:\n{combined}"
    );
    assert!(
        !combined.contains("Would update .aibox-version"),
        "aibox update --dry-run must not mention .aibox-version (removed in BACK-060):\n{combined}"
    );
}

// ─── Test 7: doctor warns on container image version mismatch ────────────────

/// `aibox doctor` must emit a warning when the running container's `aibox.version`
/// label differs from the version in aibox.toml — the container needs a rebuild.
#[test]
fn doctor_warns_on_container_version_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "doctor-ver-mismatch");

    let mock = super::mock_runtime::MockRuntime::new();

    // Simulate: container running with stale image label
    let output = run_in_with_mock(
        dir.path(),
        &["doctor"],
        &mock,
        "running", // MOCK_CONTAINER_STATE
        "0.0.1",   // MOCK_CONTAINER_VERSION — old, mismatches config
    );

    assert!(output.status.success(), "doctor always exits 0");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        combined.to_lowercase().contains("mismatch"),
        "doctor should warn about container image version mismatch, got:\n{combined}"
    );
}

// ─── Test 8: doctor warns on CLI version file mismatch ───────────────────────

/// `aibox doctor` must warn when `.aibox-version` does not match the current
/// CLI version. This indicates the project was last synced with an older CLI
/// and generated files may be stale — user should run `aibox sync`.
#[test]
fn doctor_warns_on_cli_version_file_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "doctor-cli-ver");

    // Tamper .aibox-version to simulate a project last synced by an older CLI
    fs::write(dir.path().join(".aibox-version"), "0.0.1")
        .expect("failed to write .aibox-version");

    let output = run_in(dir.path(), &["doctor"]);

    assert!(output.status.success(), "doctor always exits 0");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        combined.contains("CLI version mismatch"),
        "doctor should warn about CLI version mismatch when .aibox-version is stale, got:\n{combined}"
    );
    assert!(
        combined.contains("aibox sync"),
        "doctor warning should suggest `aibox sync` to update generated files, got:\n{combined}"
    );
}
