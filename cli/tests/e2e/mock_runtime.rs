//! Mock runtime setup for testing CLI command construction.
//!
//! Places mock `docker` and `podman` scripts on PATH and provides helpers
//! for asserting what commands were invoked.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// A mock runtime environment that intercepts docker/podman commands.
///
/// Creates a temporary directory with mock scripts on PATH that log all
/// invocations. Tests can then inspect the log to verify command construction.
pub struct MockRuntime {
    /// Temp directory containing the mock scripts (kept alive for lifetime).
    _dir: tempfile::TempDir,
    /// Path to the mock bin directory (prepended to PATH).
    pub bin_dir: PathBuf,
    /// Path to the invocation log file.
    pub log_file: PathBuf,
}

impl MockRuntime {
    /// Create a new mock runtime environment.
    ///
    /// Copies the mock scripts into a temp directory and returns a MockRuntime
    /// with paths ready to be used as environment overrides.
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("failed to create mock runtime tempdir");
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("failed to create mock bin dir");
        let log_file = dir.path().join("mock.log");

        // Copy mock scripts from the infra directory
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let infra_dir = Path::new(manifest_dir).join("tests/e2e/infra");

        for (script, name) in &[
            ("mock-docker.sh", "docker"),
            ("mock-podman.sh", "podman"),
        ] {
            let src = infra_dir.join(script);
            let dst = bin_dir.join(name);
            fs::copy(&src, &dst).unwrap_or_else(|e| {
                panic!("failed to copy {} to {}: {}", src.display(), dst.display(), e)
            });
            let mut perms = fs::metadata(&dst).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dst, perms).unwrap();
        }

        Self {
            _dir: dir,
            bin_dir,
            log_file,
        }
    }

    /// Build a PATH string with the mock bin dir prepended.
    pub fn path_env(&self) -> String {
        let existing = std::env::var("PATH").unwrap_or_default();
        format!("{}:{}", self.bin_dir.display(), existing)
    }

    /// Read the invocation log and return all lines.
    pub fn invocations(&self) -> Vec<String> {
        match fs::read_to_string(&self.log_file) {
            Ok(content) => content.lines().map(|l| l.to_string()).collect(),
            Err(_) => vec![],
        }
    }

    /// Assert that a specific command pattern appears in the log.
    pub fn assert_invoked(&self, pattern: &str) {
        let invocations = self.invocations();
        assert!(
            invocations.iter().any(|line| line.contains(pattern)),
            "expected invocation matching '{}' but got:\n{}",
            pattern,
            invocations.join("\n")
        );
    }

    /// Assert that a specific command pattern does NOT appear in the log.
    pub fn assert_not_invoked(&self, pattern: &str) {
        let invocations = self.invocations();
        assert!(
            !invocations.iter().any(|line| line.contains(pattern)),
            "did not expect invocation matching '{}' but found it in:\n{}",
            pattern,
            invocations.join("\n")
        );
    }

    /// Get the path to the mock log file (for MOCK_LOG_FILE env var).
    pub fn log_file_str(&self) -> &str {
        self.log_file.to_str().unwrap()
    }
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}
