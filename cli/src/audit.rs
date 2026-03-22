use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::DevBoxConfig;
use crate::output;

/// Check Rust dependencies with cargo audit.
fn check_cargo_audit() -> Result<()> {
    if !Path::new("Cargo.lock").exists() {
        return Ok(()); // Not a Rust project
    }

    if Command::new("cargo")
        .arg("audit")
        .arg("--version")
        .output()
        .is_err()
    {
        output::warn("cargo-audit not installed. Install: cargo install cargo-audit");
        return Ok(());
    }

    output::info("Running cargo audit...");
    let status = Command::new("cargo").arg("audit").status()?;
    if status.success() {
        output::ok("No known vulnerabilities in Rust dependencies");
    } else {
        output::warn("Vulnerabilities found — review cargo audit output above");
    }
    Ok(())
}

/// Check Python dependencies with pip-audit.
fn check_pip_audit() -> Result<()> {
    let has_requirements = Path::new("requirements.txt").exists();
    let has_pyproject = Path::new("pyproject.toml").exists();

    if !has_requirements && !has_pyproject {
        return Ok(()); // Not a Python project
    }

    if Command::new("pip-audit")
        .arg("--version")
        .output()
        .is_err()
    {
        output::warn("pip-audit not installed. Install: pip install pip-audit");
        return Ok(());
    }

    output::info("Running pip-audit...");
    let mut cmd = Command::new("pip-audit");
    if has_requirements {
        cmd.arg("-r").arg("requirements.txt");
    }
    let status = cmd.status()?;
    if status.success() {
        output::ok("No known vulnerabilities in Python dependencies");
    } else {
        output::warn("Vulnerabilities found — review pip-audit output above");
    }
    Ok(())
}

/// Scan the project's container image with trivy.
fn check_trivy(config: &Option<DevBoxConfig>) -> Result<()> {
    let config = match config {
        Some(c) => c,
        None => return Ok(()), // No config, can't determine image
    };

    if Command::new("trivy").arg("--version").output().is_err() {
        output::warn("trivy not installed. See: https://trivy.dev/latest/getting-started/installation/");
        return Ok(());
    }

    let image = format!(
        "ghcr.io/projectious-work/dev-box/{}:latest",
        config.dev_box.image
    );

    output::info(&format!("Running trivy image scan on {}...", image));
    let status = Command::new("trivy")
        .arg("image")
        .arg("--severity")
        .arg("HIGH,CRITICAL")
        .arg(&image)
        .status()?;
    if status.success() {
        output::ok("No HIGH/CRITICAL vulnerabilities in container image");
    } else {
        output::warn("Vulnerabilities found — review trivy output above");
    }
    Ok(())
}

/// Run all available security checks on the project.
pub fn cmd_audit(config_path: &Option<String>) -> Result<()> {
    output::info("Running security audit...");

    let config = DevBoxConfig::from_cli_option(config_path).ok();

    check_cargo_audit()?;
    check_pip_audit()?;
    check_trivy(&config)?;

    output::info("Security audit complete");
    Ok(())
}

/// Lightweight check for doctor: report whether audit tools are available.
pub fn doctor_check_audit_tools() {
    output::info("Security audit tools...");

    let cargo_audit_available = Command::new("cargo")
        .arg("audit")
        .arg("--version")
        .output()
        .is_ok();

    let pip_audit_available = Command::new("pip-audit")
        .arg("--version")
        .output()
        .is_ok();

    let trivy_available = Command::new("trivy")
        .arg("--version")
        .output()
        .is_ok();

    if cargo_audit_available {
        output::ok("cargo-audit available");
    } else {
        output::warn("cargo-audit not installed (optional: cargo install cargo-audit)");
    }

    if pip_audit_available {
        output::ok("pip-audit available");
    } else {
        output::warn("pip-audit not installed (optional: pip install pip-audit)");
    }

    if trivy_available {
        output::ok("trivy available");
    } else {
        output::warn("trivy not installed (optional: https://trivy.dev)");
    }
}
