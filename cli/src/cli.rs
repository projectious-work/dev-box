use clap::{Parser, Subcommand, ValueEnum};

use crate::config::{AiProvider, BaseImage, StarshipPreset, Theme};

/// Available Zellij IDE layouts.
#[derive(Clone, Debug, ValueEnum)]
pub enum Layout {
    /// VS Code-like: Yazi sidebar, Vim editor, stacked terminals
    Dev,
    /// One tool per tab, fullscreen, zero distraction
    Focus,
    /// Side-by-side coding with AI: yazi+vim left, claude right
    Cowork,
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Dev => write!(f, "dev"),
            Layout::Focus => write!(f, "focus"),
            Layout::Cowork => write!(f, "cowork"),
        }
    }
}

#[derive(Parser)]
#[command(
    name = "aibox",
    about = "Manage AI-ready development container environments",
    long_about = "\
aibox — manage AI-ready development container environments

aibox creates reproducible, containerized development environments with
built-in AI context structure and work process management.

Examples:
  aibox init                                 Interactive project setup
  aibox init --name my-app --image python --process product
  aibox init --image rust --process minimal  Rust project, minimal context
  aibox sync                                 Reconcile config + build image
  aibox sync --no-cache                      Force full rebuild
  aibox start                                Start and attach (dev layout)
  aibox start --layout focus                 Start with focus layout
  aibox doctor                               Validate project structure
  aibox update --check                       Check for newer versions
  aibox audio check                          Diagnose host audio setup",
    version
)]
pub struct Cli {
    /// Path to aibox.toml (default: ./aibox.toml)
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, env = "AIBOX_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project with aibox.toml and generated files
    ///
    /// Creates aibox.toml, generates .devcontainer/ files, scaffolds
    /// context directory, seeds .aibox-home/ with default configs, and
    /// sets up .gitignore.
    ///
    /// Without flags, runs interactively. With all flags, runs non-interactively.
    Init {
        /// Project name (default: current directory name)
        #[arg(long)]
        name: Option<String>,

        /// Base image (default: debian)
        #[arg(long, value_enum)]
        base: Option<BaseImage>,

        /// Process packages (comma-separated, e.g., "managed,code")
        #[arg(long, num_args = 1..)]
        process: Option<Vec<String>>,

        /// AI tool providers to configure (default: claude)
        #[arg(long, value_enum, num_args = 1..)]
        ai: Option<Vec<AiProvider>>,

        /// Container user (default: root)
        #[arg(long)]
        user: Option<String>,

        /// Color theme for all tools (default: gruvbox-dark)
        #[arg(long, value_enum)]
        theme: Option<Theme>,

        /// Starship prompt preset (default: default)
        #[arg(long, value_enum)]
        prompt: Option<StarshipPreset>,

        /// Addon names to enable (e.g., python, infrastructure, kubernetes)
        #[arg(long, num_args = 1..)]
        addons: Option<Vec<String>>,
    },
    /// Reconcile project state with aibox.toml configuration
    ///
    /// Seeds config files, regenerates .devcontainer/ files, reconciles
    /// skills, and builds the container image. The primary command for
    /// applying any config change.
    #[command(alias = "generate")]
    Sync {
        /// Build without cache (force full rebuild)
        #[arg(long)]
        no_cache: bool,
    },
    /// Start container and attach via zellij
    ///
    /// Seeds .aibox-home/ if needed, generates devcontainer files,
    /// creates/starts the container, then attaches via zellij.
    /// If already running, just attaches.
    ///
    /// Available layouts: dev (default), focus, cowork.
    Start {
        /// Zellij layout to use (dev, focus, cowork)
        #[arg(long, value_enum, default_value = "dev")]
        layout: Layout,
    },
    /// Stop the container
    Stop,
    /// Stop and remove the container
    ///
    /// Unlike `stop`, this removes the container entirely (like
    /// `docker rm`). Use before switching to VS Code or when you
    /// want a clean slate.
    #[command(alias = "rm")]
    Remove,
    /// Show container status
    Status,
    /// Validate context structure and produce migration artifacts
    ///
    /// Checks: config validity, container runtime, .aibox-home/ directories,
    /// .devcontainer/ files, context structure, .gitignore entries, and
    /// schema version. Generates migration artifacts when versions differ.
    Doctor,
    /// Generate shell completion script
    ///
    /// Example: aibox completions bash > ~/.bash_completion.d/aibox
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Check for or apply version updates
    ///
    /// Checks the latest CLI version on GitHub and the latest image
    /// version on GHCR for your configured image flavor.
    ///
    /// Without flags: upgrades image version in aibox.toml and regenerates
    /// container files. Use --check for a dry check, --dry-run to preview changes.
    Update {
        /// Only check versions, don't apply any changes
        #[arg(long)]
        check: bool,
        /// Preview what would change without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage named environments for switching between configurations
    ///
    /// Environments save aibox.toml, CLAUDE.md, and context/ (excluding
    /// context/shared/) to .aibox-env/<name>/. Switch between them to
    /// use different images, processes, and context within one project.
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },
    /// Back up aibox files to a timestamped directory
    ///
    /// Copies aibox.toml, .devcontainer/, .aibox-home/, context/,
    /// CLAUDE.md, .aibox-version, and .gitignore to a backup directory.
    Backup {
        /// Output directory for backup (default: .aibox-backup/)
        #[arg(long)]
        output_dir: Option<String>,
        /// Preview what would be backed up without copying
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove all aibox files and reset project to pre-init state
    ///
    /// DANGER ZONE: Deletes aibox.toml, .devcontainer/, .aibox-home/,
    /// context/, CLAUDE.md, and .aibox-version. Backs up first by default.
    /// .gitignore is backed up but NOT deleted.
    ///
    /// Stops any running container before deleting.
    Reset {
        /// Skip backup — permanently delete without saving
        #[arg(long)]
        no_backup: bool,
        /// Preview what would happen without modifying anything
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Run security checks on the project
    ///
    /// Checks Rust dependencies (cargo audit), Python dependencies
    /// (pip-audit), and container images (trivy) if the tools are available.
    Audit,
    /// Host-side audio diagnostics and setup for PulseAudio
    ///
    /// Manages PulseAudio configuration on the host machine for
    /// container audio support (e.g., Claude Code voice).
    Audio {
        #[command(subcommand)]
        action: AudioAction,
    },
    /// Manage add-ons (language runtimes, tools, AI agents)
    ///
    /// Browse, add, or remove add-ons that install tool sets into
    /// the container. Changes are written to aibox.toml and applied
    /// via sync.
    Addon {
        #[command(subcommand)]
        action: AddonAction,
    },
    /// Manage skills (AI agent capabilities)
    ///
    /// Browse, add, or remove skills that define what AI agents can
    /// do in this project. Skills are deployed to .claude/skills/.
    /// Changes are written to aibox.toml [skills] section.
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

#[derive(Subcommand)]
pub enum AddonAction {
    /// List all available add-ons and their install status
    List,
    /// Add an add-on to aibox.toml and sync
    ///
    /// Inserts the add-on with default-enabled tools into aibox.toml,
    /// then runs a full sync to regenerate container files.
    Add {
        /// Add-on name (e.g., python, rust, node, ai-claude)
        name: String,
    },
    /// Remove an add-on from aibox.toml and sync
    Remove {
        /// Add-on name to remove
        name: String,
    },
    /// Show detailed info about an add-on
    ///
    /// Displays available tools, supported versions, and defaults.
    Info {
        /// Add-on name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// List all available skills and their deploy status
    ///
    /// Shows skills from the registry, grouped by which process
    /// package provides them. Marks deployed vs available.
    List,
    /// Add a skill to [skills].include in aibox.toml
    ///
    /// Adds the skill to the include list and runs skill reconciliation.
    Add {
        /// Skill name (e.g., code-review, data-science)
        name: String,
    },
    /// Remove a skill by adding it to [skills].exclude
    ///
    /// If the skill was in [skills].include, removes it from there.
    /// Otherwise adds it to [skills].exclude.
    Remove {
        /// Skill name to exclude
        name: String,
    },
    /// Show info about a skill
    Info {
        /// Skill name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum AudioAction {
    /// Check if host audio is correctly configured
    ///
    /// Runs diagnostics: PulseAudio installation, daemon status,
    /// TCP module, persistence, port listening, and connectivity.
    Check {
        /// PulseAudio TCP port (default: 4714)
        #[arg(long, default_value = "4714")]
        port: Option<u16>,
    },
    /// Install and configure PulseAudio on the host (macOS)
    ///
    /// Installs PulseAudio via Homebrew, configures the TCP module,
    /// creates a launchd plist for auto-start with KeepAlive, and
    /// verifies the setup.
    Setup {
        /// PulseAudio TCP port (default: 4714)
        #[arg(long, default_value = "4714")]
        port: Option<u16>,
    },
}

#[derive(Subcommand)]
pub enum EnvAction {
    /// Save current project state as a named environment
    ///
    /// Copies aibox.toml, CLAUDE.md, and context/ (excluding context/shared/)
    /// to .aibox-env/<name>/.
    Create {
        /// Environment name (alphanumeric, hyphens, underscores)
        name: String,
    },
    /// Switch to a different environment
    ///
    /// Saves the current environment, restores the target, and regenerates
    /// .devcontainer/ files. Stops any running container first.
    Switch {
        /// Environment to switch to
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// List available environments
    List,
    /// Delete a saved environment
    Delete {
        /// Environment to delete
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Show current environment info
    Status,
}
