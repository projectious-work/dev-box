use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Container image registry base URL.
pub const IMAGE_REGISTRY: &str = "ghcr.io/projectious-work/dev-box";

/// Standard devcontainer directory name.
pub const DEVCONTAINER_DIR: &str = ".devcontainer";
/// Standard compose file name within devcontainer dir.
pub const COMPOSE_FILE: &str = ".devcontainer/docker-compose.yml";
/// Standard Dockerfile name within devcontainer dir.
pub const DOCKERFILE: &str = ".devcontainer/Dockerfile";
/// Standard devcontainer.json name.
pub const DEVCONTAINER_JSON: &str = ".devcontainer/devcontainer.json";

/// Image flavors corresponding to images/ subdirectories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageFlavor {
    Base,
    Python,
    Latex,
    Rust,
    PythonLatex,
    RustLatex,
}

impl std::fmt::Display for ImageFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFlavor::Base => write!(f, "base"),
            ImageFlavor::Python => write!(f, "python"),
            ImageFlavor::Latex => write!(f, "latex"),
            ImageFlavor::Rust => write!(f, "rust"),
            ImageFlavor::PythonLatex => write!(f, "python-latex"),
            ImageFlavor::RustLatex => write!(f, "rust-latex"),
        }
    }
}

impl ImageFlavor {
    pub fn from_str_loose(s: &str) -> Result<Self> {
        match s {
            "base" => Ok(ImageFlavor::Base),
            "python" => Ok(ImageFlavor::Python),
            "latex" => Ok(ImageFlavor::Latex),
            "rust" => Ok(ImageFlavor::Rust),
            "python-latex" => Ok(ImageFlavor::PythonLatex),
            "rust-latex" => Ok(ImageFlavor::RustLatex),
            _ => bail!("Unknown image flavor: '{}'. Valid: base, python, latex, rust, python-latex, rust-latex", s),
        }
    }

    pub fn contains_python(&self) -> bool {
        matches!(self, ImageFlavor::Python | ImageFlavor::PythonLatex)
    }

    pub fn contains_latex(&self) -> bool {
        matches!(self, ImageFlavor::Latex | ImageFlavor::PythonLatex | ImageFlavor::RustLatex)
    }

    pub fn contains_rust(&self) -> bool {
        matches!(self, ImageFlavor::Rust | ImageFlavor::RustLatex)
    }
}

/// Work process flavors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessFlavor {
    Minimal,
    Managed,
    Research,
    Product,
}

impl std::fmt::Display for ProcessFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessFlavor::Minimal => write!(f, "minimal"),
            ProcessFlavor::Managed => write!(f, "managed"),
            ProcessFlavor::Research => write!(f, "research"),
            ProcessFlavor::Product => write!(f, "product"),
        }
    }
}

impl ProcessFlavor {
    pub fn from_str_loose(s: &str) -> Result<Self> {
        match s {
            "minimal" => Ok(ProcessFlavor::Minimal),
            "managed" => Ok(ProcessFlavor::Managed),
            "research" => Ok(ProcessFlavor::Research),
            "product" => Ok(ProcessFlavor::Product),
            _ => bail!("Unknown process flavor: '{}'. Valid: minimal, managed, research, product", s),
        }
    }
}

/// Extra volume mount specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraVolume {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
}

/// Top-level [dev-box] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBoxSection {
    pub version: String,
    pub image: ImageFlavor,
    pub process: ProcessFlavor,
}

/// [container] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSection {
    pub name: String,
    #[serde(default = "default_hostname")]
    pub hostname: String,
    #[serde(default)]
    pub ports: Vec<String>,
    #[serde(default)]
    pub extra_packages: Vec<String>,
    #[serde(default)]
    pub extra_volumes: Vec<ExtraVolume>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

fn default_hostname() -> String {
    "dev-box".to_string()
}

/// [context] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSection {
    #[serde(default = "default_owner_path")]
    pub owner: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_owner_path() -> String {
    "~/.config/dev-box/OWNER.md".to_string()
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

impl Default for ContextSection {
    fn default() -> Self {
        Self {
            owner: default_owner_path(),
            schema_version: default_schema_version(),
        }
    }
}

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

/// Root config structure mapping dev-box.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBoxConfig {
    #[serde(rename = "dev-box")]
    pub dev_box: DevBoxSection,
    pub container: ContainerSection,
    #[serde(default)]
    pub context: ContextSection,
    #[serde(default)]
    pub audio: AudioSection,
}

impl DevBoxConfig {
    /// Load configuration from a specific file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: DevBoxConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
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

    /// Load from ./dev-box.toml or return an error if not found.
    pub fn load_or_default() -> Result<Self> {
        let path = PathBuf::from("dev-box.toml");
        if path.exists() {
            Self::load(&path)
        } else {
            bail!("No dev-box.toml found in the current directory. Run 'dev-box init' to create one.")
        }
    }

    /// Validate the config values.
    fn validate(&self) -> Result<()> {
        // Validate version is valid semver
        semver::Version::parse(&self.dev_box.version)
            .with_context(|| format!("Invalid version '{}': must be valid semver", self.dev_box.version))?;

        // Validate schema_version is valid semver
        semver::Version::parse(&self.context.schema_version)
            .with_context(|| format!("Invalid schema_version '{}': must be valid semver", self.context.schema_version))?;

        // Validate container name is non-empty
        if self.container.name.is_empty() {
            bail!("container.name must not be empty");
        }

        Ok(())
    }

    /// Get the host root path (.root/ directory), respecting env override.
    pub fn host_root_dir(&self) -> PathBuf {
        if let Ok(val) = std::env::var("DEV_BOX_HOST_ROOT") {
            PathBuf::from(val)
        } else {
            PathBuf::from(".root")
        }
    }

    /// Get the workspace directory, respecting env override.
    pub fn workspace_dir(&self) -> String {
        std::env::var("DEV_BOX_WORKSPACE_DIR").unwrap_or_else(|_| "..".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;

    fn valid_toml() -> &'static str {
        r#"
[dev-box]
version = "0.1.0"
image = "python"
process = "managed"

[container]
name = "test-project"
hostname = "test-host"
ports = ["8080:80"]
extra_packages = ["ripgrep", "fd-find"]

[container.environment]
MY_VAR = "hello"

[[container.extra_volumes]]
source = "/host/data"
target = "/data"
read_only = true

[context]
owner = "~/my-owner.md"
schema_version = "2.0.0"

[audio]
enabled = true
pulse_server = "tcp:localhost:4714"
"#
    }

    fn minimal_toml() -> &'static str {
        r#"
[dev-box]
version = "0.1.0"
image = "base"
process = "minimal"

[container]
name = "my-project"
"#
    }

    fn parse_toml(s: &str) -> Result<DevBoxConfig> {
        let config: DevBoxConfig = toml::from_str(s)
            .context("Failed to parse TOML")?;
        config.validate()?;
        Ok(config)
    }

    #[test]
    fn parse_valid_toml_all_fields() {
        let config = parse_toml(valid_toml()).expect("should parse valid toml");
        assert_eq!(config.dev_box.version, "0.1.0");
        assert_eq!(config.dev_box.image, ImageFlavor::Python);
        assert_eq!(config.dev_box.process, ProcessFlavor::Managed);
        assert_eq!(config.container.name, "test-project");
        assert_eq!(config.container.hostname, "test-host");
        assert_eq!(config.container.ports, vec!["8080:80"]);
        assert_eq!(config.container.extra_packages, vec!["ripgrep", "fd-find"]);
        assert_eq!(config.container.environment.get("MY_VAR").unwrap(), "hello");
        assert_eq!(config.container.extra_volumes.len(), 1);
        assert_eq!(config.container.extra_volumes[0].source, "/host/data");
        assert_eq!(config.container.extra_volumes[0].target, "/data");
        assert!(config.container.extra_volumes[0].read_only);
        assert_eq!(config.context.owner, "~/my-owner.md");
        assert_eq!(config.context.schema_version, "2.0.0");
        assert!(config.audio.enabled);
        assert_eq!(config.audio.pulse_server, "tcp:localhost:4714");
    }

    #[test]
    fn parse_minimal_toml_defaults() {
        let config = parse_toml(minimal_toml()).expect("should parse minimal toml");
        assert_eq!(config.dev_box.image, ImageFlavor::Base);
        assert_eq!(config.dev_box.process, ProcessFlavor::Minimal);
        assert_eq!(config.container.name, "my-project");
        assert_eq!(config.container.hostname, "dev-box", "hostname should default");
        assert!(config.container.ports.is_empty());
        assert!(config.container.extra_packages.is_empty());
        assert!(config.container.extra_volumes.is_empty());
        assert!(config.container.environment.is_empty());
        assert_eq!(config.context.owner, "~/.config/dev-box/OWNER.md");
        assert_eq!(config.context.schema_version, "1.0.0");
        assert!(!config.audio.enabled);
        assert_eq!(config.audio.pulse_server, "tcp:host.docker.internal:4714");
    }

    #[test]
    fn parse_invalid_image_flavor() {
        let toml = r#"
[dev-box]
version = "0.1.0"
image = "golang"
process = "minimal"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject unknown image flavor");
    }

    #[test]
    fn parse_invalid_process_flavor() {
        let toml = r#"
[dev-box]
version = "0.1.0"
image = "base"
process = "waterfall"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject unknown process flavor");
    }

    #[test]
    fn parse_invalid_semver_version() {
        let toml = r#"
[dev-box]
version = "not-a-version"
image = "base"
process = "minimal"

[container]
name = "test"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid semver");
    }

    #[test]
    fn parse_empty_container_name() {
        let toml = r#"
[dev-box]
version = "0.1.0"
image = "base"
process = "minimal"

[container]
name = ""
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject empty container name");
    }

    #[test]
    fn image_flavor_from_str_loose_all_valid() {
        assert_eq!(ImageFlavor::from_str_loose("base").unwrap(), ImageFlavor::Base);
        assert_eq!(ImageFlavor::from_str_loose("python").unwrap(), ImageFlavor::Python);
        assert_eq!(ImageFlavor::from_str_loose("latex").unwrap(), ImageFlavor::Latex);
        assert_eq!(ImageFlavor::from_str_loose("rust").unwrap(), ImageFlavor::Rust);
        assert_eq!(ImageFlavor::from_str_loose("python-latex").unwrap(), ImageFlavor::PythonLatex);
        assert_eq!(ImageFlavor::from_str_loose("rust-latex").unwrap(), ImageFlavor::RustLatex);
    }

    #[test]
    fn image_flavor_from_str_loose_invalid() {
        let result = ImageFlavor::from_str_loose("java");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("java"), "error should mention the invalid value");
    }

    #[test]
    fn image_flavor_contains_python() {
        assert!(!ImageFlavor::Base.contains_python());
        assert!(ImageFlavor::Python.contains_python());
        assert!(!ImageFlavor::Latex.contains_python());
        assert!(!ImageFlavor::Rust.contains_python());
        assert!(ImageFlavor::PythonLatex.contains_python());
        assert!(!ImageFlavor::RustLatex.contains_python());
    }

    #[test]
    fn image_flavor_contains_latex() {
        assert!(!ImageFlavor::Base.contains_latex());
        assert!(!ImageFlavor::Python.contains_latex());
        assert!(ImageFlavor::Latex.contains_latex());
        assert!(!ImageFlavor::Rust.contains_latex());
        assert!(ImageFlavor::PythonLatex.contains_latex());
        assert!(ImageFlavor::RustLatex.contains_latex());
    }

    #[test]
    fn image_flavor_contains_rust() {
        assert!(!ImageFlavor::Base.contains_rust());
        assert!(!ImageFlavor::Python.contains_rust());
        assert!(!ImageFlavor::Latex.contains_rust());
        assert!(ImageFlavor::Rust.contains_rust());
        assert!(!ImageFlavor::PythonLatex.contains_rust());
        assert!(ImageFlavor::RustLatex.contains_rust());
    }

    #[test]
    fn process_flavor_from_str_loose_all_valid() {
        assert_eq!(ProcessFlavor::from_str_loose("minimal").unwrap(), ProcessFlavor::Minimal);
        assert_eq!(ProcessFlavor::from_str_loose("managed").unwrap(), ProcessFlavor::Managed);
        assert_eq!(ProcessFlavor::from_str_loose("research").unwrap(), ProcessFlavor::Research);
        assert_eq!(ProcessFlavor::from_str_loose("product").unwrap(), ProcessFlavor::Product);
    }

    #[test]
    fn process_flavor_from_str_loose_invalid() {
        let result = ProcessFlavor::from_str_loose("waterfall");
        assert!(result.is_err());
    }

    #[test]
    fn process_flavor_display() {
        assert_eq!(format!("{}", ProcessFlavor::Minimal), "minimal");
        assert_eq!(format!("{}", ProcessFlavor::Managed), "managed");
        assert_eq!(format!("{}", ProcessFlavor::Research), "research");
        assert_eq!(format!("{}", ProcessFlavor::Product), "product");
    }

    #[test]
    fn image_flavor_display() {
        assert_eq!(format!("{}", ImageFlavor::Base), "base");
        assert_eq!(format!("{}", ImageFlavor::Python), "python");
        assert_eq!(format!("{}", ImageFlavor::Latex), "latex");
        assert_eq!(format!("{}", ImageFlavor::Rust), "rust");
        assert_eq!(format!("{}", ImageFlavor::PythonLatex), "python-latex");
        assert_eq!(format!("{}", ImageFlavor::RustLatex), "rust-latex");
    }

    #[test]
    #[serial]
    fn host_root_dir_default() {
        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.host_root_dir(), PathBuf::from(".root"));
    }

    #[test]
    #[serial]
    fn host_root_dir_env_override() {
        unsafe { std::env::set_var("DEV_BOX_HOST_ROOT", "/custom/root"); }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.host_root_dir(), PathBuf::from("/custom/root"));
        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
    }

    #[test]
    #[serial]
    fn workspace_dir_default() {
        unsafe { std::env::remove_var("DEV_BOX_WORKSPACE_DIR"); }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.workspace_dir(), "..");
    }

    #[test]
    #[serial]
    fn workspace_dir_env_override() {
        unsafe { std::env::set_var("DEV_BOX_WORKSPACE_DIR", "/my/workspace"); }
        let config = parse_toml(minimal_toml()).unwrap();
        assert_eq!(config.workspace_dir(), "/my/workspace");
        unsafe { std::env::remove_var("DEV_BOX_WORKSPACE_DIR"); }
    }

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dev-box.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(minimal_toml().as_bytes()).unwrap();
        let config = DevBoxConfig::load(&path).expect("should load from file");
        assert_eq!(config.container.name, "my-project");
    }

    #[test]
    fn load_missing_file() {
        let result = DevBoxConfig::load(Path::new("/nonexistent/dev-box.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn invalid_schema_version_semver() {
        let toml = r#"
[dev-box]
version = "0.1.0"
image = "base"
process = "minimal"

[container]
name = "test"

[context]
schema_version = "bad"
"#;
        let result = parse_toml(toml);
        assert!(result.is_err(), "should reject invalid schema_version semver");
    }

    #[test]
    fn extra_volume_read_only_defaults_false() {
        let toml = r#"
[dev-box]
version = "0.1.0"
image = "base"
process = "minimal"

[container]
name = "test"

[[container.extra_volumes]]
source = "/a"
target = "/b"
"#;
        let config = parse_toml(toml).unwrap();
        assert!(!config.container.extra_volumes[0].read_only);
    }
}
