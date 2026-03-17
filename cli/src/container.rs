use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::{DevBoxConfig, ImageFlavor, ProcessFlavor};
use crate::context;
use crate::generate;
use crate::output;
use crate::runtime::{ContainerState, Runtime};
use crate::seed;

/// Labels for image flavor selection (order matters — matches ImageFlavor variants).
const IMAGE_FLAVOR_ITEMS: &[&str] = &[
    "base",
    "python",
    "latex",
    "typst",
    "rust",
    "python-latex",
    "python-typst",
    "rust-latex",
];

/// Labels for process flavor selection (order matters — matches ProcessFlavor variants).
const PROCESS_FLAVOR_ITEMS: &[&str] = &["minimal", "managed", "research", "product"];

/// Determine the default project name from the current directory.
fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string())
}

/// Resolve init values, prompting interactively when `interactive` is true and
/// the corresponding argument is `None`.
pub fn resolve_init_values(
    name: Option<String>,
    image: Option<String>,
    process: Option<String>,
    interactive: bool,
) -> Result<(String, ImageFlavor, ProcessFlavor)> {
    // --- project name ---
    let project_name = match name {
        Some(n) => n,
        None if interactive => {
            let default = default_project_name();
            dialoguer::Input::<String>::new()
                .with_prompt("Project name")
                .default(default)
                .interact_text()?
        }
        None => default_project_name(),
    };

    // --- image flavor ---
    let image_flavor = match image {
        Some(s) => ImageFlavor::from_str_loose(&s)?,
        None if interactive => {
            let idx = dialoguer::Select::new()
                .with_prompt("Image flavor")
                .items(IMAGE_FLAVOR_ITEMS)
                .default(0)
                .interact()?;
            ImageFlavor::from_str_loose(IMAGE_FLAVOR_ITEMS[idx])?
        }
        None => ImageFlavor::Base,
    };

    // --- process flavor ---
    let process_flavor = match process {
        Some(s) => ProcessFlavor::from_str_loose(&s)?,
        None if interactive => {
            let idx = dialoguer::Select::new()
                .with_prompt("Work process")
                .items(PROCESS_FLAVOR_ITEMS)
                .default(3)
                .interact()?;
            ProcessFlavor::from_str_loose(PROCESS_FLAVOR_ITEMS[idx])?
        }
        None => ProcessFlavor::Product,
    };

    Ok((project_name, image_flavor, process_flavor))
}

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
        bail!(
            "Container '{}' is not running. Run 'dev-box start' first.",
            name
        );
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

    // Detect whether stdin is a TTY to decide if we can prompt interactively
    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());

    let (project_name, image_flavor, process_flavor) =
        resolve_init_values(name, image, process, interactive)?;

    let config = DevBoxConfig {
        dev_box: DevBoxSection {
            version: env!("CARGO_PKG_VERSION").to_string(),
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

    config.validate()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_init_values_non_interactive_defaults() {
        let (name, image, process) =
            resolve_init_values(None, None, None, false).expect("should succeed");

        // Name defaults to current directory name (or "my-project" fallback)
        assert!(!name.is_empty(), "name should not be empty");

        assert_eq!(image, ImageFlavor::Base);
        assert_eq!(process, ProcessFlavor::Product);
    }

    #[test]
    fn resolve_init_values_explicit_args_override() {
        // Even with interactive=true, explicit values should be used without prompting
        let (name, image, process) = resolve_init_values(
            Some("my-app".to_string()),
            Some("python-latex".to_string()),
            Some("research".to_string()),
            true,
        )
        .expect("should succeed with explicit args");

        assert_eq!(name, "my-app");
        assert_eq!(image, ImageFlavor::PythonLatex);
        assert_eq!(process, ProcessFlavor::Research);
    }

    #[test]
    fn resolve_init_values_explicit_args_non_interactive() {
        let (name, image, process) = resolve_init_values(
            Some("test-proj".to_string()),
            Some("rust".to_string()),
            Some("minimal".to_string()),
            false,
        )
        .expect("should succeed");

        assert_eq!(name, "test-proj");
        assert_eq!(image, ImageFlavor::Rust);
        assert_eq!(process, ProcessFlavor::Minimal);
    }

    #[test]
    fn resolve_init_values_invalid_image_rejected() {
        let result = resolve_init_values(
            Some("x".to_string()),
            Some("golang".to_string()),
            None,
            false,
        );
        assert!(result.is_err(), "should reject unknown image flavor");
    }

    #[test]
    fn resolve_init_values_invalid_process_rejected() {
        let result = resolve_init_values(
            Some("x".to_string()),
            None,
            Some("waterfall".to_string()),
            false,
        );
        assert!(result.is_err(), "should reject unknown process flavor");
    }
}
