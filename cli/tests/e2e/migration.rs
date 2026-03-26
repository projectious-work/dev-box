//! Version migration E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn sync_updates_version_file() {
    let runner = E2eRunner::new();
    let test = "migration-version";
    runner.cleanup(test);

    // Init project
    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);

    // Tamper with .aibox-version to simulate older version
    runner.write_file(test, ".aibox-version", "0.1.0");

    // Sync should update the version file
    runner.aibox(test, &["sync"]);

    let version = runner.read_file(test, ".aibox-version");
    assert!(
        !version.trim().is_empty(),
        ".aibox-version should not be empty after sync"
    );
    assert!(
        version.trim() != "0.1.0",
        ".aibox-version should be updated from 0.1.0 to current version"
    );

    runner.cleanup(test);
}
