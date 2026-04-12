use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::{AiHarness, AiProvider, AiboxConfig, BaseImage, StarshipPreset, Theme};
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
    /// Repeated `addon:tool=version` overrides for individual tools
    /// inside selected addons. See [`parse_addon_tool_override`] for
    /// the syntax. Each override skips the interactive version picker
    /// for that tool and pins the version into `aibox.toml`.
    pub addon_tool: Vec<String>,
    /// Override the processkit source URL. `None` → use the default
    /// upstream from `ProcessKitSection::default()`.
    pub processkit_source: Option<String>,
    /// Pin a specific processkit tag. `None` → list versions at the
    /// configured source and pick interactively (or pick latest in
    /// non-interactive mode; fall back to "unset" if listing fails).
    pub processkit_version: Option<String>,
    /// Track a moving processkit branch. Wins over `processkit_version`
    /// at fetch time per the existing fetcher contract.
    pub processkit_branch: Option<String>,
}

/// Build processkit-package selection items for the interactive prompt.
///
/// processkit ships five tiers under `src/packages/`. They are listed
/// here as static presets so `aibox init` can offer them without yet
/// having a fetched processkit cache to read from.
fn process_selection_items() -> (Vec<String>, Vec<String>) {
    const PRESETS: &[(&str, &str)] = &[
        ("minimal", "solo developers and small side projects"),
        ("managed", "small teams with a shared backlog (recommended)"),
        (
            "software",
            "software engineering teams building production systems",
        ),
        ("research", "research, data science, and ML projects"),
        (
            "product",
            "full product development (engineering + design + ops)",
        ),
    ];
    let labels = PRESETS
        .iter()
        .map(|(n, d)| format!("{} — {}", n, d))
        .collect();
    let values = PRESETS.iter().map(|(n, _)| n.to_string()).collect();
    (labels, values)
}

/// Pure decision: should `cmd_sync` (re-)install processkit content?
///
/// Returns true when the configured version is real (not the `unset`
/// sentinel) AND either there is no lock yet, or the lock disagrees
/// with the config on `(source, version)`. Used by the auto-install
/// path that lets users pin a version after `aibox init` and have
/// `aibox sync` materialize the content (closes the v0.16.0 bug
/// reported in BACK-110).
fn sync_should_install_processkit(
    config_version: &str,
    config_source: &str,
    lock_pair: Option<(&str, &str)>,
) -> bool {
    if config_version == crate::config::PROCESSKIT_VERSION_UNSET
        || config_version == crate::config::PROCESSKIT_VERSION_LATEST
    {
        return false;
    }
    match lock_pair {
        None => true,
        Some((src, ver)) => src != config_source || ver != config_version,
    }
}

/// Build the `[processkit]` section from CLI overrides + interactive
/// version picker.
///
/// Strategy:
/// 1. Source: `--processkit-source` if given, else the upstream default
///    from `ProcessKitSection::default()`.
/// 2. Branch: `--processkit-branch` if given, else `None`. A branch
///    override wins over the version at fetch time, but the version
///    field is still recorded so the project can drop the branch later
///    and have a sensible pin to fall back to.
/// 3. Version:
///    - `--processkit-version` if given → use as-is
///    - else: list available versions at the source.
///      - Interactive: show a `dialoguer::Select` with the latest as the
///        default. Includes an "unset (skip processkit install)" entry
///        as the escape hatch when the user explicitly wants no install.
///      - Non-interactive: pick the first (latest) entry. If listing
///        fails or returns nothing, fall back to the `unset` sentinel
///        and warn — the user can edit aibox.toml + re-run sync.
fn resolve_processkit_section(
    source_override: Option<&str>,
    version_override: Option<&str>,
    branch_override: Option<&str>,
    interactive: bool,
) -> Result<crate::config::ProcessKitSection> {
    use crate::config::{PROCESSKIT_VERSION_UNSET, ProcessKitSection};

    let mut section = ProcessKitSection::default();
    if let Some(s) = source_override {
        section.source = s.to_string();
    }
    if let Some(b) = branch_override {
        section.branch = Some(b.to_string());
    }

    if let Some(v) = version_override {
        section.version = v.to_string();
        return Ok(section);
    }

    // No version override — list available versions from the configured source.
    output::info(&format!(
        "Querying available processkit versions at {}...",
        section.source
    ));
    let versions = match crate::content_source::list_versions(&section.source) {
        Ok(v) if !v.is_empty() => v,
        Ok(_) => {
            output::warn(&format!(
                "No semver-tagged versions found at {}. Leaving processkit.version = \"{}\"; \
                 edit aibox.toml later and re-run `aibox sync` to install content.",
                section.source, PROCESSKIT_VERSION_UNSET
            ));
            return Ok(section);
        }
        Err(e) => {
            output::warn(&format!(
                "Could not list processkit versions at {}: {}. Leaving processkit.version = \"{}\"; \
                 edit aibox.toml later and re-run `aibox sync` to install content.",
                section.source, e, PROCESSKIT_VERSION_UNSET
            ));
            return Ok(section);
        }
    };

    if interactive {
        // Build the menu with the latest at the top + an explicit
        // "skip" escape hatch at the bottom.
        let mut items: Vec<String> = versions
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if i == 0 {
                    format!("{} (latest)", v)
                } else {
                    v.clone()
                }
            })
            .collect();
        items.push(format!(
            "{} — skip processkit install (configure later)",
            PROCESSKIT_VERSION_UNSET
        ));
        let idx = dialoguer::Select::new()
            .with_prompt("processkit version")
            .items(&items)
            .default(0)
            .interact()?;
        if idx == versions.len() {
            section.version = PROCESSKIT_VERSION_UNSET.to_string();
        } else {
            section.version = versions[idx].clone();
        }
    } else {
        // Non-interactive: pick the latest.
        section.version = versions[0].clone();
        output::ok(&format!(
            "Pinned processkit.version = \"{}\" (latest at {})",
            section.version, section.source
        ));
    }

    Ok(section)
}

// ---------------------------------------------------------------------------
// Addon resolution: requires expansion, default tools, version overrides
// ---------------------------------------------------------------------------

/// Parse a single `--addon-tool addon:tool=version` CLI flag value into
/// its three components. Pure function so it's unit-testable.
///
/// Examples:
/// - `python:python=3.14` → `("python", "python", "3.14")`
/// - `node:pnpm=10` → `("node", "pnpm", "10")`
fn parse_addon_tool_override(s: &str) -> Result<(String, String, String)> {
    let (addon_tool, version) = s.split_once('=').ok_or_else(|| {
        anyhow::anyhow!(
            "--addon-tool '{}' is missing '=<version>'. Expected format: addon:tool=version",
            s
        )
    })?;
    let (addon, tool) = addon_tool.split_once(':').ok_or_else(|| {
        anyhow::anyhow!(
            "--addon-tool '{}' is missing the addon prefix. Expected format: addon:tool=version",
            s
        )
    })?;
    if addon.is_empty() || tool.is_empty() || version.is_empty() {
        anyhow::bail!(
            "--addon-tool '{}' has an empty component. Expected format: addon:tool=version",
            s
        );
    }
    Ok((addon.to_string(), tool.to_string(), version.to_string()))
}

/// Map of `addon -> tool -> version` overrides built from the
/// repeated `--addon-tool` flag values. Used by both the interactive
/// resolver (to skip prompts when a version is already pinned) and
/// the populator (to override the default version).
type ToolOverrides = std::collections::HashMap<String, std::collections::HashMap<String, String>>;

fn build_tool_overrides(values: &[String]) -> Result<ToolOverrides> {
    let mut out: ToolOverrides = std::collections::HashMap::new();
    for v in values {
        let (addon, tool, version) = parse_addon_tool_override(v)?;
        out.entry(addon).or_default().insert(tool, version);
    }
    Ok(out)
}

/// Transitively expand the user's selected addon list to include every
/// addon required (directly or indirectly) by the selection.
///
/// Picking `docs-docusaurus` (which `requires: [node]`) without picking
/// `node` used to error out at sync time with "Addon 'docs-docusaurus'
/// requires 'node'". Now both `aibox init` and `aibox addon add` call
/// this helper so the resulting `aibox.toml` already has the
/// dependencies and `aibox sync` never sees a broken graph.
///
/// Pure function — no I/O. The caller is responsible for surfacing
/// `expanded - initial` to the user via `output::info` if desired.
pub(crate) fn expand_addon_requires(initial: &[String]) -> Vec<String> {
    use std::collections::{HashSet, VecDeque};
    let mut result: Vec<String> = initial.to_vec();
    let mut seen: HashSet<String> = result.iter().cloned().collect();
    let mut queue: VecDeque<String> = result.iter().cloned().collect();
    while let Some(name) = queue.pop_front() {
        if let Some(addon) = crate::addon_loader::get_addon(&name) {
            for req in &addon.requires {
                if seen.insert(req.clone()) {
                    result.push(req.clone());
                    queue.push_back(req.clone());
                }
            }
        }
    }
    result
}

/// Build the `[addons.<name>.tools]` section for a single addon at
/// init time. Populates every `default_enabled` tool at the addon's
/// `default_version`, with three layered override sources (later wins):
///
/// 1. Addon's `default_version`
/// 2. Interactive picker — only when `interactive == true` AND the
///    tool has more than one entry in `supported_versions` AND no
///    explicit override is set
/// 3. `--addon-tool addon:tool=version` CLI flag (highest priority,
///    suppresses the interactive picker for that tool)
///
/// Tools that are NOT `default_enabled` are skipped entirely. Users
/// who want them can edit `aibox.toml` directly afterwards (the
/// `aibox addon info <name>` command lists them).
fn populate_addon_tools(
    addon_name: &str,
    overrides_for_addon: Option<&std::collections::HashMap<String, String>>,
    interactive: bool,
) -> Result<crate::config::AddonToolsSection> {
    use crate::config::{AddonToolsSection, ToolEntry};
    use std::collections::HashMap;

    let mut tools: HashMap<String, ToolEntry> = HashMap::new();

    let Some(loaded) = crate::addon_loader::get_addon(addon_name) else {
        // Unknown addon — caller will surface this elsewhere; we just
        // return an empty section so the rest of init can proceed.
        return Ok(AddonToolsSection { tools });
    };

    for tool in &loaded.tools {
        if !tool.default_enabled {
            continue;
        }

        // Highest priority: explicit CLI override.
        let override_version = overrides_for_addon.and_then(|m| m.get(&tool.name)).cloned();

        // Second priority: interactive picker (only when there's a
        // real choice and the user hasn't pinned via the CLI).
        let picked_version =
            if override_version.is_none() && interactive && tool.supported_versions.len() > 1 {
                // Build version list: "latest" first, then supported versions
                // with the default marked.
                let default_idx = tool
                    .supported_versions
                    .iter()
                    .position(|v| v == &tool.default_version)
                    .unwrap_or(0);
                let mut items: Vec<String> = vec!["latest (always track newest)".to_string()];
                items.extend(tool.supported_versions.iter().enumerate().map(|(i, v)| {
                    if i == default_idx {
                        format!("{} (default)", v)
                    } else {
                        v.clone()
                    }
                }));
                // Default selection: the pinned default version (offset by 1
                // because "latest" is prepended).
                let idx = dialoguer::Select::new()
                    .with_prompt(format!("{}.{} version", addon_name, tool.name))
                    .items(&items)
                    .default(default_idx + 1)
                    .interact()?;
                if idx == 0 {
                    Some("latest".to_string())
                } else {
                    Some(tool.supported_versions[idx - 1].clone())
                }
            } else {
                None
            };

        // Default version as the floor.
        let version = override_version
            .or(picked_version)
            .unwrap_or_else(|| tool.default_version.clone());

        // Empty string means "no separate version" (e.g. rustfmt is part
        // of the rustup toolchain and has no independent version pin).
        // Represent this as None so the TOML serialises as `tool = {}`.
        let version_opt = if version.is_empty() {
            None
        } else {
            Some(version)
        };
        tools.insert(
            tool.name.clone(),
            ToolEntry {
                version: version_opt,
            },
        );
    }

    Ok(AddonToolsSection { tools })
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
        None => vec!["managed".to_string()],
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
                selections
                    .into_iter()
                    .map(|i| available[i].clone())
                    .collect()
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

    seed::ensure_runtime_dirs(&config)?;
    generate::generate_all(&config)?;

    let state = runtime.container_status(name)?;

    // Version mismatch check: if container exists, ensure its image version matches config.
    // No label = pre-BACK-060 image; allow start without check (backward compat).
    //
    // Two failure modes give the same symptom (the existing container's
    // image label != config.aibox.version) but have different fixes:
    //
    //   A) the image was already rebuilt at the new version by an earlier
    //      `aibox sync`, but the container still references the old image
    //      → fix: `aibox remove && aibox start` to recreate the container
    //   B) the image itself is still at the old version
    //      → fix: `aibox sync` to rebuild the image, then start
    //
    // We can't cheaply distinguish them from inside cmd_start without
    // poking the local image store, so we name both fixes in the error.
    //
    // Skip when aibox.toml pins "latest" — "latest" means "any version is
    // acceptable". Comparing a concrete label (e.g. "0.17.12") against the
    // literal string "latest" would always fire even though the container is
    // correct.
    if state != ContainerState::Missing
        && config.aibox.version != "latest"
        && let Ok(Some(container_version)) = runtime.get_container_image_version(name)
        && container_version != config.aibox.version
    {
        bail!(
            "Version mismatch: the existing container was built from image v{} \
             but aibox.toml pins v{}.\n\n\
             Likely cause: an old container survived an aibox upgrade. Recreate it:\n\
             \n    aibox remove && aibox start\n\n\
             If you have not yet rebuilt the image at the new version, run \
             `aibox sync` first to rebuild it, then the recreate command above.",
            container_version,
            config.aibox.version
        );
    }

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
    // Use a named session matching the container name so that `aibox start`
    // re-attaches to an existing session rather than creating a new one each
    // time. `--create` makes zellij start a fresh session (with the given
    // layout) only when no session named `name` exists yet.
    // `--layout` is a global flag that must come before the subcommand.
    runtime.exec_interactive(
        name,
        &config.container.user,
        &["zellij", "--layout", layout, "attach", "--create", name],
    )?;

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

pub fn cmd_status(config_path: &Option<String>, format: crate::cli::OutputFormat) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let runtime = Runtime::detect()?;
    let name = &config.container.name;

    let state = runtime.container_status(name)?;
    let state_str = match state {
        ContainerState::Running => "running",
        ContainerState::Stopped => "stopped",
        ContainerState::Missing => "missing",
    };

    match format {
        crate::cli::OutputFormat::Json => {
            let obj = serde_json::json!({
                "container": name,
                "state": state_str,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
        crate::cli::OutputFormat::Yaml => {
            let obj = serde_json::json!({
                "container": name,
                "state": state_str,
            });
            print!("{}", serde_yaml::to_string(&obj)?);
        }
        crate::cli::OutputFormat::Table => match state {
            ContainerState::Running => {
                output::ok(&format!("Container '{}' is running", name));
            }
            ContainerState::Stopped => {
                output::warn(&format!("Container '{}' is stopped", name));
            }
            ContainerState::Missing => {
                output::warn(&format!("Container '{}' does not exist", name));
            }
        },
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
    out.push_str(
        "# All .devcontainer/ files are generated from this. Edit here, run `aibox sync`.\n",
    );
    out.push_str(
        "# Reference: https://projectious-work.github.io/aibox/docs/reference/configuration\n",
    );
    out.push_str(sep);
    out.push('\n');

    // [aibox] section
    out.push_str("[aibox]\n");
    out.push_str(&format!(
        "version = {:20} # Target aibox CLI version. Update this when intentionally upgrading aibox.\n",
        format!("\"{}\"", config.aibox.version)
    ));
    out.push_str("                               # Use \"latest\" to always track the newest release without pinning.\n");
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

    // [context] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [context] — context system and process packages\n");
    out.push_str(sep);
    out.push_str("[context]\n");
    out.push_str(&format!(
        "schema_version = {:12} # Context schema version — updated automatically by `aibox sync`\n",
        format!("\"{}\"", config.context.schema_version)
    ));
    out.push_str("# processkit packages (one or more, choose by tier):\n");
    out.push_str("#   minimal   solo developers and small side projects\n");
    out.push_str("#   managed   small teams with a shared backlog (recommended default)\n");
    out.push_str("#   software  software engineering teams (extends managed)\n");
    out.push_str("#   research  research, data science, ML projects (extends managed)\n");
    out.push_str("#   product   full product development (extends software)\n");
    out.push_str(
        "# See context/templates/processkit/<version>/packages/ for the YAML definitions.\n",
    );
    out.push_str(&format!(
        "packages = [{}]\n",
        config
            .context
            .packages
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // [skills] section
    out.push('\n');
    out.push_str(sep);
    if config.skills.include.is_empty() && config.skills.exclude.is_empty() {
        out.push_str("# [skills] — fine-tune which skills are deployed\n");
        out.push_str(sep);
        out.push_str("# Skills are automatically selected from process packages and addons.\n");
        out.push_str("# Use include/exclude to override.\n");
        out.push_str("# [skills]\n");
        out.push_str("# include = [\"skill-name\"]    # Explicitly add skills beyond defaults\n");
        out.push_str("# exclude = [\"skill-name\"]    # Remove skills you don't need\n");
    } else {
        out.push_str("# [skills] — fine-tune which skills are deployed\n");
        out.push_str(sep);
        out.push_str("[skills]\n");
        out.push_str(&format!(
            "include = [{}]\n",
            config
                .skills
                .include
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        out.push_str(&format!(
            "exclude = [{}]\n",
            config
                .skills
                .exclude
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // [addons] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [addons] — language runtimes and tool bundles\n");
    out.push_str(sep);
    out.push_str("# Each addon installs a tool set into the container at build time.\n");
    out.push_str("# Selected addons land here pre-populated with default-enabled tools at\n");
    out.push_str("# their default versions; edit the version strings to switch.\n");
    out.push_str("#\n");
    out.push_str("# Version strings:\n");
    out.push_str("#   \"1.2.3\"  — pin to a specific version\n");
    out.push_str("#   \"latest\" — always install the newest version (skips pinning)\n");
    out.push_str("#   \"\"       — use the addon's built-in default version\n");
    out.push_str("#\n");
    out.push_str("# Run `aibox addon list` to see all available addons.\n");
    out.push_str(
        "# Run `aibox addon info <name>` to see every supported tool/version per addon.\n",
    );
    out.push_str("#\n");
    out.push_str("# To add an addon after init, edit this file and re-run `aibox sync`,\n");
    out.push_str(
        "# or use `aibox addon add <name>` (which also pulls in transitive `requires`).\n",
    );
    if !config.addons.addons.is_empty() {
        let mut addon_names: Vec<_> = config.addons.addons.keys().collect();
        addon_names.sort();
        for addon_name in addon_names {
            let addon_tools = &config.addons.addons[addon_name];
            out.push('\n');

            // Inline comments from the addon definition: description and
            // the full tool roster (versions, defaults, disabled tools).
            if let Some(def) = crate::addon_loader::get_addon(addon_name) {
                if !def.description.is_empty() {
                    out.push_str(&format!("# {}\n", def.description));
                }
                for tool in &def.tools {
                    if !tool.supported_versions.is_empty() {
                        // Show available versions, marking the default.
                        let versions: Vec<String> = tool
                            .supported_versions
                            .iter()
                            .map(|v| {
                                if *v == tool.default_version {
                                    format!("{} (default)", v)
                                } else {
                                    v.clone()
                                }
                            })
                            .collect();
                        out.push_str(&format!("# {}: {}\n", tool.name, versions.join(" | ")));
                    } else if tool.default_enabled {
                        // No curated version list — version can still be pinned freely.
                        out.push_str(&format!(
                            "# {}: pin with {} = {{ version = \"x.y.z\" }}\n",
                            tool.name, tool.name
                        ));
                    }
                    // Tools that are disabled by default: hint they exist but are off.
                    if !tool.default_enabled {
                        out.push_str(&format!(
                            "# {} — disabled by default; add to enable\n",
                            tool.name
                        ));
                    }
                }
            }

            out.push_str(&format!("[addons.{}.tools]\n", addon_name));
            let mut tool_names: Vec<_> = addon_tools.tools.keys().collect();
            tool_names.sort();
            for tool_name in tool_names {
                let tool_entry = &addon_tools.tools[tool_name];
                match &tool_entry.version {
                    Some(v) => {
                        out.push_str(&format!("{} = {{ version = \"{}\" }}\n", tool_name, v))
                    }
                    None => out.push_str(&format!("{} = {{}}\n", tool_name)),
                }
            }
        }
    }

    // [ai] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [ai] — AI agent harnesses and model providers\n");
    out.push_str(sep);
    out.push_str("# Harnesses: CLI tools installed in the container.\n");
    out.push_str("# Harness (CLI tool)          Config value   Provider (API key)\n");
    out.push_str("# Claude Code                 claude         Anthropic\n");
    out.push_str("# OpenAI Codex                codex          OpenAI\n");
    out.push_str("# Gemini CLI                  gemini         Google\n");
    out.push_str("# Aider                       aider          any (multi-provider)\n");
    out.push_str("# Continue                    continue       any (multi-provider)\n");
    out.push_str("# Cursor                      cursor         any (host-side IDE)\n");
    out.push_str("# GitHub Copilot              copilot        (uses GITHUB_TOKEN)\n");
    out.push_str("# OpenCode                    opencode       any (multi-provider)\n");
    out.push_str("# Hermes                      hermes         any (multi-provider)\n");
    out.push_str("#\n");
    out.push_str("# Model providers (optional): declare which API keys are available.\n");
    out.push_str("# Provider     Config value   Env var\n");
    out.push_str("# Anthropic    anthropic      ANTHROPIC_API_KEY\n");
    out.push_str("# OpenAI       openai         OPENAI_API_KEY\n");
    out.push_str("# Google       google         GEMINI_API_KEY\n");
    out.push_str("# Mistral      mistral        MISTRAL_API_KEY\n");
    out.push_str("[ai]\n");
    out.push_str(&format!(
        "harnesses = [{}]\n",
        config
            .ai
            .harnesses
            .iter()
            .map(|h| format!("\"{}\"", h))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    if !config.ai.model_providers.is_empty() {
        out.push_str(&format!(
            "model_providers = [{}]\n",
            config
                .ai
                .model_providers
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        out.push_str("# model_providers = [\"anthropic\", \"openai\"]  # optional: which API keys you have\n");
    }

    // [processkit] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [processkit] — content layer source (skills, primitives, processes)\n");
    out.push_str(sep);
    out.push_str("# processkit ships the skills and primitives that aibox installs into the\n");
    out.push_str("# project. The default upstream is the canonical projectious-work/processkit\n");
    out.push_str(
        "# repo. Companies can fork processkit and have their projects consume the fork\n",
    );
    out.push_str("# by changing `source` to point at their fork.\n");
    out.push_str("#\n");
    out.push_str(
        "# `version` is the git tag of the processkit source to consume. Special values:\n",
    );
    out.push_str("#   \"unset\"  — no version pinned yet; processkit content is not installed.\n");
    out.push_str("#   \"latest\" — resolve to the newest available tag at every `aibox sync`.\n");
    out.push_str("[processkit]\n");
    out.push_str(&format!("source   = \"{}\"\n", config.processkit.source));
    out.push_str(&format!("version  = \"{}\"\n", config.processkit.version));
    out.push_str(&format!("src_path = \"{}\"\n", config.processkit.src_path));
    match &config.processkit.branch {
        Some(branch) => out.push_str(&format!("branch   = \"{}\"\n", branch)),
        None => out.push_str(
            "# branch = \"main\"   # optional — for tracking a moving branch (discouraged)\n",
        ),
    }
    out.push_str("#\n");
    out.push_str("# Optional release-asset URL template for non-GitHub hosts (Gitea, GitLab,\n");
    out.push_str("# self-hosted). When unset, the fetcher uses the GitHub-style default:\n");
    out.push_str("#   {source}/releases/download/{version}/{name}-{version}.tar.gz\n");
    out.push_str("# Placeholders: {source} (.git stripped), {version}, {org}, {name}.\n");
    match &config.processkit.release_asset_url_template {
        Some(t) => out.push_str(&format!("release_asset_url_template = \"{}\"\n", t)),
        None => out.push_str("# release_asset_url_template = \"https://gitea.example.com/{org}/{name}/releases/download/{version}/payload.tar.gz\"\n"),
    }

    // [agents] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [agents] — canonical AGENTS.md + provider-specific entry files\n");
    out.push_str(sep);
    out.push_str("# AGENTS.md is the canonical, provider-neutral instruction document for AI\n");
    out.push_str("# coding agents. Provider files (CLAUDE.md, future CODEX.md, …) are thin\n");
    out.push_str("# pointers that simply say \"see AGENTS.md\". This matches the agents.md\n");
    out.push_str("# ecosystem convention and avoids keeping N copies of the same instructions.\n");
    out.push_str("#\n");
    out.push_str("# `provider_mode` options:\n");
    out.push_str("#   pointer (default) — provider files are thin pointers to AGENTS.md\n");
    out.push_str("#   full              — provider files contain rich provider-flavored content\n");
    out.push_str("#                       (use only when you genuinely need different content per harness)\n");
    out.push_str("[agents]\n");
    out.push_str(&format!(
        "canonical     = \"{}\"\n",
        config.agents.canonical
    ));
    let mode_str = match config.agents.provider_mode {
        crate::config::AgentsProviderMode::Pointer => "pointer",
        crate::config::AgentsProviderMode::Full => "full",
    };
    out.push_str(&format!("provider_mode = \"{}\"\n", mode_str));

    // [customization] section
    out.push('\n');
    out.push_str(sep);
    out.push_str("# [customization] — color theme, shell prompt, and zellij layout\n");
    out.push_str(sep);
    out.push_str("# Theme is applied consistently across Zellij, Vim, Yazi, lazygit, and bat.\n");
    out.push_str("# Options: gruvbox-dark | catppuccin-mocha | catppuccin-latte | dracula | tokyo-night | nord | projectious\n");
    out.push_str("[customization]\n");
    out.push_str(&format!("theme  = \"{}\"\n", config.customization.theme));
    out.push_str("# Starship prompt preset.\n");
    out.push_str("# Options: default | plain | minimal | nerd-font | pastel | bracketed | arrow\n");
    out.push_str(&format!("prompt = \"{}\"\n", config.customization.prompt));
    out.push_str(
        "# Default zellij layout. Options: dev | focus | cowork | cowork-swap | browse | ai\n",
    );
    out.push_str(&format!("layout = \"{}\"\n", config.customization.layout));

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
        AddonsSection, AiSection, AiboxConfig, AiboxSection, AudioSection, ContainerSection,
        ContextSection, CustomizationSection, SkillsSection,
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

    let (project_name, base_image, process_packages, addon_names) = resolve_init_values(
        params.name,
        params.base,
        params.process,
        params.addons,
        interactive,
    )?;

    let container_user = params.user.unwrap_or_else(|| "aibox".to_string());
    let ai_providers = match params.ai {
        Some(providers) => providers,
        None if interactive => {
            let all_harnesses = AiHarness::all();
            let items: Vec<String> = all_harnesses
                .iter()
                .map(|h| h.display_name().to_string())
                .collect();
            // Claude Code is the first item and pre-selected by default.
            let defaults: Vec<bool> = all_harnesses
                .iter()
                .enumerate()
                .map(|(i, _)| i == 0)
                .collect();
            let selections = dialoguer::MultiSelect::new()
                .with_prompt("AI harnesses (space to select, enter to confirm)")
                .items(&items)
                .defaults(&defaults)
                .interact()?;
            if selections.is_empty() {
                vec![AiHarness::Claude]
            } else {
                selections
                    .into_iter()
                    .map(|i| all_harnesses[i].clone())
                    .collect()
            }
        }
        None => vec![AiHarness::Claude],
    };

    // Collect AI harness addon names before they're moved into the config
    // struct so we can include them in the dependency expansion and tool
    // population below.
    let ai_addon_names: Vec<String> = ai_providers
        .iter()
        .filter(|h| h.is_active())
        .map(|h| h.addon_name())
        .filter(|n| !n.is_empty())
        .collect();

    let mut config = AiboxConfig {
        aibox: AiboxSection {
            version: env!("CARGO_PKG_VERSION").to_string(),
            base: base_image,
        },
        container: ContainerSection {
            name: project_name.clone(),
            hostname: project_name,
            user: container_user,
            post_create_command: None,
            keepalive: false,
            environment: std::collections::HashMap::new(),
            extra_volumes: vec![],
        },
        context: ContextSection {
            packages: process_packages,
            ..ContextSection::default()
        },
        ai: AiSection {
            harnesses: ai_providers,
            model_providers: Vec::new(),
            providers: Vec::new(),
        },
        process: None,
        addons: {
            // Build the addon section in four steps:
            //   1. Combine user-selected addons with AI provider addons so
            //      their `requires` deps (e.g. ai-codex → node) are pulled
            //      in transitively alongside the rest.
            //   2. Transitively expand `requires` on the combined list.
            //   3. Parse the repeated --addon-tool flag values into a
            //      nested map (addon → tool → version).
            //   4. For each (now-complete) addon, populate its tools
            //      sub-table with default-enabled tools at the right
            //      version (CLI override > interactive pick > default).
            let all_initial: Vec<String> = addon_names
                .iter()
                .chain(ai_addon_names.iter())
                .cloned()
                .collect();
            let expanded_addons = expand_addon_requires(&all_initial);
            for added in &expanded_addons {
                if !all_initial.contains(added) {
                    output::info(&format!(
                        "Adding addon '{}' (transitively required by your selection)",
                        added
                    ));
                }
            }
            let tool_overrides = build_tool_overrides(&params.addon_tool)?;
            let mut section = AddonsSection::default();
            for name in &expanded_addons {
                let tools = populate_addon_tools(name, tool_overrides.get(name), interactive)?;
                section.addons.insert(name.clone(), tools);
            }
            section
        },
        skills: SkillsSection::default(),
        processkit: resolve_processkit_section(
            params.processkit_source.as_deref(),
            params.processkit_version.as_deref(),
            params.processkit_branch.as_deref(),
            interactive,
        )?,
        customization: CustomizationSection {
            theme: params.theme.unwrap_or_default(),
            prompt: params.prompt.unwrap_or_default(),
            layout: crate::config::ConfigLayout::default(),
        },
        agents: crate::config::AgentsSection::default(),
        audio: AudioSection::default(),
        mcp: crate::config::McpSection::default(),
        local_env: std::collections::HashMap::new(),
        local_mcp_servers: vec![],
    };
    config.resolve_ai_provider_addons();

    config.validate()?;

    // --- summary page ---
    if interactive {
        println!();
        output::info("Configuration summary:");
        println!("  Project:     {}", config.container.name);
        println!("  Base:        {}", config.aibox.base);
        println!("  Process:     {}", config.context.packages.join(", "));
        let addon_list: Vec<String> = config
            .addons
            .addons
            .keys()
            .filter(|k| !k.starts_with("ai-"))
            .cloned()
            .collect();
        if addon_list.is_empty() {
            println!("  Addons:      (none)");
        } else {
            println!("  Addons:      {}", addon_list.join(", "));
        }
        let harness_list: Vec<&str> = config
            .ai
            .harnesses
            .iter()
            .map(|h| h.display_name())
            .collect();
        println!("  Harnesses:   {}", harness_list.join(", "));
        println!("  Theme:       {}", config.customization.theme);
        println!("  processkit:  {}", config.processkit.version);
        println!();
        let proceed = dialoguer::Confirm::new()
            .with_prompt("Generate project with these settings?")
            .default(true)
            .interact()?;
        if !proceed {
            bail!("Init cancelled by user.");
        }
    }

    let toml_str = serialize_config_with_comments(&config);

    std::fs::write(&toml_path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", toml_path.display(), e))?;

    output::ok(&format!("Created {}", toml_path.display()));

    generate::generate_all(&config)?;
    context::scaffold_context(&config)?;
    seed::seed_root_dir(&config)?;

    // Install processkit content (A5). Runs last, after the rest of the
    // init pipeline has succeeded. Warn-and-continue on failure so a
    // network hiccup or bad processkit URL doesn't wedge the user's
    // whole init — they get a working aibox project either way and can
    // fix the [processkit] section then re-run `aibox sync`.
    output::info("Installing processkit content...");
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("failed to resolve current directory: {}", e))?;
    match crate::content_init::install_content_source(&project_root, &config) {
        Ok(report) if report.skipped_due_to_unset => {
            output::warn(&format!(
                "Skipped processkit install — [processkit] version is \"{}\". \
                 Edit aibox.toml and run `aibox sync` to install processkit content.",
                crate::config::PROCESSKIT_VERSION_UNSET
            ));
        }
        Ok(report) => {
            output::ok(&format!(
                "Installed {} files from processkit {}@{} ({} groups, {} skipped)",
                report.files_installed,
                report.fetched_from,
                report.fetched_version,
                report.groups_touched,
                report.files_skipped,
            ));
            // After install, regenerate per-harness MCP config files.
            // Best-effort: any failure is warned-and-continued so an
            // MCP-registration glitch doesn't break the rest of init.
            if let Err(e) = crate::mcp_registration::regenerate_mcp_configs(&config, &project_root)
            {
                output::warn(&format!("MCP registration failed: {}", e));
            }
            // Sync processkit command adapter files to .claude/commands/
            // so Claude Code can tab-complete them. Best-effort.
            if let Err(e) = crate::claude_commands::sync_claude_commands(&project_root, &config) {
                output::warn(&format!("Claude command sync failed: {}", e));
            }
        }
        Err(e) => {
            output::warn(&format!(
                "Processkit install failed: {}. The project is set up but processkit \
                 content was not installed. Run `aibox sync` to retry.",
                e
            ));
        }
    }

    output::ok("Project initialized. Edit aibox.toml to customize, then run: aibox start");

    Ok(())
}

/// Sync command: force-seed theme-dependent files, seed missing configs, regenerate .devcontainer/.
///
/// See `crate::sync_perimeter` for the documented set of files this
/// command is allowed to create, modify, or delete. The tripwire below
/// snapshots a small set of representative out-of-perimeter files
/// before the sync runs and verifies after that none of them were
/// touched — providing a runtime guarantee in addition to the static
/// `is_within_perimeter` check used by sync write helpers.
pub fn cmd_sync(config_path: &Option<String>, no_cache: bool, no_build: bool) -> Result<()> {
    // Snapshot out-of-perimeter sentinels before any sync work runs.
    // The tripwire is verified at the end of cmd_sync.
    let tripwire =
        crate::sync_perimeter::Tripwire::snapshot(std::env::current_dir().ok().as_deref());
    let pre_sync_cli_version = crate::lock::read_lock(std::path::Path::new("."))
        .ok()
        .flatten()
        .map(|lock| lock.aibox.cli_version)
        .filter(|v| !v.is_empty());

    // Check for version migration before any other sync steps
    crate::migration::check_and_generate_migration()?;

    let mut config = AiboxConfig::from_cli_option(config_path)?;

    // Resolve [processkit].version = "latest" to a concrete tag before any
    // further processing. The lock always stores a concrete version; "latest"
    // is an aibox.toml-only convenience that is never written to the lock.
    //
    // Semver-aware upgrade policy:
    //   - Fresh install (no lock): take absolute latest unconditionally.
    //   - Patch or minor upgrade (same major): apply automatically.
    //   - Major upgrade: block and warn; take best available within current major.
    //     User must pin an explicit version in aibox.toml to cross a major boundary.
    if config.processkit.version == crate::config::PROCESSKIT_VERSION_LATEST {
        match crate::content_source::list_versions(&config.processkit.source) {
            Ok(versions) if !versions.is_empty() => {
                // Read the currently installed version tag from the lock file.
                let installed_tag: Option<String> =
                    crate::lock::read_lock(std::path::Path::new("."))
                        .ok()
                        .flatten()
                        .and_then(|lock| lock.processkit)
                        .map(|pk| pk.version.clone());

                let absolute_latest = versions[0].clone();

                let resolved = if let Some(ref tag) = installed_tag {
                    let installed_sv = crate::content_source::parse_loose_semver(tag);
                    let latest_sv = crate::content_source::parse_loose_semver(&absolute_latest);
                    match (installed_sv, latest_sv) {
                        (Some(installed), Some(latest)) if latest.major > installed.major => {
                            // Major upgrade: block and find best within current major.
                            crate::output::warn(&format!(
                                "processkit 'latest' ({}) would be a major upgrade from \
                                 the installed version ({}). Major version upgrades are \
                                 not applied automatically — pin an explicit version in \
                                 aibox.toml to upgrade. Staying on the latest v{}.x release.",
                                absolute_latest, tag, installed.major
                            ));
                            let best_in_major = versions
                                .iter()
                                .filter_map(|v| {
                                    crate::content_source::parse_loose_semver(v)
                                        .map(|sv| (sv, v.clone()))
                                })
                                .filter(|(sv, _)| sv.major == installed.major)
                                .max_by_key(|(sv, _)| sv.clone())
                                .map(|(_, v)| v);
                            match best_in_major {
                                Some(v) => {
                                    output::info(&format!(
                                        "Resolved processkit 'latest' \u{2192} {} \
                                         (latest v{}.x)",
                                        v, installed.major
                                    ));
                                    v
                                }
                                None => {
                                    // No releases in current major — keep installed.
                                    output::info(&format!(
                                        "No v{}.x releases found; keeping installed \
                                         version {}.",
                                        installed.major, tag
                                    ));
                                    tag.clone()
                                }
                            }
                        }
                        _ => {
                            // Same or lower major, or unparseable: auto-apply latest.
                            output::info(&format!(
                                "Resolved processkit 'latest' \u{2192} {} (upgrade from {})",
                                absolute_latest, tag
                            ));
                            absolute_latest
                        }
                    }
                } else {
                    // Fresh install (no lock): take absolute latest unconditionally.
                    output::info(&format!(
                        "Resolved processkit 'latest' \u{2192} {} (fresh install)",
                        absolute_latest
                    ));
                    absolute_latest
                };
                config.processkit.version = resolved;
            }
            Ok(_) => {
                crate::output::warn(
                    "processkit.version = \"latest\" but no versions found at source; \
                     skipping processkit install. Set an explicit version in aibox.toml.",
                );
            }
            Err(e) => {
                crate::output::warn(&format!(
                    "processkit.version = \"latest\" but version resolution failed: {}. \
                     Skipping processkit install. Check your network or set an explicit version.",
                    e
                ));
            }
        }
    }

    // Resolve [aibox].version = "latest" to a concrete image tag before
    // Dockerfile generation. "latest" is never a valid Docker image tag in
    // our registry (tags are base-<flavor>-v<semver>); we must resolve it to
    // a concrete version so the generated Dockerfile references a real image.
    if config.aibox.version == "latest" {
        let flavor = config.aibox.base.to_string();
        match crate::update::fetch_latest_image_version(&flavor) {
            Ok(v) => {
                let resolved = format!("{}.{}.{}", v.major, v.minor, v.patch);
                output::info(&format!(
                    "Resolved aibox image 'latest' \u{2192} v{}",
                    resolved
                ));
                config.aibox.version = resolved;
            }
            Err(e) => {
                crate::output::warn(&format!(
                    "[aibox].version = \"latest\" but image version resolution failed: {}. \
                     Dockerfile will reference a concrete version if one was previously resolved, \
                     or may fail. Consider setting an explicit version in aibox.toml.",
                    e
                ));
            }
        }
    }

    // Warn if running CLI version differs from the pinned target version.
    // Skip when version = "latest" (user explicitly opts out of pinning).
    let aibox_version_pin = &config.aibox.version;
    if !aibox_version_pin.is_empty()
        && aibox_version_pin != "latest"
        && aibox_version_pin != env!("CARGO_PKG_VERSION")
    {
        crate::output::warn(&format!(
            "aibox.toml pins version {} but you are running {} — consider updating [aibox].version",
            aibox_version_pin,
            env!("CARGO_PKG_VERSION")
        ));
    }

    // Warn if processkit version is below minimum for this aibox.
    // Skip when version was "latest" (already resolved to the newest available).
    let current_aibox = env!("CARGO_PKG_VERSION");
    if let Some(compat) = crate::compat::min_processkit_for(current_aibox)
        && !crate::compat::processkit_meets_minimum(
            &config.processkit.version,
            compat.processkit_version,
        )
    {
        crate::output::warn(&format!(
            "processkit {} is below the minimum recommended version {} for aibox v{} ({}). \
             Consider updating [processkit].version in aibox.toml.",
            config.processkit.version, compat.processkit_version, current_aibox, compat.note,
        ));
    }

    // Resolve "latest" addon tool versions to concrete versions.
    // The resolved versions are used in Dockerfile generation and recorded
    // in aibox.lock so builds are reproducible.
    let mut resolved_tools = std::collections::BTreeMap::new();
    for (addon_name, addon_tools) in &mut config.addons.addons {
        for (tool_name, tool_entry) in &mut addon_tools.tools {
            if tool_entry.version.as_deref() == Some("latest") {
                // Try upstream resolution for key tools
                if let Some(resolved) = crate::version_resolve::resolve_latest(tool_name) {
                    tool_entry.version = Some(resolved.clone());
                    resolved_tools.insert(tool_name.clone(), resolved);
                } else if let Some(addon) = crate::addon_loader::get_addon(addon_name)
                    && let Some(tool_def) = addon.tools.iter().find(|t| &t.name == tool_name)
                    && !tool_def.default_version.is_empty()
                {
                    // Fall back to addon's default_version
                    let ver = tool_def.default_version.clone();
                    tool_entry.version = Some(ver.clone());
                    resolved_tools.insert(tool_name.clone(), ver);
                }
            }
        }
    }
    if !resolved_tools.is_empty() {
        output::info(&format!(
            "Resolved {} 'latest' tool version(s) to concrete values",
            resolved_tools.len()
        ));
        // Write resolved versions to aibox.lock
        let project_root = std::env::current_dir().unwrap_or_default();
        if let Ok(Some(mut lock)) = crate::lock::read_lock(&project_root) {
            lock.addons = Some(crate::lock::AddonsLockSection {
                resolved_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                tools: resolved_tools,
            });
            if let Err(e) = crate::lock::write_lock(&project_root, &lock) {
                output::warn(&format!(
                    "Failed to update aibox.lock with resolved tool versions: {}",
                    e
                ));
            }
        }
    }

    output::info("Scaffolding missing runtime directories...");
    seed::ensure_runtime_dirs(&config)?;
    generate::generate_all(&config)?;

    // Skills, AGENTS.md, and the universal baseline are owned by
    // processkit since v0.16.0. The first time we see a real
    // [processkit].version (i.e. not the "unset" sentinel) we install
    // the content here so the user can pin a version after the initial
    // `aibox init` and have `aibox sync` materialize it. The install is
    // skipped when the lock already pins the same (source, version)
    // pair as aibox.toml — the existing three-way diff path further
    // down handles drift detection in that case.
    match std::env::current_dir() {
        Ok(cwd) => {
            let lock_pair = match crate::lock::read_lock(&cwd) {
                // Only the [processkit] section is relevant for the
                // re-install gating decision. Locks without a
                // [processkit] section (e.g. fresh init that has not
                // installed any content yet) are treated as None.
                Ok(Some(lock)) => lock
                    .processkit
                    .as_ref()
                    .map(|pk| (pk.source.clone(), pk.version.clone())),
                Ok(None) => None,
                Err(e) => {
                    output::warn(&format!(
                        "Failed to read aibox.lock; will re-install processkit: {}",
                        e
                    ));
                    None
                }
            };
            if sync_should_install_processkit(
                &config.processkit.version,
                &config.processkit.source,
                lock_pair.as_ref().map(|(s, v)| (s.as_str(), v.as_str())),
            ) {
                {
                    output::info(&format!(
                        "Installing processkit {}@{}...",
                        config.processkit.source, config.processkit.version
                    ));
                    match crate::content_init::install_content_source(&cwd, &config) {
                        Ok(report) if report.skipped_due_to_unset => {
                            // Defensive — we already gated on version != unset.
                        }
                        Ok(report) => {
                            output::ok(&format!(
                                "Installed {} files from processkit {}@{} ({} groups, {} skipped)",
                                report.files_installed,
                                report.fetched_from,
                                report.fetched_version,
                                report.groups_touched,
                                report.files_skipped,
                            ));
                        }
                        Err(e) => {
                            output::warn(&format!(
                                "Processkit install failed: {}. Sync will continue without \
                                 fresh content; fix the [processkit] section and re-run \
                                 `aibox sync` to retry.",
                                e
                            ));
                        }
                    }
                }
            }
        }
        Err(e) => output::warn(&format!(
            "Failed to determine working directory; skipping processkit install: {}",
            e
        )),
    }

    // Regenerate per-harness MCP config files (.mcp.json,
    // .cursor/mcp.json, .gemini/settings.json, .codex/config.toml,
    // .continue/mcpServers/*.json) based on the currently-pinned
    // processkit version and the [ai].providers list. Idempotent —
    // re-running on a stable (version, providers, skills) set
    // produces byte-identical output. Best-effort: any failure is
    // warned-and-continued. See DEC-033.
    if let Ok(cwd) = std::env::current_dir() {
        if let Err(e) = crate::mcp_registration::regenerate_mcp_configs(&config, &cwd) {
            output::warn(&format!("MCP registration failed: {}", e));
        }
        // Sync processkit command adapter files to .claude/commands/
        // so Claude Code can tab-complete them. Best-effort.
        if let Err(e) = crate::claude_commands::sync_claude_commands(&cwd, &config) {
            output::warn(&format!("Claude command sync failed: {}", e));
        }
    }

    // Three-way runtime diff for managed .aibox-home files.
    match std::env::current_dir() {
        Ok(cwd) => {
            let current_cli_version = env!("CARGO_PKG_VERSION");
            match crate::runtime_sync::run_runtime_sync(
                &cwd,
                pre_sync_cli_version.as_deref(),
                current_cli_version,
                &config,
            ) {
                Ok(report) => {
                    if report.summary.has_user_relevant_changes() {
                        output::info(&format!(
                            ".aibox-home changes detected: {} upstream-only, {} conflicts, {} new, {} removed",
                            report.summary.changed_upstream_only,
                            report.summary.conflict,
                            report.summary.new_upstream,
                            report.summary.removed_upstream,
                        ));
                        if let Some(path) = report.migration_document_path {
                            output::ok(&format!("Wrote migration document: {}", path.display()));
                        }
                    } else {
                        output::ok(
                            "Managed .aibox-home runtime files are in sync — no migration needed",
                        );
                    }
                }
                Err(e) => output::warn(&format!("Runtime config diff failed: {}", e)),
            }
        }
        Err(e) => output::warn(&format!("Failed to determine working directory: {}", e)),
    }

    // Three-way processkit diff (A6).
    //
    // If the project doesn't yet have an aibox.lock (i.e. nobody has run
    // `aibox init` against this project after A5 landed, OR the version is
    // "unset"), skip — there's nothing to compare against. Any failure is
    // warned-and-continued so a network glitch doesn't break the rest of
    // sync's work.
    match std::env::current_dir() {
        Ok(cwd) => match crate::lock::read_lock(&cwd) {
            Ok(Some(lock)) => match lock.processkit.as_ref() {
                None => {
                    // No processkit section yet — nothing to diff.
                }
                Some(pk) => {
                    output::info("Comparing processkit cache against project...");
                    match crate::content_diff::run_content_sync(&cwd, pk, &config) {
                        Ok(report) => {
                            if report.summary.has_user_relevant_changes() {
                                output::info(&format!(
                                    "Processkit changes detected: {} upstream-only, {} conflicts, {} new, {} removed",
                                    report.summary.changed_upstream_only,
                                    report.summary.conflict,
                                    report.summary.new_upstream,
                                    report.summary.removed_upstream,
                                ));
                                if let Some(path) = report.migration_document_path {
                                    output::ok(&format!(
                                        "Wrote migration document: {}",
                                        path.display()
                                    ));
                                }
                            } else {
                                output::ok("Processkit cache is in sync — no migration needed");
                            }
                        }
                        Err(e) => output::warn(&format!("Processkit diff failed: {}", e)),
                    }
                } // Some(pk)
            },
            Ok(None) => { /* No lock file yet — nothing to diff against. */ }
            Err(e) => output::warn(&format!("Failed to read processkit lock: {}", e)),
        },
        Err(e) => output::warn(&format!("Failed to determine working directory: {}", e)),
    }

    // Verify the perimeter tripwire BEFORE the (potentially long) image
    // build, so a perimeter violation aborts as fast as possible.
    tripwire.verify()?;

    // Build container image (if a container runtime is available)
    if no_build {
        output::ok("Sync complete (build skipped)");
    } else {
        match Runtime::detect() {
            Ok(runtime) => {
                output::info("Building container image...");
                runtime.compose_build(crate::config::COMPOSE_FILE, no_cache)?;
                output::ok("Sync complete — image built");
                warn_if_container_lags_image(&runtime, &config);
            }
            Err(_) => {
                output::warn("No container runtime found — skipping image build");
                output::ok("Sync complete (config files only)");
            }
        }
    }

    Ok(())
}

/// Warn the user if a container exists for this project AND its image
/// label disagrees with the just-built image. This catches the
/// "I synced but my old container is still running on the old image"
/// situation BEFORE the user runs `aibox start` and gets a hard error.
///
/// Best-effort: any failure (runtime probe, label read) is silently
/// swallowed. The warning is informational, not load-bearing — its
/// only job is to surface a stale runtime so the next `aibox start`
/// isn't a surprise.
fn warn_if_container_lags_image(runtime: &Runtime, config: &AiboxConfig) {
    let name = &config.container.name;
    let Ok(state) = runtime.container_status(name) else {
        return;
    };
    if state == ContainerState::Missing {
        return;
    }
    let Ok(Some(container_version)) = runtime.get_container_image_version(name) else {
        return;
    };
    if container_version == config.aibox.version {
        return;
    }
    output::warn(&format!(
        "Container '{}' is still running on image v{} but the freshly-built image is v{}.\n    \
         The current container will keep running on the old image until you recreate it. To upgrade:\n    \
         \n        aibox remove && aibox start\n    \
         \n    Existing in-flight work in the container (open editors, running processes) will be lost \
         on recreation; project files under /workspace are mounted from the host and survive.",
        name, container_version, config.aibox.version
    ));
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
        assert_eq!(process, vec!["managed".to_string()]);
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

    // ── parse_addon_tool_override / build_tool_overrides ────────────────────

    #[test]
    fn parse_addon_tool_override_happy_path() {
        let (a, t, v) = parse_addon_tool_override("python:python=3.14").unwrap();
        assert_eq!(a, "python");
        assert_eq!(t, "python");
        assert_eq!(v, "3.14");
    }

    #[test]
    fn parse_addon_tool_override_handles_dotted_versions() {
        let (a, t, v) = parse_addon_tool_override("node:pnpm=10.5.0").unwrap();
        assert_eq!(a, "node");
        assert_eq!(t, "pnpm");
        assert_eq!(v, "10.5.0");
    }

    #[test]
    fn parse_addon_tool_override_rejects_missing_equals() {
        let err = parse_addon_tool_override("python:python").unwrap_err();
        assert!(format!("{}", err).contains("=<version>"));
    }

    #[test]
    fn parse_addon_tool_override_rejects_missing_colon() {
        let err = parse_addon_tool_override("python=3.14").unwrap_err();
        assert!(format!("{}", err).contains("addon prefix"));
    }

    #[test]
    fn parse_addon_tool_override_rejects_empty_components() {
        assert!(parse_addon_tool_override(":python=3.14").is_err());
        assert!(parse_addon_tool_override("python:=3.14").is_err());
        assert!(parse_addon_tool_override("python:python=").is_err());
    }

    #[test]
    fn build_tool_overrides_groups_by_addon() {
        let raw = vec![
            "python:python=3.14".to_string(),
            "python:uv=0.8".to_string(),
            "node:node=20".to_string(),
        ];
        let map = build_tool_overrides(&raw).unwrap();
        assert_eq!(map["python"]["python"], "3.14");
        assert_eq!(map["python"]["uv"], "0.8");
        assert_eq!(map["node"]["node"], "20");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn build_tool_overrides_propagates_parse_error() {
        let raw = vec!["bogus".to_string()];
        assert!(build_tool_overrides(&raw).is_err());
    }

    // ── expand_addon_requires ───────────────────────────────────────────────
    //
    // The full transitive expansion is exercised by the e2e tests
    // (cli/tests/e2e/) which load the real addon registry. The unit
    // tests below cover the dedupe and ordering invariants without
    // depending on the registry.

    #[test]
    fn expand_addon_requires_preserves_initial_order() {
        // No addon registry initialized in unit tests → expansion is a
        // no-op (get_addon returns None for everything). The function
        // must still preserve the input order and not duplicate.
        let input = vec!["python".to_string(), "node".to_string()];
        let out = expand_addon_requires(&input);
        assert_eq!(out, vec!["python", "node"]);
    }

    #[test]
    fn expand_addon_requires_handles_empty() {
        let out = expand_addon_requires(&[]);
        assert!(out.is_empty());
    }

    // ── sync_should_install_processkit ──────────────────────────────────────

    #[test]
    fn sync_install_skipped_when_version_is_unset() {
        // The "unset" sentinel always disables the auto-install — even if a
        // stale lock from an earlier real version exists, sync should leave
        // it alone. The user has explicitly opted out by typing "unset".
        assert!(!sync_should_install_processkit(
            crate::config::PROCESSKIT_VERSION_UNSET,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            None,
        ));
        assert!(!sync_should_install_processkit(
            crate::config::PROCESSKIT_VERSION_UNSET,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            Some((
                crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION
            )),
        ));
    }

    #[test]
    fn sync_install_runs_when_version_pinned_and_no_lock() {
        // User pinned a real version but no lock exists yet — sync must install.
        assert!(sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            None,
        ));
    }

    #[test]
    fn sync_install_skipped_when_lock_matches_config() {
        // Steady state: the install already ran, lock matches config →
        // sync should NOT re-install. The downstream three-way diff path
        // handles drift detection from here on.
        assert!(!sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            Some((
                crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION
            )),
        ));
    }

    #[test]
    fn sync_install_runs_when_lock_version_stale() {
        // User bumped processkit.version in aibox.toml from v0.5.1 → v0.6.0.
        // Sync must re-install so the new version's content lands.
        assert!(sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            Some((crate::processkit_vocab::PROCESSKIT_GIT_SOURCE, "v0.5.1")),
        ));
    }

    #[test]
    fn sync_install_runs_when_lock_source_changed() {
        // User switched from upstream processkit to a fork (or vice versa).
        // Sync must re-install from the new source even if the version tag
        // happens to match.
        assert!(sync_should_install_processkit(
            crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION,
            "https://github.com/acme/processkit-acme.git",
            Some((
                crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
                crate::processkit_vocab::PROCESSKIT_DEFAULT_VERSION
            )),
        ));
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
