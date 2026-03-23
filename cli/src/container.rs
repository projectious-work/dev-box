use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::{
    AiProvider, BaseImage, AiboxConfig, StarshipPreset, Theme,
};
use crate::context;
use crate::generate;
use crate::output;
use crate::runtime::{ContainerState, Runtime};
use crate::seed;

/// Parameters for the init command, grouping all optional CLI arguments.
pub struct InitParams {
    pub name: Option<String>,
    pub base: Option<BaseImage>,
    pub process: Option<Vec<String>>,
    pub ai: Option<Vec<AiProvider>>,
    pub user: Option<String>,
    pub theme: Option<Theme>,
    pub prompt: Option<StarshipPreset>,
    pub addons: Option<Vec<String>>,
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
    base: Option<BaseImage>,
    process: Option<Vec<String>>,
    interactive: bool,
) -> Result<(String, BaseImage, Vec<String>)> {
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

    // --- base image ---
    let base_image = match base {
        Some(b) => b,
        None if interactive => {
            let idx = dialoguer::Select::new()
                .with_prompt("Base image")
                .items(IMAGE_FLAVOR_ITEMS)
                .default(0)
                .interact()?;
            // For now only debian is supported; ignore the selection index
            let _ = idx;
            BaseImage::Debian
        }
        None => BaseImage::Debian,
    };

    // --- process packages ---
    let process_packages = match process {
        Some(p) => p,
        None if interactive => {
            let idx = dialoguer::Select::new()
                .with_prompt("Work process")
                .items(PROCESS_FLAVOR_ITEMS)
                .default(3)
                .interact()?;
            vec![PROCESS_FLAVOR_ITEMS[idx].to_string()]
        }
        None => vec!["core".to_string()],
    };

    Ok((project_name, base_image, process_packages))
}

/// Build command: load config, generate files, run compose build.
pub fn cmd_build(config_path: &Option<String>, no_cache: bool) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;

    generate::generate_all(&config)?;

    output::info("Building container image...");
    runtime.compose_build(crate::config::COMPOSE_FILE, no_cache)?;
    output::ok("Build complete");

    Ok(())
}

/// Start command: seed, generate, ensure running, attach.
pub fn cmd_start(config_path: &Option<String>, layout: &str) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
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
    let config = AiboxConfig::from_cli_option(config_path)?;
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
    let config = AiboxConfig::from_cli_option(config_path)?;
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
    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    if state != ContainerState::Running {
        bail!(
            "Container '{}' is not running. Run 'aibox start' first.",
            name
        );
    }

    output::info(&format!("Attaching via zellij (layout: {})...", layout));
    runtime.exec_interactive(name, &["zellij", "--layout", layout])?;

    Ok(())
}

pub fn cmd_status(config_path: &Option<String>) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
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
fn serialize_config_with_comments(config: &AiboxConfig) -> String {
    let mut out = String::new();

    // [aibox] section
    out.push_str("# aibox.toml — project configuration for aibox.\n");
    out.push_str("# All generated files (.devcontainer/) derive from this file.\n");
    out.push_str("# Run `aibox sync` after editing to regenerate.\n");
    out.push_str("#\n");
    out.push_str(
        "# Full documentation: https://projectious-work.github.io/aibox/cli/configuration/\n\n",
    );
    out.push_str("[aibox]\n");
    out.push_str(&format!("version = \"{}\"\n", config.aibox.version));
    out.push_str(&format!(
        "# Base image. Options: debian\n\
         base = \"{}\"\n",
        config.aibox.base
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
    out.push_str("# keepalive = true                       # Network keepalive (prevents OrbStack/VM NAT idle dropout)\n");
    out.push_str("#\n");
    out.push_str("# Extra volumes: [[container.extra_volumes]]\n");
    out.push_str("# source = \"/host/path\"\n");
    out.push_str("# target = \"/container/path\"\n");
    out.push_str("# read_only = false\n");
    out.push_str("#\n");
    out.push_str("# Extra environment: [container.environment]\n");
    out.push_str("# MY_VAR = \"value\"\n");

    // [process] section
    out.push_str("\n# Process packages control which context files are scaffolded.\n");
    out.push_str("[process]\n");
    out.push_str(&format!(
        "packages = [{}]\n",
        config
            .process
            .packages
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // [addons] section
    out.push_str("\n# Addons install additional tool sets into the container.\n");
    out.push_str("# Example:\n");
    out.push_str("# [addons.python.tools]\n");
    out.push_str("# python = { version = \"3.13\" }\n");
    if config.addons.addons.is_empty() {
        out.push_str("# (no addons configured)\n");
    } else {
        for (addon_name, addon_tools) in &config.addons.addons {
            out.push_str(&format!("\n[addons.{}.tools]\n", addon_name));
            for (tool_name, tool_entry) in &addon_tools.tools {
                match &tool_entry.version {
                    Some(v) => out.push_str(&format!("{} = {{ version = \"{}\" }}\n", tool_name, v)),
                    None => out.push_str(&format!("{} = {{}}\n", tool_name)),
                }
            }
        }
    }

    // [context] section
    out.push_str("\n[context]\n");
    out.push_str(&format!(
        "schema_version = \"{}\"\n",
        config.context.schema_version
    ));

    // [ai] section
    out.push_str("\n# AI tool providers. Controls which AI CLI tools are installed and configured.\n");
    out.push_str("# Options: claude, aider, gemini, mistral\n");
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
    out.push_str(&format!(
        "# Starship prompt preset. Options: default, plain, minimal, nerd-font, pastel, bracketed\n\
         prompt = \"{}\"\n",
        config.appearance.prompt
    ));

    // [audio] section
    out.push_str("\n# Audio support for PulseAudio bridging (e.g., Claude Code voice).\n");
    out.push_str("# Requires host-side PulseAudio setup: run `aibox audio setup`\n");
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

/// Init command: create a aibox.toml and generate files.
pub fn cmd_init(config_path: &Option<String>, params: InitParams) -> Result<()> {
    use crate::config::{
        AddonsSection, AiSection, AppearanceSection, AudioSection, ContainerSection,
        ContextSection, AiboxConfig, AiboxSection, ProcessSection, SkillsSection,
    };

    let toml_path = config_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("aibox.toml"));

    if toml_path.exists() {
        bail!(
            "Config file already exists: {}. Delete it first or edit it directly.",
            toml_path.display()
        );
    }

    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());

    let (project_name, base_image, process_packages) =
        resolve_init_values(params.name, params.base, params.process, interactive)?;

    let container_user = params.user.unwrap_or_else(|| "root".to_string());
    let ai_providers = params.ai.unwrap_or_else(|| vec![AiProvider::Claude]);
    let _addon_names = params.addons.unwrap_or_default();

    let mut config = AiboxConfig {
        aibox: AiboxSection {
            version: env!("CARGO_PKG_VERSION").to_string(),
            base: base_image,
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
            keepalive: false,
        },
        context: ContextSection::default(),
        ai: AiSection {
            providers: ai_providers,
        },
        process: ProcessSection {
            packages: process_packages,
        },
        addons: AddonsSection::default(),
        skills: SkillsSection::default(),
        appearance: AppearanceSection {
            theme: params.theme.unwrap_or_default(),
            prompt: params.prompt.unwrap_or_default(),
        },
        audio: AudioSection::default(),
    };
    config.resolve_ai_provider_addons();

    config.validate()?;

    let toml_str = serialize_config_with_comments(&config);

    std::fs::write(&toml_path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", toml_path.display(), e))?;

    output::ok(&format!("Created {}", toml_path.display()));

    generate::generate_all(&config)?;
    context::scaffold_context(&config)?;
    seed::seed_root_dir(&config)?;

    output::ok("Project initialized. Edit aibox.toml to customize, then run: aibox start");

    Ok(())
}

/// Sync command: force-seed theme-dependent files, seed missing configs, regenerate .devcontainer/.
pub fn cmd_sync(config_path: &Option<String>) -> Result<()> {
    // Check for version migration before any other sync steps
    crate::migration::check_and_generate_migration()?;

    let config = AiboxConfig::from_cli_option(config_path)?;

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

    // Skill reconciliation
    context::reconcile_skills(&config)?;

    // Generate AIBOX.md (universal baseline)
    context::generate_aibox_md(&config)?;

    // Check agent entry points
    context::check_agent_entry_points(&config)?;

    output::ok("Sync complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_init_values_non_interactive_defaults() {
        let (name, base, process) =
            resolve_init_values(None, None, None, false).expect("should succeed");

        // Name defaults to current directory name (or "my-project" fallback)
        assert!(!name.is_empty(), "name should not be empty");

        assert_eq!(base, BaseImage::Debian);
        assert_eq!(process, vec!["core".to_string()]);
    }

    #[test]
    fn resolve_init_values_explicit_args_override() {
        // Even with interactive=true, explicit values should be used without prompting
        let (name, base, process) = resolve_init_values(
            Some("my-app".to_string()),
            Some(BaseImage::Debian),
            Some(vec!["research".to_string()]),
            true,
        )
        .expect("should succeed with explicit args");

        assert_eq!(name, "my-app");
        assert_eq!(base, BaseImage::Debian);
        assert_eq!(process, vec!["research".to_string()]);
    }

    #[test]
    fn resolve_init_values_explicit_args_non_interactive() {
        let (name, base, process) = resolve_init_values(
            Some("test-proj".to_string()),
            Some(BaseImage::Debian),
            Some(vec!["minimal".to_string()]),
            false,
        )
        .expect("should succeed");

        assert_eq!(name, "test-proj");
        assert_eq!(base, BaseImage::Debian);
        assert_eq!(process, vec!["minimal".to_string()]);
    }
}
