//! Update command E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! Tests `aibox update` behavior in a derived project context.

use serial_test::serial;

use super::runner::E2eRunner;

/// Verify that `aibox update` runs in a freshly init'd project without crashing.
///
/// Covers BACK-058: `aibox update` was exiting with `ERR http status: 401`
/// (hard failure) when GHCR returned 401 for anonymous pulls. Fixed to warn
/// and exit cleanly instead.
#[test]
#[serial]
fn update_runs_without_crashing_in_derived_project() {
    let runner = E2eRunner::new();
    let test = "update-graceful";
    runner.cleanup(test);

    // Init a derived project
    let init_out = runner.aibox(
        test,
        &[
            "init",
            "--name",
            test,
            "--base",
            "debian",
            "--process",
            "core",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init_out.stderr)
    );

    // Run update — must exit 0 regardless of registry availability.
    // Before the BACK-058 fix, a 401 from GHCR caused a hard failure (`ERR http status: 401`).
    // Now it warns and returns Ok(()).
    let output = runner.aibox(test, &["update"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        output.status.success(),
        "aibox update should exit 0 even when GHCR returns 401.\nOutput:\n{}",
        combined
    );

    // Verify command produced expected preamble output
    assert!(
        combined.contains("Current image version:"),
        "expected 'Current image version:' in output, got:\n{}",
        combined
    );

    // Verify it did not hard-fail with the bare "ERR http status: 401" message
    let stderr_str = stderr.to_string();
    assert!(
        !stderr_str.contains("ERR http status: 401"),
        "expected 401 to be handled as a warning, not a hard error.\nstderr:\n{}",
        stderr_str
    );

    runner.cleanup(test);
}

/// Verify `aibox update --check` exits cleanly in a derived project.
///
/// `--check` is a read-only mode (no writes). It should not crash even if
/// the network is unavailable.
#[test]
#[serial]
fn update_check_exits_cleanly() {
    let runner = E2eRunner::new();
    let test = "update-check";
    runner.cleanup(test);

    runner.aibox(
        test,
        &[
            "init",
            "--name",
            test,
            "--base",
            "debian",
            "--process",
            "core",
        ],
    );

    let output = runner.aibox(test, &["update", "--check"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output.status.success(),
        "aibox update --check should exit 0.\nOutput:\n{}",
        combined
    );
    assert!(
        combined.contains("Current CLI version:") || combined.contains("Checking for updates"),
        "expected check output, got:\n{}",
        combined
    );

    runner.cleanup(test);
}
