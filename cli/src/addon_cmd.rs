use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::addon_loader;
use crate::addon_registry;
use crate::config::AiboxConfig;
use crate::output;

/// Resolve the path to aibox.toml from the CLI option.
fn toml_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

/// List all available add-ons and their install status.
pub fn cmd_addon_list(config_path: &Option<String>) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path).ok();

    let addons = addon_registry::all_addons();

    // Calculate column widths
    let max_name = addons.iter().map(|a| a.name.len()).max().unwrap_or(10);
    let name_width = max_name.max(6); // minimum "ADDON" header width

    println!(
        "  {:<width$}  {:>5}  STATUS",
        "ADDON",
        "TOOLS",
        width = name_width
    );

    for addon in addons {
        let installed = config
            .as_ref()
            .is_some_and(|c| c.addons.has_addon(addon.name));
        let status = if installed { "installed" } else { "available" };
        let tool_count = addon.tools.len();

        println!(
            "  {:<width$}  {:>5}  {}",
            addon.name,
            tool_count,
            status,
            width = name_width
        );
    }

    Ok(())
}

/// Add an add-on to aibox.toml with default-enabled tools, then sync.
pub fn cmd_addon_add(config_path: &Option<String>, name: &str, no_build: bool) -> Result<()> {
    let addon_def = addon_registry::get_addon(name)
        .ok_or_else(|| anyhow::anyhow!("Unknown add-on '{}'. Run 'aibox addon list' to see available add-ons.", name))?;

    let path = toml_path(config_path);
    if !path.exists() {
        bail!("No aibox.toml found. Run 'aibox init' first.");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // Check if already present
    if doc
        .get("addons")
        .and_then(|a| a.get(name))
        .is_some()
    {
        output::warn(&format!("Add-on '{}' is already in aibox.toml", name));
        return Ok(());
    }

    // Build tools table with default-enabled tools
    let mut tools_table = toml_edit::Table::new();
    for tool in addon_def.tools.iter().filter(|t| t.default_enabled) {
        if tool.default_version.is_empty() {
            // Versionless tool: tool_name = {}
            tools_table.insert(tool.name, toml_edit::value(toml_edit::InlineTable::new()));
        } else {
            // Versioned tool: tool_name = { version = "X.Y" }
            let mut entry = toml_edit::InlineTable::new();
            entry.insert("version", tool.default_version.into());
            tools_table.insert(tool.name, toml_edit::value(entry));
        }
    }

    // Ensure [addons] table exists
    if doc.get("addons").is_none() {
        doc.insert("addons", toml_edit::Item::Table(toml_edit::Table::new()));
    }

    // Insert [addons.<name>] with tools subtable
    let mut addon_table = toml_edit::Table::new();
    addon_table.insert("tools", toml_edit::Item::Table(tools_table));
    doc["addons"][name] = toml_edit::Item::Table(addon_table);

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;

    output::ok(&format!("Added add-on '{}' to aibox.toml", name));

    // Run sync to apply changes
    crate::container::cmd_sync(config_path, false, no_build)?;

    Ok(())
}

/// Remove an add-on from aibox.toml, then sync.
pub fn cmd_addon_remove(config_path: &Option<String>, name: &str, no_build: bool) -> Result<()> {
    let path = toml_path(config_path);
    if !path.exists() {
        bail!("No aibox.toml found. Run 'aibox init' first.");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // Check if present
    let removed = doc
        .get_mut("addons")
        .and_then(|a| a.as_table_mut())
        .and_then(|t| t.remove(name))
        .is_some();

    if !removed {
        output::warn(&format!("Add-on '{}' is not in aibox.toml", name));
        return Ok(());
    }

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;

    output::ok(&format!("Removed add-on '{}' from aibox.toml", name));

    // Run sync to apply changes
    crate::container::cmd_sync(config_path, false, no_build)?;

    Ok(())
}

/// Show detailed info about an add-on.
pub fn cmd_addon_info(name: &str) -> Result<()> {
    let addon_def = addon_registry::get_addon(name)
        .ok_or_else(|| anyhow::anyhow!("Unknown add-on '{}'. Run 'aibox addon list' to see available add-ons.", name))?;

    println!("Add-on: {}", addon_def.name);
    println!("Recipe version: {}", addon_def.addon_version);

    // Show requires if present (sourced from loader which retains the full data).
    if let Some(loaded) = addon_loader::get_addon(name)
        && !loaded.requires.is_empty()
    {
        println!("Requires: {}", loaded.requires.join(", "));
    }

    println!();

    if addon_def.tools.is_empty() {
        println!("  (no tools)");
        return Ok(());
    }

    // Calculate column widths
    let max_name = addon_def
        .tools
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(4);
    let name_width = max_name.max(4);

    println!(
        "  {:<nw$}  {:>7}  {:>10}  SUPPORTED",
        "TOOL",
        "DEFAULT",
        "VERSION",
        nw = name_width
    );

    for tool in addon_def.tools {
        let default = if tool.default_enabled { "yes" } else { "no" };
        let version = if tool.default_version.is_empty() {
            "-"
        } else {
            tool.default_version
        };
        let supported = if tool.supported_versions.is_empty() {
            "-".to_string()
        } else {
            tool.supported_versions.join(", ")
        };

        println!(
            "  {:<nw$}  {:>7}  {:>10}  {}",
            tool.name,
            default,
            version,
            supported,
            nw = name_width
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn ensure_loaded() {
        let addons_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("addons");
        let _ = crate::addon_loader::init_from_dir(&addons_dir);
    }

    fn write_test_toml(dir: &std::path::Path) -> PathBuf {
        let path = dir.join("aibox.toml");
        fs::write(
            &path,
            r#"[aibox]
version = "0.9.0"

[container]
name = "test"

[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
"#,
        )
        .unwrap();
        path
    }

    #[test]
    fn addon_add_inserts_section() {
        ensure_loaded();
        let dir = tempfile::tempdir().unwrap();
        let path = write_test_toml(dir.path());

        // We can't run full cmd_addon_add (it calls sync which needs docker),
        // so test the toml_edit logic directly.
        let content = fs::read_to_string(&path).unwrap();
        let mut doc = content.parse::<toml_edit::DocumentMut>().unwrap();

        let addon_def = addon_registry::get_addon("rust").unwrap();
        let mut tools_table = toml_edit::Table::new();
        for tool in addon_def.tools.iter().filter(|t| t.default_enabled) {
            if tool.default_version.is_empty() {
                tools_table.insert(tool.name, toml_edit::value(toml_edit::InlineTable::new()));
            } else {
                let mut entry = toml_edit::InlineTable::new();
                entry.insert("version", tool.default_version.into());
                tools_table.insert(tool.name, toml_edit::value(entry));
            }
        }

        let mut addon_table = toml_edit::Table::new();
        addon_table.insert("tools", toml_edit::Item::Table(tools_table));
        doc["addons"]["rust"] = toml_edit::Item::Table(addon_table);

        let result = doc.to_string();
        assert!(result.contains("[addons.rust.tools]"), "should have rust addon section");
        assert!(result.contains("rustc"), "should have rustc tool");

        // Original python addon should still be there
        assert!(result.contains("[addons.python.tools]"), "should preserve python addon");

        // Verify the result is valid TOML that our config parser can read
        let _config: crate::config::AiboxConfig =
            toml::from_str(&result).expect("result should be valid aibox.toml");
    }

    #[test]
    fn addon_remove_deletes_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_test_toml(dir.path());

        let content = fs::read_to_string(&path).unwrap();
        let mut doc = content.parse::<toml_edit::DocumentMut>().unwrap();

        let removed = doc
            .get_mut("addons")
            .and_then(|a| a.as_table_mut())
            .and_then(|t| t.remove("python"))
            .is_some();

        assert!(removed, "should find and remove python addon");

        let result = doc.to_string();
        assert!(
            !result.contains("[addons.python"),
            "should not have python addon after removal"
        );
    }

    #[test]
    fn addon_info_finds_known_addon() {
        ensure_loaded();
        assert!(cmd_addon_info("python").is_ok());
    }

    #[test]
    fn addon_info_errors_on_unknown() {
        ensure_loaded();
        assert!(cmd_addon_info("nonexistent").is_err());
    }
}
