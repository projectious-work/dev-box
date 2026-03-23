use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::config::AiboxConfig;
use crate::context;
use crate::output;
use crate::process_registry;

/// Resolve the path to aibox.toml from the CLI option.
fn toml_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

/// Compute the effective skill set from a loaded config.
fn effective_skills(config: &AiboxConfig) -> Result<Vec<String>> {
    let packages = process_registry::resolve_packages(&config.process.packages)
        .map_err(|e| anyhow::anyhow!(e))?;
    let skills = process_registry::resolve_skills(
        &packages,
        &config.skills.include,
        &config.skills.exclude,
    )
    .map_err(|e| anyhow::anyhow!(e))?;
    Ok(skills)
}

/// List all available skills and their deploy status.
pub fn cmd_skill_list(config_path: &Option<String>) -> Result<()> {
    let all_names = context::all_skill_names();
    let config = AiboxConfig::from_cli_option(config_path).ok();

    let effective: Vec<String> = config
        .as_ref()
        .and_then(|c| effective_skills(c).ok())
        .unwrap_or_default();

    // Find which package provides each skill
    let packages = process_registry::all_packages();

    let max_name = all_names.iter().map(|n| n.len()).max().unwrap_or(10);
    let name_width = max_name.max(5);

    println!(
        "  {:<nw$}  {:<14}  STATUS",
        "SKILL",
        "PACKAGE",
        nw = name_width
    );

    for name in &all_names {
        let in_effective = effective.iter().any(|s| s == name);
        let status = if in_effective { "active" } else { "available" };

        // Find source package
        let source = packages
            .iter()
            .find(|p| p.skills.contains(name))
            .map(|p| p.name)
            .unwrap_or("-");

        println!(
            "  {:<nw$}  {:<14}  {}",
            name,
            source,
            status,
            nw = name_width
        );
    }

    println!();
    println!(
        "  {} total skills, {} active",
        all_names.len(),
        effective.len()
    );

    Ok(())
}

/// Add a skill to [skills].include in aibox.toml.
pub fn cmd_skill_add(config_path: &Option<String>, name: &str) -> Result<()> {
    // Verify skill exists in registry
    if context::skill_content(name).is_none() {
        bail!(
            "Unknown skill '{}'. Run 'aibox skill list' to see available skills.",
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

    // Check if already in include list
    let already_included = doc
        .get("skills")
        .and_then(|s| s.get("include"))
        .and_then(|i| i.as_array())
        .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(name)));

    if already_included {
        output::warn(&format!("Skill '{}' is already in [skills].include", name));
        return Ok(());
    }

    // Remove from exclude if present
    if let Some(skills) = doc.get_mut("skills")
        && let Some(exclude) = skills.get_mut("exclude")
        && let Some(arr) = exclude.as_array_mut()
    {
        arr.retain(|v| v.as_str() != Some(name));
    }

    // Add to include list
    if doc.get("skills").is_none() {
        doc.insert("skills", toml_edit::Item::Table(toml_edit::Table::new()));
    }
    if doc["skills"].get("include").is_none() {
        doc["skills"]["include"] = toml_edit::value(toml_edit::Array::new());
    }
    doc["skills"]["include"]
        .as_array_mut()
        .expect("include should be array")
        .push(name);

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;

    output::ok(&format!("Added skill '{}' to [skills].include", name));

    // Run skill reconciliation
    let config = AiboxConfig::from_cli_option(config_path)?;
    context::reconcile_skills(&config)?;

    Ok(())
}

/// Remove a skill by managing [skills].include and [skills].exclude.
pub fn cmd_skill_remove(config_path: &Option<String>, name: &str) -> Result<()> {
    // Don't allow removing core skills
    if process_registry::CORE_SKILLS.contains(&name) {
        bail!("Cannot remove core skill '{}'", name);
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

    // Check if it's in include — if so, just remove from include
    let was_in_include = if let Some(include) = doc
        .get_mut("skills")
        .and_then(|s| s.get_mut("include"))
        .and_then(|i| i.as_array_mut())
    {
        let before = include.len();
        include.retain(|v| v.as_str() != Some(name));
        include.len() < before
    } else {
        false
    };

    if was_in_include {
        std::fs::write(&path, doc.to_string())
            .with_context(|| format!("Failed to write {}", path.display()))?;
        output::ok(&format!(
            "Removed skill '{}' from [skills].include",
            name
        ));
    } else {
        // Not in include — add to exclude
        if doc.get("skills").is_none() {
            doc.insert("skills", toml_edit::Item::Table(toml_edit::Table::new()));
        }
        if doc["skills"].get("exclude").is_none() {
            doc["skills"]["exclude"] = toml_edit::value(toml_edit::Array::new());
        }

        let already_excluded = doc["skills"]["exclude"]
            .as_array()
            .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(name)));

        if already_excluded {
            output::warn(&format!("Skill '{}' is already excluded", name));
            return Ok(());
        }

        doc["skills"]["exclude"]
            .as_array_mut()
            .expect("exclude should be array")
            .push(name);

        std::fs::write(&path, doc.to_string())
            .with_context(|| format!("Failed to write {}", path.display()))?;
        output::ok(&format!("Added skill '{}' to [skills].exclude", name));
    }

    // Run skill reconciliation
    let config = AiboxConfig::from_cli_option(config_path)?;
    context::reconcile_skills(&config)?;

    Ok(())
}

/// Show info about a skill.
pub fn cmd_skill_info(name: &str) -> Result<()> {
    let content = context::skill_content(name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown skill '{}'. Run 'aibox skill list' to see available skills.",
            name
        )
    })?;

    // Find source package
    let packages = process_registry::all_packages();
    let source = packages
        .iter()
        .find(|p| p.skills.contains(&name))
        .map(|p| p.name);

    println!("Skill: {}", name);
    if let Some(pkg) = source {
        println!("Package: {}", pkg);
    } else {
        println!("Package: (standalone — add via [skills].include)");
    }
    println!();

    // Print first ~20 lines of SKILL.md content as a preview
    let lines: Vec<&str> = content.lines().collect();
    let preview_end = lines.len().min(20);
    for line in &lines[..preview_end] {
        println!("  {}", line);
    }
    if lines.len() > preview_end {
        println!("  ... ({} more lines)", lines.len() - preview_end);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_info_finds_known_skill() {
        assert!(cmd_skill_info("code-review").is_ok());
    }

    #[test]
    fn skill_info_errors_on_unknown() {
        assert!(cmd_skill_info("nonexistent-skill").is_err());
    }

    #[test]
    fn all_skill_names_not_empty() {
        let names = context::all_skill_names();
        assert!(names.len() >= 80, "expected at least 80 skills");
    }

    #[test]
    fn effective_skills_resolves_from_config() {
        let config = crate::config::test_config();
        let skills = effective_skills(&config).unwrap();
        // Default config has process.packages = ["core"]
        assert!(
            skills.contains(&"agent-management".to_string()),
            "core package should include agent-management"
        );
        assert!(
            skills.contains(&"owner-profile".to_string()),
            "core package should include owner-profile"
        );
    }

    #[test]
    fn toml_edit_add_skill_to_include() {
        let toml = r#"[aibox]
version = "0.9.0"

[container]
name = "test"
"#;
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();

        // Add [skills].include with a skill
        doc.insert("skills", toml_edit::Item::Table(toml_edit::Table::new()));
        doc["skills"]["include"] = toml_edit::value(toml_edit::Array::new());
        doc["skills"]["include"]
            .as_array_mut()
            .unwrap()
            .push("data-science");

        let result = doc.to_string();
        assert!(result.contains("[skills]"));
        assert!(result.contains("data-science"));
    }
}
