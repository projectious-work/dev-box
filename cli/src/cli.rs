use clap::{Parser, Subcommand, ValueEnum};

use crate::config::{AddonBundle, AiProvider, ImageFlavor, ProcessFlavor, Theme};

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
    name = "dev-box",
    about = "Manage AI-ready development container environments",
    long_about = "\
dev-box — manage AI-ready development container environments

dev-box creates reproducible, containerized development environments with
built-in AI context structure and work process management.

Examples:
  dev-box init                                 Interactive project setup
  dev-box init --name my-app --image python --process product
  dev-box init --image rust --process minimal  Rust project, minimal context
  dev-box sync                                 Apply config changes (theme, etc.)
  dev-box build                                Build the container image
  dev-box start                                Start and attach (dev layout)
  dev-box start --layout focus                 Start with focus layout
  dev-box doctor                               Validate project structure
  dev-box update --check                       Check for newer versions
  dev-box audio check                          Diagnose host audio setup",
    version
)]
pub struct Cli {
    /// Path to dev-box.toml (default: ./dev-box.toml)
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, env = "DEV_BOX_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project with dev-box.toml and generated files
    ///
    /// Creates dev-box.toml, generates .devcontainer/ files, scaffolds
    /// context directory, seeds .dev-box-home/ with default configs, and
    /// sets up .gitignore.
    ///
    /// Without flags, runs interactively. With all flags, runs non-interactively.
    Init {
        /// Project name (default: current directory name)
        #[arg(long)]
        name: Option<String>,

        /// Container image flavor
        #[arg(long, value_enum)]
        image: Option<ImageFlavor>,

        /// Work process flavor (default: product)
        #[arg(long, value_enum)]
        process: Option<ProcessFlavor>,

        /// AI tool providers to configure (default: claude)
        #[arg(long, value_enum, num_args = 1..)]
        ai: Option<Vec<AiProvider>>,

        /// Container user (default: root)
        #[arg(long)]
        user: Option<String>,

        /// Color theme for all tools (default: gruvbox-dark)
        #[arg(long, value_enum)]
        theme: Option<Theme>,

        /// Addon bundles to install (e.g., infrastructure, kubernetes, cloud-aws)
        #[arg(long, value_enum, num_args = 1..)]
        addons: Option<Vec<AddonBundle>>,
    },
    /// Reconcile project state with dev-box.toml configuration
    ///
    /// Re-seeds config files that depend on settings (theme, AI providers),
    /// regenerates .devcontainer/ files, and updates .dev-box-home/ configs.
    /// The primary command for applying config changes.
    #[command(alias = "generate")]
    Sync,
    /// Build the container image
    Build {
        /// Build without cache
        #[arg(long)]
        no_cache: bool,
    },
    /// Start container and attach via zellij
    ///
    /// Seeds .dev-box-home/ if needed, generates devcontainer files,
    /// creates/starts the container, then attaches via zellij.
    ///
    /// Available layouts: dev (default), focus, cowork.
    Start {
        /// Zellij layout to use (dev, focus, assist)
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
    /// Attach to running container
    ///
    /// Available layouts: dev (default), focus, cowork.
    Attach {
        /// Zellij layout to use (dev, focus, assist)
        #[arg(long, value_enum, default_value = "dev")]
        layout: Layout,
    },
    /// Show container status
    Status,
    /// Validate context structure and produce migration artifacts
    ///
    /// Checks: config validity, container runtime, .dev-box-home/ directories,
    /// .devcontainer/ files, context structure, .gitignore entries, and
    /// schema version. Generates migration artifacts when versions differ.
    Doctor,
    /// Generate shell completion script
    ///
    /// Example: dev-box completions bash > ~/.bash_completion.d/dev-box
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
    /// Without flags: upgrades image version in dev-box.toml and regenerates
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
    /// Environments save dev-box.toml, CLAUDE.md, and context/ (excluding
    /// context/shared/) to .dev-box-env/<name>/. Switch between them to
    /// use different images, processes, and context within one project.
    Env {
        #[command(subcommand)]
        action: EnvAction,
    },
    /// Back up dev-box files to a timestamped directory
    ///
    /// Copies dev-box.toml, .devcontainer/, .dev-box-home/, context/,
    /// CLAUDE.md, .dev-box-version, and .gitignore to a backup directory.
    Backup {
        /// Output directory for backup (default: .dev-box-backup/)
        #[arg(long)]
        output_dir: Option<String>,
        /// Preview what would be backed up without copying
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove all dev-box files and reset project to pre-init state
    ///
    /// DANGER ZONE: Deletes dev-box.toml, .devcontainer/, .dev-box-home/,
    /// context/, CLAUDE.md, and .dev-box-version. Backs up first by default.
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
    /// Copies dev-box.toml, CLAUDE.md, and context/ (excluding context/shared/)
    /// to .dev-box-env/<name>/.
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
