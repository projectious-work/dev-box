use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AiboxConfig;
use crate::output;
use crate::runtime::{ContainerState, Runtime};

/// Backup directory name.
const BACKUP_DIR: &str = ".aibox-backup";

/// Files and directories managed by aibox.
/// Each entry: (path, should_delete_on_reset)
/// .gitignore is backed up but not deleted.
const MANAGED_ITEMS: &[(&str, bool)] = &[
    ("aibox.toml", true),
    (".devcontainer", true),
    (".aibox-home", true),
    (".aibox-version", true),
    ("context", true),
    ("CLAUDE.md", true),
    (".gitignore", false),
    // Backward compat
    (".root", true),
    (".aibox", true),
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
    for entry in fs::read_dir(src)
        .with_context(|| format!("Failed to read directory: {}", src.display()))?
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
        eprintln!(
            "\x1b[1;31m  ║  No backup will be created. This cannot be undone.    ║\x1b[0m"
        );
        eprintln!(
            "\x1b[1;31m  ╚════════════════════════════════════════════════════════╝\x1b[0m"
        );
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

    // Delete phase
    for item in &items {
        if item.will_delete {
            delete_item(&item.path)?;
            output::ok(&format!("Deleted {}", item.path.display()));
        }
    }

    output::ok("Reset complete. Project is back to pre-aibox state.");
    if let Some(bp) = &backup_path {
        output::info(&format!("Backup saved at: {}", bp.display()));
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
    let existing_global_dirs: Vec<&PathBuf> =
        global_dirs.iter().filter(|d| d.exists()).collect();
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
    eprintln!(
        "\n\x1b[1;31m  ╔════════════════════════════════════════════════════════╗\x1b[0m"
    );
    eprintln!(
        "\x1b[1;31m  ║  DANGER: aibox will be PERMANENTLY UNINSTALLED        ║\x1b[0m"
    );
    eprintln!(
        "\x1b[1;31m  ╚════════════════════════════════════════════════════════╝\x1b[0m"
    );
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
            eprintln!(
                "    \x1b[31m\u{2717}\x1b[0m  {} (remove)",
                dir.display()
            );
        } else {
            eprintln!(
                "    \x1b[32m\u{2713}\x1b[0m  {} (kept)",
                dir.display()
            );
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
    eprintln!("  To reinstall: curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash");

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
        assert!(existing.len() >= 6); // toml, devcontainer, home, version, context, claude.md, gitignore

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
        let gitignore = items.iter().find(|i| i.path == PathBuf::from(".gitignore"));
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
}
