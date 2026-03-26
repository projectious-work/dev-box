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

/// Build process selection items for the interactive prompt.
/// Shows presets only — individual package names are still accepted via --process on the CLI.
fn process_selection_items() -> (Vec<String>, Vec<String>) {
    let presets = crate::process_registry::all_presets();
    let labels = presets
        .iter()
        .map(|p| format!("{} — {}", p.name, p.description))
        .collect();
    let values = presets.iter().map(|p| p.name.to_string()).collect();
    (labels, values)
}

/// Determine the default project name from the current directory.
fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string())
}

/// Build the list of addon names available for interactive selection,
/// excluding AI provider addons (those are handled via `[ai].providers`).
fn selectable_addon_names() -> Vec<String> {
    crate::addon_loader::all_addons()
        .iter()
        .filter(|a| !a.name.starts_with("ai-"))
        .map(|a| a.name.clone())
        .collect()
}

/// Resolve init values, prompting interactively when `interactive` is true and
/// the corresponding argument is `None`.
pub fn resolve_init_values(
    name: Option<String>,
    base: Option<BaseImage>,
    process: Option<Vec<String>>,
    addons: Option<Vec<String>>,
    interactive: bool,
) -> Result<(String, BaseImage, Vec<String>, Vec<String>)> {
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

    // --- base image (only debian for now, skip prompt) ---
    let base_image = base.unwrap_or(BaseImage::Debian);

    // --- process packages ---
    let process_packages = match process {
        Some(p) => p,
        None if interactive => {
            let (labels, values) = process_selection_items();
            let idx = dialoguer::Select::new()
                .with_prompt("Work process")
                .items(&labels)
                .default(0)
                .interact()?;
            vec![values[idx].clone()]
        }
        None => vec!["core".to_string()],
    };

    // --- addons ---
    let addon_names = match addons {
        Some(a) => a,
        None if interactive => {
            let available = selectable_addon_names();
            if available.is_empty() {
                vec![]
            } else {
                let selections = dialoguer::MultiSelect::new()
                    .with_prompt("Addons (space to select, enter to confirm)")
                    .items(&available)
                    .interact()?;
                selections.into_iter().map(|i| available[i].clone()).collect()
            }
        }
        None => vec![],
    };

    Ok((project_name, base_image, process_packages, addon_names))
}

/// Build command: load config, generate files, run compose build.
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
    runtime.exec_interactive(name, &config.container.user, &["zellij", "--layout", layout])?;

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
    let sep = "# =============================================================================\n";

    // File header
    out.push_str(sep);
    out.push_str("# aibox.toml — single source of truth for your aibox project.\n");
    out.push_str("# All .devcontainer/ files are generated from this. Edit here, run `aibox sync`.\n");
    out.push_str("# Reference: https://projectious-work.github.io/aibox/docs/reference/configuration\n");
    out.push_str(sep);
    out.push('\n');

    // [aibox] section
    out.push_str("[aibox]\n");
    out.push_str(&format!(
        "version = {:20} # Set by aibox — do not edit manually\n",
        format!("\"{}\"", config.aibox.version)
    ));
    out.push_str(&format!(
        "base    = {:20} # Base image flavor. Options: debian\n",
        format!("\"{}\"", config.aibox.base)
    ));

    // [container] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [container] — runtime and build configuration\n");
    out.push_str(sep);
    out.push_str("[container]\n");
    out.push_str(&format!(
        "name     = {:20} # Container name used by docker/podman\n",
        format!("\"{}\"", config.container.name)
    ));
    out.push_str(&format!(
        "hostname = {:20} # Hostname visible inside the container\n",
        format!("\"{}\"", config.container.hostname)
    ));

    // user — active if non-root, commented if root
    if config.container.user != "root" {
        out.push_str(&format!(
            "user     = {:20} # User inside the container (controls mount paths)\n",
            format!("\"{}\"", config.container.user)
        ));
    } else {
        out.push_str("# user     = \"root\"               # User inside the container. Options: root, aibox, or any username\n");
        out.push_str("#                                  # Controls mount paths (e.g. /root vs /home/<user>/.vim)\n");
    }

    // --- Ports ---
    out.push_str("\n# --- Ports ---\n");
    if !config.container.ports.is_empty() {
        out.push_str(&format!(
            "ports = [{}]                   # Host:container port forwarding\n",
            config
                .container
                .ports
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        out.push_str("# ports = [\"8080:80\", \"5432:5432\"]  # Host:container port forwarding (can list multiple)\n");
    }

    // --- Extra packages ---
    out.push_str("\n# --- Extra packages ---\n");
    if !config.container.extra_packages.is_empty() {
        out.push_str(&format!(
            "extra_packages = [{}]          # Additional apt packages installed at build time\n",
            config
                .container
                .extra_packages
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        out.push_str("# extra_packages = [\"ripgrep\", \"fd-find\", \"jq\"]  # Additional apt packages installed at build time\n");
    }

    // --- VS Code ---
    out.push_str("\n# --- VS Code ---\n");
    if !config.container.vscode_extensions.is_empty() {
        out.push_str(&format!(
            "vscode_extensions = [{}]       # Auto-installed in Dev Containers\n",
            config
                .container
                .vscode_extensions
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        out.push_str("# vscode_extensions = [\"eamodio.gitlens\", \"rust-lang.rust-analyzer\"]  # Auto-installed in Dev Containers\n");
    }

    // --- Lifecycle ---
    out.push_str("\n# --- Lifecycle ---\n");
    if let Some(cmd) = &config.container.post_create_command {
        out.push_str(&format!(
            "post_create_command = {:20} # Shell command run once after container first starts\n",
            format!("\"{}\"", cmd)
        ));
    } else {
        out.push_str("# post_create_command = \"npm install\"  # Shell command run once after container first starts\n");
    }
    if config.container.keepalive {
        out.push_str("keepalive = true               # Send periodic keepalive (prevents NAT idle dropout in OrbStack/VMs)\n");
    } else {
        out.push_str("# keepalive           = true           # Send periodic keepalive (prevents NAT idle dropout in OrbStack/VMs)\n");
    }

    // --- Extra volumes ---
    out.push_str("\n# --- Extra volumes ---\n");
    if !config.container.extra_volumes.is_empty() {
        for vol in &config.container.extra_volumes {
            out.push_str("[[container.extra_volumes]]\n");
            out.push_str(&format!("source    = \"{}\"", vol.source));
            out.push_str("                # Absolute path on the host\n");
            out.push_str(&format!("target    = \"{}\"", vol.target));
            out.push_str("                # Absolute path inside the container\n");
            out.push_str(&format!("read_only = {}                # Mount read-only (default: false)\n", vol.read_only));
        }
    } else {
        out.push_str("# [[container.extra_volumes]]\n");
        out.push_str("# source    = \"/host/path\"       # Absolute path on the host\n");
        out.push_str("# target    = \"/container/path\"  # Absolute path inside the container\n");
        out.push_str("# read_only = false               # Mount read-only (default: false)\n");
    }

    // --- Extra environment ---
    out.push_str("\n# --- Extra environment ---\n");
    if !config.container.environment.is_empty() {
        out.push_str("[container.environment]\n");
        let mut env_keys: Vec<_> = config.container.environment.keys().collect();
        env_keys.sort();
        for key in env_keys {
            out.push_str(&format!(
                "{} = \"{}\"                # Injected as environment variable into the container\n",
                key, config.container.environment[key]
            ));
        }
    } else {
        out.push_str("# [container.environment]\n");
        out.push_str("# MY_API_KEY = \"value\"           # Injected as environment variable into the container\n");
    }

    // [process] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [process] — context files and skills scaffolded for this project\n");
    out.push_str(sep);
    out.push_str("# Presets (use one of these):\n");
    out.push_str("#   managed          core + tracking + standups + handover  (recommended default)\n");
    out.push_str("#   software         managed + code + architecture\n");
    out.push_str("#   research-project managed + research + documentation\n");
    out.push_str("#   full-product     managed + code + architecture + design + product + security + operations\n");
    out.push_str("# Individual packages (advanced): core, tracking, standups, handover, code, architecture,\n");
    out.push_str("#   design, product, security, data, operations, research, documentation\n");
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
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [addons] — language runtimes and tool bundles\n");
    out.push_str(sep);
    out.push_str("# Each addon installs a tool set into the container at build time.\n");
    out.push_str("# Run `aibox addon list` to see all available addons.\n");
    out.push_str("# Run `aibox addon info <name>` for tool details and supported versions.\n");
    out.push_str("#\n");
    out.push_str("# Examples (uncomment and adjust versions as needed):\n");
    out.push_str("# [addons.python.tools]\n");
    out.push_str("# python = { version = \"3.13\" }     # CPython interpreter\n");
    out.push_str("# uv     = { version = \"0.7\" }      # Fast Python package manager\n");
    out.push_str("#\n");
    out.push_str("# [addons.rust.tools]\n");
    out.push_str("# rust = { version = \"stable\" }     # Rust toolchain via rustup (stable/beta/nightly)\n");
    out.push_str("#\n");
    out.push_str("# [addons.node.tools]\n");
    out.push_str("# node = { version = \"22\" }         # Node.js LTS\n");
    if !config.addons.addons.is_empty() {
        let mut addon_names: Vec<_> = config.addons.addons.keys().collect();
        addon_names.sort();
        for addon_name in addon_names {
            let addon_tools = &config.addons.addons[addon_name];
            out.push_str(&format!("\n[addons.{}.tools]\n", addon_name));
            let mut tool_names: Vec<_> = addon_tools.tools.keys().collect();
            tool_names.sort();
            for tool_name in tool_names {
                let tool_entry = &addon_tools.tools[tool_name];
                match &tool_entry.version {
                    Some(v) => out.push_str(&format!("{} = {{ version = \"{}\" }}\n", tool_name, v)),
                    None => out.push_str(&format!("{} = {{}}\n", tool_name)),
                }
            }
        }
    }

    // [context] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [context] — context system versioning\n");
    out.push_str(sep);
    out.push_str("[context]\n");
    out.push_str(&format!(
        "schema_version = {:12} # Context schema version — updated automatically by `aibox sync`\n",
        format!("\"{}\"", config.context.schema_version)
    ));

    // [ai] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [ai] — AI coding assistant providers\n");
    out.push_str(sep);
    out.push_str("# Each provider listed here is automatically installed as an addon.\n");
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
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [appearance] — color theme and shell prompt\n");
    out.push_str(sep);
    out.push_str("# Theme is applied consistently across Zellij, Vim, Yazi, lazygit, and bat.\n");
    out.push_str("# Options: gruvbox-dark | catppuccin-mocha | catppuccin-latte | dracula | tokyo-night | nord | projectious\n");
    out.push_str("[appearance]\n");
    out.push_str(&format!("theme  = \"{}\"\n", config.appearance.theme));
    out.push_str("# Starship prompt preset.\n");
    out.push_str("# Options: default | minimal | nerd-font | pastel | bracketed\n");
    out.push_str(&format!("prompt = \"{}\"\n", config.appearance.prompt));

    // [audio] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [audio] — PulseAudio bridging for voice features (e.g., Claude Code voice)\n");
    out.push_str(sep);
    out.push_str("# Requires host-side setup: run `aibox audio setup` on the host first.\n");
    out.push_str("[audio]\n");
    out.push_str(&format!("enabled = {}\n", config.audio.enabled));
    if config.audio.enabled {
        out.push_str(&format!(
            "pulse_server = \"{}\"  # PulseAudio TCP endpoint (default port: 4714)\n",
            config.audio.pulse_server
        ));
    } else {
        out.push_str("# pulse_server = \"tcp:host.docker.internal:4714\"  # PulseAudio TCP endpoint (default port: 4714)\n");
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

    let (project_name, base_image, process_packages, addon_names) =
        resolve_init_values(params.name, params.base, params.process, params.addons, interactive)?;

    let container_user = params.user.unwrap_or_else(|| "aibox".to_string());
    let ai_providers = params.ai.unwrap_or_else(|| vec![AiProvider::Claude]);

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
        addons: {
            let mut section = AddonsSection::default();
            for name in &addon_names {
                section.addons.entry(name.clone()).or_insert_with(|| {
                    crate::config::AddonToolsSection {
                        tools: std::collections::HashMap::new(),
                    }
                });
            }
            section
        },
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
pub fn cmd_sync(config_path: &Option<String>, no_cache: bool, no_build: bool) -> Result<()> {
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

    // Build container image (if a container runtime is available)
    if no_build {
        output::ok("Sync complete (build skipped)");
    } else {
        match Runtime::detect() {
            Ok(runtime) => {
                output::info("Building container image...");
                runtime.compose_build(crate::config::COMPOSE_FILE, no_cache)?;
                output::ok("Sync complete — image built");
            }
            Err(_) => {
                output::warn("No container runtime found — skipping image build");
                output::ok("Sync complete (config files only)");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_init_values_non_interactive_defaults() {
        let (name, base, process, addons) =
            resolve_init_values(None, None, None, None, false).expect("should succeed");

        // Name defaults to current directory name (or "my-project" fallback)
        assert!(!name.is_empty(), "name should not be empty");

        assert_eq!(base, BaseImage::Debian);
        assert_eq!(process, vec!["core".to_string()]);
        assert!(addons.is_empty());
    }

    #[test]
    fn resolve_init_values_explicit_args_override() {
        // Even with interactive=true, explicit values should be used without prompting
        let (name, base, process, addons) = resolve_init_values(
            Some("my-app".to_string()),
            Some(BaseImage::Debian),
            Some(vec!["research".to_string()]),
            Some(vec!["latex".to_string()]),
            true,
        )
        .expect("should succeed with explicit args");

        assert_eq!(name, "my-app");
        assert_eq!(base, BaseImage::Debian);
        assert_eq!(process, vec!["research".to_string()]);
        assert_eq!(addons, vec!["latex".to_string()]);
    }

    #[test]
    fn resolve_init_values_explicit_args_non_interactive() {
        let (name, base, process, addons) = resolve_init_values(
            Some("test-proj".to_string()),
            Some(BaseImage::Debian),
            Some(vec!["minimal".to_string()]),
            Some(vec!["python".to_string(), "latex".to_string()]),
            false,
        )
        .expect("should succeed");

        assert_eq!(name, "test-proj");
        assert_eq!(base, BaseImage::Debian);
        assert_eq!(process, vec!["minimal".to_string()]);
        assert_eq!(addons, vec!["python".to_string(), "latex".to_string()]);
    }
}
