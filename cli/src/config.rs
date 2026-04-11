use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// ExtraVolume — user-defined bind mount
// ---------------------------------------------------------------------------

/// A user-defined bind mount entry, from `[[container.extra_volumes]]` in
/// `aibox.toml` or `.aibox-local.toml`.
///
/// In TOML:
/// ```toml
/// [[container.extra_volumes]]
/// source = "~/.config/gh"
/// target = "/home/aibox/.config/gh"
///
/// [[container.extra_volumes]]
/// source = "~/.aws"
/// target = "/home/aibox/.aws"
/// read_only = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtraVolume {
    /// Host-side path. Supports `~` expansion by Docker Compose. No `..` allowed.
    pub source: String,
    /// Container-side absolute path. Must start with `/`. No `..` allowed.
    pub target: String,
    /// Mount read-only. Defaults to false.
    #[serde(default)]
    pub read_only: bool,
}

/// Container image registry base URL.
pub const IMAGE_REGISTRY: &str = "ghcr.io/projectious-work/aibox";

/// Standard devcontainer directory name.
pub const DEVCONTAINER_DIR: &str = ".devcontainer";
/// Standard compose file name within devcontainer dir.
pub const COMPOSE_FILE: &str = ".devcontainer/docker-compose.yml";
/// Standard Dockerfile name within devcontainer dir.
pub const DOCKERFILE: &str = ".devcontainer/Dockerfile";
/// Standard devcontainer.json name.
pub const DEVCONTAINER_JSON: &str = ".devcontainer/devcontainer.json";

// ---------------------------------------------------------------------------
// Base image
// ---------------------------------------------------------------------------

/// Base image for the aibox container. Currently only Debian is supported;
/// Alpine is planned for later.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum BaseImage {
    #[default]
    Debian,
    // Alpine, // planned
}

impl std::fmt::Display for BaseImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseImage::Debian => write!(f, "debian"),
        }
    }
}

// ---------------------------------------------------------------------------
// [aibox] section
// ---------------------------------------------------------------------------

/// Top-level [aibox] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiboxSection {
    pub version: String,
    #[serde(default)]
    pub base: BaseImage,
}

// ---------------------------------------------------------------------------
// [container] section — UNCHANGED
// ---------------------------------------------------------------------------

/// [container] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSection {
    pub name: String,
    #[serde(default = "default_hostname")]
    pub hostname: String,
    /// Container user (default: "aibox"). Determines mount paths inside container.
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default)]
    pub post_create_command: Option<String>,
    /// Network keepalive — prevents OrbStack/VM NAT from dropping idle connections.
    #[serde(default)]
    pub keepalive: bool,
    /// Extra environment variables injected into the container.
    /// Committed entries go in `aibox.toml`; secrets go in `.aibox-local.toml`.
    ///
    /// ```toml
    /// [container.environment]
    /// AWS_DEFAULT_REGION = "eu-west-1"
    /// ```
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Additional bind mounts beyond the aibox defaults.
    /// Committed entries (shared caches) go in `aibox.toml`; personal credential
    /// directories go in `.aibox-local.toml` (gitignored).
    ///
    /// ```toml
    /// [[container.extra_volumes]]
    /// source = "~/.config/gh"
    /// target = "/home/aibox/.config/gh"
    /// ```
    #[serde(default)]
    pub extra_volumes: Vec<ExtraVolume>,
}

fn default_user() -> String {
    "aibox".to_string()
}

fn default_hostname() -> String {
    "aibox".to_string()
}

// ---------------------------------------------------------------------------
// [context] section — merged with former [process]
// ---------------------------------------------------------------------------

/// [context] section — context system versioning and process packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSection {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default = "default_context_packages")]
    pub packages: Vec<String>,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

fn default_context_packages() -> Vec<String> {
    vec!["managed".to_string()]
}

impl Default for ContextSection {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            packages: default_context_packages(),
        }
    }
}

/// Legacy [process] section for backward compatibility.
/// If present, packages are merged into [context].packages during load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyProcessSection {
    #[serde(default)]
    pub packages: Vec<String>,
}

// ---------------------------------------------------------------------------
// [ai] section
// ---------------------------------------------------------------------------

/// AI tool providers supported in aibox containers.
///
/// MCP-capable providers (have a built-in MCP client and a project-level
/// config file aibox can write to):
/// - `Claude` — `.mcp.json` at project root
/// - `Cursor` — `.cursor/mcp.json` at project root (host-side IDE only)
/// - `Gemini` — `.gemini/settings.json` (Gemini CLI)
/// - `Codex` — `.codex/config.toml` (OpenAI Codex CLI, binary: `codex`)
/// - `Continue` — `.continue/mcpServers/<name>.json` (Continue CLI, binary: `cn`)
/// - `Copilot` — `.mcp.json` at project root (GitHub Copilot CLI, binary: `copilot`)
///
/// Special MCP routing:
/// - `Mistral` — has MCP client capability via Python SDK and Le Chat,
///   but no local file-based config. aibox writes `.mcp.json` (the
///   Claude shape) when Mistral is selected so a custom Mistral
///   SDK-based CLI tool can read MCP server registrations from there.
///
/// Non-MCP providers (no built-in MCP client; aibox cannot register
/// processkit MCP servers; sync emits a warning):
/// - `Aider` — no native MCP client. Third-party experimental bridges
///   exist but are not yet stable.
///
/// Note: `Cursor` is a host-side IDE extension only — it has no container
/// CLI binary and no in-container persistence directory. All other providers
/// have a corresponding `ai-<name>` addon that installs their CLI in the image.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum AiProvider {
    Claude,
    Aider,
    Gemini,
    Mistral,
    Cursor,
    Codex,
    Continue,
    Copilot,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Claude => write!(f, "claude"),
            AiProvider::Aider => write!(f, "aider"),
            AiProvider::Gemini => write!(f, "gemini"),
            AiProvider::Mistral => write!(f, "mistral"),
            AiProvider::Cursor => write!(f, "cursor"),
            AiProvider::Codex => write!(f, "codex"),
            AiProvider::Continue => write!(f, "continue"),
            AiProvider::Copilot => write!(f, "copilot"),
        }
    }
}

impl AiProvider {
    /// Returns the actual CLI binary name for this provider.
    ///
    /// This differs from `Display` for providers where the display name and
    /// binary name diverge (e.g. `Continue` displays as `"continue"` but the
    /// binary is `cn`). Use this wherever a shell command is needed.
    pub fn binary_name(&self) -> &'static str {
        match self {
            AiProvider::Claude => "claude",
            AiProvider::Aider => "aider",
            AiProvider::Gemini => "gemini",
            AiProvider::Mistral => "mistral",
            AiProvider::Cursor => "cursor",
            AiProvider::Codex => "codex",
            AiProvider::Continue => "cn",
            AiProvider::Copilot => "copilot",
        }
    }
}

fn default_ai_tools() -> Vec<AiProvider> {
    vec![AiProvider::Claude]
}

/// [ai] section — AI tool provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSection {
    #[serde(default = "default_ai_tools")]
    pub providers: Vec<AiProvider>,
}

impl Default for AiSection {
    fn default() -> Self {
        Self {
            providers: default_ai_tools(),
        }
    }
}

// ---------------------------------------------------------------------------
// [addons] section — REWRITTEN
// ---------------------------------------------------------------------------

/// Configuration for a single tool within an addon.
///
/// In TOML this appears as e.g. `python = { version = "3.13" }` or `clippy = {}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolEntry {
    /// Tool version. `None` when no version is specified (e.g. `clippy = {}`).
    pub version: Option<String>,
}

/// The `tools` sub-table of an addon, e.g. `[addons.python.tools]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddonToolsSection {
    #[serde(default)]
    pub tools: HashMap<String, ToolEntry>,
}

/// [addons] section — each key is an addon name mapping to its tools table.
///
/// In TOML:
/// ```toml
/// [addons.python.tools]
/// python = { version = "3.13" }
/// uv = { version = "0.7" }
/// ```
///
/// Deserialized as `HashMap<String, AddonToolsSection>` where the outer key
/// is the addon name (e.g. "python") and the inner map contains tool entries.
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct AddonsSection {
    pub addons: HashMap<String, AddonToolsSection>,
}

// Custom deserialization: the TOML section `[addons]` is a table where each
// key is an addon name and each value is an `AddonToolsSection`. Serde by
// default would look for a field called `addons` inside the `[addons]` table,
// but in our TOML the addon names ARE the keys of the `[addons]` table. We
// use `deserialize_with` at the AiboxConfig level via a transparent wrapper.
impl<'de> Deserialize<'de> for AddonsSection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let addons = HashMap::<String, AddonToolsSection>::deserialize(deserializer)?;
        Ok(AddonsSection { addons })
    }
}

#[allow(dead_code)]
impl AddonsSection {
    /// Check whether a specific addon is configured.
    pub fn has_addon(&self, name: &str) -> bool {
        self.addons.contains_key(name)
    }

    /// Get the tools section for a specific addon, if present.
    pub fn get_addon(&self, name: &str) -> Option<&AddonToolsSection> {
        self.addons.get(name)
    }

    /// Iterate over all configured addons.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AddonToolsSection)> {
        self.addons.iter()
    }

    /// Check whether a specific addon contains a specific tool.
    pub fn has_tool(&self, addon: &str, tool: &str) -> bool {
        self.addons
            .get(addon)
            .is_some_and(|a| a.tools.contains_key(tool))
    }

    /// Get the version of a specific tool in an addon, if configured.
    pub fn tool_version(&self, addon: &str, tool: &str) -> Option<&str> {
        self.addons
            .get(addon)
            .and_then(|a| a.tools.get(tool))
            .and_then(|t| t.version.as_deref())
    }

    /// Convenience: check if the python addon is configured.
    pub fn has_python(&self) -> bool {
        self.has_addon("python")
    }

    /// Convenience: check if the rust addon is configured.
    pub fn has_rust(&self) -> bool {
        self.has_addon("rust")
    }

    /// Convenience: check if the node addon is configured.
    pub fn has_node(&self) -> bool {
        self.has_addon("node")
    }

    /// Convenience: check if the latex addon is configured.
    pub fn has_latex(&self) -> bool {
        self.has_addon("latex")
    }
}

// ---------------------------------------------------------------------------
// [skills] section — NEW
// ---------------------------------------------------------------------------

/// [skills] section — include/exclude overrides for skill deployment.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillsSection {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

// ---------------------------------------------------------------------------
// [appearance] section — UNCHANGED
// ---------------------------------------------------------------------------

/// Color themes available across all tools (Zellij, Vim, Yazi, lazygit).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    GruvboxDark,
    CatppuccinMocha,
    CatppuccinLatte,
    Dracula,
    TokyoNight,
    Nord,
    Projectious,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::GruvboxDark => write!(f, "gruvbox-dark"),
            Theme::CatppuccinMocha => write!(f, "catppuccin-mocha"),
            Theme::CatppuccinLatte => write!(f, "catppuccin-latte"),
            Theme::Dracula => write!(f, "dracula"),
            Theme::TokyoNight => write!(f, "tokyo-night"),
            Theme::Nord => write!(f, "nord"),
            Theme::Projectious => write!(f, "projectious"),
        }
    }
}

fn default_theme() -> Theme {
    Theme::default()
}

/// Starship prompt presets.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum StarshipPreset {
    #[default]
    Default, // Clean, informative — dir, git, language, duration
    Plain,     // ASCII only — no Nerd Font needed
    Minimal,   // Just directory + git branch
    NerdFont,  // Full Nerd Font symbols
    Pastel,    // Soft powerline segments
    Bracketed, // [segments] in brackets
    Arrow,     // Powerline-style chevron/arrow segments (airline-style)
}

impl std::fmt::Display for StarshipPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StarshipPreset::Default => write!(f, "default"),
            StarshipPreset::Plain => write!(f, "plain"),
            StarshipPreset::Minimal => write!(f, "minimal"),
            StarshipPreset::NerdFont => write!(f, "nerd-font"),
            StarshipPreset::Pastel => write!(f, "pastel"),
            StarshipPreset::Bracketed => write!(f, "bracketed"),
            StarshipPreset::Arrow => write!(f, "arrow"),
        }
    }
}

fn default_prompt() -> StarshipPreset {
    StarshipPreset::default()
}

/// Default zellij layout for `aibox start`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum ConfigLayout {
    /// VS Code-like: Yazi sidebar, Vim editor, stacked terminals
    #[default]
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

impl std::fmt::Display for ConfigLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLayout::Dev => write!(f, "dev"),
            ConfigLayout::Focus => write!(f, "focus"),
            ConfigLayout::Cowork => write!(f, "cowork"),
            ConfigLayout::CoworkSwap => write!(f, "cowork-swap"),
            ConfigLayout::Browse => write!(f, "browse"),
            ConfigLayout::Ai => write!(f, "ai"),
        }
    }
}

fn default_layout() -> ConfigLayout {
    ConfigLayout::default()
}

/// [customization] section — color theme, shell prompt, and zellij layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizationSection {
    #[serde(default = "default_theme")]
    pub theme: Theme,
    #[serde(default = "default_prompt")]
    pub prompt: StarshipPreset,
    #[serde(default = "default_layout")]
    pub layout: ConfigLayout,
}

impl Default for CustomizationSection {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            prompt: default_prompt(),
            layout: default_layout(),
        }
    }
}

// ---------------------------------------------------------------------------
// [processkit] section — content layer source (skills, primitives, processes)
// ---------------------------------------------------------------------------

/// [processkit] section — configures the processkit-compatible source
/// the project consumes content from.
///
/// processkit ships skills and primitives that aibox installs into the
/// project. The default upstream is the canonical projectious-work/processkit
/// repo. Companies can fork processkit and have projects consume the fork by
/// changing `source` to point at their fork.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessKitSection {
    /// Git URL of the processkit-compatible source.
    #[serde(default = "default_processkit_source")]
    pub source: String,
    /// Semver tag of the processkit source to consume. The sentinel value
    /// `"unset"` means "no version pinned yet" — downstream code can detect
    /// this and skip processkit fetching until a real version is set.
    #[serde(default = "default_processkit_version")]
    pub version: String,
    /// Subdirectory within the source repo containing the processkit content.
    #[serde(default = "default_processkit_src_path")]
    pub src_path: String,
    /// Optional branch name. If set, tracks a moving branch instead of a
    /// pinned tag (discouraged but supported).
    #[serde(default)]
    pub branch: Option<String>,
    /// URL template for the release-asset tarball, with `{source}`,
    /// `{version}`, `{org}`, and `{name}` placeholders. When unset, the
    /// fetcher uses the GitHub-style default
    /// `{source}/releases/download/{version}/{name}-{version}.tar.gz`.
    /// Set this to point at non-GitHub hosts (Gitea, GitLab, self-hosted)
    /// that serve release assets at a different URL shape.
    #[serde(default)]
    pub release_asset_url_template: Option<String>,
}

fn default_processkit_source() -> String {
    crate::processkit_vocab::PROCESSKIT_GIT_SOURCE.to_string()
}

fn default_processkit_version() -> String {
    "unset".to_string()
}

fn default_processkit_src_path() -> String {
    "src".to_string()
}

/// Sentinel version value meaning "no processkit version pinned yet".
pub const PROCESSKIT_VERSION_UNSET: &str = "unset";
/// Sentinel value meaning "resolve to the latest available tag at sync time".
pub const PROCESSKIT_VERSION_LATEST: &str = "latest";

// ---------------------------------------------------------------------------
// .aibox-local.toml — gitignored personal overlay
// ---------------------------------------------------------------------------

/// Personal, gitignored overlay that merges on top of `aibox.toml`.
///
/// Only a subset of the config is overridable locally — specifically the fields
/// that vary per developer (credentials, personal mount paths). Shared settings
/// like container name, aibox version, and addon list stay in `aibox.toml`.
///
/// Location: `.aibox-local.toml` in the project root (same dir as `aibox.toml`).
/// This file is added to `.gitignore` by `aibox init` / `aibox sync`.
///
/// Example `.aibox-local.toml`:
/// ```toml
/// [container.environment]
/// GH_TOKEN            = "ghp_..."
/// ANTHROPIC_API_KEY   = "sk-ant-..."
///
/// [[container.extra_volumes]]
/// source = "~/.config/gh"
/// target = "/home/aibox/.config/gh"
///
/// [[container.extra_volumes]]
/// source = "~/.aws"
/// target = "/home/aibox/.aws"
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AiboxLocalConfig {
    #[serde(default)]
    pub container: LocalContainerSection,
    /// Personal MCP servers — never committed to git.
    #[serde(default)]
    pub mcp: McpSection,
}

/// The `[container]` sub-section of `.aibox-local.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocalContainerSection {
    /// Environment variables to inject — merged with (and override) `aibox.toml` entries.
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Additional bind mounts — appended after `aibox.toml` extra_volumes.
    #[serde(default)]
    pub extra_volumes: Vec<ExtraVolume>,
}

/// One extra MCP server entry defined in `aibox.toml` (team-shared) or
/// `.aibox-local.toml` (personal). Supplements the processkit-managed
/// servers that aibox discovers from the installed skills.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtraMcpServer {
    /// Server name — used as the key in the `mcpServers` JSON object.
    pub name: String,
    /// Executable to spawn (e.g. `uv`, `npx`, `python3`).
    pub command: String,
    /// Arguments passed to `command`.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables injected into the server process.
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}

/// `[mcp]` section shared by `aibox.toml` (team-shared servers) and
/// `.aibox-local.toml` (personal servers). Same shape, different semantics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpSection {
    /// Extra MCP servers to register alongside processkit-managed ones.
    #[serde(default)]
    pub servers: Vec<ExtraMcpServer>,
}

impl Default for ProcessKitSection {
    fn default() -> Self {
        Self {
            source: default_processkit_source(),
            version: default_processkit_version(),
            src_path: default_processkit_src_path(),
            branch: None,
            release_asset_url_template: None,
        }
    }
}

// ---------------------------------------------------------------------------
// [agents] section — canonical AGENTS.md + provider pointer files
// ---------------------------------------------------------------------------

/// How aibox scaffolds provider-specific agent entry files (e.g.
/// `CLAUDE.md`) when both `AGENTS.md` and a provider file exist.
///
/// - `Pointer` (default): provider files are thin pointers that say
///   "see `AGENTS.md`". Canonical instructions live exclusively in
///   `AGENTS.md`. This is the recommended mode and matches the
///   `agents.md` ecosystem convention.
/// - `Full`: provider files contain the rich, provider-flavoured
///   content. Use only when a project genuinely needs different
///   instructions per harness.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentsProviderMode {
    #[default]
    Pointer,
    Full,
}

/// `[agents]` section — controls how aibox scaffolds the canonical
/// agent entry file (`AGENTS.md`) and the provider-specific pointer
/// files (`CLAUDE.md`, future `CODEX.md`, …).
///
/// The principle is provider neutrality: every agent harness reads the
/// same `AGENTS.md` so projects don't have to keep N versions of the
/// same instructions in sync. Provider files exist only to satisfy the
/// auto-load convention of specific harnesses (Claude Code auto-loads
/// `CLAUDE.md` at startup, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentsSection {
    /// Filename of the canonical agent entry document. Almost no one
    /// should override this — the default `AGENTS.md` matches the
    /// growing ecosystem convention at <https://agents.md/>.
    #[serde(default = "default_agents_canonical")]
    pub canonical: String,

    /// How provider-specific files are scaffolded. See [`AgentsProviderMode`].
    #[serde(default)]
    pub provider_mode: AgentsProviderMode,
}

fn default_agents_canonical() -> String {
    crate::processkit_vocab::AGENTS_FILENAME.to_string()
}

impl Default for AgentsSection {
    fn default() -> Self {
        Self {
            canonical: default_agents_canonical(),
            provider_mode: AgentsProviderMode::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [audio] section — UNCHANGED
// ---------------------------------------------------------------------------

/// [audio] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_pulse_server")]
    pub pulse_server: String,
}

fn default_pulse_server() -> String {
    "tcp:host.docker.internal:4714".to_string()
}

impl Default for AudioSection {
    fn default() -> Self {
        Self {
            enabled: false,
            pulse_server: default_pulse_server(),
        }
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Check that a string is a safe container/hostname identifier.
/// Must start with alphanumeric and contain only [a-zA-Z0-9._-].
fn is_safe_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// Check that an addon/tool/skill name uses only safe characters: [a-zA-Z0-9_-].
fn is_safe_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Check that a tool version string is safe: non-empty and contains only
/// alphanumeric characters or [.-_+].
fn is_safe_version(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

// ---------------------------------------------------------------------------
// Root config — AiboxConfig
// ---------------------------------------------------------------------------

/// Root config structure mapping aibox.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiboxConfig {
    #[serde(rename = "aibox")]
    pub aibox: AiboxSection,
    pub container: ContainerSection,
    #[serde(default)]
    pub context: ContextSection,
    #[serde(default)]
    pub ai: AiSection,
    #[serde(default)]
    pub addons: AddonsSection,
    #[serde(default)]
    pub skills: SkillsSection,
    #[serde(default)]
    pub processkit: ProcessKitSection,
    #[serde(default)]
    pub agents: AgentsSection,
    #[serde(default, alias = "appearance")]
    pub customization: CustomizationSection,
    #[serde(default)]
    pub audio: AudioSection,

    /// Legacy [process] section — if present, packages are merged into [context].
    #[serde(default, skip_serializing)]
    pub(crate) process: Option<LegacyProcessSection>,

    /// Team-shared custom MCP servers from `aibox.toml [mcp.servers]`.
    #[serde(default)]
    pub mcp: McpSection,

    /// Environment variables from `.aibox-local.toml` only — tracked separately
    /// so `generate.rs` can write them to `.aibox-local.env` rather than
    /// embedding literal credential values in `docker-compose.yml`.
    /// Not part of the TOML schema; populated programmatically at load time.
    #[serde(skip)]
    pub local_env: HashMap<String, String>,

    /// Personal MCP servers from `.aibox-local.toml [mcp.servers]` — tracked
    /// separately so they are never committed to git (same principle as
    /// `local_env` / `.aibox-local.env`). Not part of the TOML schema;
    /// populated programmatically at load time.
    #[serde(skip)]
    pub local_mcp_servers: Vec<ExtraMcpServer>,
}

impl AiboxConfig {
    /// Load configuration from a specific file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let mut config: AiboxConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        config.migrate_legacy_sections();
        config.resolve_ai_provider_addons();
        config.validate()?;
        Ok(config)
    }

    /// Migrate legacy [process] section into [context].packages.
    fn migrate_legacy_sections(&mut self) {
        if let Some(legacy) = self.process.take()
            && !legacy.packages.is_empty()
        {
            // Only override if context.packages is still at the default
            if self.context.packages == default_context_packages() {
                self.context.packages = legacy.packages;
            }
            crate::output::warn(
                "Deprecated: [process] section found in aibox.toml. \
                 Please move 'packages' into the [context] section.",
            );
        }
    }

    /// Load config from an optional CLI path argument.
    pub fn from_cli_option(config_path: &Option<String>) -> Result<Self> {
        match config_path {
            Some(path) => Self::load(&PathBuf::from(path)),
            None => Self::load_or_default(),
        }
    }

    /// Load from ./aibox.toml, then merge .aibox-local.toml if present.
    ///
    /// `.aibox-local.toml` is a gitignored personal overlay for per-developer
    /// settings (credentials, personal mount paths). Its `[container.environment]`
    /// entries are merged into the base config (local wins on key conflicts) and its
    /// `[[container.extra_volumes]]` entries are appended after the base ones.
    pub fn load_or_default() -> Result<Self> {
        let path = PathBuf::from("aibox.toml");
        if !path.exists() {
            bail!("No aibox.toml found in the current directory. Run 'aibox init' to create one.");
        }
        let mut config = Self::load(&path)?;

        // Merge .aibox-local.toml if present (gitignored personal overlay).
        let local_path = PathBuf::from(".aibox-local.toml");
        if local_path.exists() {
            let local_content =
                std::fs::read_to_string(&local_path).context("Failed to read .aibox-local.toml")?;
            let local: AiboxLocalConfig =
                toml::from_str(&local_content).context("Failed to parse .aibox-local.toml")?;
            // Capture local env vars before merging so generate.rs can write
            // them to .aibox-local.env instead of embedding them in compose.
            config.local_env = local.container.environment.clone();
            // Environment: local wins on key conflicts.
            config
                .container
                .environment
                .extend(local.container.environment);
            // Extra volumes: additive.
            config
                .container
                .extra_volumes
                .extend(local.container.extra_volumes);
            // Validate merged extra_volumes from both sources.
            config.validate_extra_volumes()?;
            // Personal MCP servers — stored separately so they never get
            // embedded in a committed file (same principle as local_env).
            config.local_mcp_servers = local.mcp.servers;
        }

        Ok(config)
    }

    /// Validate all `[[container.extra_volumes]]` entries for path safety.
    /// Called after merging `.aibox-local.toml` so both sources are covered.
    fn validate_extra_volumes(&self) -> Result<()> {
        for vol in &self.container.extra_volumes {
            if vol.source.is_empty() {
                bail!("container.extra_volumes entry has an empty 'source'");
            }
            if vol.target.is_empty() {
                bail!("container.extra_volumes entry has an empty 'target'");
            }
            if vol.source.contains("..") {
                bail!(
                    "container.extra_volumes source '{}' must not contain '..'",
                    vol.source
                );
            }
            if vol.target.contains("..") {
                bail!(
                    "container.extra_volumes target '{}' must not contain '..'",
                    vol.target
                );
            }
            if !vol.target.starts_with('/') {
                bail!(
                    "container.extra_volumes target '{}' must be an absolute path (start with '/')",
                    vol.target
                );
            }
        }
        Ok(())
    }

    /// Parse config from a TOML string. Useful for testing and programmatic use.
    #[allow(dead_code)]
    pub fn from_str(toml_str: &str) -> Result<Self> {
        let mut config: AiboxConfig =
            toml::from_str(toml_str).context("Failed to parse TOML config")?;
        config.migrate_legacy_sections();
        config.resolve_ai_provider_addons();
        config.validate()?;
        Ok(config)
    }

    /// Validate the config values. Called internally by `load`, but also
    /// available for validating programmatically-constructed configs.
    pub fn validate(&self) -> Result<()> {
        // Validate version is valid semver (allow "latest" sentinel)
        if self.aibox.version != "latest" {
            semver::Version::parse(&self.aibox.version).with_context(|| {
                format!(
                    "Invalid version '{}': must be valid semver",
                    self.aibox.version
                )
            })?;
        }

        // Validate schema_version is valid semver
        semver::Version::parse(&self.context.schema_version).with_context(|| {
            format!(
                "Invalid schema_version '{}': must be valid semver",
                self.context.schema_version
            )
        })?;

        // Validate container name is non-empty and safe
        if self.container.name.is_empty() {
            bail!("container.name must not be empty");
        }
        if !is_safe_identifier(&self.container.name) {
            bail!(
                "container.name '{}' contains invalid characters. \
                 Must start with alphanumeric and contain only [a-zA-Z0-9._-]",
                self.container.name
            );
        }
        if !self.container.hostname.is_empty() && !is_safe_identifier(&self.container.hostname) {
            bail!(
                "container.hostname '{}' contains invalid characters. \
                 Must start with alphanumeric and contain only [a-zA-Z0-9._-]",
                self.container.hostname
            );
        }

        // Validate context packages have safe names
        if self.context.packages.is_empty() {
            bail!("context.packages must not be empty (at minimum ['core'] is required)");
        }
        for pkg in &self.context.packages {
            if !is_safe_name(pkg) {
                bail!(
                    "context.packages entry '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    pkg
                );
            }
        }

        // Validate addon names and tool names are safe identifiers
        for (addon_name, addon_tools) in &self.addons.addons {
            if !is_safe_name(addon_name) {
                bail!(
                    "addon name '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    addon_name
                );
            }
            for (tool_name, tool_entry) in &addon_tools.tools {
                if !is_safe_name(tool_name) {
                    bail!(
                        "tool name '{}' in addon '{}' contains invalid characters. \
                         Must contain only [a-zA-Z0-9_-]",
                        tool_name,
                        addon_name
                    );
                }
                if let Some(version) = &tool_entry.version
                    && !version.is_empty() // empty string means "use addon default" — valid
                    && !is_safe_version(version)
                {
                    bail!(
                        "tool version '{}' for '{}' in addon '{}' contains invalid characters. \
                         Must contain only [a-zA-Z0-9._\\-+]",
                        version,
                        tool_name,
                        addon_name
                    );
                }
            }
        }

        // Validate skill names are safe
        for skill in &self.skills.include {
            if !is_safe_name(skill) {
                bail!(
                    "skills.include entry '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    skill
                );
            }
        }
        for skill in &self.skills.exclude {
            if !is_safe_name(skill) {
                bail!(
                    "skills.exclude entry '{}' contains invalid characters. \
                     Must contain only [a-zA-Z0-9_-]",
                    skill
                );
            }
        }

        // Validate [processkit]
        self.validate_processkit()?;

        // Validate extra volumes path safety
        self.validate_extra_volumes()?;

        Ok(())
    }

    /// Validate the [processkit] section. Split out for testability.
    fn validate_processkit(&self) -> Result<()> {
        let pk = &self.processkit;

        // source must be a non-empty URL-ish string
        if pk.source.trim().is_empty() {
            bail!("processkit.source must not be empty");
        }
        if !(pk.source.starts_with("http://")
            || pk.source.starts_with("https://")
            || pk.source.starts_with("git@")
            || pk.source.starts_with("file://")
            || pk.source.starts_with("ssh://"))
        {
            bail!(
                "processkit.source '{}' does not look like a URL. \
                 Expected one of: http://, https://, git@, ssh://, file://",
                pk.source
            );
        }

        // version: allow the "unset" sentinel, OR a leading-`v` semver-ish
        // tag, OR a bare semver string. We don't pin to strict semver because
        // git tags vary; just sanity check.
        if pk.version != PROCESSKIT_VERSION_UNSET && pk.version != PROCESSKIT_VERSION_LATEST {
            let stripped = pk.version.strip_prefix('v').unwrap_or(&pk.version);
            // Either parses as semver, or matches a relaxed `numbers + dots`
            // shape (e.g. "0.4", "1.0.0-rc1").
            let semver_ok = semver::Version::parse(stripped).is_ok();
            let relaxed_ok = !stripped.is_empty()
                && stripped
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
                && stripped.chars().any(|c| c.is_ascii_digit());
            if !semver_ok && !relaxed_ok {
                bail!(
                    "processkit.version '{}' is not a valid version tag. \
                     Use the sentinel \"unset\", \"latest\", a semver string like \"0.4.0\", \
                     or a tag like \"v0.4.0\".",
                    pk.version
                );
            }
        }

        // src_path: no traversal, no absolute paths
        if pk.src_path.contains("..") {
            bail!(
                "processkit.src_path '{}' must not contain '..'",
                pk.src_path
            );
        }
        if pk.src_path.starts_with('/') {
            bail!(
                "processkit.src_path '{}' must not be an absolute path",
                pk.src_path
            );
        }

        // branch: if set, must not be empty
        if let Some(branch) = &pk.branch
            && branch.trim().is_empty()
        {
            bail!("processkit.branch is set but empty; remove it or provide a name");
        }

        Ok(())
    }

    /// Resolve `[ai].providers` into addon entries so the addon pipeline
    /// handles AI tool installation. Called before `validate()` during load.
    /// Idempotent — won't overwrite if the user already configured the addon
    /// explicitly in `[addons]`.
    pub fn resolve_ai_provider_addons(&mut self) {
        for provider in &self.ai.providers {
            let addon_name = format!("ai-{}", provider);
            self.addons
                .addons
                .entry(addon_name)
                .or_insert_with(|| AddonToolsSection {
                    tools: HashMap::new(),
                });
        }
    }

    /// Get the host root path (.aibox-home/ directory), respecting env override.
    /// Falls back to `.root/` if that directory exists (backward compatibility).
    pub fn host_root_dir(&self) -> PathBuf {
        if let Ok(val) = std::env::var("AIBOX_HOST_ROOT") {
            return PathBuf::from(val);
        }
        // Backward compat: use .root/ if it exists and .aibox-home/ doesn't
        let new_path = PathBuf::from(".aibox-home");
        let old_path = PathBuf::from(".root");
        if old_path.exists() && !new_path.exists() {
            old_path
        } else {
            new_path
        }
    }

    /// Get the container-side home directory based on the configured user.
    pub fn container_home(&self) -> String {
        if self.container.user == "root" {
            "/root".to_string()
        } else {
            format!("/home/{}", self.container.user)
        }
    }

    /// Get the workspace directory, respecting env override.
    pub fn workspace_dir(&self) -> String {
        std::env::var("AIBOX_WORKSPACE_DIR").unwrap_or_else(|_| "..".to_string())
    }
}

// ---------------------------------------------------------------------------
// Test helper
// ---------------------------------------------------------------------------

/// Create a `AiboxConfig` for testing with sensible defaults.
/// Only available in test builds to reduce boilerplate across test modules.
#[cfg(test)]
pub fn test_config() -> AiboxConfig {
    let mut config = AiboxConfig {
        aibox: AiboxSection {
            version: "0.9.0".to_string(),
            base: BaseImage::Debian,
        },
        container: ContainerSection {
            name: "test-proj".to_string(),
            hostname: "test-proj".to_string(),
            user: "root".to_string(),
            post_create_command: None,
            keepalive: false,
            environment: HashMap::new(),
            extra_volumes: vec![],
        },
        context: ContextSection::default(),
        ai: AiSection::default(),
        addons: AddonsSection::default(),
        skills: SkillsSection::default(),
        processkit: ProcessKitSection::default(),
        agents: AgentsSection::default(),
        customization: CustomizationSection::default(),
        audio: AudioSection::default(),
        process: None,
        mcp: McpSection::default(),
        local_env: HashMap::new(),
        local_mcp_servers: vec![],
    };
    config.resolve_ai_provider_addons();
    config
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;

    // -- TOML fixtures ------------------------------------------------------

    fn full_toml() -> &'static str {
        r#"
[aibox]
version = "0.9.0"
base = "debian"

[container]
name = "my-project"
hostname = "my-project"
user = "root"
keepalive = false
post_create_command = "npm install"

[context]
schema_version = "2.0.0"

[ai]
providers = ["claude", "aider", "mistral"]

[process]
packages = ["managed", "code", "documentation"]

[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }

[addons.node.tools]
node = { version = "22" }
pnpm = { version = "10" }

[addons.rust.tools]
rustc = { version = "1.89" }
clippy = {}
rustfmt = {}

[addons.latex.tools]
texlive-core = {}
texlive-recommended = {}

[addons.infrastructure.tools]
opentofu = {}
ansible = {}

[addons.kubernetes.tools]
kubectl = {}
helm = {}

[addons.cloud-aws.tools]
aws-cli = {}

[addons.docs-docusaurus.tools]
docusaurus = { version = "3" }

[skills]
exclude = ["standup-context"]
include = ["flutter-development"]

[appearance]
theme = "gruvbox-dark"
prompt = "default"

[audio]
enabled = false
"#
    }

    fn minimal_toml() -> &'static str {
        r#"
[aibox]
version = "0.9.0"

[container]
name = "my-project"
"#
    }

    fn parse_toml(s: &str) -> Result<AiboxConfig> {
        let mut config: AiboxConfig = toml::from_str(s).context("Failed to parse TOML")?;
        config.migrate_legacy_sections();
        config.validate()?;
        Ok(config)
    }

    // -- Full config parsing ------------------------------------------------

    #[test]
    fn parse_full_toml_all_fields() {
        let config = parse_toml(full_toml()).expect("should parse full toml");

        // [aibox]
        assert_eq!(config.aibox.version, "0.9.0");
        assert_eq!(config.aibox.base, BaseImage::Debian);

        // [container]
        assert_eq!(config.container.name, "my-project");
        assert_eq!(config.container.hostname, "my-project");
        assert_eq!(config.container.user, "root");
        assert_eq!(
            config.container.post_create_command.as_deref(),
            Some("npm install")
        );
        assert!(!config.container.keepalive);

        // [context]
        assert_eq!(config.context.schema_version, "2.0.0");

        // [ai]
        assert_eq!(config.ai.providers.len(), 3);
        assert_eq!(config.ai.providers[0], AiProvider::Claude);
        assert_eq!(config.ai.providers[1], AiProvider::Aider);
        assert_eq!(config.ai.providers[2], AiProvider::Mistral);

        // [context].packages (migrated from legacy [process])
        assert_eq!(
            config.context.packages,
            vec!["managed", "code", "documentation"]
        );

        // [addons]
        assert_eq!(config.addons.addons.len(), 8);
        assert!(config.addons.has_addon("python"));
        assert!(config.addons.has_addon("node"));
        assert!(config.addons.has_addon("rust"));
        assert!(config.addons.has_addon("latex"));
        assert!(config.addons.has_addon("infrastructure"));
        assert!(config.addons.has_addon("kubernetes"));
        assert!(config.addons.has_addon("cloud-aws"));

        // Check specific tool versions
        assert_eq!(config.addons.tool_version("python", "python"), Some("3.13"));
        assert_eq!(config.addons.tool_version("python", "uv"), Some("0.7"));
        assert_eq!(config.addons.tool_version("rust", "rustc"), Some("1.87"));
        assert_eq!(config.addons.tool_version("rust", "clippy"), None);
        assert_eq!(config.addons.tool_version("rust", "rustfmt"), None);
        assert!(config.addons.has_tool("kubernetes", "kubectl"));
        assert!(config.addons.has_tool("kubernetes", "helm"));
        assert_eq!(
            config.addons.tool_version("docs-docusaurus", "docusaurus"),
            Some("3")
        );

        // [skills]
        assert_eq!(config.skills.exclude, vec!["standup-context"]);
        assert_eq!(config.skills.include, vec!["flutter-development"]);

        // [customization] (parsed from legacy [appearance] via serde alias)
        assert_eq!(config.customization.theme, Theme::GruvboxDark);
        assert_eq!(config.customization.prompt, StarshipPreset::Default);

        // [audio]
        assert!(!config.audio.enabled);
    }

    // -- Minimal config with defaults ---------------------------------------

    #[test]
    fn parse_minimal_toml_defaults() {
        let config = parse_toml(minimal_toml()).expect("should parse minimal toml");
        assert_eq!(config.aibox.base, BaseImage::Debian);
        assert_eq!(config.container.name, "my-project");
        assert_eq!(config.container.hostname, "aibox");
        assert_eq!(config.context.schema_version, "1.0.0");
        assert_eq!(config.ai.providers, vec![AiProvider::Claude]);
        assert_eq!(config.context.packages, vec!["managed"]);
        assert!(config.addons.addons.is_empty());
        assert!(config.skills.include.is_empty());
        assert!(config.skills.exclude.is_empty());
        assert!(!config.audio.enabled);
        assert_eq!(config.audio.pulse_server, "tcp:host.docker.internal:4714");
    }

    // -- Validation errors --------------------------------------------------

    #[test]
    fn parse_invalid_semver_version() {
        let toml = r#"
[aibox]
version = "not-a-version"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid semver");
    }

    #[test]
    fn parse_aibox_version_latest_sentinel() {
        let toml = r#"
[aibox]
version = "latest"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_ok(), "should accept 'latest' as aibox version");
        assert_eq!(result.unwrap().aibox.version, "latest");
    }

    #[test]
    fn parse_empty_container_name() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty container name");
    }

    #[test]
    fn invalid_schema_version_semver() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
schema_version = "bad"
"#;
        let result = parse_toml(toml);
        assert!(
            result.is_err(),
            "should reject invalid schema_version semver"
        );
    }

    #[test]
    fn invalid_container_name_chars() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "my project!"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid container name");
    }

    #[test]
    fn empty_context_packages_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[context]
packages = []
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty context packages");
    }

    #[test]
    fn legacy_process_section_migrated() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[process]
packages = ["managed", "code"]

[context]
schema_version = "2.0.0"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.context.packages, vec!["managed", "code"]);
        assert_eq!(config.context.schema_version, "2.0.0");
    }

    #[test]
    fn legacy_appearance_alias_works() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[appearance]
theme = "dracula"
prompt = "minimal"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.customization.theme, Theme::Dracula);
        assert_eq!(config.customization.prompt, StarshipPreset::Minimal);
    }

    #[test]
    fn invalid_skill_name_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[skills]
include = ["valid-skill", "bad skill!"]
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid skill name");
    }

    #[test]
    fn invalid_addon_name_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[addons."bad addon!".tools]
tool = {}
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid addon name");
    }

    // -- AI providers -------------------------------------------------------

    #[test]
    fn ai_provider_display() {
        assert_eq!(format!("{}", AiProvider::Claude), "claude");
        assert_eq!(format!("{}", AiProvider::Aider), "aider");
        assert_eq!(format!("{}", AiProvider::Gemini), "gemini");
        assert_eq!(format!("{}", AiProvider::Mistral), "mistral");
        assert_eq!(format!("{}", AiProvider::Codex), "codex");
        assert_eq!(format!("{}", AiProvider::Continue), "continue");
        assert_eq!(format!("{}", AiProvider::Copilot), "copilot");
    }

    #[test]
    fn ai_provider_binary_name() {
        // Most providers: binary name matches display name.
        assert_eq!(AiProvider::Claude.binary_name(), "claude");
        assert_eq!(AiProvider::Aider.binary_name(), "aider");
        assert_eq!(AiProvider::Codex.binary_name(), "codex");
        assert_eq!(AiProvider::Copilot.binary_name(), "copilot");
        // Continue is the exception: display = "continue", binary = "cn".
        assert_eq!(AiProvider::Continue.binary_name(), "cn");
    }

    #[test]
    fn parse_all_ai_providers() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[ai]
providers = ["claude", "aider", "gemini", "mistral"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.ai.providers.len(), 4);
        assert_eq!(config.ai.providers[3], AiProvider::Mistral);
    }

    #[test]
    fn parse_new_ai_providers() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[ai]
providers = ["codex", "copilot", "continue"]
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(config.ai.providers.len(), 3);
        assert_eq!(config.ai.providers[0], AiProvider::Codex);
        assert_eq!(config.ai.providers[1], AiProvider::Copilot);
        assert_eq!(config.ai.providers[2], AiProvider::Continue);
        assert!(config.addons.has_addon("ai-codex"));
        assert!(config.addons.has_addon("ai-copilot"));
        assert!(config.addons.has_addon("ai-continue"));
    }

    #[test]
    fn parse_empty_ai_providers() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[ai]
providers = []
"#;
        let config = parse_toml(toml).unwrap();
        assert!(config.ai.providers.is_empty());
    }

    #[test]
    fn default_ai_providers_is_claude() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.ai.providers, vec![AiProvider::Claude]);
    }

    // -- Base image ---------------------------------------------------------

    #[test]
    fn base_image_display() {
        assert_eq!(format!("{}", BaseImage::Debian), "debian");
    }

    #[test]
    fn base_image_default_is_debian() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.aibox.base, BaseImage::Debian);
    }

    // -- Addons helpers -----------------------------------------------------

    #[test]
    fn addons_convenience_methods() {
        let config = parse_toml(full_toml()).unwrap();
        assert!(config.addons.has_python());
        assert!(config.addons.has_rust());
        assert!(config.addons.has_node());
        assert!(config.addons.has_latex());
    }

    #[test]
    fn addons_tool_lookup() {
        let config = parse_toml(full_toml()).unwrap();
        assert!(config.addons.has_tool("python", "python"));
        assert!(config.addons.has_tool("python", "uv"));
        assert!(!config.addons.has_tool("python", "poetry"));
        assert_eq!(config.addons.tool_version("node", "node"), Some("22"));
        assert_eq!(config.addons.tool_version("node", "pnpm"), Some("10"));
        assert_eq!(config.addons.tool_version("cloud-aws", "aws-cli"), None);
    }

    #[test]
    fn addons_empty_by_default() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert!(config.addons.addons.is_empty());
        assert!(!config.addons.has_python());
        assert!(!config.addons.has_rust());
    }

    #[test]
    fn addon_with_only_versionless_tools() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[addons.rust.tools]
clippy = {}
rustfmt = {}
"#;
        let config = parse_toml(toml).unwrap();
        assert!(config.addons.has_rust());
        assert!(config.addons.has_tool("rust", "clippy"));
        assert_eq!(config.addons.tool_version("rust", "clippy"), None);
    }

    // -- Context packages ---------------------------------------------------

    #[test]
    fn context_packages_default_is_managed() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.context.packages, vec!["managed"]);
    }

    #[test]
    fn context_packages_custom_via_legacy_process() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[process]
packages = ["managed", "code", "research"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(config.context.packages, vec!["managed", "code", "research"]);
    }

    // -- Skills section -----------------------------------------------------

    #[test]
    fn skills_default_empty() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert!(config.skills.include.is_empty());
        assert!(config.skills.exclude.is_empty());
    }

    #[test]
    fn skills_include_only() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[skills]
include = ["flutter-development", "rust-conventions"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.skills.include,
            vec!["flutter-development", "rust-conventions"]
        );
        assert!(config.skills.exclude.is_empty());
    }

    #[test]
    fn skills_exclude_only() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[skills]
exclude = ["standup-context"]
"#;
        let config = parse_toml(toml).unwrap();
        assert!(config.skills.include.is_empty());
        assert_eq!(config.skills.exclude, vec!["standup-context"]);
    }

    // -- Appearance ---------------------------------------------------------

    #[test]
    fn appearance_all_themes() {
        for (input, expected) in [
            ("gruvbox-dark", Theme::GruvboxDark),
            ("catppuccin-mocha", Theme::CatppuccinMocha),
            ("catppuccin-latte", Theme::CatppuccinLatte),
            ("dracula", Theme::Dracula),
            ("tokyo-night", Theme::TokyoNight),
            ("nord", Theme::Nord),
        ] {
            let toml = format!(
                r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[appearance]
theme = "{input}"
"#
            );
            let config = parse_toml(&toml).unwrap();
            assert_eq!(config.customization.theme, expected);
        }
    }

    // -- File loading -------------------------------------------------------

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aibox.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(minimal_toml().as_bytes()).unwrap();
        let config = AiboxConfig::load(&path).expect("should load from file");
        assert_eq!(config.container.name, "my-project");
    }

    #[test]
    fn load_missing_file() {
        let result = AiboxConfig::load(Path::new("/nonexistent/aibox.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn from_str_parses_and_validates() {
        let config = AiboxConfig::from_str(minimal_toml()).unwrap();
        assert_eq!(config.container.name, "my-project");
    }

    // -- Host/container path helpers ----------------------------------------

    #[test]
    #[serial]
    fn host_root_dir_default() {
        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.host_root_dir(), PathBuf::from(".aibox-home"));
    }

    #[test]
    #[serial]
    fn host_root_dir_env_override() {
        unsafe {
            std::env::set_var("AIBOX_HOST_ROOT", "/custom/root");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.host_root_dir(), PathBuf::from("/custom/root"));
        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn workspace_dir_default() {
        unsafe {
            std::env::remove_var("AIBOX_WORKSPACE_DIR");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.workspace_dir(), "..");
    }

    #[test]
    #[serial]
    fn workspace_dir_env_override() {
        unsafe {
            std::env::set_var("AIBOX_WORKSPACE_DIR", "/my/workspace");
        }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.workspace_dir(), "/my/workspace");
        unsafe {
            std::env::remove_var("AIBOX_WORKSPACE_DIR");
        }
    }

    // -- test_config helper -------------------------------------------------

    #[test]
    fn test_config_validates() {
        let config = test_config();
        config.validate().expect("test_config should be valid");
    }

    // -- AI provider → addon resolution ------------------------------------

    #[test]
    fn resolve_ai_providers_creates_addon_entries() {
        let config = test_config(); // default: providers = [Claude]
        assert!(
            config.addons.has_addon("ai-claude"),
            "ai-claude addon should be auto-resolved from [ai].providers"
        );
    }

    #[test]
    fn resolve_ai_providers_multiple() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            providers = ["claude", "aider", "gemini", "mistral"]
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(config.addons.has_addon("ai-claude"));
        assert!(config.addons.has_addon("ai-aider"));
        assert!(config.addons.has_addon("ai-gemini"));
        assert!(config.addons.has_addon("ai-mistral"));
    }

    #[test]
    fn resolve_ai_providers_empty_creates_no_addons() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            providers = []
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert!(!config.addons.has_addon("ai-claude"));
        assert!(!config.addons.has_addon("ai-aider"));
    }

    // -- ProcessKit section -------------------------------------------------

    #[test]
    fn processkit_section_default_values() {
        let pk = ProcessKitSection::default();
        assert_eq!(pk.source, crate::processkit_vocab::PROCESSKIT_GIT_SOURCE);
        assert_eq!(pk.version, "unset");
        assert_eq!(pk.src_path, "src");
        assert_eq!(pk.branch, None);
    }

    #[test]
    fn processkit_section_parses_from_toml() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://example.com/forks/processkit.git"
version = "v0.4.0"
src_path = "content"
branch = "develop"
"#;
        let config = parse_toml(toml).expect("should parse processkit section");
        assert_eq!(
            config.processkit.source,
            "https://example.com/forks/processkit.git"
        );
        assert_eq!(config.processkit.version, "v0.4.0");
        assert_eq!(config.processkit.src_path, "content");
        assert_eq!(config.processkit.branch.as_deref(), Some("develop"));
    }

    #[test]
    fn processkit_section_parses_with_only_source() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://example.com/forks/processkit.git"
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.processkit.source,
            "https://example.com/forks/processkit.git"
        );
        assert_eq!(config.processkit.version, "unset");
        assert_eq!(config.processkit.src_path, "src");
        assert_eq!(config.processkit.branch, None);
    }

    #[test]
    fn processkit_section_parses_when_section_missing() {
        // An old-style aibox.toml with no [processkit] block should parse
        // cleanly with all defaults filled in.
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(
            config.processkit.source,
            "https://github.com/projectious-work/processkit.git"
        );
        assert_eq!(config.processkit.version, "unset");
        assert_eq!(config.processkit.src_path, "src");
        assert_eq!(config.processkit.branch, None);
    }

    #[test]
    fn processkit_validate_rejects_empty_source() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty source");
    }

    #[test]
    fn processkit_validate_rejects_non_url_source() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "not-a-url"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject non-URL source");
    }

    #[test]
    fn processkit_validate_rejects_path_traversal_in_src_path() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
src_path = "../etc"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject path traversal in src_path");
    }

    #[test]
    fn processkit_validate_rejects_absolute_src_path() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
src_path = "/etc"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject absolute src_path");
    }

    #[test]
    fn processkit_validate_accepts_unset_version() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
version = "unset"
"#;
        let config = parse_toml(toml).expect("unset sentinel should validate");
        assert_eq!(config.processkit.version, "unset");
    }

    #[test]
    fn processkit_validate_accepts_semver_version() {
        for ver in ["v0.4.0", "0.4.0", "v1.0.0-rc1", "v0.4"] {
            let toml = format!(
                r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
version = "{ver}"
"#
            );
            parse_toml(&toml)
                .unwrap_or_else(|e| panic!("version {ver} should validate, but got error: {e}"));
        }
    }

    #[test]
    fn processkit_validate_rejects_empty_branch() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[processkit]
source = "https://github.com/projectious-work/processkit.git"
branch = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty branch");
    }

    #[test]
    fn resolve_ai_providers_does_not_overwrite_explicit_addon() {
        let toml = r#"
            [aibox]
            version = "0.9.0"
            [container]
            name = "test"
            [ai]
            providers = ["aider"]
            [addons.ai-aider.tools]
            aider = { version = "custom" }
        "#;
        let config = AiboxConfig::from_str(toml).unwrap();
        // Should keep the user's explicit config, not overwrite with empty tools
        let aider_tools = &config.addons.get_addon("ai-aider").unwrap().tools;
        assert!(
            aider_tools.contains_key("aider"),
            "should preserve user-configured tool entry"
        );
    }

    // -- ExtraVolume / .aibox-local.toml tests --------------------------------

    #[test]
    fn extra_volumes_parse_from_toml() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"

[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"

[[container.extra_volumes]]
source = "~/.aws"
target = "/home/aibox/.aws"
read_only = true
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(config.container.extra_volumes.len(), 2);
        assert_eq!(config.container.extra_volumes[0].source, "~/.config/gh");
        assert_eq!(
            config.container.extra_volumes[0].target,
            "/home/aibox/.config/gh"
        );
        assert!(!config.container.extra_volumes[0].read_only);
        assert!(config.container.extra_volumes[1].read_only);
    }

    #[test]
    fn environment_parses_from_toml() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"

[container.environment]
GH_TOKEN = "ghp_abc"
MY_VAR = "hello"
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        assert_eq!(
            config
                .container
                .environment
                .get("GH_TOKEN")
                .map(|s| s.as_str()),
            Some("ghp_abc")
        );
        assert_eq!(
            config
                .container
                .environment
                .get("MY_VAR")
                .map(|s| s.as_str()),
            Some("hello")
        );
    }

    #[test]
    fn extra_volumes_rejects_dotdot_in_source() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[[container.extra_volumes]]
source = "../../../etc/passwd"
target = "/home/aibox/passwd"
"#;
        let result = AiboxConfig::from_str(toml);
        // from_str calls validate() which calls validate_extra_volumes()
        assert!(result.is_err(), "should reject .. in source");
    }

    #[test]
    fn extra_volumes_rejects_relative_target() {
        let toml = r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[[container.extra_volumes]]
source = "~/.config/gh"
target = "home/aibox/.config/gh"
"#;
        let result = AiboxConfig::from_str(toml);
        assert!(result.is_err(), "should reject non-absolute target");
    }

    #[test]
    fn aibox_local_toml_merges_environment_and_volumes() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // Write aibox.toml
        std::fs::write(
            dir.join("aibox.toml"),
            r#"
[aibox]
version = "0.9.0"
[container]
name = "test"
[container.environment]
SHARED = "from-main"
"#,
        )
        .unwrap();

        // Write .aibox-local.toml
        std::fs::write(
            dir.join(".aibox-local.toml"),
            r#"
[container.environment]
GH_TOKEN = "ghp_secret"
SHARED = "local-wins"

[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"
"#,
        )
        .unwrap();

        // Load from the temp dir
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        let config = AiboxConfig::load_or_default().unwrap();
        std::env::set_current_dir(orig).unwrap();

        // Local env merged: GH_TOKEN added, SHARED overridden by local
        assert_eq!(
            config
                .container
                .environment
                .get("GH_TOKEN")
                .map(|s| s.as_str()),
            Some("ghp_secret")
        );
        assert_eq!(
            config
                .container
                .environment
                .get("SHARED")
                .map(|s| s.as_str()),
            Some("local-wins")
        );
        // Volume appended
        assert_eq!(config.container.extra_volumes.len(), 1);
        assert_eq!(config.container.extra_volumes[0].source, "~/.config/gh");
    }
}
