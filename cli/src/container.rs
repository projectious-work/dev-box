use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::DevBoxConfig;
use crate::context;
use crate::generate;
use crate::output;
use crate::runtime::{ContainerState, Runtime};
use crate::seed;

/// Get the compose file path.
fn compose_file() -> &'static str {
    crate::config::COMPOSE_FILE
}

/// Build command: load config, generate files, run compose build.
pub fn cmd_build(config_path: &Option<String>, no_cache: bool) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;

    generate::generate_all(&config)?;

    output::info("Building container image...");
    runtime.compose_build(compose_file(), no_cache)?;
    output::ok("Build complete");

    Ok(())
}

/// Start command: seed, generate, ensure running, attach.
pub fn cmd_start(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    // Seed .root/ directory
    seed::seed_root_dir(&config)?;

    // Generate devcontainer files
    generate::generate_all(&config)?;

    // Check current state
    let state = runtime.container_status(name)?;
    match state {
        ContainerState::Running => {
            output::info("Container already running");
        }
        state @ (ContainerState::Stopped | ContainerState::Missing) => {
            let action = if state == ContainerState::Stopped {
                "Starting stopped"
            } else {
                "Creating and starting"
            };
            output::info(&format!("{} container...", action));
            runtime.compose_up(compose_file(), name)?;
            runtime.wait_for_running(name, 7500)?;
            output::ok("Container started");
        }
    }

    // Attach via zellij
    output::info("Attaching via zellij...");
    runtime.exec_interactive(name, &["zellij", "--layout", "dev"])?;

    Ok(())
}

/// Stop command.
pub fn cmd_stop(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    match state {
        ContainerState::Running => {
            output::info("Stopping container...");
            runtime.compose_stop(compose_file(), name)?;
            output::ok("Container stopped");
        }
        ContainerState::Stopped => {
            output::info("Container is already stopped");
        }
        ContainerState::Missing => {
            output::warn("No container found");
        }
    }

    Ok(())
}

/// Attach command.
pub fn cmd_attach(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    if state != ContainerState::Running {
        bail!("Container '{}' is not running. Run 'dev-box start' first.", name);
    }

    output::info("Attaching via zellij...");
    runtime.exec_interactive(name, &["zellij", "--layout", "dev"])?;

    Ok(())
}

/// Status command.
pub fn cmd_status(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    match state {
        ContainerState::Running => {
            output::ok(&format!("Container '{}' is running", name));
        }
        ContainerState::Stopped => {
            output::warn(&format!("Container '{}' is stopped", name));
        }
        ContainerState::Missing => {
            output::warn(&format!("Container '{}' does not exist", name));
        }
    }

    Ok(())
}

/// Init command: create a dev-box.toml and generate files.
pub fn cmd_init(
    config_path: &Option<String>,
    name: Option<String>,
    image: Option<String>,
    process: Option<String>,
) -> Result<()> {
    use crate::config::{
        AudioSection, ContainerSection, ContextSection, DevBoxConfig, DevBoxSection,
        ImageFlavor, ProcessFlavor,
    };

    let toml_path = config_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dev-box.toml"));

    if toml_path.exists() {
        bail!(
            "Config file already exists: {}. Delete it first or edit it directly.",
            toml_path.display()
        );
    }

    // Determine project name from arg or current directory name
    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-project".to_string())
    });

    let image_flavor = match image {
        Some(s) => ImageFlavor::from_str_loose(&s)?,
        None => ImageFlavor::Base,
    };

    let process_flavor = match process {
        Some(s) => ProcessFlavor::from_str_loose(&s)?,
        None => ProcessFlavor::Product,
    };

    let config = DevBoxConfig {
        dev_box: DevBoxSection {
            version: "0.1.0".to_string(),
            image: image_flavor,
            process: process_flavor,
        },
        container: ContainerSection {
            name: project_name.clone(),
            hostname: project_name,
            ports: vec![],
            extra_packages: vec![],
            extra_volumes: vec![],
            environment: std::collections::HashMap::new(),
        },
        context: ContextSection::default(),
        audio: AudioSection::default(),
    };

    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    std::fs::write(&toml_path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", toml_path.display(), e))?;

    output::ok(&format!("Created {}", toml_path.display()));

    // Generate devcontainer files
    generate::generate_all(&config)?;

    // Scaffold context directory based on process flavor
    context::scaffold_context(&config.dev_box.process, &config.container.name)?;

    output::ok("Project initialized. Edit dev-box.toml to customize, then run: dev-box start");

    Ok(())
}

/// Generate command.
pub fn cmd_generate(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    generate::generate_all(&config)?;
    output::ok("Generation complete");
    Ok(())
}

