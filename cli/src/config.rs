use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
// Extra volume mount
// ---------------------------------------------------------------------------

/// Extra volume mount specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraVolume {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
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
    pub ports: Vec<String>,
    #[serde(default)]
    pub extra_packages: Vec<String>,
    #[serde(default)]
    pub extra_volumes: Vec<ExtraVolume>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    #[serde(default)]
    pub post_create_command: Option<String>,
    #[serde(default)]
    pub vscode_extensions: Vec<String>,
    /// Network keepalive — prevents OrbStack/VM NAT from dropping idle connections.
    #[serde(default)]
    pub keepalive: bool,
}

fn default_user() -> String {
    "aibox".to_string()
}

fn default_hostname() -> String {
    "aibox".to_string()
}

// ---------------------------------------------------------------------------
// [context] section — UNCHANGED
// ---------------------------------------------------------------------------

/// [context] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSection {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

impl Default for ContextSection {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
        }
    }
}

// ---------------------------------------------------------------------------
// [ai] section
// ---------------------------------------------------------------------------

/// AI tool providers supported in aibox containers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum AiProvider {
    Claude,
    Aider,
    Gemini,
    Mistral,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Claude => write!(f, "claude"),
            AiProvider::Aider => write!(f, "aider"),
            AiProvider::Gemini => write!(f, "gemini"),
            AiProvider::Mistral => write!(f, "mistral"),
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
// [process] section — NEW
// ---------------------------------------------------------------------------

fn default_process_packages() -> Vec<String> {
    vec!["core".to_string()]
}

/// [process] section — composable process packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSection {
    #[serde(default = "default_process_packages")]
    pub packages: Vec<String>,
}

impl Default for ProcessSection {
    fn default() -> Self {
        Self {
            packages: default_process_packages(),
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
    Default,       // Clean, informative — dir, git, language, duration
    Plain,         // ASCII only — no Nerd Font needed
    Minimal,       // Just directory + git branch
    NerdFont,      // Full Nerd Font symbols
    Pastel,        // Soft powerline segments
    Bracketed,     // [segments] in brackets
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
        }
    }
}

fn default_prompt() -> StarshipPreset {
    StarshipPreset::default()
}

/// [appearance] section — color theme and prompt configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSection {
    #[serde(default = "default_theme")]
    pub theme: Theme,
    #[serde(default = "default_prompt")]
    pub prompt: StarshipPreset,
}

impl Default for AppearanceSection {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            prompt: default_prompt(),
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

/// Check that a string is a valid Debian package name.
/// Must start with alphanumeric and contain only [a-zA-Z0-9.+\-].
fn is_safe_package_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '+' || c == '-')
}

/// Check that an addon/tool/skill name uses only safe characters: [a-zA-Z0-9_-].
fn is_safe_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
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
    pub process: ProcessSection,
    #[serde(default)]
    pub addons: AddonsSection,
    #[serde(default)]
    pub skills: SkillsSection,
    #[serde(default)]
    pub appearance: AppearanceSection,
    #[serde(default)]
    pub audio: AudioSection,
}

impl AiboxConfig {
    /// Load configuration from a specific file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let mut config: AiboxConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        config.resolve_ai_provider_addons();
        config.validate()?;
        Ok(config)
    }

    /// Load config from an optional CLI path argument.
    pub fn from_cli_option(config_path: &Option<String>) -> Result<Self> {
        match config_path {
            Some(path) => Self::load(&PathBuf::from(path)),
            None => Self::load_or_default(),
        }
    }

    /// Load from ./aibox.toml or return an error if not found.
    pub fn load_or_default() -> Result<Self> {
        let path = PathBuf::from("aibox.toml");
        if path.exists() {
            Self::load(&path)
        } else {
            bail!(
                "No aibox.toml found in the current directory. Run 'aibox init' to create one."
            )
        }
    }

    /// Parse config from a TOML string. Useful for testing and programmatic use.
    #[allow(dead_code)]
    pub fn from_str(toml_str: &str) -> Result<Self> {
        let mut config: AiboxConfig =
            toml::from_str(toml_str).context("Failed to parse TOML config")?;
        config.resolve_ai_provider_addons();
        config.validate()?;
        Ok(config)
    }

    /// Validate the config values. Called internally by `load`, but also
    /// available for validating programmatically-constructed configs.
    pub fn validate(&self) -> Result<()> {
        // Validate version is valid semver
        semver::Version::parse(&self.aibox.version).with_context(|| {
            format!(
                "Invalid version '{}': must be valid semver",
                self.aibox.version
            )
        })?;

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

        // Validate extra_packages contain only safe package names
        for pkg in &self.container.extra_packages {
            if !is_safe_package_name(pkg) {
                bail!(
                    "extra_packages entry '{}' contains invalid characters. \
                     Must match Debian package naming: [a-zA-Z0-9][a-zA-Z0-9.+\\-]+",
                    pkg
                );
            }
        }

        // Validate process packages have safe names
        if self.process.packages.is_empty() {
            bail!("process.packages must not be empty (at minimum ['core'] is required)");
        }
        for pkg in &self.process.packages {
            if !is_safe_name(pkg) {
                bail!(
                    "process.packages entry '{}' contains invalid characters. \
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
            for tool_name in addon_tools.tools.keys() {
                if !is_safe_name(tool_name) {
                    bail!(
                        "tool name '{}' in addon '{}' contains invalid characters. \
                         Must contain only [a-zA-Z0-9_-]",
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
            ports: vec![],
            extra_packages: vec![],
            extra_volumes: vec![],
            environment: HashMap::new(),
            post_create_command: None,
            vscode_extensions: vec![],
            keepalive: false,
        },
        context: ContextSection::default(),
        ai: AiSection::default(),
        process: ProcessSection::default(),
        addons: AddonsSection::default(),
        skills: SkillsSection::default(),
        appearance: AppearanceSection::default(),
        audio: AudioSection::default(),
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
ports = ["8080:80"]
extra_packages = ["ffmpeg"]
vscode_extensions = ["ms-python.python"]
keepalive = false
post_create_command = "npm install"

[container.environment]
MY_VAR = "hello"

[[container.extra_volumes]]
source = "/host/data"
target = "/data"
read_only = true

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
rustc = { version = "1.87" }
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
        let config: AiboxConfig = toml::from_str(s).context("Failed to parse TOML")?;
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
        assert_eq!(config.container.ports, vec!["8080:80"]);
        assert_eq!(config.container.extra_packages, vec!["ffmpeg"]);
        assert_eq!(config.container.environment.get("MY_VAR").unwrap(), "hello");
        assert_eq!(config.container.extra_volumes.len(), 1);
        assert_eq!(config.container.extra_volumes[0].source, "/host/data");
        assert!(config.container.extra_volumes[0].read_only);
        assert_eq!(
            config.container.post_create_command.as_deref(),
            Some("npm install")
        );
        assert_eq!(
            config.container.vscode_extensions,
            vec!["ms-python.python"]
        );
        assert!(!config.container.keepalive);

        // [context]
        assert_eq!(config.context.schema_version, "2.0.0");

        // [ai]
        assert_eq!(config.ai.providers.len(), 3);
        assert_eq!(config.ai.providers[0], AiProvider::Claude);
        assert_eq!(config.ai.providers[1], AiProvider::Aider);
        assert_eq!(config.ai.providers[2], AiProvider::Mistral);

        // [process]
        assert_eq!(
            config.process.packages,
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
        assert_eq!(
            config.addons.tool_version("python", "python"),
            Some("3.13")
        );
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

        // [appearance]
        assert_eq!(config.appearance.theme, Theme::GruvboxDark);
        assert_eq!(config.appearance.prompt, StarshipPreset::Default);

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
        assert!(config.container.ports.is_empty());
        assert!(config.container.extra_packages.is_empty());
        assert!(config.container.extra_volumes.is_empty());
        assert!(config.container.environment.is_empty());
        assert_eq!(config.context.schema_version, "1.0.0");
        assert_eq!(config.ai.providers, vec![AiProvider::Claude]);
        assert_eq!(config.process.packages, vec!["core"]);
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
    fn invalid_extra_package_name() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"
extra_packages = ["good-pkg", "bad pkg!"]
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid package name");
    }

    #[test]
    fn empty_process_packages_rejected() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[process]
packages = []
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty process packages");
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

    // -- Process section ----------------------------------------------------

    #[test]
    fn process_packages_default_is_core() {
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.process.packages, vec!["core"]);
    }

    #[test]
    fn process_packages_custom() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[process]
packages = ["managed", "code", "research"]
"#;
        let config = parse_toml(toml).unwrap();
        assert_eq!(
            config.process.packages,
            vec!["managed", "code", "research"]
        );
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
            assert_eq!(config.appearance.theme, expected);
        }
    }

    // -- Extra volumes ------------------------------------------------------

    #[test]
    fn extra_volume_read_only_defaults_false() {
        let toml = r#"
[aibox]
version = "0.9.0"

[container]
name = "test"

[[container.extra_volumes]]
source = "/a"
target = "/b"
"#;
        let config = parse_toml(toml).unwrap();
        assert!(!config.container.extra_volumes[0].read_only);
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
}
