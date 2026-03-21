use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::{AiProvider, DevBoxConfig, ImageFlavor, ProcessFlavor};
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
    image: Option<ImageFlavor>,
    process: Option<ProcessFlavor>,
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
        Some(f) => f,
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
        Some(f) => f,
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
pub fn cmd_start(config_path: &Option<String>, layout: &str) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    // Seed .dev-box-home/ directory
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
    output::info(&format!("Attaching via zellij (layout: {})...", layout));
    runtime.exec_interactive(name, &["zellij", "--layout", layout])?;

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

/// Remove command: stop and remove the container.
pub fn cmd_remove(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    if state == ContainerState::Missing {
        output::info("No container found");
        return Ok(());
    }

    output::info("Stopping and removing container...");
    runtime.compose_down(compose_file())?;
    output::ok(&format!("Container '{}' removed", name));

    Ok(())
}

/// Attach command.
pub fn cmd_attach(config_path: &Option<String>, layout: &str) -> Result<()> {
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

    output::info(&format!("Attaching via zellij (layout: {})...", layout));
    runtime.exec_interactive(name, &["zellij", "--layout", layout])?;

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

/// Serialize config to TOML with comprehensive comments.
fn serialize_config_with_comments(config: &DevBoxConfig) -> String {
    let mut out = String::new();

    // [dev-box] section
    out.push_str("# dev-box.toml — project configuration for dev-box.\n");
    out.push_str("# All generated files (.devcontainer/) derive from this file.\n");
    out.push_str("# Run `dev-box generate` after editing to regenerate.\n");
    out.push_str("#\n");
    out.push_str(
        "# Full documentation: https://projectious-work.github.io/dev-box/cli/configuration/\n\n",
    );
    out.push_str("[dev-box]\n");
    out.push_str(&format!("version = \"{}\"\n", config.dev_box.version));
    out.push_str(&format!(
        "# Container image flavor. Options: base, python, latex, typst, rust,\n\
         # python-latex, python-typst, rust-latex\n\
         image = \"{}\"\n",
        config.dev_box.image
    ));
    out.push_str(&format!(
        "# Work process flavor. Controls which context files are scaffolded.\n\
         # Options: minimal (CLAUDE.md only), managed (backlog + decisions),\n\
         #          research (progress + notes), product (full: PRD + backlog + standups)\n\
         process = \"{}\"\n",
        config.dev_box.process
    ));

    // [container] section
    out.push_str("\n[container]\n");
    out.push_str(&format!("name = \"{}\"\n", config.container.name));
    out.push_str(&format!("hostname = \"{}\"\n", config.container.hostname));
    if config.container.user != "root" {
        out.push_str(&format!(
            "# Container user. Determines mount paths inside container.\n\
             user = \"{}\"\n",
            config.container.user
        ));
    } else {
        out.push_str(
            "# user = \"root\"  # Container user (default: root). Change to run as non-root.\n",
        );
    }
    out.push_str("# ports = [\"8080:80\"]  # Host:container port forwarding\n");
    if !config.container.ports.is_empty() {
        out.push_str(&format!(
            "ports = [{}]\n",
            config
                .container
                .ports
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    out.push_str("# extra_packages = [\"ripgrep\", \"fd-find\"]  # Additional apt packages\n");
    if !config.container.extra_packages.is_empty() {
        out.push_str(&format!(
            "extra_packages = [{}]\n",
            config
                .container
                .extra_packages
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    out.push_str("# vscode_extensions = [\"eamodio.gitlens\"]  # Additional VS Code extensions\n");
    out.push_str("# post_create_command = \"npm install\"  # Run after container creation\n");
    out.push_str("#\n");
    out.push_str("# Extra volumes: [[container.extra_volumes]]\n");
    out.push_str("# source = \"/host/path\"\n");
    out.push_str("# target = \"/container/path\"\n");
    out.push_str("# read_only = false\n");
    out.push_str("#\n");
    out.push_str("# Extra environment: [container.environment]\n");
    out.push_str("# MY_VAR = \"value\"\n");

    // [context] section
    out.push_str("\n[context]\n");
    out.push_str(&format!(
        "schema_version = \"{}\"\n",
        config.context.schema_version
    ));

    // [ai] section
    out.push_str("\n# AI tool providers. Controls which AI CLI tools are mounted/configured.\n");
    out.push_str("# Options: claude (more providers planned)\n");
    out.push_str("[ai]\n");
    out.push_str(&format!(
        "providers = [{}]\n",
        config
            .ai
            .providers
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // [audio] section
    out.push_str("\n# Audio support for PulseAudio bridging (e.g., Claude Code voice).\n");
    out.push_str("# Requires host-side PulseAudio setup: run `dev-box audio setup`\n");
    out.push_str("[audio]\n");
    out.push_str(&format!("enabled = {}\n", config.audio.enabled));
    if config.audio.enabled {
        out.push_str(&format!(
            "pulse_server = \"{}\"\n",
            config.audio.pulse_server
        ));
    } else {
        out.push_str("# pulse_server = \"tcp:host.docker.internal:4714\"\n");
    }

    out
}

/// Init command: create a dev-box.toml and generate files.
pub fn cmd_init(
    config_path: &Option<String>,
    name: Option<String>,
    image: Option<ImageFlavor>,
    process: Option<ProcessFlavor>,
    ai: Option<Vec<AiProvider>>,
    user: Option<String>,
) -> Result<()> {
    use crate::config::{
        AiSection, AudioSection, ContainerSection, ContextSection, DevBoxConfig, DevBoxSection,
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

    let container_user = user.unwrap_or_else(|| "root".to_string());
    let ai_providers = ai.unwrap_or_else(|| vec![AiProvider::Claude]);

    let config = DevBoxConfig {
        dev_box: DevBoxSection {
            version: env!("CARGO_PKG_VERSION").to_string(),
            image: image_flavor,
            process: process_flavor,
        },
        container: ContainerSection {
            name: project_name.clone(),
            hostname: project_name,
            user: container_user,
            ports: vec![],
            extra_packages: vec![],
            extra_volumes: vec![],
            environment: std::collections::HashMap::new(),
            post_create_command: None,
            vscode_extensions: vec![],
        },
        context: ContextSection::default(),
        ai: AiSection {
            providers: ai_providers,
        },
        audio: AudioSection::default(),
    };

    config.validate()?;

    let toml_str = serialize_config_with_comments(&config);

    std::fs::write(&toml_path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", toml_path.display(), e))?;

    output::ok(&format!("Created {}", toml_path.display()));

    // Generate devcontainer files
    generate::generate_all(&config)?;

    // Scaffold context directory based on process flavor
    context::scaffold_context(&config)?;

    // Seed .dev-box-home/ directory with default configs
    seed::seed_root_dir(&config)?;

    output::ok("Project initialized. Edit dev-box.toml to customize, then run: dev-box start");

    Ok(())
}

/// Generate command.
pub fn cmd_generate(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;

    // Re-seed .dev-box-home/ in case config changed (e.g., new AI provider,
    // audio toggled). seed_root_dir is idempotent — never overwrites.
    seed::seed_root_dir(&config)?;

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
            Some(ImageFlavor::PythonLatex),
            Some(ProcessFlavor::Research),
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
            Some(ImageFlavor::Rust),
            Some(ProcessFlavor::Minimal),
            false,
        )
        .expect("should succeed");

        assert_eq!(name, "test-proj");
        assert_eq!(image, ImageFlavor::Rust);
        assert_eq!(process, ProcessFlavor::Minimal);
    }
}
