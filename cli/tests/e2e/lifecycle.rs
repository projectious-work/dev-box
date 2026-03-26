//! Container lifecycle E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! Tests the full init → sync → start → stop → remove lifecycle.

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn companion_is_reachable() {
    let runner = E2eRunner::new();
    runner.assert_reachable();
}

#[test]
#[serial]
fn lifecycle_init_sync() {
    let runner = E2eRunner::new();
    let test = "lifecycle-init-sync";
    runner.cleanup(test);

    // Init
    let output = runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files created
    assert!(runner.file_exists(test, "aibox.toml"), "aibox.toml should exist");
    assert!(runner.file_exists(test, ".devcontainer/Dockerfile"), "Dockerfile should exist");
    assert!(runner.file_exists(test, ".devcontainer/docker-compose.yml"), "docker-compose.yml should exist");
    assert!(runner.file_exists(test, "CLAUDE.md"), "CLAUDE.md should exist");

    // Sync (--no-build: config-only, no GHCR pull needed)
    let output = runner.aibox(test, &["sync", "--no-build"]);
    assert!(
        output.status.success(),
        "sync failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn claudemd_preserved_on_sync() {
    let runner = E2eRunner::new();
    let test = "claudemd-preserve";
    runner.cleanup(test);

    // Init
    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);

    // Modify CLAUDE.md with user content
    runner.write_file(test, "CLAUDE.md", "# My Custom CLAUDE.md\n\nUser-specific content here.\n");

    // Sync should not overwrite CLAUDE.md
    runner.aibox(test, &["sync"]);

    let content = runner.read_file(test, "CLAUDE.md");
    assert!(
        content.contains("User-specific content"),
        "CLAUDE.md user content should be preserved after sync"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn generated_files_overwritten_on_sync() {
    let runner = E2eRunner::new();
    let test = "gen-overwrite";
    runner.cleanup(test);

    // Init
    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);

    // Tamper with generated Dockerfile
    runner.write_file(test, ".devcontainer/Dockerfile", "# tampered\nFROM scratch\n");

    // Sync should regenerate it
    runner.aibox(test, &["sync"]);

    let dockerfile = runner.read_file(test, ".devcontainer/Dockerfile");
    assert!(
        !dockerfile.contains("# tampered"),
        "Dockerfile should be regenerated, not contain tampered content"
    );
    assert!(
        dockerfile.contains("ghcr.io") || dockerfile.contains("FROM"),
        "Dockerfile should contain valid generated content"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn status_without_container_shows_missing() {
    let runner = E2eRunner::new();
    let test = "status-missing";
    runner.cleanup(test);

    runner.aibox(test, &["init", "--name", test, "--base", "debian", "--process", "core"]);

    let output = runner.aibox(test, &["status"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("missing") || combined.contains("Missing") || combined.contains("not found"),
        "status should report missing when no container exists: {}",
        combined
    );

    runner.cleanup(test);
}
/// Verify that `aibox init --process managed` creates the expected context files.
///
/// Covers BACK-053: the `managed` preset (core + tracking + standups + handover)
/// must scaffold BACKLOG.md, DECISIONS.md, STANDUPS.md, and a session-template.
#[test]
#[serial]
fn init_with_managed_preset_creates_context_files() {
    let runner = E2eRunner::new();
    let test = "init-managed-preset";
    runner.cleanup(test);

    let output = runner.aibox(
        test,
        &["init", "--name", test, "--base", "debian", "--process", "managed"],
    );
    assert!(
        output.status.success(),
        "init --process managed failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(runner.file_exists(test, "CLAUDE.md"), "CLAUDE.md should exist");
    assert!(runner.file_exists(test, "aibox.toml"), "aibox.toml should exist");

    // tracking package (managed preset includes tracking)
    assert!(
        runner.file_exists(test, "context/BACKLOG.md"),
        "context/BACKLOG.md should exist for managed preset"
    );
    assert!(
        runner.file_exists(test, "context/DECISIONS.md"),
        "context/DECISIONS.md should exist for managed preset"
    );

    // standups package
    assert!(
        runner.file_exists(test, "context/STANDUPS.md"),
        "context/STANDUPS.md should exist for managed preset"
    );

    // handover package
    assert!(
        runner.file_exists(test, "context/project-notes/session-template.md"),
        "session-template.md should exist for managed preset"
    );

    // aibox.toml should record the preset name
    let toml = runner.read_file(test, "aibox.toml");
    assert!(
        toml.contains("managed"),
        "aibox.toml should reference managed process, got:\n{}",
        toml
    );

    runner.cleanup(test);
}

/// Verify that `aibox init --process software` scaffolds architecture processes.
///
/// Covers BACK-053: the `software` preset must scaffold its additional packages.
#[test]
#[serial]
fn init_with_software_preset_creates_code_files() {
    let runner = E2eRunner::new();
    let test = "init-software-preset";
    runner.cleanup(test);

    let output = runner.aibox(
        test,
        &["init", "--name", test, "--base", "debian", "--process", "software"],
    );
    assert!(
        output.status.success(),
        "init --process software failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // tracking + standups from managed base
    assert!(runner.file_exists(test, "context/BACKLOG.md"));
    assert!(runner.file_exists(test, "context/STANDUPS.md"));

    // software preset adds process declarations
    assert!(
        runner.dir_exists(test, "context/processes"),
        "context/processes/ should exist for software preset"
    );

    runner.cleanup(test);
}
