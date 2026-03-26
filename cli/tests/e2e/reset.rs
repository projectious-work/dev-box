//! Reset and backup E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn reset_creates_backup() {
    let runner = E2eRunner::new();
    let test = "reset-backup";
    runner.cleanup(test);

    // Init project
    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);
    assert!(runner.file_exists(test, "aibox.toml"));

    // Reset (with backup, auto-confirm)
    let output = runner.aibox(test, &["reset", "--yes"]);
    assert!(
        output.status.success(),
        "reset failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // aibox.toml should be deleted after reset
    assert!(
        !runner.file_exists(test, "aibox.toml"),
        "aibox.toml should be deleted after reset"
    );

    // Backup directory should exist
    assert!(
        runner.dir_exists(test, ".aibox-backup"),
        ".aibox-backup should exist after reset"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn reset_no_backup_deletes_all() {
    let runner = E2eRunner::new();
    let test = "reset-no-backup";
    runner.cleanup(test);

    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);

    let output = runner.aibox(test, &["reset", "--no-backup", "--yes"]);
    assert!(
        output.status.success(),
        "reset --no-backup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!runner.file_exists(test, "aibox.toml"), "aibox.toml should be gone");
    assert!(!runner.dir_exists(test, ".devcontainer"), ".devcontainer should be gone");
    assert!(
        !runner.dir_exists(test, ".aibox-backup"),
        ".aibox-backup should not exist with --no-backup"
    );

    runner.cleanup(test);
}
