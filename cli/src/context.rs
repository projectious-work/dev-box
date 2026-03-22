use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::{DevBoxConfig, ImageFlavor, ProcessFlavor};
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
const RESEARCH_NOTE_TEMPLATE: &str = include_str!("../../templates/research/research-note.md");
const EXPERIMENTS_README: &str = include_str!("../../templates/research/experiments-README.md");

// --- Product templates ---
const PRODUCT_CLAUDE_MD: &str = include_str!("../../templates/product/CLAUDE.md.template");
const PRODUCT_DECISIONS: &str = include_str!("../../templates/product/DECISIONS.md");
const PRODUCT_BACKLOG: &str = include_str!("../../templates/product/BACKLOG.md");
const PRODUCT_STANDUPS: &str = include_str!("../../templates/product/STANDUPS.md");
const PRODUCT_PROJECTS: &str = include_str!("../../templates/product/PROJECTS.md");
const PRODUCT_PRD: &str = include_str!("../../templates/product/PRD.md");
const PRODUCT_GENERAL: &str = include_str!("../../templates/product/work-instructions/GENERAL.md");
const PRODUCT_DEVELOPMENT: &str =
    include_str!("../../templates/product/work-instructions/DEVELOPMENT.md");
const PRODUCT_TEAM: &str = include_str!("../../templates/product/work-instructions/TEAM.md");

// --- Process templates ---
const PROCESS_README: &str = include_str!("../../templates/processes/README.md");
const PROCESS_RELEASE: &str = include_str!("../../templates/processes/release.md");
const PROCESS_CODE_REVIEW: &str = include_str!("../../templates/processes/code-review.md");
const PROCESS_FEATURE_DEV: &str = include_str!("../../templates/processes/feature-development.md");
const PROCESS_BUG_FIX: &str = include_str!("../../templates/processes/bug-fix.md");

// --- Skill templates ---
const SKILL_BACKLOG_CONTEXT: &str =
    include_str!("../../templates/skills/backlog-context/SKILL.md");
const SKILL_DECISIONS_ADR: &str = include_str!("../../templates/skills/decisions-adr/SKILL.md");
const SKILL_STANDUP_CONTEXT: &str =
    include_str!("../../templates/skills/standup-context/SKILL.md");

/// Default OWNER.md content — created locally in each project's context/ directory.
const OWNER_CONTENT: &str = r#"# Owner Profile

This file describes the project owner. It helps AI agents understand who they
are working with and tailor their communication and technical approach accordingly.

## About

- **Name:**
- **Role:**
- **Contact:**

## Background

- **Domain expertise:** <!-- e.g., backend systems, data science, DevOps -->
- **Primary languages:** <!-- e.g., Python, Rust, TypeScript -->
- **Years of experience:**

## Preferences

- **Communication style:** <!-- e.g., concise and direct, detailed explanations -->
- **Communication language:** <!-- e.g., English, German, prefer English for code comments -->
- **Code style preferences:** <!-- e.g., minimal comments, explicit types, functional style -->
- **Review preferences:** <!-- e.g., prefer small PRs, want tests for every change -->

## Working Context

- **Timezone:** <!-- e.g., Europe/Berlin -->
- **Working hours:** <!-- e.g., 09:00-18:00 CET -->
- **Current focus:** <!-- e.g., migrating auth system, learning Kubernetes -->
"#;

/// Scaffold the context/ directory based on the chosen work process flavor.
///
/// - Creates context/ directory and populates it with template files
/// - Creates CLAUDE.md at project root from the template
/// - Replaces {{project_name}} placeholders with the actual project name
/// - Creates .dev-box-version file
/// - Updates .gitignore with generated file entries and language-specific blocks
pub fn scaffold_context(config: &DevBoxConfig) -> Result<()> {
    let process = &config.dev_box.process;
    let project_name = &config.container.name;
    let image = &config.dev_box.image;

    output::info(&format!("Scaffolding context for '{}' process...", process));

    match process {
        ProcessFlavor::Minimal => scaffold_minimal(project_name)?,
        ProcessFlavor::Managed => scaffold_managed(project_name)?,
        ProcessFlavor::Research => scaffold_research(project_name)?,
        ProcessFlavor::Product => scaffold_product(project_name)?,
    }

    // Create .dev-box-version
    write_if_missing(Path::new(".dev-box-version"), env!("CARGO_PKG_VERSION"))?;
    output::ok("Created .dev-box-version");

    // Update .gitignore with dev-box entries and language-specific blocks
    update_gitignore(image)?;

    // Create Dockerfile.local placeholder
    let local_dockerfile = Path::new(crate::config::DEVCONTAINER_DIR).join("Dockerfile.local");
    write_if_missing(
        &local_dockerfile,
        "# Project-specific Dockerfile layers.\n\
         # This file is appended to the generated Dockerfile by `dev-box sync`.\n\
         # It is never overwritten — you own this file.\n\
         #\n\
         # The generated base image is available as the \"dev-box\" stage:\n\
         #   FROM ghcr.io/projectious-work/dev-box:<image>-v<version> AS dev-box\n\
         #\n\
         # Simple usage — add layers directly:\n\
         #   RUN apt-get update && apt-get install -y some-package\n\
         #   RUN npx playwright install --with-deps chromium\n\
         #\n\
         # Advanced usage — multi-stage build referencing the dev-box stage:\n\
         #   FROM node:20 AS builder\n\
         #   RUN npm ci && npm run build\n\
         #\n\
         #   FROM dev-box\n\
         #   COPY --from=builder /app/dist /workspace/dist\n",
    )?;

    output::ok(&format!("Context scaffolded ({} process)", process));
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

    // Process declarations and skills
    scaffold_processes(context)?;
    scaffold_skills()?;

    // OWNER.md (local copy)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold research process.
fn scaffold_research(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("research")).context("Failed to create context/research")?;
    fs::create_dir_all(context.join("analysis")).context("Failed to create context/analysis")?;

    // CLAUDE.md at root
    let claude_md = render(RESEARCH_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("PROGRESS.md"), RESEARCH_PROGRESS)?;
    output::ok("Created context/PROGRESS.md");

    // Research note template
    write_if_missing(
        &context.join("research").join("_template.md"),
        RESEARCH_NOTE_TEMPLATE,
    )?;
    output::ok("Created context/research/_template.md");

    // .gitkeep for empty dirs
    write_if_missing(&context.join("analysis").join(".gitkeep"), "")?;
    output::ok("Created context/analysis/");

    // Experiments directory
    let experiments = Path::new("experiments");
    fs::create_dir_all(experiments).context("Failed to create experiments/")?;
    write_if_missing(&experiments.join("README.md"), EXPERIMENTS_README)?;
    output::ok("Created experiments/README.md");

    // Process declarations and skills
    scaffold_processes(context)?;
    scaffold_skills()?;

    // OWNER.md (local copy)
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
    fs::create_dir_all(context.join("ideas")).context("Failed to create context/ideas")?;

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

    // Research subfolder with template
    fs::create_dir_all(context.join("research"))
        .context("Failed to create context/research")?;
    write_if_missing(
        &context.join("research").join("_template.md"),
        RESEARCH_NOTE_TEMPLATE,
    )?;
    output::ok("Created context/research/_template.md");

    // Experiments directory
    let experiments = Path::new("experiments");
    fs::create_dir_all(experiments).context("Failed to create experiments/")?;
    write_if_missing(&experiments.join("README.md"), EXPERIMENTS_README)?;
    output::ok("Created experiments/README.md");

    // Process declarations and skills
    scaffold_processes(context)?;
    scaffold_skills()?;

    // OWNER.md (local copy)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold process declaration files into context/processes/.
fn scaffold_processes(context: &Path) -> Result<()> {
    let processes = context.join("processes");
    fs::create_dir_all(&processes).context("Failed to create context/processes")?;

    write_if_missing(&processes.join("README.md"), PROCESS_README)?;
    write_if_missing(&processes.join("release.md"), PROCESS_RELEASE)?;
    write_if_missing(&processes.join("code-review.md"), PROCESS_CODE_REVIEW)?;
    write_if_missing(
        &processes.join("feature-development.md"),
        PROCESS_FEATURE_DEV,
    )?;
    write_if_missing(&processes.join("bug-fix.md"), PROCESS_BUG_FIX)?;
    output::ok("Created context/processes/");

    Ok(())
}

/// Scaffold the .claude/skills/ directory with example skill templates.
fn scaffold_skills() -> Result<()> {
    let skills_dir = Path::new(".claude").join("skills");
    fs::create_dir_all(&skills_dir).context("Failed to create .claude/skills")?;

    let backlog_dir = skills_dir.join("backlog-context");
    fs::create_dir_all(&backlog_dir).context("Failed to create .claude/skills/backlog-context")?;
    write_if_missing(&backlog_dir.join("SKILL.md"), SKILL_BACKLOG_CONTEXT)?;

    let decisions_dir = skills_dir.join("decisions-adr");
    fs::create_dir_all(&decisions_dir).context("Failed to create .claude/skills/decisions-adr")?;
    write_if_missing(&decisions_dir.join("SKILL.md"), SKILL_DECISIONS_ADR)?;

    let standup_dir = skills_dir.join("standup-context");
    fs::create_dir_all(&standup_dir).context("Failed to create .claude/skills/standup-context")?;
    write_if_missing(&standup_dir.join("SKILL.md"), SKILL_STANDUP_CONTEXT)?;

    output::ok("Created .claude/skills/");

    Ok(())
}

/// Create OWNER.md in context/shared/ directory.
/// Falls back to context/OWNER.md check for backward compatibility.
fn setup_owner_md(context: &Path) -> Result<()> {
    // Backward compat: if context/OWNER.md exists, don't create shared/ version
    let legacy_path = context.join("OWNER.md");
    if legacy_path.exists() {
        tracing::debug!("context/OWNER.md already exists (legacy location), skipping");
        return Ok(());
    }

    let shared_dir = context.join("shared");
    fs::create_dir_all(&shared_dir)
        .with_context(|| format!("Failed to create {}", shared_dir.display()))?;

    let owner_path = shared_dir.join("OWNER.md");
    if owner_path.exists() {
        tracing::debug!("context/shared/OWNER.md already exists, skipping");
        return Ok(());
    }

    fs::write(&owner_path, OWNER_CONTENT)
        .with_context(|| format!("Failed to write {}", owner_path.display()))?;
    output::ok("Created context/shared/OWNER.md");

    Ok(())
}

/// Returns the list of expected context files for a given process flavor.
pub fn expected_context_files(process: &ProcessFlavor) -> Vec<&'static str> {
    match process {
        ProcessFlavor::Minimal => vec!["CLAUDE.md"],
        ProcessFlavor::Managed => vec![
            "CLAUDE.md",
            "context/shared/OWNER.md",
            "context/DECISIONS.md",
            "context/BACKLOG.md",
            "context/STANDUPS.md",
            "context/work-instructions/GENERAL.md",
            "context/processes/README.md",
            "context/processes/release.md",
            "context/processes/code-review.md",
            "context/processes/feature-development.md",
            "context/processes/bug-fix.md",
            ".claude/skills/backlog-context/SKILL.md",
            ".claude/skills/decisions-adr/SKILL.md",
            ".claude/skills/standup-context/SKILL.md",
        ],
        ProcessFlavor::Research => vec![
            "CLAUDE.md",
            "context/shared/OWNER.md",
            "context/PROGRESS.md",
            "context/research/_template.md",
            "context/analysis/.gitkeep",
            "experiments/README.md",
            "context/processes/README.md",
            "context/processes/release.md",
            "context/processes/code-review.md",
            "context/processes/feature-development.md",
            "context/processes/bug-fix.md",
            ".claude/skills/backlog-context/SKILL.md",
            ".claude/skills/decisions-adr/SKILL.md",
            ".claude/skills/standup-context/SKILL.md",
        ],
        ProcessFlavor::Product => vec![
            "CLAUDE.md",
            "context/shared/OWNER.md",
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
            "context/research/_template.md",
            "experiments/README.md",
            "context/processes/README.md",
            "context/processes/release.md",
            "context/processes/code-review.md",
            "context/processes/feature-development.md",
            "context/processes/bug-fix.md",
            ".claude/skills/backlog-context/SKILL.md",
            ".claude/skills/decisions-adr/SKILL.md",
            ".claude/skills/standup-context/SKILL.md",
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

    fs::write(path, content).with_context(|| format!("Failed to write: {}", path.display()))?;
    Ok(())
}

/// Write content to a file only if it differs from the current content.
/// Creates parent directories if needed. Returns true if the file was written.
pub(crate) fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if existing == content {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

/// Generate a .gitignore with dev-box entries, project-specific section,
/// and language-specific blocks based on the image flavor.
pub(crate) fn update_gitignore(image: &ImageFlavor) -> Result<()> {
    let gitignore_path = Path::new(".gitignore");

    // If .gitignore already exists, just ensure dev-box entries are present
    if gitignore_path.exists() {
        return ensure_devbox_entries(gitignore_path);
    }

    // Create a new .gitignore with full structure
    let mut content = String::new();

    // Project-specific section
    content.push_str(
        "# ── Project-specific ─────────────────────────────────────────────────────────\n",
    );
    content.push_str("# Add your project-specific ignore patterns here.\n\n\n");

    // dev-box generated
    content.push_str(
        "# ── dev-box generated ────────────────────────────────────────────────────────\n",
    );
    content.push_str("# Files generated by dev-box — do not remove these entries.\n");
    content.push_str(".devcontainer/Dockerfile\n");
    content.push_str(".devcontainer/docker-compose.yml\n");
    content.push_str(".devcontainer/devcontainer.json\n");
    content.push_str(".dev-box-home/\n");
    content.push_str(".root/\n");
    content.push_str(".dev-box-version\n");
    content.push_str(".dev-box/\n");
    content.push_str(".dev-box-backup/\n");
    content.push_str(".dev-box-env/\n\n");

    // OS generated
    content.push_str(
        "# ── OS generated files ───────────────────────────────────────────────────────\n",
    );
    content.push_str(".DS_Store\n");
    content.push_str(".DS_Store?\n");
    content.push_str("._*\n");
    content.push_str(".Spotlight-V100\n");
    content.push_str(".Trashes\n");
    content.push_str("Thumbs.db\n");
    content.push_str("ehthumbs.db\n\n");

    // Editor/IDE
    content.push_str(
        "# ── Editor / IDE ─────────────────────────────────────────────────────────────\n",
    );
    content.push_str("*.swp\n");
    content.push_str("*.swo\n");
    content.push_str("*~\n");
    content.push_str(".idea/\n\n");

    // Language-specific blocks based on image flavor
    if image.contains_python() {
        content.push_str(
            "# ── Python ───────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("__pycache__/\n");
        content.push_str("*.py[cod]\n");
        content.push_str("*$py.class\n");
        content.push_str("*.egg-info/\n");
        content.push_str("*.egg\n");
        content.push_str("dist/\n");
        content.push_str("build/\n");
        content.push_str(".eggs/\n");
        content.push_str(".venv/\n");
        content.push_str("venv/\n");
        content.push_str(".pytest_cache/\n");
        content.push_str(".mypy_cache/\n");
        content.push_str(".ruff_cache/\n");
        content.push_str("htmlcov/\n");
        content.push_str(".coverage\n");
        content.push_str(".coverage.*\n");
        content.push_str("site/\n\n");
    }

    if image.contains_latex() {
        content.push_str(
            "# ── LaTeX ────────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("*.aux\n");
        content.push_str("*.bbl\n");
        content.push_str("*.blg\n");
        content.push_str("*.fdb_latexmk\n");
        content.push_str("*.fls\n");
        content.push_str("*.lof\n");
        content.push_str("*.log\n");
        content.push_str("*.lot\n");
        content.push_str("*.out\n");
        content.push_str("*.toc\n");
        content.push_str("*.synctex.gz\n");
        content.push_str("*.nav\n");
        content.push_str("*.snm\n");
        content.push_str("*.vrb\n");
        content.push_str("*.bcf\n");
        content.push_str("*.run.xml\n");
        content.push_str("out/\n\n");
    }

    if image.contains_typst() {
        content.push_str(
            "# ── Typst ────────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("# Typst produces PDFs directly — ignore build outputs if applicable\n\n");
    }

    if image.contains_rust() {
        content.push_str(
            "# ── Rust ─────────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("target/\n");
        content.push_str("Cargo.lock\n\n");
    }

    fs::write(gitignore_path, content).context("Failed to write .gitignore")?;
    output::ok("Created .gitignore with dev-box and language-specific entries");

    Ok(())
}

/// Ensure dev-box entries exist in an existing .gitignore.
fn ensure_devbox_entries(gitignore_path: &Path) -> Result<()> {
    let required_entries = [
        "# dev-box generated",
        crate::config::DOCKERFILE,
        crate::config::COMPOSE_FILE,
        crate::config::DEVCONTAINER_JSON,
        ".dev-box-home/",
        ".dev-box-version",
        ".dev-box-backup/",
        ".dev-box-env/",
    ];

    let existing = fs::read_to_string(gitignore_path).context("Failed to read .gitignore")?;
    let existing_lines: Vec<&str> = existing.lines().map(|l| l.trim()).collect();

    let mut additions = Vec::new();
    for entry in &required_entries {
        if !existing_lines.contains(entry) {
            // Also check for .root/ (backward compat)
            if *entry == ".dev-box-home/" && existing_lines.contains(&".root/") {
                continue;
            }
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

    fs::write(gitignore_path, content).context("Failed to write .gitignore")?;
    output::ok("Updated .gitignore with dev-box entries");

    Ok(())
}

/// Check that .gitignore has required entries. Used by doctor.
pub fn check_gitignore_entries() -> Vec<String> {
    let gitignore_path = Path::new(".gitignore");
    let mut warnings = Vec::new();

    if !gitignore_path.exists() {
        warnings.push(".gitignore not found — run 'dev-box init' or create one".to_string());
        return warnings;
    }

    let content = match fs::read_to_string(gitignore_path) {
        Ok(c) => c,
        Err(_) => {
            warnings.push("Could not read .gitignore".to_string());
            return warnings;
        }
    };

    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();

    let required = [
        (".devcontainer/Dockerfile", "generated Dockerfile"),
        (
            ".devcontainer/docker-compose.yml",
            "generated docker-compose",
        ),
        (
            ".devcontainer/devcontainer.json",
            "generated devcontainer.json",
        ),
        (".dev-box-version", "version lockfile"),
    ];

    for (entry, desc) in &required {
        if !lines.contains(entry) {
            warnings.push(format!(".gitignore missing '{}' ({})", entry, desc));
        }
    }

    // Check for .dev-box-home/ or .root/
    if !lines.contains(&".dev-box-home/") && !lines.contains(&".root/") {
        warnings
            .push(".gitignore missing '.dev-box-home/' (persisted config directory)".to_string());
    }

    warnings
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
            let config = test_config(ProcessFlavor::Minimal, ImageFlavor::Base);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists(), "CLAUDE.md should exist");
            assert!(
                Path::new(".dev-box-version").exists(),
                ".dev-box-version should exist"
            );
        });
    }

    #[test]
    #[serial]
    fn scaffold_managed_creates_expected_files() {
        in_temp_dir(|| {
            let config = test_config(ProcessFlavor::Managed, ImageFlavor::Base);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new(".dev-box-version").exists());
            assert!(Path::new("context/DECISIONS.md").exists());
            assert!(Path::new("context/BACKLOG.md").exists());
            assert!(Path::new("context/STANDUPS.md").exists());
            assert!(Path::new("context/work-instructions/GENERAL.md").exists());
            assert!(Path::new("context/shared/OWNER.md").exists());
            assert!(Path::new("context/processes/README.md").exists());
            assert!(Path::new("context/processes/release.md").exists());
            assert!(Path::new("context/processes/code-review.md").exists());
            assert!(Path::new("context/processes/feature-development.md").exists());
            assert!(Path::new("context/processes/bug-fix.md").exists());
            assert!(Path::new(".claude/skills/backlog-context/SKILL.md").exists());
            assert!(Path::new(".claude/skills/decisions-adr/SKILL.md").exists());
            assert!(Path::new(".claude/skills/standup-context/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_research_creates_expected_files() {
        in_temp_dir(|| {
            let config = test_config(ProcessFlavor::Research, ImageFlavor::Base);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new("context/PROGRESS.md").exists());
            assert!(Path::new("context/research/_template.md").exists());
            assert!(Path::new("context/analysis/.gitkeep").exists());
            assert!(Path::new("context/shared/OWNER.md").exists());
            assert!(Path::new("experiments/README.md").exists());
            assert!(Path::new("context/processes/README.md").exists());
            assert!(Path::new("context/processes/release.md").exists());
            assert!(Path::new(".claude/skills/backlog-context/SKILL.md").exists());
            assert!(Path::new(".claude/skills/decisions-adr/SKILL.md").exists());
            assert!(Path::new(".claude/skills/standup-context/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_product_creates_all_expected_files() {
        in_temp_dir(|| {
            let config = test_config(ProcessFlavor::Product, ImageFlavor::Base);
            scaffold_context(&config).unwrap();
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
            assert!(Path::new("context/research/_template.md").exists());
            assert!(Path::new("experiments/README.md").exists());
            assert!(Path::new("context/shared/OWNER.md").exists());
            assert!(Path::new("context/processes/README.md").exists());
            assert!(Path::new("context/processes/release.md").exists());
            assert!(Path::new("context/processes/code-review.md").exists());
            assert!(Path::new("context/processes/feature-development.md").exists());
            assert!(Path::new("context/processes/bug-fix.md").exists());
            assert!(Path::new(".claude/skills/backlog-context/SKILL.md").exists());
            assert!(Path::new(".claude/skills/decisions-adr/SKILL.md").exists());
            assert!(Path::new(".claude/skills/standup-context/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn claude_md_contains_project_name() {
        in_temp_dir(|| {
            let config = test_config(ProcessFlavor::Minimal, ImageFlavor::Base);
            scaffold_context(&config).unwrap();
            let content = fs::read_to_string("CLAUDE.md").unwrap();
            assert!(
                content.contains("test-proj"),
                "CLAUDE.md should contain project name"
            );
        });
    }

    #[test]
    #[serial]
    fn gitignore_includes_python_block() {
        in_temp_dir(|| {
            update_gitignore(&ImageFlavor::Python).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("__pycache__/"));
            assert!(content.contains("*.py[cod]"));
            assert!(content.contains(".dev-box-home/"));
        });
    }

    #[test]
    #[serial]
    fn gitignore_includes_latex_block() {
        in_temp_dir(|| {
            update_gitignore(&ImageFlavor::Latex).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("*.aux"));
            assert!(content.contains("*.synctex.gz"));
        });
    }

    #[test]
    #[serial]
    fn gitignore_includes_rust_block() {
        in_temp_dir(|| {
            update_gitignore(&ImageFlavor::Rust).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("target/"));
        });
    }

    #[test]
    #[serial]
    fn gitignore_combined_flavor() {
        in_temp_dir(|| {
            update_gitignore(&ImageFlavor::PythonLatex).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("__pycache__/"));
            assert!(content.contains("*.aux"));
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_preserves_existing_content() {
        in_temp_dir(|| {
            fs::write(".gitignore", "node_modules/\n*.log\n").unwrap();
            update_gitignore(&ImageFlavor::Base).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("node_modules/"));
            assert!(content.contains("*.log"));
            assert!(content.contains(".dev-box-home/") || content.contains(".root/"));
        });
    }

    #[test]
    #[serial]
    fn owner_md_has_extended_fields() {
        in_temp_dir(|| {
            let config = test_config(ProcessFlavor::Managed, ImageFlavor::Base);
            scaffold_context(&config).unwrap();
            let content = fs::read_to_string("context/shared/OWNER.md").unwrap();
            assert!(content.contains("Domain expertise"));
            assert!(content.contains("Timezone"));
            assert!(content.contains("Communication language"));
        });
    }

    fn test_config(process: ProcessFlavor, image: ImageFlavor) -> DevBoxConfig {
        crate::config::test_config(image, process)
    }
}
