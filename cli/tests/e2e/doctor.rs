//! Doctor diagnostic E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn doctor_reports_missing_files() {
    let runner = E2eRunner::new();
    let test = "doctor-missing";
    runner.cleanup(test);

    // Run doctor without any aibox project
    let output = runner.aibox(test, &["doctor"]);
    // Doctor always exits 0 (it's a diagnostic tool)
    assert!(output.status.success(), "doctor should always exit 0");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("aibox.toml") || stderr.contains("Config"),
        "doctor should report missing config: {}",
        stderr
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn doctor_after_init_reports_healthy() {
    let runner = E2eRunner::new();
    let test = "doctor-healthy";
    runner.cleanup(test);

    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);

    let output = runner.aibox(test, &["doctor"]);
    assert!(output.status.success());

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // After init, most checks should pass (except container runtime if not available)
    assert!(
        combined.contains("ok") || combined.contains("OK") || combined.contains("✓") || combined.contains("pass"),
        "doctor should report some healthy checks after init"
    );

    runner.cleanup(test);
}
