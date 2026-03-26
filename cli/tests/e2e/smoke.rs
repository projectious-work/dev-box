//! Application smoke tests — verify tools run inside containers.
//!
//! Requires the e2e-runner companion container with podman (feature = "e2e").
//! These tests build actual container images and verify installed tools.
//!
//! NOTE: These tests are slow (image build takes time). They are the most
//! comprehensive validation that the full pipeline works end-to-end.
//!
//! These tests are placeholders — they will be fully implemented once
//! the companion container's podman setup is validated.

use serial_test::serial;

use super::runner::E2eRunner;

/// Verify that podman is available on the companion container.
#[test]
#[serial]
fn podman_available_on_companion() {
    let runner = E2eRunner::new();
    let output = runner.exec("podman --version");
    assert!(
        output.status.success(),
        "podman should be available on e2e-runner: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("podman"),
        "podman --version should output version info"
    );
}

/// Verify that podman can pull a minimal image (validates rootless setup).
#[test]
#[serial]
#[ntest::timeout(120_000)] // 2 minutes for image pull
fn podman_can_pull_image() {
    let runner = E2eRunner::new();
    let output = runner.exec("podman pull --quiet docker.io/library/alpine:latest");
    assert!(
        output.status.success(),
        "podman pull should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Verify that podman can run a container (validates full rootless pipeline).
#[test]
#[serial]
#[ntest::timeout(120_000)]
fn podman_can_run_container() {
    let runner = E2eRunner::new();
    let output = runner.exec("podman run --rm docker.io/library/alpine:latest echo hello-e2e");
    assert!(
        output.status.success(),
        "podman run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello-e2e"),
        "container should output hello-e2e"
    );
}
