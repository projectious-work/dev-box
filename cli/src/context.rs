use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::ProcessFlavor;
use crate::output;

// --- Minimal templates ---
const MINIMAL_CLAUDE_MD: &str = include_str!("../../templates/minimal/CLAUDE.md.template");

// --- Managed templates ---
const MANAGED_CLAUDE_MD: &str = include_str!("../../templates/managed/CLAUDE.md.template");
const MANAGED_DECISIONS: &str = include_str!("../../templates/managed/DECISIONS.md");
const MANAGED_BACKLOG: &str = include_str!("../../templates/managed/BACKLOG.md");
const MANAGED_STANDUPS: &str = include_str!("../../templates/managed/STANDUPS.md");
const MANAGED_GENERAL: &str = include_str!("../../templates/managed/work-instructions/GENERAL.md");

// --- Research templates ---
const RESEARCH_CLAUDE_MD: &str = include_str!("../../templates/research/CLAUDE.md.template");
const RESEARCH_PROGRESS: &str = include_str!("../../templates/research/PROGRESS.md");

// --- Product templates ---
const PRODUCT_CLAUDE_MD: &str = include_str!("../../templates/product/CLAUDE.md.template");
const PRODUCT_DECISIONS: &str = include_str!("../../templates/product/DECISIONS.md");
const PRODUCT_BACKLOG: &str = include_str!("../../templates/product/BACKLOG.md");
const PRODUCT_STANDUPS: &str = include_str!("../../templates/product/STANDUPS.md");
const PRODUCT_PROJECTS: &str = include_str!("../../templates/product/PROJECTS.md");
const PRODUCT_PRD: &str = include_str!("../../templates/product/PRD.md");
const PRODUCT_GENERAL: &str =
    include_str!("../../templates/product/work-instructions/GENERAL.md");
const PRODUCT_DEVELOPMENT: &str =
    include_str!("../../templates/product/work-instructions/DEVELOPMENT.md");
const PRODUCT_TEAM: &str = include_str!("../../templates/product/work-instructions/TEAM.md");

/// Default OWNER.md placeholder content.
const OWNER_PLACEHOLDER: &str = r#"# Owner Profile

This file describes the project owner and is shared across projects.

To share this file across projects, create it at:
  ~/.config/dev-box/OWNER.md

Then re-run `dev-box init` — it will be symlinked automatically.

## About

- **Name:**
- **Role:**
- **Contact:**

## Preferences

- **Communication style:**
- **Code style preferences:**
- **Review preferences:**
"#;

/// Scaffold the context/ directory based on the chosen work process flavor.
///
/// - Creates context/ directory and populates it with template files
/// - Creates CLAUDE.md at project root from the template
/// - Replaces {{project_name}} placeholders with the actual project name
/// - Creates .dev-box-version file
/// - Updates .gitignore with generated file entries
pub fn scaffold_context(process: &ProcessFlavor, project_name: &str) -> Result<()> {
    output::info(&format!(
        "Scaffolding context for '{}' process...",
        process
    ));

    match process {
        ProcessFlavor::Minimal => scaffold_minimal(project_name)?,
        ProcessFlavor::Managed => scaffold_managed(project_name)?,
        ProcessFlavor::Research => scaffold_research(project_name)?,
        ProcessFlavor::Product => scaffold_product(project_name)?,
    }

    // Create .dev-box-version
    write_if_missing(
        Path::new(".dev-box-version"),
        env!("CARGO_PKG_VERSION"),
    )?;
    output::ok("Created .dev-box-version");

    // Update .gitignore
    update_gitignore()?;

    output::ok(&format!(
        "Context scaffolded ({} process)",
        process
    ));
    Ok(())
}

/// Scaffold minimal process: just CLAUDE.md at root, no context/ directory.
fn scaffold_minimal(project_name: &str) -> Result<()> {
    let claude_md = render(MINIMAL_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");
    Ok(())
}

/// Scaffold managed process.
fn scaffold_managed(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("work-instructions"))
        .context("Failed to create context/work-instructions")?;

    // CLAUDE.md at root
    let claude_md = render(MANAGED_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("DECISIONS.md"), MANAGED_DECISIONS)?;
    output::ok("Created context/DECISIONS.md");

    write_if_missing(&context.join("BACKLOG.md"), MANAGED_BACKLOG)?;
    output::ok("Created context/BACKLOG.md");

    write_if_missing(&context.join("STANDUPS.md"), MANAGED_STANDUPS)?;
    output::ok("Created context/STANDUPS.md");

    write_if_missing(
        &context.join("work-instructions").join("GENERAL.md"),
        MANAGED_GENERAL,
    )?;
    output::ok("Created context/work-instructions/GENERAL.md");

    // OWNER.md (symlink or placeholder)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold research process.
fn scaffold_research(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("research"))
        .context("Failed to create context/research")?;
    fs::create_dir_all(context.join("analysis"))
        .context("Failed to create context/analysis")?;

    // CLAUDE.md at root
    let claude_md = render(RESEARCH_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("PROGRESS.md"), RESEARCH_PROGRESS)?;
    output::ok("Created context/PROGRESS.md");

    // .gitkeep for empty dirs
    write_if_missing(&context.join("research").join(".gitkeep"), "")?;
    write_if_missing(&context.join("analysis").join(".gitkeep"), "")?;
    output::ok("Created context/research/ and context/analysis/");

    // OWNER.md (symlink or placeholder)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold product process (full set).
fn scaffold_product(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("work-instructions"))
        .context("Failed to create context/work-instructions")?;
    fs::create_dir_all(context.join("project-notes"))
        .context("Failed to create context/project-notes")?;
    fs::create_dir_all(context.join("ideas"))
        .context("Failed to create context/ideas")?;

    // CLAUDE.md at root
    let claude_md = render(PRODUCT_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("DECISIONS.md"), PRODUCT_DECISIONS)?;
    output::ok("Created context/DECISIONS.md");

    write_if_missing(&context.join("BACKLOG.md"), PRODUCT_BACKLOG)?;
    output::ok("Created context/BACKLOG.md");

    write_if_missing(&context.join("STANDUPS.md"), PRODUCT_STANDUPS)?;
    output::ok("Created context/STANDUPS.md");

    write_if_missing(&context.join("PROJECTS.md"), PRODUCT_PROJECTS)?;
    output::ok("Created context/PROJECTS.md");

    write_if_missing(&context.join("PRD.md"), PRODUCT_PRD)?;
    output::ok("Created context/PRD.md");

    write_if_missing(
        &context.join("work-instructions").join("GENERAL.md"),
        PRODUCT_GENERAL,
    )?;
    output::ok("Created context/work-instructions/GENERAL.md");

    write_if_missing(
        &context.join("work-instructions").join("DEVELOPMENT.md"),
        PRODUCT_DEVELOPMENT,
    )?;
    output::ok("Created context/work-instructions/DEVELOPMENT.md");

    write_if_missing(
        &context.join("work-instructions").join("TEAM.md"),
        PRODUCT_TEAM,
    )?;
    output::ok("Created context/work-instructions/TEAM.md");

    // .gitkeep for empty dirs
    write_if_missing(&context.join("project-notes").join(".gitkeep"), "")?;
    write_if_missing(&context.join("ideas").join(".gitkeep"), "")?;
    output::ok("Created context/project-notes/ and context/ideas/");

    // OWNER.md (symlink or placeholder)
    setup_owner_md(context)?;

    Ok(())
}

/// Set up OWNER.md: symlink from ~/.config/dev-box/OWNER.md if it exists,
/// otherwise create a placeholder with instructions.
fn setup_owner_md(context: &Path) -> Result<()> {
    let owner_path = context.join("OWNER.md");
    if owner_path.exists() || owner_path.symlink_metadata().is_ok() {
        tracing::debug!("OWNER.md already exists, skipping");
        return Ok(());
    }

    let global_owner = dirs::config_dir()
        .map(|d| d.join("dev-box").join("OWNER.md"));

    if let Some(ref global) = global_owner
        && global.exists()
    {
        std::os::unix::fs::symlink(global, &owner_path)
            .with_context(|| {
                format!(
                    "Failed to symlink {} -> {}",
                    owner_path.display(),
                    global.display()
                )
            })?;
        output::ok(&format!(
            "Symlinked context/OWNER.md -> {}",
            global.display()
        ));
        return Ok(());
    }

    // No global OWNER.md found — create placeholder
    fs::write(&owner_path, OWNER_PLACEHOLDER)
        .with_context(|| format!("Failed to write {}", owner_path.display()))?;
    output::ok("Created context/OWNER.md (placeholder)");
    output::info("Tip: create ~/.config/dev-box/OWNER.md to share your profile across projects");

    Ok(())
}

/// Returns the list of expected context files for a given process flavor.
pub fn expected_context_files(process: &ProcessFlavor) -> Vec<&'static str> {
    match process {
        ProcessFlavor::Minimal => vec!["CLAUDE.md"],
        ProcessFlavor::Managed => vec![
            "CLAUDE.md",
            "context/OWNER.md",
            "context/DECISIONS.md",
            "context/BACKLOG.md",
            "context/STANDUPS.md",
            "context/work-instructions/GENERAL.md",
        ],
        ProcessFlavor::Research => vec![
            "CLAUDE.md",
            "context/OWNER.md",
            "context/PROGRESS.md",
            "context/research/.gitkeep",
            "context/analysis/.gitkeep",
        ],
        ProcessFlavor::Product => vec![
            "CLAUDE.md",
            "context/OWNER.md",
            "context/DECISIONS.md",
            "context/BACKLOG.md",
            "context/STANDUPS.md",
            "context/PROJECTS.md",
            "context/PRD.md",
            "context/work-instructions/GENERAL.md",
            "context/work-instructions/DEVELOPMENT.md",
            "context/work-instructions/TEAM.md",
            "context/project-notes/.gitkeep",
            "context/ideas/.gitkeep",
        ],
    }
}

/// Replace {{project_name}} in template content.
pub(crate) fn render(template: &str, project_name: &str) -> String {
    template.replace("{{project_name}}", project_name)
}

/// Write content to a file only if it doesn't already exist.
pub(crate) fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        tracing::debug!("Skipping existing file: {}", path.display());
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write: {}", path.display()))?;
    Ok(())
}

/// Ensure .gitignore contains entries for generated devcontainer files.
pub(crate) fn update_gitignore() -> Result<()> {
    let gitignore_path = Path::new(".gitignore");
    let required_entries = [
        "# dev-box generated",
        crate::config::DOCKERFILE,
        crate::config::COMPOSE_FILE,
        crate::config::DEVCONTAINER_JSON,
        ".root/",
        ".dev-box-version",
    ];

    let existing = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path)
            .context("Failed to read .gitignore")?
    } else {
        String::new()
    };

    let mut additions = Vec::new();
    for entry in &required_entries {
        if !existing.contains(entry) {
            additions.push(*entry);
        }
    }

    if additions.is_empty() {
        return Ok(());
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    if !content.is_empty() {
        content.push('\n');
    }

    content.push_str(&additions.join("\n"));
    content.push('\n');

    fs::write(gitignore_path, content)
        .context("Failed to write .gitignore")?;
    output::ok("Updated .gitignore with dev-box entries");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Helper to run a closure inside a temp directory, restoring the original
    /// cwd afterwards (best-effort).
    fn in_temp_dir<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        f();
        // Restore — ignore errors (dir may be deleted)
        let _ = std::env::set_current_dir(&original);
    }

    #[test]
    fn render_replaces_project_name() {
        let result = render("Hello {{project_name}}!", "my-app");
        assert_eq!(result, "Hello my-app!");
    }

    #[test]
    fn render_replaces_multiple_occurrences() {
        let result = render("{{project_name}} is {{project_name}}", "foo");
        assert_eq!(result, "foo is foo");
    }

    #[test]
    fn write_if_missing_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        write_if_missing(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn write_if_missing_does_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();
        write_if_missing(&path, "new content").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    }

    #[test]
    fn write_if_missing_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("c.txt");
        write_if_missing(&path, "deep").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "deep");
    }

    #[test]
    #[serial]
    fn scaffold_minimal_creates_claude_md_and_version() {
        in_temp_dir(|| {
            scaffold_context(&ProcessFlavor::Minimal, "test-proj").unwrap();
            assert!(Path::new("CLAUDE.md").exists(), "CLAUDE.md should exist");
            assert!(Path::new(".dev-box-version").exists(), ".dev-box-version should exist");
            // Minimal should NOT create context/ directory
            // (it only creates CLAUDE.md at root)
        });
    }

    #[test]
    #[serial]
    fn scaffold_managed_creates_expected_files() {
        in_temp_dir(|| {
            scaffold_context(&ProcessFlavor::Managed, "test-proj").unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new(".dev-box-version").exists());
            assert!(Path::new("context/DECISIONS.md").exists());
            assert!(Path::new("context/BACKLOG.md").exists());
            assert!(Path::new("context/STANDUPS.md").exists());
            assert!(Path::new("context/work-instructions/GENERAL.md").exists());
            assert!(Path::new("context/OWNER.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_research_creates_expected_files() {
        in_temp_dir(|| {
            scaffold_context(&ProcessFlavor::Research, "test-proj").unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new("context/PROGRESS.md").exists());
            assert!(Path::new("context/research/.gitkeep").exists());
            assert!(Path::new("context/analysis/.gitkeep").exists());
            assert!(Path::new("context/OWNER.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_product_creates_all_expected_files() {
        in_temp_dir(|| {
            scaffold_context(&ProcessFlavor::Product, "test-proj").unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new(".dev-box-version").exists());
            assert!(Path::new("context/DECISIONS.md").exists());
            assert!(Path::new("context/BACKLOG.md").exists());
            assert!(Path::new("context/STANDUPS.md").exists());
            assert!(Path::new("context/PROJECTS.md").exists());
            assert!(Path::new("context/PRD.md").exists());
            assert!(Path::new("context/work-instructions/GENERAL.md").exists());
            assert!(Path::new("context/work-instructions/DEVELOPMENT.md").exists());
            assert!(Path::new("context/work-instructions/TEAM.md").exists());
            assert!(Path::new("context/project-notes/.gitkeep").exists());
            assert!(Path::new("context/ideas/.gitkeep").exists());
            assert!(Path::new("context/OWNER.md").exists());
        });
    }

    #[test]
    #[serial]
    fn claude_md_contains_project_name() {
        in_temp_dir(|| {
            scaffold_context(&ProcessFlavor::Minimal, "awesome-project").unwrap();
            let content = fs::read_to_string("CLAUDE.md").unwrap();
            assert!(content.contains("awesome-project"), "CLAUDE.md should contain project name");
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_adds_entries() {
        in_temp_dir(|| {
            update_gitignore().unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains(".devcontainer/Dockerfile"));
            assert!(content.contains(".devcontainer/docker-compose.yml"));
            assert!(content.contains(".devcontainer/devcontainer.json"));
            assert!(content.contains(".root/"));
            assert!(content.contains(".dev-box-version"));
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_idempotent() {
        in_temp_dir(|| {
            update_gitignore().unwrap();
            let first = fs::read_to_string(".gitignore").unwrap();
            update_gitignore().unwrap();
            let second = fs::read_to_string(".gitignore").unwrap();
            assert_eq!(first, second, ".gitignore should not change on second run");
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_preserves_existing_content() {
        in_temp_dir(|| {
            fs::write(".gitignore", "node_modules/\n*.log\n").unwrap();
            update_gitignore().unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("node_modules/"));
            assert!(content.contains("*.log"));
            assert!(content.contains(".root/"));
        });
    }
}
