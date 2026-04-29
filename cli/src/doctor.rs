use anyhow::Result;
use std::path::Path;

use crate::config::AiboxConfig;
use crate::output;
use crate::processkit_vocab::AGENTS_FILENAME;
use crate::runtime::{ContainerState, Runtime};

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

/// Return the list of project-side files `aibox doctor` checks for.
///
/// Since v0.16.0 the bulk of context content (BACKLOG, DECISIONS, skills,
/// AGENTS.md, …) is owned by processkit and may or may not be present
/// depending on whether the user has run `aibox init` against a real
/// processkit version. Doctor only checks the slice that aibox itself
/// creates: the version marker, the gitignore, and the canonical
/// agent entrypoint installed by processkit.
fn expected_files(_packages: &[String]) -> Vec<&'static str> {
    vec![AGENTS_FILENAME, "aibox.lock", ".gitignore"]
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
    let config = match AiboxConfig::from_cli_option(config_path) {
        Ok(c) => {
            output::ok(&format!(
                "Config: valid (v{}, {}, {:?})",
                c.aibox.version, c.aibox.base, c.context.packages
            ));
            Some(c)
        }
        Err(e) => {
            output::error(&format!("Config: {}", e));
            diag.errors += 1;
            None
        }
    };

    // 2. Check container runtime (informational — not required for init/generate/doctor)
    match Runtime::detect() {
        Ok(rt) => output::ok(&format!("Container runtime: {} detected", rt.runtime_bin)),
        Err(_) => {
            output::warn(
                "No container runtime found (podman or docker needed for build/start/stop/attach)",
            );
            diag.warnings += 1;
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

    // 3. Check .aibox-home/ directory (or legacy .root/)
    let root = config.host_root_dir();
    let root_label = root.display().to_string();
    if root.exists() {
        output::ok(&format!("{} directory exists", root_label));
        // Check expected subdirectories
        check_root_subdirs(&root, &root_label, &mut diag);

        // Check mount source paths match config (AI providers, audio)
        check_mount_sources(&root, &root_label, &config, &mut diag);

        // Suggest migration from .root/ to .aibox-home/
        if root_label == ".root" && !std::path::Path::new(".aibox-home").exists() {
            output::warn(
                ".root/ is the legacy name — consider renaming to .aibox-home/ \
                 (mv .root .aibox-home)",
            );
            diag.warnings += 1;
        }
    } else {
        output::warn(&format!(
            "{} directory not found -- run 'aibox init' or 'aibox start' to create it",
            root_label
        ));
        diag.warnings += 1;
    }

    // 4. Check .devcontainer/ files
    check_devcontainer_files(&mut diag);

    // 5. Check context structure
    output::info(&format!(
        "Checking context structure ({:?})...",
        config.context.packages
    ));
    check_context_structure(&config.context.packages, &mut diag);

    // 6. Check .gitignore
    output::info("Checking .gitignore...");
    let gitignore_warnings = crate::context::check_gitignore_entries();
    if gitignore_warnings.is_empty() {
        output::ok(".gitignore has all required entries");
    } else {
        for warning in &gitignore_warnings {
            output::warn(warning);
            diag.warnings += 1;
        }
    }

    // 6b. [skills].include / [skills].exclude validation (DEC-035)
    output::info("Validating [skills] overrides...");
    if let Ok(cwd) = std::env::current_dir() {
        match crate::content_init::validate_skill_overrides(&cwd, &config) {
            Ok(unknown) if unknown.is_empty() => {
                output::ok("[skills] overrides reference known skills");
            }
            Ok(unknown) => {
                for u in &unknown {
                    output::warn(u);
                    diag.warnings += 1;
                }
            }
            Err(e) => {
                output::warn(&format!("[skills] override validation failed: {}", e));
                diag.warnings += 1;
            }
        }
    }

    // 6c. Check command file registrations (BACK-20260423_2050-EagerStone)
    output::info("Checking command file registrations...");
    check_command_registrations(&config, &mut diag);

    // 6d. Codex prompt-path drift check (BACK-20260426_1627-StrongHawk).
    // Loud failure if `pk-*` managed files reappear in the legacy
    // `~/.codex/prompts/` path that aibox v0.21.1 mistakenly used —
    // catches a regression in the codex profile of harness_commands.
    check_codex_prompt_path_drift(&config, &mut diag);

    // 7. Security audit tools
    crate::audit::doctor_check_audit_tools();

    // 8. Schema version check
    output::info("Schema version check");
    check_schema_version(&config, &mut diag)?;

    // 9. Container image version check (only if a runtime is available)
    if let Ok(runtime) = Runtime::detect() {
        check_container_image_version(&runtime, &config, &mut diag);
    }

    // 10. CLI version file check
    check_cli_version_file(&mut diag);

    print_summary(&diag);
    Ok(())
}

/// Check that installed skills have their command files registered in
/// each enabled harness's command directory.
///
/// For every `context/skills/*/commands/*.md` (source), validates that for
/// every enabled scaffolded harness the corresponding deployed file exists
/// under that harness's commands dir (Claude is always-on; others gated on
/// `[ai].harnesses`). Helps detect incomplete skill distributions and stale
/// scaffolds that were dropped before `aibox sync` was rerun.
fn check_command_registrations(config: &AiboxConfig, diag: &mut DiagResult) {
    let skills_dir = std::path::Path::new("context/skills");
    if !skills_dir.is_dir() {
        output::ok("No context/skills/ found (expected in new projects)");
        return;
    }

    // Gather every command basename from the live skills tree.
    let mut source_commands: Vec<(std::path::PathBuf, String)> = Vec::new();
    if let Ok(categories) = std::fs::read_dir(skills_dir) {
        for category in categories.flatten() {
            if !category.path().is_dir() {
                continue;
            }
            if let Ok(skills) = std::fs::read_dir(category.path()) {
                for skill in skills.flatten() {
                    let skill_path = skill.path();
                    if !skill_path.is_dir() {
                        continue;
                    }
                    let commands_src = skill_path.join("commands");
                    if !commands_src.is_dir() {
                        continue;
                    }
                    if let Ok(cmds) = std::fs::read_dir(&commands_src) {
                        for cmd in cmds.flatten() {
                            let cmd_path = cmd.path();
                            if let Some(filename) = cmd_path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .filter(|s| s.ends_with(".md"))
                            {
                                source_commands.push((commands_src.clone(), filename.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    // Per-harness target dirs. `path_template` is a `{stem}` substitution
    // pattern relative to the project root. Mirrors the profiles in
    // `harness_commands::profile_for`. Keep this list in sync when
    // adding new scaffoldable harnesses.
    use crate::config::AiHarness;
    // (label, target_dir_for_message, path_template_with_{stem}, enabled)
    let mut targets: Vec<(&'static str, &'static str, &'static str, bool)> = Vec::new();
    targets.push((
        "claude",
        ".claude/commands",
        ".claude/commands/{stem}.md",
        true, // always-on
    ));
    targets.push((
        "codex",
        ".agents/skills",
        ".agents/skills/{stem}/SKILL.md",
        config.ai.harnesses.contains(&AiHarness::Codex),
    ));
    targets.push((
        "cursor",
        ".cursor/commands",
        ".cursor/commands/{stem}.md",
        config.ai.harnesses.contains(&AiHarness::Cursor),
    ));
    targets.push((
        "gemini",
        ".gemini/commands",
        ".gemini/commands/{stem}.toml",
        config.ai.harnesses.contains(&AiHarness::Gemini),
    ));
    targets.push((
        "opencode",
        ".opencode/commands",
        ".opencode/commands/{stem}.md",
        config.ai.harnesses.contains(&AiHarness::OpenCode),
    ));

    for (harness, target_dir, path_template, enabled) in &targets {
        if !*enabled {
            continue;
        }
        let mut missing_count = 0;
        for (commands_src, filename) in &source_commands {
            let stem = match filename.strip_suffix(".md") {
                Some(s) => s,
                None => continue,
            };
            let deployed = std::path::PathBuf::from(path_template.replace("{stem}", stem));
            if !deployed.exists() {
                output::warn(&format!(
                    "{harness}: command file missing: {}/{} exists but {} is not registered",
                    commands_src.display(),
                    filename,
                    deployed.display()
                ));
                diag.warnings += 1;
                missing_count += 1;
            }
        }
        if missing_count == 0 {
            output::ok(&format!(
                "[{harness}] all installed skill commands are registered in {target_dir}/"
            ));
        } else {
            output::warn(&format!(
                "[{harness}] {missing_count} command file(s) missing — run 'aibox sync' to register them"
            ));
        }
    }
}

/// Detect drift on the Codex slash-command path. Codex CLI 0.125.0 surfaces
/// custom workflows as Skills under `<workspace>/.agents/skills/<name>/SKILL.md`
/// — NOT from `~/.codex/prompts/` (the legacy aibox v0.21.1 location). If
/// any managed `pk-*.md` file reappears in the legacy path, treat that as
/// a regression error: aibox is again writing to the wrong place. Also
/// errors if Codex is enabled but no skills landed under `.agents/skills/`.
///
/// See DEC-20260426_1636-MightySky and BACK-20260426_1627-StrongHawk.
fn check_codex_prompt_path_drift(config: &AiboxConfig, diag: &mut DiagResult) {
    use crate::config::AiHarness;
    let codex_enabled = config.ai.harnesses.contains(&AiHarness::Codex);

    let legacy_dir = std::path::Path::new(".aibox-home/.codex/prompts");
    if let Ok(entries) = std::fs::read_dir(legacy_dir) {
        let stale: Vec<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.starts_with("pk-") && n.ends_with(".md"))
            .collect();
        if !stale.is_empty() {
            output::error(&format!(
                "codex: stale managed prompt(s) in legacy path .aibox-home/.codex/prompts/: \
                 {}. Codex 0.125.0 ignores this directory; commands must be Codex Skills \
                 under .agents/skills/<name>/SKILL.md (DEC-20260426_1636-MightySky). \
                 Run 'aibox sync' to migrate.",
                stale.join(", ")
            ));
            diag.errors += 1;
        }
    }

    if codex_enabled {
        let skills_dir = std::path::Path::new(".agents/skills");
        let has_pk_skill = std::fs::read_dir(skills_dir)
            .map(|it| {
                it.flatten().any(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.starts_with("pk-"))
                        .unwrap_or(false)
                        && e.path().join("SKILL.md").is_file()
                })
            })
            .unwrap_or(false);
        if !has_pk_skill {
            output::warn(
                "codex: no pk-* Codex Skills found under .agents/skills/ — \
                 run 'aibox sync' to scaffold them (Codex 0.125.0 surfaces \
                 these as $skill-name mentions and via /skills)",
            );
            diag.warnings += 1;
        } else {
            output::ok("codex: pk-* Codex Skills present under .agents/skills/");
        }
    }
}

/// Check that mount source directories exist for configured features.
fn check_mount_sources(root: &Path, root_label: &str, config: &AiboxConfig, diag: &mut DiagResult) {
    // AI providers — check the .aibox-home/<provider>/ persistence dir
    // for the in-container CLI tools that have one. Cursor is the only
    // provider with no container CLI binary (host-side IDE extension only).
    for provider in &config.ai.harnesses {
        let Some(dir_name) = provider.config_dir() else {
            continue;
        };
        let path = root.join(dir_name);
        if path.exists() {
            output::ok(&format!(
                "{}/{} exists ({})",
                root_label, dir_name, provider
            ));
        } else {
            output::warn(&format!(
                "{}/{} missing — run 'aibox sync' to create it",
                root_label, dir_name
            ));
            diag.warnings += 1;
        }
    }

    // Audio
    if config.audio.enabled {
        let asoundrc = root.join(".asoundrc");
        if asoundrc.exists() {
            output::ok(&format!("{}/.asoundrc exists", root_label));
        } else {
            output::warn(&format!(
                "{}/.asoundrc missing — run 'aibox sync' to create it",
                root_label
            ));
            diag.warnings += 1;
        }
    }
}

/// Check home directory subdirectories.
fn check_root_subdirs(root: &Path, root_label: &str, diag: &mut DiagResult) {
    let expected_dirs = [".ssh", ".vim", ".config/zellij", ".config/git"];
    for dir in &expected_dirs {
        let path = root.join(dir);
        if path.exists() {
            output::ok(&format!("{}/{} exists", root_label, dir));
        } else {
            output::warn(&format!("{}/{} missing", root_label, dir));
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
            output::warn(&format!("{} missing -- run 'aibox sync'", f));
            diag.warnings += 1;
            all_present = false;
        }
    }

    if all_present {
        output::ok(".devcontainer/ files present");
    }
}

/// Check context structure against the process packages.
fn check_context_structure(packages: &[String], diag: &mut DiagResult) {
    let expected = expected_files(packages);

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
            output::warn(&format!(
                "Extra file: {} (not in {} schema)",
                normalized, "context"
            ));
            diag.warnings += 1;
        }
    }
}

/// Check schema version and generate migration artifacts if needed.
fn check_schema_version(config: &AiboxConfig, diag: &mut DiagResult) -> Result<()> {
    let target_version = &config.context.schema_version;

    let lock = match crate::lock::read_lock(Path::new("."))? {
        Some(l) => l,
        None => {
            output::warn("aibox.lock not found -- run 'aibox init' to create it");
            diag.warnings += 1;
            return Ok(());
        }
    };

    let current_version = &lock.aibox.cli_version;

    if current_version == target_version {
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
        generate_migration_artifacts(current_version, target_version, config)?;
    }

    Ok(())
}

/// Generate migration artifacts when schema versions differ.
fn generate_migration_artifacts(
    current_version: &str,
    target_version: &str,
    config: &AiboxConfig,
) -> Result<()> {
    let migration_dir = Path::new(".aibox/migration");
    std::fs::create_dir_all(migration_dir)?;

    // Write schema-current.md
    let current_schema = schema_for_version(current_version)
        .unwrap_or("# Unknown Schema Version\n\nNo embedded schema found for this version.\n");
    std::fs::write(migration_dir.join("schema-current.md"), current_schema)?;
    output::ok("Generated .aibox/migration/schema-current.md");

    // Write schema-target.md
    let target_schema = schema_for_version(target_version)
        .unwrap_or("# Unknown Schema Version\n\nNo embedded schema found for this version.\n");
    std::fs::write(migration_dir.join("schema-target.md"), target_schema)?;
    output::ok("Generated .aibox/migration/schema-target.md");

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
    output::ok("Generated .aibox/migration/diff.md");

    // Write migration-prompt.md
    let prompt_content = format!(
        "# Migration Prompt\n\n\
         You are migrating the project context structure for **{}**.\n\n\
         ## Current State\n\n\
         - Schema version: {}\n\
         - Process packages: {:?}\n\
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
         - `.aibox/migration/schema-current.md`\n\
         - `.aibox/migration/schema-target.md`\n\
         - `.aibox/migration/diff.md`\n\
         - `context/` directory (current project files)\n\
         - `CLAUDE.md` (project root)\n",
        config.container.name,
        current_version,
        config.context.packages,
        config.container.name,
        target_version,
    );
    std::fs::write(migration_dir.join("migration-prompt.md"), prompt_content)?;
    output::ok("Generated .aibox/migration/migration-prompt.md");

    output::info(&format!(
        "Migration artifacts written to {}",
        migration_dir.display()
    ));

    Ok(())
}

/// Check that the running container's image version matches the config version.
///
/// Reads the `aibox.version` Docker label set at build time. Skips silently
/// if the container is missing or has no label (pre-BACK-060 image).
fn check_container_image_version(runtime: &Runtime, config: &AiboxConfig, diag: &mut DiagResult) {
    let name = &config.container.name;
    let state = match runtime.container_status(name) {
        Ok(s) => s,
        Err(_) => return,
    };
    if state == ContainerState::Missing {
        return;
    }

    match runtime.get_container_image_version(name) {
        Ok(Some(container_ver)) => {
            if container_ver == config.aibox.version {
                output::ok(&format!(
                    "Container image version: {} (matches config)",
                    container_ver
                ));
            } else {
                output::warn(&format!(
                    "Container image version mismatch: container={} config={} — \
                     run `aibox sync` to rebuild",
                    container_ver, config.aibox.version
                ));
                diag.warnings += 1;
            }
        }
        Ok(None) => {
            // Pre-BACK-060 image: no label — informational only
            output::ok("Container image version: no label (pre-v0.13 image, rebuild recommended)");
        }
        Err(_) => {} // inspect failed — skip silently
    }
}

/// Warn if `aibox.lock [aibox].cli_version` doesn't match the current CLI version.
///
/// `cli_version` is written at init/sync time. A mismatch means generated files
/// may be stale for this CLI version.
fn check_cli_version_file(diag: &mut DiagResult) {
    let lock = match crate::lock::read_lock(Path::new(".")) {
        Ok(Some(l)) => l,
        _ => return, // No lock or read error — already reported by check_schema_version.
    };

    let file_version = &lock.aibox.cli_version;
    if file_version.is_empty() {
        return; // Unknown version — skip.
    }

    let cli_version = env!("CARGO_PKG_VERSION");
    if file_version != cli_version {
        output::warn(&format!(
            "CLI version mismatch: aibox.lock cli_version={} current={} — \
             run `aibox sync` to update generated files",
            file_version, cli_version
        ));
        diag.warnings += 1;
    }
}

/// Print final summary.
fn print_summary(diag: &DiagResult) {
    output::info(&format!(
        "Diagnostics complete: {} warning(s), {} error(s)",
        diag.warnings, diag.errors
    ));
}
