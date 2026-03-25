//! Loads addon definitions from YAML files and renders Dockerfile templates.
//!
//! Addon YAML files are stored in `$XDG_CONFIG_HOME/aibox/addons/` with
//! category subdirectories (languages/, tools/, docs/, ai/).
//!
//! Each YAML file defines:
//! - Metadata (name, version, builder_weight)
//! - Tools with version selection
//! - Optional builder stage (Dockerfile template)
//! - Runtime commands (Dockerfile template)
//!
//! Templates use minijinja syntax with `tools.<name>.version` and
//! `tools.<name>.enabled` as context variables.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::addon_registry::{ToolConfig, ToolDef};

// ---------------------------------------------------------------------------
// YAML data model
// ---------------------------------------------------------------------------

/// Deserialized from a single addon YAML file.
#[derive(Debug, Deserialize)]
pub struct AddonYaml {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub builder_weight: Option<String>,
    #[serde(default)]
    pub tools: Vec<ToolYaml>,
    #[serde(default)]
    pub builder: Option<String>,
    #[serde(default)]
    pub runtime: Option<String>,
}

/// A tool entry in the YAML file.
#[derive(Debug, Deserialize)]
pub struct ToolYaml {
    pub name: String,
    #[serde(default = "default_true")]
    pub default_enabled: bool,
    #[serde(default)]
    pub default_version: Option<String>,
    #[serde(default)]
    pub supported_versions: Vec<String>,
}

fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Loaded addon storage
// ---------------------------------------------------------------------------

/// A fully loaded addon with owned data (not static references).
#[derive(Debug)]
pub struct LoadedAddon {
    pub name: String,
    pub addon_version: String,
    pub builder_weight: Option<String>,
    pub tools: Vec<LoadedTool>,
    pub builder_template: Option<String>,
    pub runtime_template: Option<String>,
}

#[derive(Debug)]
pub struct LoadedTool {
    pub name: String,
    pub default_enabled: bool,
    pub default_version: String,
    pub supported_versions: Vec<String>,
}

/// Global addon store, initialized once.
static ADDONS: OnceLock<Vec<LoadedAddon>> = OnceLock::new();

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Get the addons directory path.
/// Checks `AIBOX_ADDONS_DIR` env var first, then falls back to XDG config.
pub fn addons_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("AIBOX_ADDONS_DIR") {
        return Ok(PathBuf::from(dir));
    }
    crate::dirs::config_dir()
        .map(|d| d.join("addons"))
        .ok_or_else(|| anyhow::anyhow!("Could not determine XDG config directory"))
}

/// Load all addon YAML files from the addons directory.
/// Walks subdirectories (languages/, tools/, docs/, ai/).
fn load_from_dir(dir: &Path) -> Result<Vec<LoadedAddon>> {
    if !dir.exists() {
        bail!(
            "Addon definitions not found at {}\n\
             Run the install script to set them up:\n\
             curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash",
            dir.display()
        );
    }

    let mut addons = Vec::new();

    // Walk subdirectories
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Category subdirectory — read YAML files inside
            for file_entry in fs::read_dir(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?
            {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                if file_path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                    let addon = load_yaml_file(&file_path)?;
                    addons.push(addon);
                }
            }
        } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            // Top-level YAML file (for flexibility)
            let addon = load_yaml_file(&path)?;
            addons.push(addon);
        }
    }

    if addons.is_empty() {
        bail!(
            "No addon YAML files found in {}\n\
             The directory exists but contains no .yaml files.",
            dir.display()
        );
    }

    // Sort by name for consistent ordering
    addons.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(addons)
}

/// Parse a single YAML file into a LoadedAddon.
fn load_yaml_file(path: &Path) -> Result<LoadedAddon> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read addon file: {}", path.display()))?;
    let yaml: AddonYaml = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse addon YAML: {}", path.display()))?;

    Ok(LoadedAddon {
        name: yaml.name,
        addon_version: yaml.version,
        builder_weight: yaml.builder_weight,
        tools: yaml
            .tools
            .into_iter()
            .map(|t| LoadedTool {
                name: t.name,
                default_enabled: t.default_enabled,
                default_version: t.default_version.unwrap_or_default(),
                supported_versions: t.supported_versions,
            })
            .collect(),
        builder_template: yaml.builder,
        runtime_template: yaml.runtime,
    })
}

// ---------------------------------------------------------------------------
// Global access
// ---------------------------------------------------------------------------

/// Initialize the addon store from the default XDG path. Call once at startup.
pub fn init() -> Result<()> {
    let dir = addons_dir()?;
    init_from_dir(&dir)
}

/// Initialize the addon store from a specific directory.
/// Used by tests to point at the repo's addons/ directory.
pub fn init_from_dir(dir: &Path) -> Result<()> {
    let addons = load_from_dir(dir)?;
    ADDONS
        .set(addons)
        .map_err(|_| anyhow::anyhow!("Addon store already initialized"))?;
    Ok(())
}

/// Get all loaded addons.
pub fn all_addons() -> &'static [LoadedAddon] {
    ADDONS.get().map(|v| v.as_slice()).unwrap_or(&[])
}

/// Find an addon by name.
pub fn get_addon(name: &str) -> Option<&'static LoadedAddon> {
    all_addons().iter().find(|a| a.name == name)
}

// ---------------------------------------------------------------------------
// Conversion to legacy types (for backward compat with addons.rs)
// ---------------------------------------------------------------------------

impl LoadedAddon {
    /// Convert to an AddonDef for backward compatibility.
    /// Note: returns owned ToolDefs, not static references.
    pub fn to_tool_defs(&self) -> Vec<ToolDef> {
        self.tools
            .iter()
            .map(|t| ToolDef {
                name: leak_str(&t.name),
                default_enabled: t.default_enabled,
                supported_versions: leak_str_slice(&t.supported_versions),
                default_version: leak_str(&t.default_version),
            })
            .collect()
    }

    /// Builder weight as a numeric sort key for ordering.
    /// heavy=0, medium=1, light=2, none=3
    pub fn builder_order_key(&self) -> usize {
        match self.builder_weight.as_deref() {
            Some("heavy") => 0,
            Some("medium") => 1,
            Some("light") => 2,
            _ => 3,
        }
    }
}

// Leak strings to get &'static str — these live for the program's lifetime
// since addons are loaded once at startup via OnceLock.
pub fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn leak_str_slice(v: &[String]) -> &'static [&'static str] {
    let leaked: Vec<&'static str> = v.iter().map(|s| leak_str(s)).collect();
    Box::leak(leaked.into_boxed_slice())
}

// ---------------------------------------------------------------------------
// Template rendering
// ---------------------------------------------------------------------------

/// Build the minijinja context for an addon's templates.
fn build_template_context(
    addon: &LoadedAddon,
    tools: &HashMap<String, ToolConfig>,
) -> minijinja::Value {
    let mut tool_map = HashMap::new();

    for tool_def in &addon.tools {
        let enabled = tools
            .get(&tool_def.name)
            .is_some_and(|t| t.enabled);
        let version = tools
            .get(&tool_def.name)
            .and_then(|t| {
                if t.version.is_empty() {
                    None
                } else {
                    Some(t.version.clone())
                }
            })
            .unwrap_or_else(|| tool_def.default_version.clone());

        let mut entry = HashMap::new();
        entry.insert("enabled".to_string(), minijinja::Value::from(enabled));
        entry.insert("version".to_string(), minijinja::Value::from(version));
        tool_map.insert(tool_def.name.clone(), minijinja::Value::from_serialize(&entry));
    }

    minijinja::Value::from_serialize(HashMap::from([("tools", tool_map)]))
}

/// Render the builder stage template for an addon. Returns None if no builder.
pub fn render_builder(
    addon: &LoadedAddon,
    tools: &HashMap<String, ToolConfig>,
) -> Result<Option<String>> {
    let template_str = match &addon.builder_template {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut env = minijinja::Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.add_template("builder", template_str)
        .with_context(|| format!("Invalid builder template for addon '{}'", addon.name))?;

    let tmpl = env.get_template("builder").unwrap();
    let ctx = build_template_context(addon, tools);
    let rendered = tmpl
        .render(&ctx)
        .with_context(|| format!("Failed to render builder template for addon '{}'", addon.name))?;

    Ok(Some(rendered))
}

/// Render the runtime commands template for an addon.
pub fn render_runtime(
    addon: &LoadedAddon,
    tools: &HashMap<String, ToolConfig>,
) -> Result<String> {
    let template_str = match &addon.runtime_template {
        Some(t) => t,
        None => return Ok(String::new()),
    };

    let mut env = minijinja::Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.add_template("runtime", template_str)
        .with_context(|| format!("Invalid runtime template for addon '{}'", addon.name))?;

    let tmpl = env.get_template("runtime").unwrap();
    let ctx = build_template_context(addon, tools);
    let rendered = tmpl
        .render(&ctx)
        .with_context(|| format!("Failed to render runtime template for addon '{}'", addon.name))?;

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_yaml(dir: &Path, category: &str, name: &str, content: &str) {
        let cat_dir = dir.join(category);
        fs::create_dir_all(&cat_dir).unwrap();
        let mut f = fs::File::create(cat_dir.join(format!("{}.yaml", name))).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn load_from_dir_finds_yaml_files() {
        let dir = tempfile::tempdir().unwrap();
        write_test_yaml(
            dir.path(),
            "languages",
            "test-addon",
            r#"
name: test-addon
version: "1.0.0"
tools:
  - name: test-tool
    default_enabled: true
    default_version: "1.0"
    supported_versions: ["1.0", "2.0"]
runtime: |
  RUN echo "hello"
"#,
        );

        let addons = load_from_dir(dir.path()).unwrap();
        assert_eq!(addons.len(), 1);
        assert_eq!(addons[0].name, "test-addon");
        assert_eq!(addons[0].tools.len(), 1);
        assert_eq!(addons[0].tools[0].name, "test-tool");
        assert_eq!(addons[0].tools[0].default_version, "1.0");
    }

    #[test]
    fn render_runtime_substitutes_versions() {
        let addon = LoadedAddon {
            name: "test".to_string(),
            addon_version: "1.0.0".to_string(),
            builder_weight: None,
            tools: vec![LoadedTool {
                name: "mytool".to_string(),
                default_enabled: true,
                default_version: "3.0".to_string(),
                supported_versions: vec!["3.0".to_string()],
            }],
            builder_template: None,
            runtime_template: Some("RUN install mytool={{ tools.mytool.version }}".to_string()),
        };

        let mut tools = HashMap::new();
        tools.insert(
            "mytool".to_string(),
            ToolConfig {
                enabled: true,
                version: "3.0".to_string(),
            },
        );

        let result = render_runtime(&addon, &tools).unwrap();
        assert!(result.contains("mytool=3.0"), "got: {}", result);
    }

    #[test]
    fn render_runtime_handles_conditionals() {
        let addon = LoadedAddon {
            name: "test".to_string(),
            addon_version: "1.0.0".to_string(),
            builder_weight: None,
            tools: vec![
                LoadedTool {
                    name: "required".to_string(),
                    default_enabled: true,
                    default_version: "1.0".to_string(),
                    supported_versions: vec![],
                },
                LoadedTool {
                    name: "optional".to_string(),
                    default_enabled: false,
                    default_version: "2.0".to_string(),
                    supported_versions: vec![],
                },
            ],
            builder_template: None,
            runtime_template: Some(
                "RUN install required\n\
                 {% if tools.optional.enabled %}RUN install optional{% endif %}"
                    .to_string(),
            ),
        };

        // Only required is enabled
        let mut tools = HashMap::new();
        tools.insert(
            "required".to_string(),
            ToolConfig {
                enabled: true,
                version: "1.0".to_string(),
            },
        );

        let result = render_runtime(&addon, &tools).unwrap();
        assert!(result.contains("install required"));
        assert!(!result.contains("install optional"));
    }

    #[test]
    fn builder_order_key_sorts_correctly() {
        let heavy = LoadedAddon {
            name: "a".to_string(),
            addon_version: "1.0.0".to_string(),
            builder_weight: Some("heavy".to_string()),
            tools: vec![],
            builder_template: Some("FROM debian".to_string()),
            runtime_template: None,
        };
        let medium = LoadedAddon {
            name: "b".to_string(),
            addon_version: "1.0.0".to_string(),
            builder_weight: Some("medium".to_string()),
            tools: vec![],
            builder_template: Some("FROM debian".to_string()),
            runtime_template: None,
        };
        let none = LoadedAddon {
            name: "c".to_string(),
            addon_version: "1.0.0".to_string(),
            builder_weight: None,
            tools: vec![],
            builder_template: None,
            runtime_template: None,
        };

        assert!(heavy.builder_order_key() < medium.builder_order_key());
        assert!(medium.builder_order_key() < none.builder_order_key());
    }

    #[test]
    fn missing_dir_gives_clear_error() {
        let result = load_from_dir(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("install script"), "error should mention install: {}", err);
    }
}
