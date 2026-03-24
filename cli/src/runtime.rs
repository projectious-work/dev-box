use anyhow::{Context, Result, bail};
use std::process::Command;

/// Container state as detected by runtime inspect.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerState {
    Running,
    Stopped,
    Missing,
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerState::Running => write!(f, "running"),
            ContainerState::Stopped => write!(f, "stopped"),
            ContainerState::Missing => write!(f, "missing"),
        }
    }
}

/// Abstraction over podman/docker runtime.
#[derive(Debug, Clone)]
pub struct Runtime {
    pub compose_bin: Vec<String>,
    pub runtime_bin: String,
}

impl Runtime {
    /// Detect available container runtime.
    ///
    /// Checks both presence on PATH and whether the daemon is actually
    /// responsive (`docker info` / `podman info`). Prefers docker over
    /// podman because OrbStack (common on macOS) provides docker and
    /// podman may be installed but not running.
    pub fn detect() -> Result<Self> {
        // Prefer docker — OrbStack and Docker Desktop both provide it
        if runtime_is_responsive("docker") {
            return Ok(Runtime {
                compose_bin: vec!["docker".to_string(), "compose".to_string()],
                runtime_bin: "docker".to_string(),
            });
        }

        // Fall back to podman if docker isn't responsive
        if runtime_is_responsive("podman") {
            return Ok(Runtime {
                compose_bin: vec!["podman".to_string(), "compose".to_string()],
                runtime_bin: "podman".to_string(),
            });
        }

        // Neither is responsive — give a helpful error
        let docker_present = command_exists("docker");
        let podman_present = command_exists("podman");

        match (docker_present, podman_present) {
            (true, _) => bail!(
                "docker found on PATH but not responding. \
                 Is Docker Desktop or OrbStack running?"
            ),
            (_, true) => bail!(
                "podman found on PATH but not responding. \
                 Try: podman machine start"
            ),
            _ => bail!("Neither podman nor docker found. Please install one of them."),
        }
    }

    /// Get the container state by inspecting it directly.
    pub fn container_status(&self, name: &str) -> Result<ContainerState> {
        let output = Command::new(&self.runtime_bin)
            .args(["inspect", "--format", "{{.State.Status}}", name])
            .output()
            .context("Failed to run container inspect")?;

        if !output.status.success() {
            // Container doesn't exist
            return Ok(ContainerState::Missing);
        }

        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match status.as_str() {
            "running" => Ok(ContainerState::Running),
            _ => Ok(ContainerState::Stopped),
        }
    }

    /// Run compose build.
    pub fn compose_build(&self, compose_file: &str, no_cache: bool) -> Result<()> {
        let mut args: Vec<&str> = vec!["-f", compose_file, "build"];
        if no_cache {
            args.push("--no-cache");
        }

        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose build failed");
        }
        Ok(())
    }

    /// Run compose up -d for a service.
    pub fn compose_up(&self, compose_file: &str, service: &str) -> Result<()> {
        let args = vec!["-f", compose_file, "up", "-d", service];
        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose up failed");
        }
        Ok(())
    }

    /// Run compose stop for a service.
    pub fn compose_stop(&self, compose_file: &str, service: &str) -> Result<()> {
        let args = vec!["-f", compose_file, "stop", service];
        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose stop failed");
        }
        Ok(())
    }

    /// Run compose down for a service (stop + remove container and network).
    pub fn compose_down(&self, compose_file: &str) -> Result<()> {
        let args = vec!["-f", compose_file, "down"];
        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose down failed");
        }
        Ok(())
    }

    /// Exec interactively into a container as the specified user.
    pub fn exec_interactive(&self, container: &str, user: &str, cmd: &[&str]) -> Result<()> {
        let mut args = vec!["exec", "-it", "-u", user, container];
        args.extend_from_slice(cmd);

        let status = Command::new(&self.runtime_bin)
            .args(&args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to exec into container")?;

        if !status.success() {
            bail!(
                "Exec into container failed (exit code: {:?})",
                status.code()
            );
        }
        Ok(())
    }

    /// Wait for a container to reach running state, polling every 500ms.
    pub fn wait_for_running(&self, name: &str, timeout_ms: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let poll_interval = std::time::Duration::from_millis(500);

        loop {
            if let Ok(ContainerState::Running) = self.container_status(name) {
                return Ok(());
            }

            if start.elapsed() > timeout {
                bail!(
                    "Timed out waiting for container '{}' to start ({}ms)",
                    name,
                    timeout_ms
                );
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Run a compose command, inheriting stdio.
    fn run_compose(&self, args: &[&str]) -> Result<std::process::ExitStatus> {
        let (program, base_args) = match self.compose_bin.len() {
            1 => (self.compose_bin[0].as_str(), vec![]),
            _ => (
                self.compose_bin[0].as_str(),
                self.compose_bin[1..]
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            ),
        };

        let mut full_args = base_args;
        full_args.extend_from_slice(args);

        Command::new(program)
            .args(&full_args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to run compose command")
    }
}

/// Check if a command exists on PATH.
pub(crate) fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a container runtime is on PATH AND responsive.
/// Runs `<runtime> info` with a short timeout to verify the daemon is up.
fn runtime_is_responsive(cmd: &str) -> bool {
    if !command_exists(cmd) {
        return false;
    }

    // `docker info` / `podman info` exits 0 if the daemon is reachable
    Command::new(cmd)
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_state_display_running() {
        assert_eq!(format!("{}", ContainerState::Running), "running");
    }

    #[test]
    fn container_state_display_stopped() {
        assert_eq!(format!("{}", ContainerState::Stopped), "stopped");
    }

    #[test]
    fn container_state_display_missing() {
        assert_eq!(format!("{}", ContainerState::Missing), "missing");
    }

    #[test]
    fn container_state_equality() {
        assert_eq!(ContainerState::Running, ContainerState::Running);
        assert_ne!(ContainerState::Running, ContainerState::Stopped);
        assert_ne!(ContainerState::Stopped, ContainerState::Missing);
    }

    #[test]
    fn command_exists_for_bash() {
        // bash should exist on any Linux system
        assert!(command_exists("bash"), "bash should be found on PATH");
    }

    #[test]
    fn command_exists_for_nonexistent() {
        assert!(!command_exists("this_command_does_not_exist_xyz123"));
    }

    #[test]
    fn runtime_struct_fields() {
        let rt = Runtime {
            compose_bin: vec!["docker".to_string(), "compose".to_string()],
            runtime_bin: "docker".to_string(),
        };
        assert_eq!(rt.runtime_bin, "docker");
        assert_eq!(rt.compose_bin.len(), 2);
    }

    #[test]
    fn container_status_for_nonexistent_container() {
        // This test requires docker or podman to be available
        let rt = if runtime_is_responsive("docker") {
            Runtime {
                compose_bin: vec!["docker".to_string(), "compose".to_string()],
                runtime_bin: "docker".to_string(),
            }
        } else if runtime_is_responsive("podman") {
            Runtime {
                compose_bin: vec!["podman".to_string(), "compose".to_string()],
                runtime_bin: "podman".to_string(),
            }
        } else {
            // No responsive runtime — skip
            return;
        };
        let state = rt.container_status("nonexistent_container_xyz123").unwrap();
        assert_eq!(state, ContainerState::Missing);
    }
}
