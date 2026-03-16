use anyhow::Result;
use std::path::Path;

use crate::config::{DevBoxConfig, ProcessFlavor};
use crate::output;
use crate::runtime::Runtime;

/// Embedded schema document for v1.0.0.
const SCHEMA_V1_0_0: &str = include_str!("../../schemas/v1.0.0/context-schema.md");

/// Diagnostic counters.
struct DiagResult {
    warnings: u32,
    errors: u32,
}

impl DiagResult {
    fn new() -> Self {
        Self {
            warnings: 0,
            errors: 0,
        }
    }
}

/// Return the list of expected files for a given process flavor.
/// Delegates to the shared list in context.rs and adds infrastructure files.
fn expected_files(process: &ProcessFlavor) -> Vec<&'static str> {
    let mut files = crate::context::expected_context_files(process);
    // Doctor also checks infrastructure files
    files.extend_from_slice(&[".dev-box-version", ".gitignore"]);
    files
}

/// Look up the embedded schema for a given version string.
fn schema_for_version(version: &str) -> Option<&'static str> {
    match version {
        "1.0.0" => Some(SCHEMA_V1_0_0),
        _ => None,
    }
}

/// Run full diagnostics.
pub fn cmd_doctor(config_path: &Option<String>) -> Result<()> {
    let mut diag = DiagResult::new();

    output::info("Running diagnostics...");

    // 1. Load and validate config
    let config = match DevBoxConfig::from_cli_option(config_path) {
        Ok(c) => {
            output::ok(&format!(
                "Config: valid (v{}, {}, {})",
                c.dev_box.version, c.dev_box.image, c.dev_box.process
            ));
            Some(c)
        }
        Err(e) => {
            output::error(&format!("Config: {}", e));
            diag.errors += 1;
            None
        }
    };

    // 2. Check container runtime
    match Runtime::detect() {
        Ok(rt) => output::ok(&format!("Container runtime: {} detected", rt.runtime_bin)),
        Err(_) => {
            output::error("No container runtime found (need podman or docker)");
            diag.errors += 1;
        }
    }

    // If we couldn't load config, we can't do the remaining checks
    let config = match config {
        Some(c) => c,
        None => {
            print_summary(&diag);
            return Ok(());
        }
    };

    // 3. Check .root/ directory
    let root = config.host_root_dir();
    if root.exists() {
        output::ok(".root/ directory exists");
        // Check expected subdirectories
        check_root_subdirs(&root, &mut diag);
    } else {
        output::warn(".root/ directory not found -- run 'dev-box start' to create it");
        diag.warnings += 1;
    }

    // 4. Check .devcontainer/ files
    check_devcontainer_files(&mut diag);

    // 5. Check context structure
    output::info(&format!(
        "Checking context structure ({})...",
        config.dev_box.process
    ));
    check_context_structure(&config.dev_box.process, &mut diag);

    // 6. Schema version check
    output::info("Schema version check");
    check_schema_version(&config, &mut diag)?;

    print_summary(&diag);
    Ok(())
}

/// Check .root/ subdirectories.
fn check_root_subdirs(root: &Path, diag: &mut DiagResult) {
    let expected_dirs = [".ssh", ".vim", ".config/zellij", ".config/git"];
    for dir in &expected_dirs {
        let path = root.join(dir);
        if path.exists() {
            output::ok(&format!(".root/{} exists", dir));
        } else {
            output::warn(&format!(".root/{} missing", dir));
            diag.warnings += 1;
        }
    }
}

/// Check .devcontainer/ files.
fn check_devcontainer_files(diag: &mut DiagResult) {
    let files = [
        crate::config::DOCKERFILE,
        crate::config::COMPOSE_FILE,
        crate::config::DEVCONTAINER_JSON,
    ];

    let mut all_present = true;
    for f in &files {
        if !Path::new(f).exists() {
            output::warn(&format!("{} missing -- run 'dev-box generate'", f));
            diag.warnings += 1;
            all_present = false;
        }
    }

    if all_present {
        output::ok(".devcontainer/ files present");
    }
}

/// Check context structure against the process flavor.
fn check_context_structure(process: &ProcessFlavor, diag: &mut DiagResult) {
    let expected = expected_files(process);

    for file in &expected {
        let path = Path::new(file);
        // For OWNER.md, also accept symlinks
        if path.exists() || path.symlink_metadata().is_ok() {
            output::ok(&format!("{} exists", file));
        } else {
            output::warn(&format!("{} missing", file));
            diag.warnings += 1;
        }
    }

    // Check for extra files in context/ that aren't expected (warning only)
    if Path::new("context").exists() {
        check_extra_files("context", &expected, diag);
    }
}

/// Walk the context/ directory and report files not in the expected list.
fn check_extra_files(dir: &str, expected: &[&str], diag: &mut DiagResult) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.to_string_lossy().to_string();

        if path.is_dir() {
            check_extra_files(&rel, expected, diag);
            continue;
        }

        // Normalize path separators and check against expected list
        let normalized = rel.replace('\\', "/");
        if !expected.iter().any(|e| normalized == *e) {
            // Don't warn about OWNER.md if it's a symlink (it's always expected via the list)
            output::warn(&format!("Extra file: {} (not in {} schema)", normalized, "context"));
            diag.warnings += 1;
        }
    }
}

/// Check schema version and generate migration artifacts if needed.
fn check_schema_version(config: &DevBoxConfig, diag: &mut DiagResult) -> Result<()> {
    let version_file = Path::new(".dev-box-version");
    let target_version = &config.context.schema_version;

    if !version_file.exists() {
        output::warn(".dev-box-version file not found -- run 'dev-box init' to create it");
        diag.warnings += 1;
        return Ok(());
    }

    let current_version = std::fs::read_to_string(version_file)?
        .trim()
        .to_string();

    if current_version == *target_version {
        output::ok(&format!(
            "Current: {}, Target: {} (up to date)",
            current_version, target_version
        ));
    } else {
        output::warn(&format!(
            "Current: {}, Target: {} (migration needed)",
            current_version, target_version
        ));
        diag.warnings += 1;
        generate_migration_artifacts(&current_version, target_version, config)?;
    }

    Ok(())
}

/// Generate migration artifacts when schema versions differ.
fn generate_migration_artifacts(
    current_version: &str,
    target_version: &str,
    config: &DevBoxConfig,
) -> Result<()> {
    let migration_dir = Path::new(".dev-box/migration");
    std::fs::create_dir_all(migration_dir)?;

    // Write schema-current.md
    let current_schema = schema_for_version(current_version)
        .unwrap_or("# Unknown Schema Version\n\nNo embedded schema found for this version.\n");
    std::fs::write(migration_dir.join("schema-current.md"), current_schema)?;
    output::ok("Generated .dev-box/migration/schema-current.md");

    // Write schema-target.md
    let target_schema = schema_for_version(target_version)
        .unwrap_or("# Unknown Schema Version\n\nNo embedded schema found for this version.\n");
    std::fs::write(migration_dir.join("schema-target.md"), target_schema)?;
    output::ok("Generated .dev-box/migration/schema-target.md");

    // Write diff.md
    let diff_content = format!(
        "# Schema Diff: {} -> {}\n\n\
         ## Summary\n\n\
         Migration from schema version {} to {}.\n\n\
         ## Structural Differences\n\n\
         {}\n",
        current_version,
        target_version,
        current_version,
        target_version,
        if current_schema == target_schema {
            "No structural differences detected between these schema versions.".to_string()
        } else {
            "Schema content differs. Review schema-current.md and schema-target.md for details."
                .to_string()
        }
    );
    std::fs::write(migration_dir.join("diff.md"), diff_content)?;
    output::ok("Generated .dev-box/migration/diff.md");

    // Write migration-prompt.md
    let prompt_content = format!(
        "# Migration Prompt\n\n\
         You are migrating the project context structure for **{}**.\n\n\
         ## Current State\n\n\
         - Schema version: {}\n\
         - Process flavor: {}\n\
         - Container name: {}\n\n\
         ## Target State\n\n\
         - Schema version: {}\n\n\
         ## Instructions\n\n\
         1. Read `schema-current.md` to understand the current structure\n\
         2. Read `schema-target.md` to understand the target structure\n\
         3. Read `diff.md` for a summary of differences\n\
         4. Examine the project's `context/` directory\n\
         5. Generate a migration plan that:\n\
            - Adds any missing files or sections\n\
            - Never removes or overwrites existing user content\n\
            - Preserves all existing formatting and IDs\n\
            - Marks each change as \"required\" or \"recommended\"\n\n\
         ## Files to Reference\n\n\
         - `.dev-box/migration/schema-current.md`\n\
         - `.dev-box/migration/schema-target.md`\n\
         - `.dev-box/migration/diff.md`\n\
         - `context/` directory (current project files)\n\
         - `CLAUDE.md` (project root)\n",
        config.container.name,
        current_version,
        config.dev_box.process,
        config.container.name,
        target_version,
    );
    std::fs::write(migration_dir.join("migration-prompt.md"), prompt_content)?;
    output::ok("Generated .dev-box/migration/migration-prompt.md");

    output::info(&format!(
        "Migration artifacts written to {}",
        migration_dir.display()
    ));

    Ok(())
}

/// Print final summary.
fn print_summary(diag: &DiagResult) {
    output::info(&format!(
        "Diagnostics complete: {} warning(s), {} error(s)",
        diag.warnings, diag.errors
    ));
}
