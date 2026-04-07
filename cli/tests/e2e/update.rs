//! Update command E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! Tests `aibox update` behavior in a derived project context.

use serial_test::serial;

use super::runner::E2eRunner;

/// Verify that `aibox update --check` successfully fetches version info from GHCR.
///
/// The GHCR packages are public, so anonymous token exchange should succeed and
/// the CLI should find published tags matching the `base-debian-v*` pattern.
/// This test catches tag-prefix mismatches between the CLI and the registry.
#[test]
#[serial]
fn update_check_fetches_from_registry() {
    let runner = E2eRunner::new();
    let test = "update-registry-fetch";
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
            "managed",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init_out.stderr)
    );

    // Run update --check — should fetch real version info from GHCR.
    let output = runner.aibox(test, &["update", "--check"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        output.status.success(),
        "aibox update --check should exit 0.\nOutput:\n{}",
        combined
    );

    // Verify the registry fetch succeeded — output should show the image is
    // up to date or that a new version is available, NOT a "Could not" warning.
    assert!(
        !combined.contains("Could not fetch latest image version"),
        "expected successful registry fetch, but got a warning.\nOutput:\n{}",
        combined
    );
    assert!(
        !combined.contains("No published tags found"),
        "tag prefix mismatch: no tags matched the expected pattern.\nOutput:\n{}",
        combined
    );

    // Should report image status (either up-to-date or upgrade available)
    assert!(
        combined.contains("is up to date") || combined.contains("New image version available"),
        "expected image version status in output, got:\n{}",
        combined
    );

    runner.cleanup(test);
}

/// Verify `aibox update --dry-run` fetches the latest version from GHCR without
/// applying changes.
///
/// This exercises the full `do_upgrade` code path including the tag-prefix
/// matching, but stops before writing to aibox.toml thanks to `--dry-run`.
#[test]
#[serial]
fn update_dry_run_fetches_from_registry() {
    let runner = E2eRunner::new();
    let test = "update-dry-run";
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
            "managed",
        ],
    );

    let output = runner.aibox(test, &["update", "--dry-run"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        output.status.success(),
        "aibox update --dry-run should exit 0.\nOutput:\n{}",
        combined
    );

    // Verify the registry fetch succeeded (not a warning/fallback)
    assert!(
        !combined.contains("Could not fetch latest image version"),
        "expected successful registry fetch, but got a warning.\nOutput:\n{}",
        combined
    );

    // Should show current version and either "already at the latest" or "[dry-run]"
    assert!(
        combined.contains("Current image version:"),
        "expected 'Current image version:' in output, got:\n{}",
        combined
    );
    assert!(
        combined.contains("is already at the latest") || combined.contains("[dry-run]"),
        "expected dry-run or up-to-date output, got:\n{}",
        combined
    );

    runner.cleanup(test);
}
