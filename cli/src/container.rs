use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::{AddonBundle, AiProvider, DevBoxConfig, ImageFlavor, ProcessFlavor, Theme};
use crate::context;
use crate::generate;
use crate::output;
use crate::runtime::{ContainerState, Runtime};
use crate::seed;

/// Parameters for the init command, grouping all optional CLI arguments.
pub struct InitParams {
    pub name: Option<String>,
    pub image: Option<ImageFlavor>,
    pub process: Option<ProcessFlavor>,
    pub ai: Option<Vec<AiProvider>>,
    pub user: Option<String>,
    pub theme: Option<Theme>,
    pub addons: Option<Vec<AddonBundle>>,
}

/// Labels for image flavor selection (order matters — matches ImageFlavor variants).
const IMAGE_FLAVOR_ITEMS: &[&str] = &[
    "base",
    "python",
    "latex",
    "typst",
    "rust",
    "node",
    "go",
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

/// Build command: load config, generate files, run compose build.
pub fn cmd_build(config_path: &Option<String>, no_cache: bool) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;

    generate::generate_all(&config)?;

    output::info("Building container image...");
    runtime.compose_build(crate::config::COMPOSE_FILE, no_cache)?;
    output::ok("Build complete");

    Ok(())
}

/// Start command: seed, generate, ensure running, attach.
pub fn cmd_start(config_path: &Option<String>, layout: &str) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    seed::seed_root_dir(&config)?;
    generate::generate_all(&config)?;

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
            runtime.compose_up(crate::config::COMPOSE_FILE, name)?;
            runtime.wait_for_running(name, 7500)?;
            output::ok("Container started");
        }
    }

    output::info(&format!("Attaching via zellij (layout: {})...", layout));
    runtime.exec_interactive(name, &["zellij", "--layout", layout])?;

    Ok(())
}

pub fn cmd_stop(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    match state {
        ContainerState::Running => {
            output::info("Stopping container...");
            runtime.compose_stop(crate::config::COMPOSE_FILE, name)?;
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
    runtime.compose_down(crate::config::COMPOSE_FILE)?;
    output::ok(&format!("Container '{}' removed", name));

    Ok(())
}

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
    out.push_str("# Run `dev-box sync` after editing to regenerate.\n");
    out.push_str("#\n");
    out.push_str(
        "# Full documentation: https://projectious-work.github.io/dev-box/cli/configuration/\n\n",
    );
    out.push_str("[dev-box]\n");
    out.push_str(&format!("version = \"{}\"\n", config.dev_box.version));
    out.push_str(&format!(
        "# Container image flavor. Options: base, python, latex, typst, rust, node, go,\n\
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

    // [addons] section
    out.push_str("\n# Addon bundles install additional tool sets into the container.\n");
    out.push_str("# Options: infrastructure, kubernetes, cloud-aws, cloud-gcp, cloud-azure,\n");
    out.push_str("#          docs-mkdocs, docs-zensical, docs-docusaurus, docs-starlight,\n");
    out.push_str("#          docs-mdbook, docs-hugo\n");
    out.push_str("[addons]\n");
    if config.addons.bundles.is_empty() {
        out.push_str("# bundles = [\"infrastructure\", \"kubernetes\"]\n");
    } else {
        out.push_str(&format!(
            "bundles = [{}]\n",
            config
                .addons
                .bundles
                .iter()
                .map(|b| format!("\"{}\"", b))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // [context] section
    out.push_str("\n[context]\n");
    out.push_str(&format!(
        "schema_version = \"{}\"\n",
        config.context.schema_version
    ));

    // [ai] section
    out.push_str("\n# AI tool providers. Controls which AI CLI tools are installed and configured.\n");
    out.push_str("# Options: claude, aider, gemini\n");
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

    // [appearance] section
    out.push_str("\n# Color theme applied across Zellij, Vim, Yazi, and lazygit.\n");
    out.push_str("# Options: gruvbox-dark, catppuccin-mocha, catppuccin-latte, dracula, tokyo-night, nord\n");
    out.push_str("[appearance]\n");
    out.push_str(&format!("theme = \"{}\"\n", config.appearance.theme));

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
pub fn cmd_init(config_path: &Option<String>, params: InitParams) -> Result<()> {
    use crate::config::{
        AddonsSection, AiSection, AppearanceSection, AudioSection, ContainerSection,
        ContextSection, DevBoxConfig, DevBoxSection,
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

    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());

    let (project_name, image_flavor, process_flavor) =
        resolve_init_values(params.name, params.image, params.process, interactive)?;

    let container_user = params.user.unwrap_or_else(|| "root".to_string());
    let ai_providers = params.ai.unwrap_or_else(|| vec![AiProvider::Claude]);
    let addon_bundles = params.addons.unwrap_or_default();

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
        addons: AddonsSection {
            bundles: addon_bundles,
        },
        appearance: AppearanceSection {
            theme: params.theme.unwrap_or_default(),
        },
        audio: AudioSection::default(),
    };

    config.validate()?;

    let toml_str = serialize_config_with_comments(&config);

    std::fs::write(&toml_path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", toml_path.display(), e))?;

    output::ok(&format!("Created {}", toml_path.display()));

    generate::generate_all(&config)?;
    context::scaffold_context(&config)?;
    seed::seed_root_dir(&config)?;

    output::ok("Project initialized. Edit dev-box.toml to customize, then run: dev-box start");

    Ok(())
}

/// Sync command: force-seed theme-dependent files, seed missing configs, regenerate .devcontainer/.
pub fn cmd_sync(config_path: &Option<String>) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;

    output::info("Syncing config files...");
    let updated = seed::sync_theme_files(&config)?;

    if updated.is_empty() {
        output::ok("All config files already up to date");
    } else {
        for file in &updated {
            output::ok(&format!("Updated {}", file));
        }
    }

    seed::seed_root_dir(&config)?;
    generate::generate_all(&config)?;
    output::ok("Sync complete");

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
