use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dev-box", about = "Manage AI-ready development container environments")]
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
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// Container image flavor
        #[arg(long)]
        image: Option<String>,
        /// Work process flavor
        #[arg(long)]
        process: Option<String>,
    },
    /// Re-generate devcontainer files from dev-box.toml
    Generate,
    /// Build the container image
    Build {
        /// Build without cache
        #[arg(long)]
        no_cache: bool,
    },
    /// Start container and attach via zellij
    Start,
    /// Stop the container
    Stop,
    /// Attach to running container
    Attach,
    /// Show container status
    Status,
    /// Validate context structure and produce migration artifacts
    Doctor,
    /// Check for or apply version updates
    Update {
        /// Only check, don't apply
        #[arg(long)]
        check: bool,
    },
}
