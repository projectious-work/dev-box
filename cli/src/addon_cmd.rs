use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::PathBuf;

use crate::addon_loader;
use crate::addon_registry;
use crate::cli::OutputFormat;
use crate::config::AiboxConfig;
use crate::output;

/// Resolve the path to aibox.toml from the CLI option.
fn toml_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

fn category_order(cat: &str) -> usize {
    match cat {
        "AI Providers" => 0,
        "Languages" => 1,
        "Documentation" => 2,
        "Tools" => 3,
        _ => 4,
    }
}

/// List all available add-ons and their install status.
pub fn cmd_addon_list(config_path: &Option<String>, format: OutputFormat) -> Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        name: &'a str,
        category: &'a str,
        description: &'a str,
        tools: usize,
        status: &'static str,
    }

    let config = AiboxConfig::from_cli_option(config_path).ok();
    let addons = addon_loader::all_addons();

    let mut rows: Vec<Row> = addons
        .iter()
        .map(|a| {
            let installed = config.as_ref().is_some_and(|c| c.addons.has_addon(&a.name));
            Row {
                name: &a.name,
                category: &a.category,
                description: &a.description,
                tools: a.tools.len(),
                status: if installed { "installed" } else { "available" },
            }
        })
        .collect();

    // Sort: category order first, then name alphabetically
    rows.sort_by(|a, b| {
        category_order(a.category)
            .cmp(&category_order(b.category))
            .then(a.name.cmp(b.name))
    });

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        OutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&rows)?);
        }
        OutputFormat::Table => {
            let name_width = rows.iter().map(|r| r.name.len()).max().unwrap_or(10).max(5);
            let desc_width = rows.iter().map(|r| r.description.len()).max().unwrap_or(20).max(11);

            // Group by category and print with headers
            let mut current_cat = "";
            for r in &rows {
                if r.category != current_cat {
                    if !current_cat.is_empty() {
                        println!();
                    }
                    println!("  \x1b[1m{}\x1b[0m", r.category);
                    println!(
                        "  {:<nw$}  {:<dw$}  {:>5}  STATUS",
                        "ADDON", "DESCRIPTION", "TOOLS",
                        nw = name_width, dw = desc_width
                    );
                    current_cat = r.category;
                }
                println!(
                    "  {:<nw$}  {:<dw$}  {:>5}  {}",
                    r.name, r.description, r.tools, r.status,
                    nw = name_width, dw = desc_width
                );
            }
        }
    }

    Ok(())
}

/// Add an add-on to aibox.toml with default-enabled tools, then sync.
pub fn cmd_addon_add(config_path: &Option<String>, name: &str, no_build: bool) -> Result<()> {
    // Verify the named addon exists before touching the config file.
    if addon_registry::get_addon(name).is_none() {
        bail!(
            "Unknown add-on '{}'. Run 'aibox addon list' to see available add-ons.",
            name
        );
    }

    let path = toml_path(config_path);
    if !path.exists() {
        bail!("No aibox.toml found. Run 'aibox init' first.");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // Transitively expand `requires`. Picking `docs-docusaurus` here
    // also pulls in `node`, etc. The expansion is idempotent: any
    // addon already present in aibox.toml is a no-op.
    let expanded = crate::container::expand_addon_requires(std::slice::from_ref(&name.to_string()));

    // Ensure [addons] table exists.
    if doc.get("addons").is_none() {
        doc.insert("addons", toml_edit::Item::Table(toml_edit::Table::new()));
    }

    let mut added_any = false;
    for addon_name in &expanded {
        if doc.get("addons").and_then(|a| a.get(addon_name)).is_some() {
            if addon_name == name {
                output::warn(&format!("Add-on '{}' is already in aibox.toml", name));
                return Ok(());
            }
            // Transitive dep already present — nothing to do for this entry.
            continue;
        }

        let addon_def = addon_registry::get_addon(addon_name).ok_or_else(|| {
            anyhow::anyhow!(
                "Internal: addon '{}' was returned by expand_addon_requires \
                 but is unknown to the registry",
                addon_name
            )
        })?;

        // Build tools table with default-enabled tools at default versions.
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
        doc["addons"][addon_name.as_str()] = toml_edit::Item::Table(addon_table);

        if addon_name == name {
            output::ok(&format!("Added add-on '{}' to aibox.toml", name));
        } else {
            output::info(&format!(
                "Adding addon '{}' (transitively required by '{}')",
                addon_name, name
            ));
        }
        added_any = true;
    }

    if !added_any {
        return Ok(());
    }

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;

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
pub fn cmd_addon_info(name: &str, format: OutputFormat) -> Result<()> {
    let loaded = addon_loader::get_addon(name)
        .ok_or_else(|| anyhow::anyhow!("Unknown add-on '{}'. Run 'aibox addon list' to see available add-ons.", name))?;

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            #[derive(Serialize)]
            struct ToolRow<'a> {
                name: &'a str,
                default_enabled: bool,
                default_version: &'a str,
                supported_versions: &'a [String],
            }
            #[derive(Serialize)]
            struct InfoOut<'a> {
                name: &'a str,
                category: &'a str,
                description: &'a str,
                addon_version: &'a str,
                requires: &'a [String],
                tools: Vec<ToolRow<'a>>,
            }
            let out = InfoOut {
                name: &loaded.name,
                category: &loaded.category,
                description: &loaded.description,
                addon_version: &loaded.addon_version,
                requires: &loaded.requires,
                tools: loaded.tools.iter().map(|t| ToolRow {
                    name: &t.name,
                    default_enabled: t.default_enabled,
                    default_version: &t.default_version,
                    supported_versions: &t.supported_versions,
                }).collect(),
            };
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{}", serde_yaml::to_string(&out)?);
            }
        }
        OutputFormat::Table => {
            println!("Add-on:       {}", loaded.name);
            println!("Category:     {}", loaded.category);
            if !loaded.description.is_empty() {
                println!("Description:  {}", loaded.description);
            }
            println!("Version:      {}", loaded.addon_version);
            if !loaded.requires.is_empty() {
                println!("Requires:     {}", loaded.requires.join(", "));
            }
            println!();

            if loaded.tools.is_empty() {
                println!("  (no tools)");
                return Ok(());
            }

            let name_width = loaded.tools.iter().map(|t| t.name.len()).max().unwrap_or(4).max(4);
            println!(
                "  {:<nw$}  {:>7}  {:>10}  SUPPORTED",
                "TOOL", "DEFAULT", "VERSION", nw = name_width
            );
            for tool in &loaded.tools {
                let default = if tool.default_enabled { "yes" } else { "no" };
                let version = if tool.default_version.is_empty() { "-" } else { &tool.default_version };
                let supported = if tool.supported_versions.is_empty() {
                    "-".to_string()
                } else {
                    tool.supported_versions.join(", ")
                };
                println!(
                    "  {:<nw$}  {:>7}  {:>10}  {}",
                    tool.name, default, version, supported, nw = name_width
                );
            }
        }
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
        assert!(cmd_addon_info("python", OutputFormat::Table).is_ok());
    }

    #[test]
    fn addon_info_errors_on_unknown() {
        ensure_loaded();
        assert!(cmd_addon_info("nonexistent", OutputFormat::Table).is_err());
    }
}
