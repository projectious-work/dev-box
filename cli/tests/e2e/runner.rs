//! E2E test runner — SSH harness for the companion container.
//!
//! Connects to the `aibox-e2e-testrunner` companion container via SSH and executes
//! aibox commands in isolated workspace directories.
//!
//! The aibox binary and addon definitions are deployed to the companion
//! via SCP — no shared volumes. This makes the companion a realistic
//! simulation of a user's host machine.

use std::path::Path;
use std::process::{Command, Output};
use std::sync::Once;

/// Remote paths on the companion container.
const REMOTE_AIBOX_BIN: &str = "/usr/local/bin/aibox";
const REMOTE_ADDONS_DIR: &str = "/opt/aibox/addons";

/// Ensure the binary + addons are deployed exactly once per test run.
static DEPLOY_ONCE: Once = Once::new();

/// SSH-based runner for executing commands on the aibox-e2e-testrunner companion container.
pub struct E2eRunner {
    ssh_key: String,
    host: String,
    port: u16,
    user: String,
}

impl E2eRunner {
    /// Create a runner pointing at the companion container.
    ///
    /// By default, connects to `aibox-e2e-testrunner:22` using the pre-seeded test SSH key.
    /// The `aibox-e2e-testrunner` hostname is resolved via Docker DNS (same compose network).
    pub fn new() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        Self {
            ssh_key: format!("{}/../.devcontainer/ssh-e2e/id_ed25519", manifest_dir),
            host: std::env::var("E2E_HOST").unwrap_or_else(|_| "aibox-e2e-testrunner".to_string()),
            port: std::env::var("E2E_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(22),
            user: "testuser".to_string(),
        }
    }

    /// Common SSH args (reused by exec and scp).
    fn ssh_opts(&self) -> Vec<String> {
        vec![
            "-i".to_string(),
            self.ssh_key.clone(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            "-o".to_string(),
            "ConnectTimeout=5".to_string(),
            "-o".to_string(),
            "LogLevel=ERROR".to_string(),
        ]
    }

    /// Execute a raw command on the companion container via SSH.
    pub fn exec(&self, cmd: &str) -> Output {
        let mut args = self.ssh_opts();
        args.extend([
            "-p".to_string(),
            self.port.to_string(),
            format!("{}@{}", self.user, self.host),
            cmd.to_string(),
        ]);
        Command::new("ssh")
            .args(&args)
            .output()
            .expect("SSH command failed — is aibox-e2e-testrunner running?")
    }

    /// Copy a local file to the companion container via SCP.
    fn scp(&self, local_path: &str, remote_path: &str) {
        let mut args = self.ssh_opts();
        args.extend([
            "-P".to_string(),
            self.port.to_string(),
            local_path.to_string(),
            format!("{}@{}:{}", self.user, self.host, remote_path),
        ]);
        let output = Command::new("scp")
            .args(&args)
            .output()
            .expect("SCP command failed — is aibox-e2e-testrunner running?");
        assert!(
            output.status.success(),
            "scp {} -> {} failed: {}",
            local_path,
            remote_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Recursively copy a local directory to the companion via SCP.
    fn scp_recursive(&self, local_path: &str, remote_path: &str) {
        let mut args = self.ssh_opts();
        args.extend([
            "-r".to_string(),
            "-P".to_string(),
            self.port.to_string(),
            local_path.to_string(),
            format!("{}@{}:{}", self.user, self.host, remote_path),
        ]);
        let output = Command::new("scp")
            .args(&args)
            .output()
            .expect("SCP command failed — is aibox-e2e-testrunner running?");
        assert!(
            output.status.success(),
            "scp -r {} -> {} failed: {}",
            local_path,
            remote_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Deploy the aibox binary and addon definitions to the companion.
    ///
    /// Called once per test run (guarded by `Once`). SCPs the freshly-built
    /// binary to `/usr/local/bin/aibox`, the addon YAMLs to `/opt/aibox/addons/`,
    /// and container image assets (vimrc, bin scripts) to `/opt/aibox/`.
    pub fn deploy(&self) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let binary = format!("{}/target/debug/aibox", manifest_dir);
        let addons = format!("{}/../addons", manifest_dir);

        assert!(
            Path::new(&binary).exists(),
            "aibox binary not found at {}. Run `cargo build` first.",
            binary
        );

        // Create remote directories
        self.exec(&format!(
            "sudo mkdir -p {} && sudo chown testuser:testuser {}",
            REMOTE_ADDONS_DIR, REMOTE_ADDONS_DIR
        ));

        // Deploy binary
        let tmp_bin = "/tmp/aibox";
        self.scp(&binary, tmp_bin);
        self.exec(&format!(
            "sudo mv {} {} && sudo chmod +x {}",
            tmp_bin, REMOTE_AIBOX_BIN, REMOTE_AIBOX_BIN
        ));

        // Deploy addons (recursive copy)
        self.exec(&format!("rm -rf {}/*", REMOTE_ADDONS_DIR));
        self.scp_recursive(&addons, "/opt/aibox/");

        // Deploy container image assets for visual keybinding tests.
        // The full vimrc (with leader key mappings) and bin scripts live in the
        // container image, not in the seeded .aibox-home. Deploy them so the
        // aibox-e2e-testrunner can simulate the full container environment.
        let image_config = format!("{}/../images/base-debian/config", manifest_dir);
        self.deploy_image_asset(&format!("{}/vimrc", image_config), "/opt/aibox/vimrc", false);
        for (src, dst) in &[
            ("bin/open-in-editor.sh", "open-in-editor"),
            ("bin/vim-loop.sh", "vim-loop"),
        ] {
            self.deploy_image_asset(
                &format!("{}/{}", image_config, src),
                &format!("/usr/local/bin/{}", dst),
                true,
            );
        }

        // Verify deployment
        let output = self.exec(&format!("{} --version", REMOTE_AIBOX_BIN));
        assert!(
            output.status.success(),
            "deployed aibox binary is not executable: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Deploy a single file from a local path to a remote path on the companion.
    /// Skips silently if the local file does not exist.
    fn deploy_image_asset(&self, local_path: &str, remote_path: &str, executable: bool) {
        if !Path::new(local_path).exists() {
            return;
        }
        let file_name = Path::new(local_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "asset".to_string());
        let tmp = format!("/tmp/aibox_asset_{}", file_name);
        self.scp(local_path, &tmp);
        let chmod = if executable { "755" } else { "644" };
        self.exec(&format!(
            "sudo mv {} {} && sudo chmod {} {}",
            tmp, remote_path, chmod, remote_path
        ));
    }

    /// Ensure the binary is deployed (called automatically, once per test run).
    pub fn ensure_deployed(&self) {
        // We need to capture `self` for the closure, but Once::call_once
        // requires a static lifetime. Work around by checking a file marker.
        DEPLOY_ONCE.call_once(|| {
            self.deploy();
        });
    }

    /// Execute an aibox command in an isolated workspace directory.
    ///
    /// Creates `/workspaces/<test_name>/` on the companion if it doesn't exist.
    /// Automatically ensures the binary is deployed on first call.
    pub fn aibox(&self, test_name: &str, args: &[&str]) -> Output {
        self.ensure_deployed();
        let workspace = format!("/workspaces/{}", test_name);
        let cmd = format!(
            "mkdir -p {workspace} && cd {workspace} && AIBOX_ADDONS_DIR={} {} {}",
            REMOTE_ADDONS_DIR,
            REMOTE_AIBOX_BIN,
            args.join(" ")
        );
        self.exec(&cmd)
    }

    /// Read a file from the companion container.
    pub fn read_file(&self, test_name: &str, path: &str) -> String {
        let cmd = format!("cat /workspaces/{}/{}", test_name, path);
        let output = self.exec(&cmd);
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    /// Write content to a file on the companion container.
    pub fn write_file(&self, test_name: &str, path: &str, content: &str) {
        let workspace = format!("/workspaces/{}", test_name);
        let full_path = format!("{}/{}", workspace, path);
        let cmd = format!(
            "mkdir -p {workspace} && mkdir -p $(dirname {full_path}) && cat > {full_path} << 'AIBOX_E2E_EOF'\n{content}\nAIBOX_E2E_EOF"
        );
        let output = self.exec(&cmd);
        assert!(
            output.status.success(),
            "write_file failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Check if a file exists on the companion container.
    pub fn file_exists(&self, test_name: &str, path: &str) -> bool {
        let cmd = format!("test -f /workspaces/{}/{}", test_name, path);
        self.exec(&cmd).status.success()
    }

    /// Check if a directory exists on the companion container.
    pub fn dir_exists(&self, test_name: &str, path: &str) -> bool {
        let cmd = format!("test -d /workspaces/{}/{}", test_name, path);
        self.exec(&cmd).status.success()
    }

    /// Clean up a test workspace directory.
    pub fn cleanup(&self, test_name: &str) {
        self.exec(&format!("rm -rf /workspaces/{}", test_name));
    }

    /// Execute a command inside a running aibox container (via podman exec).
    ///
    /// Used for smoke tests that verify tools are installed and functional.
    pub fn container_exec(&self, container_name: &str, cmd: &str) -> Output {
        self.exec(&format!("podman exec {} {}", container_name, cmd))
    }

    /// Assert the companion container is reachable.
    pub fn assert_reachable(&self) {
        let output = self.exec("echo ok");
        assert!(
            output.status.success(),
            "aibox-e2e-testrunner is not reachable via SSH. Is the companion container running?\n\
             stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim() == "ok",
            "unexpected response from aibox-e2e-testrunner: '{}'",
            stdout.trim()
        );
    }
}

impl Default for E2eRunner {
    fn default() -> Self {
        Self::new()
    }
}
