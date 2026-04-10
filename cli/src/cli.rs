use clap::{Parser, Subcommand, ValueEnum};

use crate::config::{AiProvider, BaseImage, StarshipPreset, Theme};

/// Output format for list commands.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table (default)
    #[default]
    Table,
    /// JSON array
    Json,
    /// YAML sequence
    Yaml,
}

/// Available Zellij IDE layouts.
#[derive(Clone, Debug, ValueEnum)]
pub enum Layout {
    /// VS Code-like: Yazi sidebar, Vim editor, stacked terminals
    Dev,
    /// One tool per tab, fullscreen, zero distraction
    Focus,
    /// Side-by-side coding with AI: yazi+vim left (50%), claude right (50%)
    Cowork,
    /// Cowork swapped: yazi+ai left (40%), vim editor right (60%)
    CoworkSwap,
    /// Yazi-focused with large preview and AI pane
    Browse,
    /// AI-first: Yazi left (60%), AI agent right (40%), no editor on first screen
    Ai,
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Dev => write!(f, "dev"),
            Layout::Focus => write!(f, "focus"),
            Layout::Cowork => write!(f, "cowork"),
            Layout::CoworkSwap => write!(f, "cowork-swap"),
            Layout::Browse => write!(f, "browse"),
            Layout::Ai => write!(f, "ai"),
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
  aibox start --layout focus                 Start with a specific layout
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

    /// Skip all confirmation prompts (like apt-get -y)
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

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

        /// Addon names to enable (e.g., python, infrastructure, kubernetes).
        /// Each selected addon's `requires` are auto-added transitively
        /// (e.g. selecting `docs-docusaurus` also pulls in `node`).
        #[arg(long, num_args = 1..)]
        addons: Option<Vec<String>>,

        /// Pin a specific tool version inside an addon. Repeatable.
        /// Format: `addon:tool=version`. Examples:
        ///
        ///   --addon-tool python:python=3.14 --addon-tool node:pnpm=10
        ///
        /// Overrides the addon's default version and skips the
        /// interactive version picker for that tool.
        #[arg(long = "addon-tool")]
        addon_tool: Vec<String>,

        /// processkit source URL (default: projectious-work/processkit upstream).
        /// Use this to point at a fork or a compatible alternative repo.
        #[arg(long)]
        processkit_source: Option<String>,

        /// processkit version tag to pin. If omitted, aibox lists the
        /// available versions at the source and (interactively) lets you
        /// pick one or (non-interactively) defaults to the latest.
        #[arg(long)]
        processkit_version: Option<String>,

        /// processkit branch override. Tracks the moving HEAD of a branch
        /// instead of a pinned tag — discouraged for production use, fine
        /// for testing pre-release work. Mutually informative with
        /// `--processkit-version`: when both are set the branch wins at
        /// fetch time but the version is still recorded in aibox.toml.
        #[arg(long)]
        processkit_branch: Option<String>,
    },
    /// Reconcile project state with aibox.toml configuration
    ///
    /// Seeds config files, regenerates .devcontainer/ files, runs the
    /// processkit content diff, and builds the container image. The
    /// primary command for applying any config change.
    ///
    /// Sync perimeter (files aibox sync may create, modify, or delete):
    ///   - aibox.toml                          (one-time schema migrations)
    ///   - aibox.lock                          (CLI version + processkit pin)
    ///   - .aibox-home/**                      (runtime config seed; gitignored)
    ///   - .devcontainer/Dockerfile            (regenerated)
    ///   - .devcontainer/docker-compose.yml    (regenerated)
    ///   - .devcontainer/devcontainer.json     (regenerated)
    ///   - context/migrations/**               (additive migration documents)
    ///
    /// Anything else (README.md, AGENTS.md, CLAUDE.md, src/, tests/,
    /// context/BACKLOG.md, context/skills/, etc.) is OUT of perimeter and
    /// will never be touched. Attempts to write outside the perimeter via
    /// aibox internals are blocked at the source. See
    /// cli/src/sync_perimeter.rs.
    #[command(alias = "generate")]
    Sync {
        /// Build without cache (force full rebuild)
        #[arg(long)]
        no_cache: bool,

        /// Skip the container image build step (config-only sync)
        #[arg(long)]
        no_build: bool,
    },
    /// Start container and attach via zellij
    ///
    /// Seeds .aibox-home/ if needed, generates devcontainer files,
    /// creates/starts the container, then attaches via zellij.
    /// If already running, just attaches.
    ///
    /// Available layouts: dev (default), focus, cowork, cowork-swap, browse, ai.
    Start {
        /// Zellij layout to use (dev, focus, cowork, cowork-swap, browse, ai)
        #[arg(long, value_enum)]
        layout: Option<Layout>,
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
    Status {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
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
    /// Copies aibox.toml, aibox.lock, .devcontainer/, .aibox-home/, context/,
    /// CLAUDE.md, and .gitignore to a backup directory.
    Backup {
        /// Output directory for backup (default: .aibox/backup/)
        #[arg(long)]
        output_dir: Option<String>,
        /// Preview what would be backed up without copying
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove all aibox files and reset project to pre-init state
    ///
    /// DANGER ZONE: Deletes aibox.toml, aibox.lock, .devcontainer/, .aibox-home/,
    /// context/, and CLAUDE.md. Backs up first by default.
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
        /// Skip the confirmation prompt (alias: --force)
        #[arg(long, visible_alias = "force")]
        yes: bool,
    },
    /// Uninstall the aibox CLI binary
    ///
    /// DANGER ZONE: Removes the aibox binary from its install location.
    /// Global config (~/.aibox/) is kept by default. Use --purge to
    /// also remove it. Does NOT touch project files — use `reset` for that.
    Uninstall {
        /// Preview what would be removed without deleting anything
        #[arg(long)]
        dry_run: bool,
        /// Also remove global config and cache (~/.aibox/)
        #[arg(long)]
        purge: bool,
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
    /// Manage processkit migration documents
    ///
    /// Migration documents are generated by `aibox sync` when the
    /// processkit version changes. They live under
    /// `context/migrations/{pending,in-progress,applied}/` and move
    /// between those subdirectories as you work through them.
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
    /// Query and manage processkit content (skills, processes, schemas)
    ///
    /// Inspect what processkit content is installed in this project
    /// under `context/`. Skills can be selectively installed or
    /// uninstalled via `[skills].include` / `[skills].exclude` in
    /// aibox.toml.
    ///
    /// Examples:
    ///   aibox kit list                       Summary of installed content
    ///   aibox kit skill list                 All installed skills by category
    ///   aibox kit skill list --all           All available skills with status
    ///   aibox kit skill list --category ai   Filter by category
    ///   aibox kit skill categories           Category summary
    ///   aibox kit skill info python-best-practices
    ///   aibox kit skill install python-best-practices
    ///   aibox kit skill uninstall pandas-polars
    ///   aibox kit process list               List installed processes
    Kit {
        #[command(subcommand)]
        action: KitAction,
    },
}

#[derive(Subcommand)]
pub enum MigrateAction {
    /// Show pending and in-progress migrations and suggest the next one
    ///
    /// Read-only: does not change any files. Refreshes
    /// `context/migrations/INDEX.md` and prints the suggested next
    /// migration's briefing.
    Continue,
    /// Start working on a pending migration (transitions pending → in-progress)
    Start {
        /// Migration ID (e.g. MIG-bright-owl)
        id: String,
    },
    /// Mark an in-progress (or pending) migration as applied
    Apply {
        /// Migration ID (e.g. MIG-bright-owl)
        id: String,
    },
    /// Reject a pending or in-progress migration with a reason
    ///
    /// The rejection reason is written into the document's frontmatter
    /// under `spec.rejection_reason` and the file is moved to
    /// `context/migrations/applied/` (the terminal home for rejected
    /// migrations).
    Reject {
        /// Migration ID (e.g. MIG-bright-owl)
        id: String,
        /// Reason for rejection
        #[arg(long)]
        reason: String,
    },
}

#[derive(Subcommand)]
pub enum AddonAction {
    /// List all available add-ons and their install status
    #[command(alias = "ls")]
    List {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Add an add-on to aibox.toml and sync
    ///
    /// Inserts the add-on with default-enabled tools into aibox.toml,
    /// then runs a full sync to regenerate container files.
    Add {
        /// Add-on name (e.g., python, rust, node, ai-claude)
        name: String,

        /// Skip the container image build step after sync
        #[arg(long)]
        no_build: bool,
    },
    /// Remove an add-on from aibox.toml and sync
    Remove {
        /// Add-on name to remove
        name: String,

        /// Skip the container image build step after sync
        #[arg(long)]
        no_build: bool,
    },
    /// Show detailed info about an add-on
    ///
    /// Displays available tools, supported versions, and defaults.
    Info {
        /// Add-on name
        name: String,
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
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
    #[command(alias = "ls")]
    List {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
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

// ---------------------------------------------------------------------------
// aibox kit subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum KitAction {
    /// Show a summary of installed processkit content
    ///
    /// Counts of installed skills, processes, schemas, and state machines.
    List {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Manage and inspect processkit skills
    Skill {
        #[command(subcommand)]
        action: KitSkillAction,
    },
    /// Inspect processkit processes
    Process {
        #[command(subcommand)]
        action: KitProcessAction,
    },
}

#[derive(Subcommand)]
pub enum KitSkillAction {
    /// List processkit skills installed in this project
    ///
    /// Without --all: shows skills present in context/skills/.
    /// With --all: shows all available skills from the processkit templates
    /// mirror, with installed status.
    #[command(alias = "ls")]
    List {
        /// Show all available skills (requires processkit templates mirror)
        #[arg(long)]
        all: bool,
        /// Filter by category (e.g. language, ai, process, security)
        #[arg(long)]
        category: Option<String>,
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Show skill categories with counts
    Categories {
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Show details for a specific skill
    Info {
        /// Skill name (e.g. python-best-practices)
        name: String,
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Add a skill to the active set (modifies aibox.toml)
    ///
    /// Adjusts [skills].include / [skills].exclude so the skill is
    /// present on the next 'aibox sync'. Does not run sync automatically.
    Install {
        /// Skill name
        name: String,
    },
    /// Remove a skill from the active set (modifies aibox.toml)
    ///
    /// Adjusts [skills].include / [skills].exclude so the skill is
    /// excluded on the next 'aibox sync'. Does not run sync automatically.
    Uninstall {
        /// Skill name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum KitProcessAction {
    /// List processkit processes installed in this project
    #[command(alias = "ls")]
    List {
        /// Show all available processes (requires processkit templates mirror)
        #[arg(long)]
        all: bool,
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Show details for a specific process
    Info {
        /// Process name (filename without .md)
        name: String,
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value = "table")]
        format: OutputFormat,
    },
}
