use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AiboxConfig;
use crate::output;
use crate::runtime::{ContainerState, Runtime};

/// Backup directory name.
const BACKUP_DIR: &str = ".aibox/backup";

/// Files and directories managed by aibox.
/// Each entry: (path, should_delete_on_reset)
/// .gitignore is backed up but not deleted.
///
/// Keep this list in sync with `crate::sync_perimeter::SYNC_PERIMETER`.
/// Every path that sync or init can CREATE must be deleted here on reset.
/// For provider directories (.cursor/, .gemini/, …) where aibox owns only
/// a specific file rather than the whole directory, list only the owned
/// file/subdirectory so user config in the same directory is left intact.
const MANAGED_ITEMS: &[(&str, bool)] = &[
    // ── Core aibox files ────────────────────────────────────────────────
    ("aibox.toml", true),
    ("aibox.lock", true),
    // Backup the full .devcontainer tree so repo-local files survive reset.
    // Deletion remains selective via the file-level entries below.
    (".devcontainer", false),
    (".devcontainer/Dockerfile", true),
    (".devcontainer/docker-compose.yml", true),
    (".devcontainer/devcontainer.json", true),
    (".devcontainer/Dockerfile.local", true),
    (".devcontainer/docker-compose.override.yml", true),
    (".aibox-home", true),
    ("context", true),
    ("CLAUDE.md", true),
    (".gitignore", false),
    // ── processkit-installed files ───────────────────────────────────────
    // AGENTS.md is written by processkit at init/sync time; reset must
    // remove it so the project is back to a pre-aibox state.
    ("AGENTS.md", true),
    // ── MCP server registration files (DEC-033, v0.16.5+) ───────────────
    // .mcp.json is fully owned by aibox. For the per-harness provider
    // directories, only the specific file/subdir that aibox writes is
    // listed — the rest of each directory may contain user config.
    (".mcp.json", true),
    (".cursor/mcp.json", true),
    (".gemini/settings.json", true),
    (".codex/config.toml", true),
    (".continue/mcpServers", true),
    // ── Claude Code slash-command adapters (aibox#37, v0.17.3+) ─────────
    // .claude/commands/ is NOT deleted wholesale: users may have their own
    // custom commands there. Only the specific files aibox installed are
    // removed (see selective cleanup in cmd_reset via remove_managed_commands).
    // The directory is still backed up so users can restore any content.
    (".claude/commands", false),
    // ── Backward compat ──────────────────────────────────────────────────
    (".root", true),
    (".aibox-version", true), // legacy — removed by migrate_legacy_lock_files; still cleaned up on reset
];

/// Represents an item to be processed during backup/reset.
struct ManagedItem {
    path: PathBuf,
    exists: bool,
    will_backup: bool,
    will_delete: bool,
    backup_target: Option<PathBuf>,
}

/// Build the list of managed items, checking which exist on disk.
fn discover_items(backup_dir: Option<&Path>, delete: bool) -> Vec<ManagedItem> {
    MANAGED_ITEMS
        .iter()
        .map(|(path_str, deletable)| {
            let path = PathBuf::from(path_str);
            let exists = path.exists();
            let will_backup = exists && backup_dir.is_some();
            let will_delete = exists && delete && *deletable;
            let backup_target = if will_backup {
                backup_dir.map(|d| d.join(path_str))
            } else {
                None
            };
            ManagedItem {
                path,
                exists,
                will_backup,
                will_delete,
                backup_target,
            }
        })
        .collect()
}

/// Print the action table for managed items.
fn print_table(items: &[ManagedItem]) {
    // Header
    let header_delete = "Delete";
    eprintln!(
        "\n  {:<25} {:<10} {:<45} {}",
        "File/Directory", "Backup", "Target", header_delete
    );
    eprintln!("  {}", "-".repeat(95));

    for item in items {
        if !item.exists {
            continue;
        }

        let backup_str = if item.will_backup {
            "\x1b[32m\u{2713}\x1b[0m"
        } else {
            "\x1b[31m\u{2717}\x1b[0m"
        };
        let target_str = match &item.backup_target {
            Some(t) => t.display().to_string(),
            None => "-".to_string(),
        };
        let delete_str = if item.will_delete {
            "\x1b[31m\u{2713}\x1b[0m"
        } else {
            "\x1b[32m\u{2717}\x1b[0m"
        };

        eprintln!(
            "  {:<25} {:<17} {:<52} {}",
            item.path.display(),
            backup_str,
            target_str,
            delete_str,
        );
    }
    eprintln!();
}

/// Generate the backup subdirectory name: aibox-<version>-backup-<date>-<time>
fn backup_subdir_name(version: &str) -> String {
    let now = chrono::Local::now();
    format!(
        "aibox-{}-backup-{}-{}",
        version,
        now.format("%Y-%m-%d"),
        now.format("%H%M"),
    )
}

/// Copy a file or directory recursively to the backup location.
pub fn copy_item(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        copy_dir_recursive(src, dst)?;
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create backup dir: {}", parent.display()))?;
        }
        fs::copy(src, dst)
            .with_context(|| format!("Failed to copy {} -> {}", src.display(), dst.display()))?;
    }
    Ok(())
}

/// Recursively copy a directory.
/// Symlinks are reproduced as symlinks; broken symlinks are skipped.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory: {}", dst.display()))?;
    for entry in
        fs::read_dir(src).with_context(|| format!("Failed to read directory: {}", src.display()))?
    {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = fs::symlink_metadata(&src_path)
            .with_context(|| format!("Failed to read metadata: {}", src_path.display()))?
            .file_type();

        if file_type.is_symlink() {
            // Reproduce symlink; skip if we can't read the target path
            if let Ok(target) = fs::read_link(&src_path) {
                #[cfg(unix)]
                {
                    let _ = std::os::unix::fs::symlink(&target, &dst_path);
                }
            }
        } else if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "Failed to copy {} -> {}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}

/// Delete a file or directory.
pub fn delete_item(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
    } else {
        fs::remove_file(path)
            .with_context(|| format!("Failed to remove file: {}", path.display()))?;
    }
    Ok(())
}

fn remove_dir_if_empty(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;
    if entries.next().is_none() {
        fs::remove_dir(path)
            .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
    }

    Ok(())
}

/// Stop the container if it's running.
pub fn ensure_container_stopped(config: &AiboxConfig) -> Result<()> {
    let runtime = match Runtime::detect() {
        Ok(r) => r,
        Err(_) => return Ok(()), // No runtime available, nothing to stop
    };

    let name = &config.container.name;
    let state = runtime.container_status(name)?;
    if state == ContainerState::Running {
        output::info(&format!("Stopping running container '{}'...", name));
        runtime.compose_stop(crate::config::COMPOSE_FILE, name)?;
        output::ok("Container stopped");
    }
    Ok(())
}

/// Confirm a dangerous action interactively.
pub fn confirm(prompt: &str, confirm_word: &str) -> Result<bool> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        bail!("Cannot confirm in non-interactive mode. Use --yes to skip confirmation.");
    }
    eprintln!("{}", prompt);
    eprint!("  Type '{}' to confirm: ", confirm_word);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim() == confirm_word)
}

// =============================================================================
// Public commands
// =============================================================================

/// Backup command: save current aibox state to a timestamped backup.
pub fn cmd_backup(
    config_path: &Option<String>,
    output_dir: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let version = &config.aibox.version;

    let base_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(BACKUP_DIR));
    let subdir_name = backup_subdir_name(version);
    let backup_path = base_dir.join(&subdir_name);

    output::info(&format!("Backup target: {}", backup_path.display()));

    let items = discover_items(Some(&backup_path), false);

    let existing_count = items.iter().filter(|i| i.exists).count();
    if existing_count == 0 {
        output::warn("No aibox files found to back up.");
        return Ok(());
    }

    print_table(&items);

    if dry_run {
        output::warn("[dry-run] No files were copied.");
        return Ok(());
    }

    // Create backup directory
    fs::create_dir_all(&backup_path)
        .with_context(|| format!("Failed to create backup dir: {}", backup_path.display()))?;

    // Copy items
    for item in &items {
        if item.will_backup {
            let target = item.backup_target.as_ref().unwrap();
            copy_item(&item.path, target)?;
            output::ok(&format!("Backed up {}", item.path.display()));
        }
    }

    output::ok(&format!("Backup complete: {}", backup_path.display()));
    Ok(())
}

/// Reset command: backup (unless --no-backup) then delete all aibox files.
pub fn cmd_reset(
    config_path: &Option<String>,
    no_backup: bool,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let version = &config.aibox.version;

    let backup_path = if no_backup {
        None
    } else {
        let base_dir = PathBuf::from(BACKUP_DIR);
        let subdir_name = backup_subdir_name(version);
        Some(base_dir.join(subdir_name))
    };

    let items = discover_items(backup_path.as_deref(), true);

    let existing_count = items.iter().filter(|i| i.exists).count();
    if existing_count == 0 {
        output::warn("No aibox files found. Nothing to reset.");
        return Ok(());
    }

    let delete_count = items.iter().filter(|i| i.will_delete).count();

    // Show what will happen
    if no_backup {
        eprintln!(
            "\n\x1b[1;31m  ╔════════════════════════════════════════════════════════╗\x1b[0m"
        );
        eprintln!(
            "\x1b[1;31m  ║  DANGER: {:<6} items will be PERMANENTLY DELETED    ║\x1b[0m",
            delete_count
        );
        eprintln!("\x1b[1;31m  ║  No backup will be created. This cannot be undone.    ║\x1b[0m");
        eprintln!("\x1b[1;31m  ╚════════════════════════════════════════════════════════╝\x1b[0m");
    } else {
        output::info(&format!(
            "Backup target: {}",
            backup_path.as_ref().unwrap().display()
        ));
    }

    print_table(&items);

    if dry_run {
        output::warn("[dry-run] No files were modified.");
        return Ok(());
    }

    // Confirm
    if !yes {
        let confirm_word = if no_backup { "DELETE" } else { "reset" };
        let prompt = if no_backup {
            "\x1b[1;31m  This will permanently delete all aibox files WITHOUT backup.\x1b[0m"
                .to_string()
        } else {
            format!(
                "  This will back up to {} and then delete aibox files.",
                backup_path.as_ref().unwrap().display()
            )
        };

        if !confirm(&prompt, confirm_word)? {
            output::warn("Aborted.");
            return Ok(());
        }
    }

    // Stop container before deleting
    ensure_container_stopped(&config)?;

    // Backup phase
    if let Some(ref bp) = backup_path {
        fs::create_dir_all(bp)
            .with_context(|| format!("Failed to create backup dir: {}", bp.display()))?;

        for item in &items {
            if item.will_backup {
                let target = item.backup_target.as_ref().unwrap();
                copy_item(&item.path, target)?;
                output::ok(&format!("Backed up {}", item.path.display()));
            }
        }
    }

    // Selectively remove aibox-managed command files from .claude/commands/
    // BEFORE the delete phase removes context/ (which contains the templates
    // mirror we need to know which filenames are ours). User-authored commands
    // in the same directory are left untouched. Best-effort: failure is
    // warned-and-continued so a missing mirror doesn't abort the whole reset.
    let cwd = std::env::current_dir().unwrap_or_default();
    if let Err(e) = crate::claude_commands::remove_managed_commands(&cwd, &config) {
        output::warn(&format!("Could not clean .claude/commands/: {}", e));
    }

    // Preserve auth and cache directories from .aibox-home before deletion.
    // These contain login tokens and cached data that should survive reset.
    let auth_preserve_dir = PathBuf::from(".aibox/.preserved-auth");
    let auth_dirs_to_preserve: &[&str] = &[
        ".claude", // Claude Code login state and memory
        ".codex",  // OpenAI Codex login state
        ".ssh",    // SSH keys
        ".cargo/registry",
        ".cargo/git",
        ".aider",
        ".gemini",
    ];
    let aibox_home = PathBuf::from(".aibox-home");
    let mut preserved_any = false;
    if aibox_home.exists() {
        for sub in auth_dirs_to_preserve {
            let src = aibox_home.join(sub);
            if src.exists() {
                let dst = auth_preserve_dir.join(sub);
                if let Err(e) = copy_item(&src, &dst) {
                    output::warn(&format!("Could not preserve {}: {}", src.display(), e));
                } else {
                    preserved_any = true;
                }
            }
        }
    }

    // Delete phase
    for item in &items {
        if item.will_delete {
            delete_item(&item.path)?;
            output::ok(&format!("Deleted {}", item.path.display()));
        }
    }

    remove_dir_if_empty(Path::new(".devcontainer"))?;

    // Restore preserved auth directories into .aibox-home
    if preserved_any && auth_preserve_dir.exists() {
        fs::create_dir_all(&aibox_home).ok();
        for sub in auth_dirs_to_preserve {
            let src = auth_preserve_dir.join(sub);
            if src.exists() {
                let dst = aibox_home.join(sub);
                if let Err(e) = copy_item(&src, &dst) {
                    output::warn(&format!("Could not restore {}: {}", dst.display(), e));
                } else {
                    output::ok(&format!("Preserved auth: {}", sub));
                }
            }
        }
        // Clean up temp preserve dir
        let _ = fs::remove_dir_all(&auth_preserve_dir);
    }

    // Generate post-reset migration briefing if we have a backup.
    // This gives agents a structured document to guide reconciliation
    // between the backup (old project state) and the fresh scaffold.
    if let Some(ref bp) = backup_path {
        match generate_reset_migration_briefing(bp) {
            Ok(Some(path)) => {
                output::ok(&format!("Reset recovery migration: {}", path.display()));
            }
            Ok(None) => {} // no user content found in backup
            Err(e) => {
                output::warn(&format!(
                    "Could not generate reset migration briefing: {}",
                    e
                ));
            }
        }
    }

    output::ok("Reset complete. Project is back to pre-aibox state.");
    if let Some(bp) = &backup_path {
        output::info(&format!("Backup saved at: {}", bp.display()));
        output::info(
            "Run `aibox sync` to re-scaffold, then check context/migrations/pending/ \
             for the reset recovery migration.",
        );
    }

    Ok(())
}

/// Uninstall command: remove the CLI binary, optionally purge global config.
pub fn cmd_uninstall(dry_run: bool, purge: bool, yes: bool) -> Result<()> {
    // Find the CLI binary path (the currently running executable)
    let binary_path = std::env::current_exe()
        .context("Could not determine the path of the running aibox binary")?;

    // Global directories (XDG Base Directory Specification)
    let global_dirs = crate::dirs::all_global_dirs();
    let existing_global_dirs: Vec<&PathBuf> = global_dirs.iter().filter(|d| d.exists()).collect();
    let has_global_config = !existing_global_dirs.is_empty();

    // Determine whether to remove global directories:
    // --purge → always remove
    // no --purge, interactive → ask (default: keep)
    // no --purge, --yes → keep (safe default)
    let remove_global = if purge {
        true
    } else if has_global_config && !yes {
        ask_yes_no(
            "  Remove global config/cache (XDG directories)? [y/N] ",
            false, // default: no
        )?
    } else {
        false
    };

    if !binary_path.exists() && !remove_global {
        output::warn("Nothing to uninstall.");
        return Ok(());
    }

    // Show what will be removed
    eprintln!("\n\x1b[1;31m  ╔════════════════════════════════════════════════════════╗\x1b[0m");
    eprintln!("\x1b[1;31m  ║  DANGER: aibox will be PERMANENTLY UNINSTALLED        ║\x1b[0m");
    eprintln!("\x1b[1;31m  ╚════════════════════════════════════════════════════════╝\x1b[0m");
    eprintln!();
    eprintln!("  The following will be removed:");
    if binary_path.exists() {
        eprintln!(
            "    \x1b[31m\u{2717}\x1b[0m  {} (CLI binary)",
            binary_path.display()
        );
    }
    for dir in &existing_global_dirs {
        if remove_global {
            eprintln!("    \x1b[31m\u{2717}\x1b[0m  {} (remove)", dir.display());
        } else {
            eprintln!("    \x1b[32m\u{2713}\x1b[0m  {} (kept)", dir.display());
        }
    }
    eprintln!();
    eprintln!("  Project files (aibox.toml, .devcontainer/, context/) are NOT affected.");
    eprintln!("  Use 'aibox reset' to remove project files.");
    eprintln!();

    if dry_run {
        output::warn("[dry-run] No files were removed.");
        return Ok(());
    }

    // Confirm
    if !yes
        && !confirm(
            "  \x1b[1;31mThis will permanently remove the aibox CLI.\x1b[0m",
            "uninstall",
        )?
    {
        output::warn("Aborted.");
        return Ok(());
    }

    // Remove global directories first (while the binary is still running)
    if remove_global {
        for dir in &existing_global_dirs {
            delete_item(dir)?;
            output::ok(&format!("Removed {}", dir.display()));
        }
    }

    // Remove the binary last
    if binary_path.exists() {
        // On Unix, a running binary can be deleted — the OS keeps the inode
        // alive until the process exits.
        fs::remove_file(&binary_path).with_context(|| {
            format!(
                "Failed to remove binary: {}. You may need to remove it manually.",
                binary_path.display()
            )
        })?;
        output::ok(&format!("Removed {}", binary_path.display()));
    }

    eprintln!();
    output::ok("aibox has been uninstalled.");
    eprintln!(
        "  To reinstall: curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash"
    );

    Ok(())
}

/// Ask a yes/no question with a default. Returns the user's choice.
/// Empty input (just Enter) returns the default.
fn ask_yes_no(prompt: &str, default: bool) -> Result<bool> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Ok(default);
    }
    eprint!("{}", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed == "y" || trimmed == "yes")
    }
}

// =============================================================================
// Post-reset migration briefing
// =============================================================================

/// Walk a directory recursively and return relative paths (as strings).
fn walk_dir_relative(base: &Path, current: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                paths.extend(walk_dir_relative(base, &p));
            } else if let Ok(rel) = p.strip_prefix(base) {
                paths.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    paths
}

/// Generate a post-reset migration briefing document.
///
/// Scans the backup directory for user-created content (entities, custom
/// configs, additional files) and writes a migration document to
/// `context/migrations/pending/` that guides the agent through
/// reconciliation after `aibox sync` re-scaffolds the project.
fn generate_reset_migration_briefing(backup_dir: &Path) -> Result<Option<PathBuf>> {
    let pending_dir = PathBuf::from("context/migrations/pending");

    // Collect user content from the backup.
    let mut user_entities: Vec<String> = Vec::new();
    let mut custom_configs: Vec<String> = Vec::new();
    let mut additional_devcontainer: Vec<String> = Vec::new();

    // Scan context/ subdirectories for user-created entity files.
    let backup_context = backup_dir.join("context");
    let entity_dirs = [
        "workitems",
        "decisions",
        "discussions",
        "logs",
        "actors",
        "artifacts",
        "notes",
    ];
    for dir_name in &entity_dirs {
        let dir = backup_context.join(dir_name);
        if dir.is_dir() {
            for file in walk_dir_relative(&backup_context, &dir) {
                if file.ends_with(".md") || file.ends_with(".yaml") || file.ends_with(".yml") {
                    user_entities.push(format!("context/{}", file));
                }
            }
        }
    }

    // Check for customized AGENTS.md
    let agents_md = backup_dir.join("AGENTS.md");
    if agents_md.is_file() {
        custom_configs.push("AGENTS.md".to_string());
    }

    // Check for custom .devcontainer files beyond the managed set.
    let backup_devcontainer = backup_dir.join(".devcontainer");
    if backup_devcontainer.is_dir() {
        let managed = [
            "Dockerfile",
            "docker-compose.yml",
            "devcontainer.json",
            "Dockerfile.local",
            "docker-compose.override.yml",
        ];
        for file in walk_dir_relative(&backup_devcontainer, &backup_devcontainer) {
            if !managed.contains(&file.as_str()) {
                additional_devcontainer.push(format!(".devcontainer/{}", file));
            }
        }
    }

    // Check for skill customizations (edited SKILL.md files etc.)
    let backup_skills = backup_context.join("skills");
    if backup_skills.is_dir() {
        // We'll note that skills existed; the agent should diff them.
        custom_configs.push("context/skills/ (check for local edits)".to_string());
    }

    // If nothing interesting, skip.
    if user_entities.is_empty() && custom_configs.is_empty() && additional_devcontainer.is_empty() {
        return Ok(None);
    }

    // Generate the migration document.
    fs::create_dir_all(&pending_dir)
        .with_context(|| format!("failed to create {}", pending_dir.display()))?;

    let now = chrono::Utc::now();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = format!("MIG-RESET-{}", now.format("%Y%m%dT%H%M%S"));
    let out_path = pending_dir.join(format!("{}.md", id));

    let total = user_entities.len() + custom_configs.len() + additional_devcontainer.len();

    let mut body = String::new();
    body.push_str("---\n");
    body.push_str("apiVersion: processkit.projectious.work/v1\n");
    body.push_str("kind: Migration\n");
    body.push_str("metadata:\n");
    body.push_str(&format!("  id: {}\n", id));
    body.push_str(&format!("  created: {}\n", now_iso));
    body.push_str("spec:\n");
    body.push_str("  source: aibox-reset-recovery\n");
    body.push_str("  source_url: aibox://reset-recovery\n");
    body.push_str(&format!(
        "  from_version: \"{}\"\n",
        backup_dir
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default()
    ));
    body.push_str("  to_version: fresh-scaffold\n");
    body.push_str("  state: pending\n");
    body.push_str("  generated_by: aibox reset\n");
    body.push_str(&format!("  generated_at: {}\n", now_iso));
    body.push_str(&format!(
        "  summary: \"{} items from backup to review for recovery\"\n",
        total
    ));
    body.push_str("  affected_groups:\n");
    if !user_entities.is_empty() {
        body.push_str("    - user-entities\n");
    }
    if !custom_configs.is_empty() {
        body.push_str("    - custom-configs\n");
    }
    if !additional_devcontainer.is_empty() {
        body.push_str("    - additional-devcontainer\n");
    }
    body.push_str("---\n\n");
    body.push_str(&format!("# Reset Recovery — {}\n\n", id));
    body.push_str(&format!(
        "This migration was generated after `aibox reset`. The backup at\n\
         `{}` contains the previous project state.\n\n\
         Review each section below and decide what to bring forward into\n\
         the freshly scaffolded project. Use 3-way reasoning:\n\
         - **Backup** = what the user had before reset\n\
         - **Fresh scaffold** = what `aibox sync` just created\n\
         - **Desired state** = merge of both, guided by user intent\n\n",
        backup_dir.display()
    ));

    if !user_entities.is_empty() {
        body.push_str("## User-created entities\n\n");
        body.push_str(
            "These entity files existed in the backup. Review each and copy\n\
             those that are still relevant into the fresh `context/` tree.\n\n",
        );
        for f in &user_entities {
            body.push_str(&format!("- `{}`\n", f));
        }
        body.push('\n');
    }

    if !custom_configs.is_empty() {
        body.push_str("## Customized configuration files\n\n");
        body.push_str(
            "These files were customized in the backup. Diff them against\n\
             the fresh versions and merge relevant changes.\n\n",
        );
        for f in &custom_configs {
            body.push_str(&format!("- `{}`\n", f));
        }
        body.push('\n');
    }

    if !additional_devcontainer.is_empty() {
        body.push_str("## Additional .devcontainer files\n\n");
        body.push_str(
            "These files are not managed by aibox and were in the backup.\n\
             Copy them back if they are still needed.\n\n",
        );
        for f in &additional_devcontainer {
            body.push_str(&format!("- `{}`\n", f));
        }
        body.push('\n');
    }

    body.push_str("## Auth state\n\n");
    body.push_str(
        "Login credentials and SSH keys from `.aibox-home/` were automatically\n\
         preserved during reset (`.claude/`, `.codex/`, `.ssh/`, `.cargo/` caches).\n\
         No manual action needed for auth.\n\n",
    );

    body.push_str("## AGENTS.md review\n\n");
    body.push_str(
        "After re-scaffolding with `aibox sync`, verify AGENTS.md is up to date:\n\
         - processkit version matches `aibox.lock`\n\
         - Configured AI harnesses/providers match `[ai]` in `aibox.toml`\n\
         - Build / test / lint commands are still accurate\n\
         - Project-specific notes and operational gotchas are current\n",
    );

    fs::write(&out_path, body)
        .with_context(|| format!("failed to write {}", out_path.display()))?;

    Ok(Some(out_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_project(dir: &Path) {
        // Create a minimal aibox project structure
        fs::write(
            dir.join("aibox.toml"),
            r#"[aibox]
version = "0.3.8"
image = "base"
process = "minimal"

[container]
name = "test-project"
"#,
        )
        .unwrap();
        fs::create_dir_all(dir.join(".devcontainer")).unwrap();
        fs::write(dir.join(".devcontainer/Dockerfile"), "FROM debian").unwrap();
        fs::create_dir_all(dir.join(".aibox-home/.vim")).unwrap();
        fs::write(dir.join(".aibox-home/.vim/vimrc"), "set nocp").unwrap();
        fs::write(dir.join(".aibox-version"), "0.3.8").unwrap();
        fs::create_dir_all(dir.join("context")).unwrap();
        fs::write(dir.join("context/DECISIONS.md"), "# Decisions").unwrap();
        fs::write(dir.join("CLAUDE.md"), "# Project").unwrap();
        fs::write(dir.join(".gitignore"), ".aibox-home/\n").unwrap();
    }

    #[test]
    fn backup_subdir_name_format() {
        let name = backup_subdir_name("0.3.8");
        assert!(name.starts_with("aibox-0.3.8-backup-"));
        // Should match YYYY-MM-DD-HHMM pattern
        assert!(name.len() > 30);
    }

    #[test]
    #[serial_test::serial]
    fn discover_items_finds_existing() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        setup_project(dir.path());

        let items = discover_items(None, true);
        let existing: Vec<_> = items.iter().filter(|i| i.exists).collect();
        assert!(existing.len() >= 6); // toml, devcontainer, home, legacy .aibox-version, context, claude.md, gitignore

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn discover_items_gitignore_not_deleted() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        setup_project(dir.path());

        let items = discover_items(None, true);
        let gitignore = items.iter().find(|i| i.path == Path::new(".gitignore"));
        assert!(gitignore.is_some());
        assert!(!gitignore.unwrap().will_delete);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn copy_dir_recursive_works() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        let dst = dir.path().join("dst");

        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("file.txt"), "hello").unwrap();
        fs::write(src.join("sub/nested.txt"), "world").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();

        assert!(dst.join("file.txt").exists());
        assert!(dst.join("sub/nested.txt").exists());
        assert_eq!(fs::read_to_string(dst.join("file.txt")).unwrap(), "hello");
        assert_eq!(
            fs::read_to_string(dst.join("sub/nested.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    #[serial_test::serial]
    fn reset_backup_preserves_user_edited_generated_and_local_files() {
        let dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        setup_project(dir.path());

        fs::write(
            dir.path().join("aibox.toml"),
            "[aibox]\nversion = \"0.3.8\"\n# user edit\n\n[container]\nname = \"test-project\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("aibox.lock"),
            "version = \"0.3.8\"\n# user edit\n",
        )
        .unwrap();
        fs::write(
            dir.path().join(".devcontainer/Dockerfile"),
            "FROM debian\n# user edit\n",
        )
        .unwrap();
        fs::write(
            dir.path().join(".devcontainer/docker-compose.yml"),
            "services:\n  app:\n    image: test\n# user edit\n",
        )
        .unwrap();
        fs::write(
            dir.path().join(".devcontainer/devcontainer.json"),
            "{\n  \"name\": \"test\"\n}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join(".devcontainer/Dockerfile.local"),
            "RUN echo local-user-layer\n",
        )
        .unwrap();
        fs::write(
            dir.path().join(".devcontainer/docker-compose.override.yml"),
            "services:\n  db:\n    image: postgres:16\n",
        )
        .unwrap();
        fs::write(
            dir.path().join(".devcontainer/local-secrets.txt"),
            "token=super-secret\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join(".aibox-home/.config/yazi")).unwrap();
        fs::write(
            dir.path().join(".aibox-home/.config/yazi/keymap.toml"),
            "# user tweak\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("context/DECISIONS.md"),
            "# Decisions\nuser note\n",
        )
        .unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "# Project\nuser note\n").unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# Agents\nuser note\n").unwrap();
        fs::write(
            dir.path().join(".gitignore"),
            ".aibox-home/\n# user tweak\n",
        )
        .unwrap();

        cmd_reset(&None, false, false, true).unwrap();

        let backup_root = dir.path().join(BACKUP_DIR);
        assert!(backup_root.is_dir(), "backup root should exist");
        let backup_dir = fs::read_dir(&backup_root)
            .unwrap()
            .next()
            .expect("expected one backup directory")
            .unwrap()
            .path();

        let assert_backup = |rel: &str, expected: &str| {
            let path = backup_dir.join(rel);
            assert!(path.exists(), "backup should contain {}", rel);
            let content = fs::read_to_string(&path).unwrap();
            assert!(
                content.contains(expected),
                "backup for {} should preserve user content; got: {}",
                rel,
                content
            );
        };

        assert_backup("aibox.toml", "# user edit");
        assert_backup("aibox.lock", "# user edit");
        assert_backup(".devcontainer/Dockerfile", "# user edit");
        assert_backup(".devcontainer/docker-compose.yml", "# user edit");
        assert_backup(".devcontainer/devcontainer.json", "\"name\": \"test\"");
        assert_backup(".devcontainer/Dockerfile.local", "local-user-layer");
        assert_backup(".devcontainer/docker-compose.override.yml", "postgres:16");
        assert_backup(".devcontainer/local-secrets.txt", "super-secret");
        assert_backup(".aibox-home/.config/yazi/keymap.toml", "user tweak");
        assert_backup("context/DECISIONS.md", "user note");
        assert_backup("CLAUDE.md", "user note");
        assert_backup("AGENTS.md", "user note");
        assert_backup(".gitignore", "user tweak");

        assert!(!dir.path().join("aibox.toml").exists());
        assert!(!dir.path().join(".devcontainer/Dockerfile").exists());
        assert!(!dir.path().join(".devcontainer/docker-compose.yml").exists());
        assert!(!dir.path().join(".devcontainer/devcontainer.json").exists());
        assert!(!dir.path().join(".devcontainer/Dockerfile.local").exists());
        assert!(
            !dir.path()
                .join(".devcontainer/docker-compose.override.yml")
                .exists()
        );
        assert!(
            dir.path().join(".devcontainer/local-secrets.txt").exists(),
            "unknown local .devcontainer files should be left in place"
        );
        assert!(!dir.path().join(".aibox-home").exists());
        assert!(dir.path().join(".gitignore").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
