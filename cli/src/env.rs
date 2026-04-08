use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::OutputFormat;
use crate::config::AiboxConfig;
use crate::generate;
use crate::output;
use crate::reset;

/// Environment storage directory.
const ENV_DIR: &str = ".aibox-env";
/// State file tracking current environment.
const STATE_FILE: &str = "state.toml";

/// Per-environment files to save/restore. Includes the canonical
/// `AGENTS.md` (owned by processkit, but `write_if_missing` lets users
/// hand-customize it; envs should snapshot those customizations) and
/// the thin-pointer `CLAUDE.md`.
const ENV_FILES: &[&str] = &["aibox.toml", "AGENTS.md", "CLAUDE.md"];
/// Per-environment directory (excluding shared/).
const ENV_CONTEXT_DIR: &str = "context";
/// Shared subdirectory name — excluded from env copy.
const SHARED_DIR: &str = "shared";

// =============================================================================
// State management
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Default)]
struct EnvState {
    current: Option<String>,
}

fn state_path() -> PathBuf {
    PathBuf::from(ENV_DIR).join(STATE_FILE)
}

fn load_state() -> Result<EnvState> {
    let path = state_path();
    if !path.exists() {
        return Ok(EnvState::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let state: EnvState = toml::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(state)
}

fn save_state(state: &EnvState) -> Result<()> {
    let path = state_path();
    fs::create_dir_all(ENV_DIR).context("Failed to create .aibox-env directory")?;
    let content = toml::to_string_pretty(state).context("Failed to serialize state")?;
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

// =============================================================================
// Environment save/restore
// =============================================================================

/// List existing environment names from .aibox-env/ subdirectories.
fn list_env_names() -> Result<Vec<String>> {
    let env_dir = PathBuf::from(ENV_DIR);
    if !env_dir.exists() {
        return Ok(vec![]);
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&env_dir).context("Failed to read .aibox-env/")? {
        let entry = entry?;
        if entry.path().is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// Save current project state to a named environment slot.
fn save_env(name: &str) -> Result<()> {
    let env_path = PathBuf::from(ENV_DIR).join(name);
    fs::create_dir_all(&env_path)
        .with_context(|| format!("Failed to create env dir: {}", env_path.display()))?;

    // Copy individual files
    for file in ENV_FILES {
        let src = PathBuf::from(file);
        if src.exists() {
            let dst = env_path.join(file);
            reset::copy_item(&src, &dst)?;
        }
    }

    // Copy context/ excluding shared/
    let context_src = PathBuf::from(ENV_CONTEXT_DIR);
    if context_src.exists() {
        let context_dst = env_path.join(ENV_CONTEXT_DIR);
        copy_context_excluding_shared(&context_src, &context_dst)?;
    }

    Ok(())
}

/// Restore a named environment to the project root.
fn restore_env(name: &str) -> Result<()> {
    let env_path = PathBuf::from(ENV_DIR).join(name);
    if !env_path.exists() {
        bail!("Environment '{}' not found", name);
    }

    // Restore individual files
    for file in ENV_FILES {
        let src = env_path.join(file);
        if src.exists() {
            let dst = PathBuf::from(file);
            reset::copy_item(&src, &dst)?;
        }
    }

    // Restore context/ — delete env-specific content first, then copy back
    let context_dst = PathBuf::from(ENV_CONTEXT_DIR);
    if context_dst.exists() {
        delete_context_excluding_shared(&context_dst)?;
    }
    let context_src = env_path.join(ENV_CONTEXT_DIR);
    if context_src.exists() {
        // Ensure context/ exists (shared/ might be the only thing there)
        fs::create_dir_all(&context_dst).context("Failed to create context/")?;
        // Copy all saved env-specific content back
        for entry in fs::read_dir(&context_src).context("Failed to read saved context")? {
            let entry = entry?;
            let dst_path = context_dst.join(entry.file_name());
            reset::copy_item(&entry.path(), &dst_path)?;
        }
    }

    Ok(())
}

/// Copy context/ directory excluding the shared/ subdirectory.
fn copy_context_excluding_shared(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create {}", dst.display()))?;

    for entry in fs::read_dir(src)
        .with_context(|| format!("Failed to read {}", src.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        // Skip shared/
        if name == SHARED_DIR {
            continue;
        }
        let dst_path = dst.join(&name);
        reset::copy_item(&entry.path(), &dst_path)?;
    }
    Ok(())
}

/// Delete all entries in context/ except shared/.
fn delete_context_excluding_shared(context_dir: &Path) -> Result<()> {
    if !context_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(context_dir)
        .with_context(|| format!("Failed to read {}", context_dir.display()))?
    {
        let entry = entry?;
        if entry.file_name() == SHARED_DIR {
            continue;
        }
        reset::delete_item(&entry.path())?;
    }
    Ok(())
}

/// Validate environment name.
fn validate_env_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Environment name cannot be empty");
    }
    if name == SHARED_DIR {
        bail!("'{}' is reserved", SHARED_DIR);
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("Environment name must contain only alphanumeric characters, hyphens, and underscores");
    }
    Ok(())
}

// =============================================================================
// Public commands
// =============================================================================

/// Create a named environment from the current project state.
pub fn cmd_env_create(config_path: &Option<String>, name: &str) -> Result<()> {
    // Validate config exists
    let _config = AiboxConfig::from_cli_option(config_path)?;
    validate_env_name(name)?;

    let env_path = PathBuf::from(ENV_DIR).join(name);
    if env_path.exists() {
        bail!(
            "Environment '{}' already exists. Delete it first with: aibox env delete {}",
            name,
            name
        );
    }

    output::info(&format!("Creating environment '{}'...", name));
    save_env(name)?;

    // Update state
    let mut state = load_state()?;
    state.current = Some(name.to_string());
    save_state(&state)?;

    output::ok(&format!(
        "Environment '{}' created and set as current",
        name
    ));
    Ok(())
}

/// Switch to a named environment.
pub fn cmd_env_switch(config_path: &Option<String>, name: &str, yes: bool) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    validate_env_name(name)?;

    let env_path = PathBuf::from(ENV_DIR).join(name);
    if !env_path.exists() {
        bail!(
            "Environment '{}' not found. Available: {}",
            name,
            list_env_names()?.join(", ")
        );
    }

    let state = load_state()?;
    if state.current.as_deref() == Some(name) {
        output::info(&format!("Already on environment '{}'", name));
        return Ok(());
    }

    let current_name = state.current.as_deref().unwrap_or("(unnamed)");
    output::info(&format!(
        "Switching from '{}' to '{}'",
        current_name, name
    ));

    if !yes {
        let prompt = format!(
            "  This will save '{}' and restore '{}'. Container will be stopped.",
            current_name, name
        );
        if !reset::confirm(&prompt, "switch")? {
            output::warn("Aborted.");
            return Ok(());
        }
    }

    // Stop container
    reset::ensure_container_stopped(&config)?;

    // Save current state
    if let Some(ref current) = state.current {
        output::info(&format!("Saving current environment '{}'...", current));
        save_env(current)?;
        output::ok(&format!("Saved '{}'", current));
    }

    // Restore target
    output::info(&format!("Restoring environment '{}'...", name));
    restore_env(name)?;
    output::ok(&format!("Restored '{}'", name));

    // Update state
    let new_state = EnvState {
        current: Some(name.to_string()),
    };
    save_state(&new_state)?;

    // Regenerate .devcontainer/ from restored config
    let restored_config = AiboxConfig::from_cli_option(config_path)?;
    generate::generate_all(&restored_config)?;

    output::ok(&format!("Switched to environment '{}'", name));
    output::info("Run 'aibox build' then 'aibox start' to apply changes.");

    Ok(())
}

/// List available environments.
pub fn cmd_env_list(format: OutputFormat) -> Result<()> {
    #[derive(Serialize)]
    struct Row {
        name: String,
        current: bool,
    }

    let names = list_env_names()?;
    let state = load_state()?;

    let rows: Vec<Row> = names
        .into_iter()
        .map(|name| {
            let current = state.current.as_deref() == Some(name.as_str());
            Row { name, current }
        })
        .collect();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        OutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&rows)?);
        }
        OutputFormat::Table => {
            if rows.is_empty() {
                output::info("No environments created yet.");
                output::info("Create one with: aibox env create <name>");
                return Ok(());
            }
            eprintln!("\n  {:<25} Status", "Environment");
            eprintln!("  {}", "-".repeat(40));
            for r in &rows {
                let marker = if r.current { "\x1b[32m\u{25cf} current\x1b[0m" } else { "" };
                eprintln!("  {:<25} {}", r.name, marker);
            }
            eprintln!();
        }
    }

    Ok(())
}

/// Delete a named environment.
pub fn cmd_env_delete(name: &str, yes: bool) -> Result<()> {
    validate_env_name(name)?;

    let env_path = PathBuf::from(ENV_DIR).join(name);
    if !env_path.exists() {
        bail!("Environment '{}' not found", name);
    }

    let state = load_state()?;
    let is_current = state.current.as_deref() == Some(name);

    if is_current {
        output::warn(&format!(
            "Environment '{}' is the current environment.",
            name
        ));
    }

    if !yes {
        let prompt = format!("  This will permanently delete environment '{}'.", name);
        if !reset::confirm(&prompt, name)? {
            output::warn("Aborted.");
            return Ok(());
        }
    }

    reset::delete_item(&env_path)?;
    output::ok(&format!("Deleted environment '{}'", name));

    // Clear current if we deleted it
    if is_current {
        let new_state = EnvState { current: None };
        save_state(&new_state)?;
        output::warn("No current environment set. Create or switch to one.");
    }

    Ok(())
}

/// Show current environment status.
pub fn cmd_env_status(config_path: &Option<String>) -> Result<()> {
    let state = load_state()?;
    let names = list_env_names()?;

    match &state.current {
        Some(name) => {
            output::ok(&format!("Current environment: {}", name));

            // Show config summary if available
            if let Ok(config) = AiboxConfig::from_cli_option(config_path) {
                eprintln!("  Base:    {}", config.aibox.base);
                eprintln!("  Packages: {:?}", config.context.packages);
                eprintln!("  Version: {}", config.aibox.version);
            }
        }
        None => {
            output::info("No current environment set.");
        }
    }

    if !names.is_empty() {
        eprintln!(
            "  Available environments: {}",
            names.join(", ")
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project(dir: &Path) {
        fs::write(
            dir.join("aibox.toml"),
            r#"[aibox]
version = "0.3.9"
image = "python"
process = "research"

[container]
name = "test-project"
"#,
        )
        .unwrap();
        fs::write(dir.join("CLAUDE.md"), "# Test Project").unwrap();
        fs::create_dir_all(dir.join("context/shared")).unwrap();
        fs::write(dir.join("context/shared/OWNER.md"), "# Owner").unwrap();
        fs::create_dir_all(dir.join("context/research")).unwrap();
        fs::write(dir.join("context/PROGRESS.md"), "# Progress").unwrap();
        fs::write(
            dir.join("context/research/notes.md"),
            "# Research notes",
        )
        .unwrap();
    }

    #[test]
    fn validate_env_name_rejects_empty() {
        assert!(validate_env_name("").is_err());
    }

    #[test]
    fn validate_env_name_rejects_shared() {
        assert!(validate_env_name("shared").is_err());
    }

    #[test]
    fn validate_env_name_rejects_special_chars() {
        assert!(validate_env_name("my env").is_err());
        assert!(validate_env_name("my/env").is_err());
        assert!(validate_env_name("my.env").is_err());
    }

    #[test]
    fn validate_env_name_accepts_valid() {
        assert!(validate_env_name("research").is_ok());
        assert!(validate_env_name("my-env").is_ok());
        assert!(validate_env_name("env_1").is_ok());
        assert!(validate_env_name("Research2").is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn copy_context_excluding_shared_works() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        setup_project(dir.path());

        let dst = dir.path().join("backup-context");
        copy_context_excluding_shared(&dir.path().join("context"), &dst).unwrap();

        // shared/ should NOT be copied
        assert!(!dst.join("shared").exists());
        // But other files should be
        assert!(dst.join("PROGRESS.md").exists());
        assert!(dst.join("research/notes.md").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn save_and_restore_env() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        setup_project(dir.path());

        // Save as "research"
        save_env("research").unwrap();

        // Verify saved files
        let env_path = dir.path().join(ENV_DIR).join("research");
        assert!(env_path.join("aibox.toml").exists());
        assert!(env_path.join("CLAUDE.md").exists());
        assert!(env_path.join("context/PROGRESS.md").exists());
        assert!(!env_path.join("context/shared").exists()); // shared not copied

        // Modify project files
        fs::write(dir.path().join("CLAUDE.md"), "# Modified").unwrap();
        fs::write(dir.path().join("context/PROGRESS.md"), "# Modified progress").unwrap();

        // Restore "research"
        restore_env("research").unwrap();

        // Verify restored content
        let claude = fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert_eq!(claude, "# Test Project");
        let progress = fs::read_to_string(dir.path().join("context/PROGRESS.md")).unwrap();
        assert_eq!(progress, "# Progress");

        // shared/ should still be there
        assert!(dir.path().join("context/shared/OWNER.md").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn env_state_persistence() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let state = EnvState {
            current: Some("research".to_string()),
        };
        save_state(&state).unwrap();

        let loaded = load_state().unwrap();
        assert_eq!(loaded.current, Some("research".to_string()));

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn list_env_names_discovers_dirs() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        fs::create_dir_all(dir.path().join(ENV_DIR).join("alpha")).unwrap();
        fs::create_dir_all(dir.path().join(ENV_DIR).join("beta")).unwrap();
        // state.toml is a file, should not appear
        fs::write(
            dir.path().join(ENV_DIR).join(STATE_FILE),
            "current = \"alpha\"",
        )
        .unwrap();

        let names = list_env_names().unwrap();
        assert_eq!(names, vec!["alpha", "beta"]);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn delete_context_excluding_shared_preserves_shared() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        setup_project(dir.path());

        delete_context_excluding_shared(&dir.path().join("context")).unwrap();

        // shared/ should survive
        assert!(dir.path().join("context/shared/OWNER.md").exists());
        // Everything else should be gone
        assert!(!dir.path().join("context/PROGRESS.md").exists());
        assert!(!dir.path().join("context/research").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
