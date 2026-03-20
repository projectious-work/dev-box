use clap::{Parser, Subcommand};

use crate::config::{AiProvider, ImageFlavor, ProcessFlavor};

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
  dev-box generate                             Regenerate files from dev-box.toml
  dev-box build                                Build the container image
  dev-box start                                Start and attach to container
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
    },
    /// Re-generate devcontainer files from dev-box.toml
    ///
    /// Regenerates .devcontainer/Dockerfile, docker-compose.yml, and
    /// devcontainer.json from the current dev-box.toml configuration.
    /// Does not touch context files or .dev-box-home/.
    Generate,
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
    Start,
    /// Stop the container
    Stop,
    /// Attach to running container
    Attach,
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
