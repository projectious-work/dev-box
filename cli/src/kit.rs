//! `aibox kit` — query processkit content installed in this project.
//!
//! Provides read-only inspection of skills, processes, and schemas installed
//! under `context/`. Skill install/uninstall commands modify `[skills].include`
//! / `[skills].exclude` in `aibox.toml` (same pattern as `aibox addon add/rm`).
//!
//! ## Paths
//!
//! | Content | Installed live path | Templates mirror |
//! |---------|--------------------|--------------------|
//! | Skills | `context/skills/<name>/SKILL.md` | `context/templates/processkit/<version>/skills/<name>/SKILL.md` |
//! | Processes | `context/processes/<name>.md` | `context/templates/processkit/<version>/processes/<name>.md` |
//! | Schemas | `context/schemas/<name>.yaml` | `context/templates/processkit/<version>/primitives/schemas/<name>.yaml` |
//! | State machines | `context/state-machines/<name>.yaml` | `context/templates/processkit/<version>/primitives/state-machines/<name>.yaml` |
//!
//! ## Category vocabulary (processkit v0.5+)
//!
//! Skills carry `metadata.processkit.category` in their YAML frontmatter.
//! 14 values: `process`, `meta`, `design`, `language`, `infrastructure`,
//! `data`, `architecture`, `ai`, `security`, `framework`, `observability`,
//! `database`, `api`, `performance`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::OutputFormat;
use crate::config::AiboxConfig;
use crate::lock;
use crate::output;

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn toml_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

fn project_root(config_path: &Option<String>) -> PathBuf {
    let p = toml_path(config_path);
    p.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Find the processkit templates mirror directory for the current lock version,
/// if it exists. Returns `None` when no lock or no templates mirror is present.
fn templates_skills_dir(root: &Path) -> Option<PathBuf> {
    let lock = lock::read_lock(root).ok()??;
    let pk = lock.processkit?;
    let dir = root
        .join("context/templates/processkit")
        .join(&pk.version)
        .join("skills");
    if dir.is_dir() { Some(dir) } else { None }
}

fn templates_processes_dir(root: &Path) -> Option<PathBuf> {
    let lock = lock::read_lock(root).ok()??;
    let pk = lock.processkit?;
    let dir = root
        .join("context/templates/processkit")
        .join(&pk.version)
        .join("processes");
    if dir.is_dir() { Some(dir) } else { None }
}

// ---------------------------------------------------------------------------
// Frontmatter parsing
// ---------------------------------------------------------------------------

/// Parsed YAML frontmatter from a processkit SKILL.md file.
#[derive(Debug, Clone, Deserialize, Default)]
struct SkillFrontmatter {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub metadata: Option<SkillMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
struct SkillMetadata {
    pub processkit: Option<SkillProcesskitMeta>,
}

#[derive(Debug, Clone, Deserialize)]
struct SkillProcesskitMeta {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub version: String,
}

/// Parse YAML frontmatter from a markdown file (between the first two `---` fences).
/// Returns an empty `SkillFrontmatter` if no frontmatter is found.
fn parse_frontmatter(path: &Path) -> Result<SkillFrontmatter> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    // Find `---` fences
    let mut lines = content.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Ok(SkillFrontmatter::default());
    }

    let yaml_block: String = lines
        .take_while(|l| l.trim() != "---")
        .collect::<Vec<_>>()
        .join("\n");

    let fm: SkillFrontmatter = serde_yaml::from_str(&yaml_block)
        .unwrap_or_default();

    Ok(fm)
}

// ---------------------------------------------------------------------------
// Skill discovery
// ---------------------------------------------------------------------------

/// A single skill entry (installed or available).
#[derive(Debug, Clone, Serialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub category: String,
    /// Whether this skill is currently installed in `context/skills/`.
    pub installed: bool,
}

/// Walk a skills directory (either live or templates mirror) and return skill entries.
/// `installed_names` is used to set the `installed` flag for each entry.
fn walk_skills_dir(dir: &Path, installed_names: &std::collections::HashSet<String>) -> Vec<SkillEntry> {
    let mut skills = Vec::new();

    let Ok(entries) = std::fs::read_dir(dir) else {
        return skills;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_file = path.join("SKILL.md");
        if !skill_file.exists() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip meta/internal directories
        if dir_name.starts_with('_') {
            continue;
        }

        let fm = parse_frontmatter(&skill_file).unwrap_or_default();

        // Use directory name as fallback for `name` when frontmatter is absent/empty.
        let name = if fm.name.is_empty() { dir_name.clone() } else { fm.name };
        let category = fm
            .metadata
            .as_ref()
            .and_then(|m| m.processkit.as_ref())
            .map(|p| p.category.clone())
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| "uncategorized".to_string());

        let installed = installed_names.contains(&dir_name);

        skills.push(SkillEntry {
            name,
            description: fm.description,
            category,
            installed,
        });
    }

    skills.sort_by(|a, b| {
        category_order(&a.category)
            .cmp(&category_order(&b.category))
            .then(a.name.cmp(&b.name))
    });

    skills
}

/// Collect the set of skill directory names present in `context/skills/`.
fn installed_skill_names(root: &Path) -> std::collections::HashSet<String> {
    let dir = root.join("context/skills");
    if !dir.is_dir() {
        return std::collections::HashSet::new();
    }
    std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.path().is_dir()
                && e.path().join("SKILL.md").exists()
                && !e
                    .file_name()
                    .to_str()
                    .map(|n| n.starts_with('_'))
                    .unwrap_or(false)
        })
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect()
}

/// Canonical sort order for the 14-value processkit category vocabulary.
fn category_order(cat: &str) -> usize {
    match cat {
        "process" => 0,
        "meta" => 1,
        "architecture" => 2,
        "language" => 3,
        "framework" => 4,
        "ai" => 5,
        "data" => 6,
        "infrastructure" => 7,
        "database" => 8,
        "api" => 9,
        "security" => 10,
        "observability" => 11,
        "design" => 12,
        "performance" => 13,
        _ => 99,
    }
}

// ---------------------------------------------------------------------------
// Process discovery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ProcessEntry {
    pub name: String,
    pub description: String,
    pub installed: bool,
}

fn walk_processes_dir(dir: &Path, installed_names: &std::collections::HashSet<String>) -> Vec<ProcessEntry> {
    let mut processes = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return processes };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if stem.is_empty() || stem == "INDEX" {
            continue;
        }

        // Extract first heading or first paragraph as description
        let description = extract_process_description(&path).unwrap_or_default();
        let installed = installed_names.contains(&stem);

        processes.push(ProcessEntry { name: stem, description, installed });
    }

    processes.sort_by(|a, b| a.name.cmp(&b.name));
    processes
}

fn installed_process_names(root: &Path) -> std::collections::HashSet<String> {
    let dir = root.join("context/processes");
    if !dir.is_dir() {
        return std::collections::HashSet::new();
    }
    std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("md")
                && e.file_name().to_str().map(|n| n != "INDEX.md").unwrap_or(false)
        })
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect()
}

/// Extract a short description from a process markdown file.
/// Returns the first non-heading paragraph (stripped) or empty string.
fn extract_process_description(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let mut in_frontmatter = false;
    let mut fm_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if !fm_seen && trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            if !in_frontmatter {
                fm_seen = true;
            }
            continue;
        }
        if in_frontmatter {
            continue;
        }
        // Skip blank lines and headings
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Return first content paragraph
        let desc: String = trimmed.chars().take(120).collect();
        return Ok(if desc.len() < trimmed.len() { format!("{}…", desc) } else { desc });
    }
    Ok(String::new())
}

// ---------------------------------------------------------------------------
// Schema / state-machine discovery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SchemaEntry {
    pub name: String,
    pub kind: String, // "schema" or "state-machine"
}

fn collect_schema_entries(root: &Path) -> Vec<SchemaEntry> {
    let mut entries = Vec::new();

    for (dir, kind) in [
        (root.join("context/schemas"), "schema"),
        (root.join("context/state-machines"), "state-machine"),
    ] {
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() || name == "INDEX" {
                continue;
            }
            entries.push(SchemaEntry { name, kind: kind.to_string() });
        }
    }

    entries.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
    entries
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// `aibox kit list` — summary of installed processkit content.
pub fn cmd_kit_list(config_path: &Option<String>, format: OutputFormat) -> Result<()> {
    let root = project_root(config_path);

    let installed_skills = installed_skill_names(&root).len();
    let installed_processes = installed_process_names(&root).len();
    let schemas = collect_schema_entries(&root);
    let schema_count = schemas.iter().filter(|s| s.kind == "schema").count();
    let state_machine_count = schemas.iter().filter(|s| s.kind == "state-machine").count();

    // Count all-available from templates mirror
    let available_skills = templates_skills_dir(&root).as_deref().and_then(|d| {
        std::fs::read_dir(d).ok().map(|entries| {
            entries.flatten()
                .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists()
                    && !e.file_name().to_str().map(|n| n.starts_with('_')).unwrap_or(false))
                .count()
        })
    });
    let available_processes = templates_processes_dir(&root).as_deref().and_then(|d| {
        std::fs::read_dir(d).ok().map(|entries| {
            entries.flatten()
                .filter(|e| {
                    e.path().extension().and_then(|x| x.to_str()) == Some("md")
                        && e.file_name().to_str().map(|n| n != "INDEX.md").unwrap_or(false)
                })
                .count()
        })
    });

    #[derive(Serialize)]
    struct Summary {
        skills_installed: usize,
        skills_available: Option<usize>,
        processes_installed: usize,
        processes_available: Option<usize>,
        schemas: usize,
        state_machines: usize,
    }

    let summary = Summary {
        skills_installed: installed_skills,
        skills_available: available_skills,
        processes_installed: installed_processes,
        processes_available: available_processes,
        schemas: schema_count,
        state_machines: state_machine_count,
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&summary)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&summary)?),
        OutputFormat::Table => {
            println!("  processkit content");
            println!();
            let skill_avail = available_skills
                .map(|n| format!("{} available", n))
                .unwrap_or_else(|| "templates not installed".to_string());
            let proc_avail = available_processes
                .map(|n| format!("{} available", n))
                .unwrap_or_else(|| "templates not installed".to_string());
            println!("  Skills       {:>4} installed  ({})", installed_skills, skill_avail);
            println!("  Processes    {:>4} installed  ({})", installed_processes, proc_avail);
            println!("  Schemas      {:>4}", schema_count);
            println!("  State machines {:>2}", state_machine_count);
            println!();
            println!("  Run 'aibox kit skill list' for skill details.");
        }
    }

    Ok(())
}

/// `aibox kit skill list [--category <cat>] [--all] [--format]`
pub fn cmd_kit_skill_list(
    config_path: &Option<String>,
    filter_category: Option<&str>,
    all: bool,
    format: OutputFormat,
) -> Result<()> {
    let root = project_root(config_path);
    let installed_names = installed_skill_names(&root);

    let skills: Vec<SkillEntry> = if all {
        // Walk templates mirror; mark installed status
        match templates_skills_dir(&root) {
            Some(ref tmpl_dir) => walk_skills_dir(tmpl_dir, &installed_names),
            None => {
                output::warn(
                    "No templates mirror found. Run 'aibox init' or 'aibox sync' with a \
                     pinned processkit version first.",
                );
                // Fall back to installed only
                walk_skills_dir(&root.join("context/skills"), &installed_names)
            }
        }
    } else {
        walk_skills_dir(&root.join("context/skills"), &installed_names)
    };

    let skills: Vec<&SkillEntry> = if let Some(cat) = filter_category {
        skills.iter().filter(|s| s.category == cat).collect()
    } else {
        skills.iter().collect()
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&skills)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&skills)?),
        OutputFormat::Table => {
            if skills.is_empty() {
                if let Some(cat) = filter_category {
                    output::warn(&format!(
                        "No skills found in category '{}'.",
                        cat
                    ));
                } else {
                    output::warn(
                        "No skills installed. Run 'aibox init' or 'aibox sync' to install processkit content.",
                    );
                }
                return Ok(());
            }

            let name_w = skills.iter().map(|s| s.name.len()).max().unwrap_or(5).max(5);
            let desc_w = skills.iter().map(|s| s.description.len().min(60)).max().unwrap_or(11).max(11);

            let mut cur_cat = "";
            for s in &skills {
                if s.category != cur_cat {
                    if !cur_cat.is_empty() { println!(); }
                    println!("  \x1b[1m{}\x1b[0m", s.category);
                    if all {
                        println!("  {:<nw$}  {:<dw$}  STATUS", "SKILL", "DESCRIPTION", nw = name_w, dw = desc_w);
                    } else {
                        println!("  {:<nw$}  {:<dw$}", "SKILL", "DESCRIPTION", nw = name_w, dw = desc_w);
                    }
                    cur_cat = &s.category;
                }
                let desc_trunc: String = s.description.chars().take(60).collect();
                let desc_display = if s.description.len() > 60 {
                    format!("{}…", desc_trunc)
                } else {
                    s.description.clone()
                };
                if all {
                    let status = if s.installed { "installed" } else { "available" };
                    println!("  {:<nw$}  {:<dw$}  {}", s.name, desc_display, status, nw = name_w, dw = desc_w);
                } else {
                    println!("  {:<nw$}  {:<dw$}", s.name, desc_display, nw = name_w, dw = desc_w);
                }
            }
        }
    }

    Ok(())
}

/// `aibox kit skill categories [--format]`
pub fn cmd_kit_skill_categories(config_path: &Option<String>, format: OutputFormat) -> Result<()> {
    let root = project_root(config_path);
    let installed_names = installed_skill_names(&root);

    // Use templates mirror if available for full picture; fall back to installed
    let skills = match templates_skills_dir(&root) {
        Some(ref tmpl_dir) => walk_skills_dir(tmpl_dir, &installed_names),
        None => walk_skills_dir(&root.join("context/skills"), &installed_names),
    };

    // Aggregate: category → (total, installed)
    let mut counts: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for s in &skills {
        let e = counts.entry(s.category.clone()).or_insert((0, 0));
        e.0 += 1;
        if s.installed { e.1 += 1; }
    }

    // Sort by category order
    let mut rows: Vec<(String, usize, usize)> = counts
        .into_iter()
        .map(|(cat, (total, inst))| (cat, total, inst))
        .collect();
    rows.sort_by(|a, b| category_order(&a.0).cmp(&category_order(&b.0)));

    #[derive(Serialize)]
    struct CatRow {
        category: String,
        total: usize,
        installed: usize,
    }

    let serializable: Vec<CatRow> = rows.iter().map(|(c, t, i)| CatRow {
        category: c.clone(),
        total: *t,
        installed: *i,
    }).collect();

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&serializable)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&serializable)?),
        OutputFormat::Table => {
            println!("  {:<20}  {:>5}  {:>9}", "CATEGORY", "TOTAL", "INSTALLED");
            for (cat, total, inst) in &rows {
                println!("  {:<20}  {:>5}  {:>9}", cat, total, inst);
            }
        }
    }

    Ok(())
}

/// `aibox kit skill info <name> [--format]`
pub fn cmd_kit_skill_info(config_path: &Option<String>, name: &str, format: OutputFormat) -> Result<()> {
    let root = project_root(config_path);
    let installed_names = installed_skill_names(&root);

    // Try live install first, then templates mirror
    let skill_file = root.join("context/skills").join(name).join("SKILL.md");
    let tmpl_file = templates_skills_dir(&root).map(|d| d.join(name).join("SKILL.md"));

    let (path, installed) = if skill_file.exists() {
        (skill_file, true)
    } else if let Some(ref t) = tmpl_file {
        if t.exists() {
            (t.clone(), false)
        } else {
            bail!(
                "Skill '{}' not found. Run 'aibox kit skill list' to see available skills.",
                name
            );
        }
    } else {
        bail!(
            "Skill '{}' not found. Run 'aibox kit skill list' to see available skills.",
            name
        );
    };

    let fm = parse_frontmatter(&path)?;
    let category = fm
        .metadata
        .as_ref()
        .and_then(|m| m.processkit.as_ref())
        .map(|p| p.category.clone())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "uncategorized".to_string());
    let version = fm
        .metadata
        .as_ref()
        .and_then(|m| m.processkit.as_ref())
        .map(|p| p.version.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_default();

    #[derive(Serialize)]
    struct SkillDetail {
        name: String,
        description: String,
        category: String,
        version: String,
        installed: bool,
    }

    let detail = SkillDetail {
        name: if fm.name.is_empty() { name.to_string() } else { fm.name },
        description: fm.description,
        category,
        version,
        installed,
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&detail)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&detail)?),
        OutputFormat::Table => {
            println!("  Skill:       {}", detail.name);
            println!("  Category:    {}", detail.category);
            if !detail.version.is_empty() {
                println!("  Version:     {}", detail.version);
            }
            println!("  Installed:   {}", if detail.installed { "yes" } else { "no" });
            println!();
            if !detail.description.is_empty() {
                println!("  {}", detail.description);
            }
        }
    }

    let _ = installed_names; // used by walk functions; suppress warning here
    Ok(())
}

/// `aibox kit skill install <name>`
///
/// Modifies `[skills].include` / `[skills].exclude` in `aibox.toml` so the
/// skill will be present on the next `aibox sync`. Mirrors the logic of
/// `aibox addon add`.
pub fn cmd_kit_skill_install(config_path: &Option<String>, name: &str) -> Result<()> {
    let path = toml_path(config_path);
    if !path.exists() {
        bail!("No aibox.toml found. Run 'aibox init' first.");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", path.display()))?;

    // Read current include/exclude via config
    let config = AiboxConfig::from_cli_option(config_path)?;

    if !config.skills.exclude.is_empty() {
        // We're in exclude-mode: remove name from exclude list
        if !config.skills.exclude.contains(&name.to_string()) {
            output::warn(&format!("Skill '{}' is already in the active set (not in exclude list).", name));
            return Ok(());
        }
        let new_exclude: Vec<&str> = config.skills.exclude.iter()
            .filter(|s| s.as_str() != name)
            .map(|s| s.as_str())
            .collect();
        set_skills_array(&mut doc, "exclude", &new_exclude);
    } else if !config.skills.include.is_empty() {
        // We're in include-mode: add name to include list
        if config.skills.include.contains(&name.to_string()) {
            output::warn(&format!("Skill '{}' is already in [skills].include.", name));
            return Ok(());
        }
        let mut new_include: Vec<&str> = config.skills.include.iter().map(|s| s.as_str()).collect();
        new_include.push(name);
        new_include.sort();
        set_skills_array(&mut doc, "include", &new_include);
    } else {
        // No filter mode: all skills are installed. Nothing to do.
        output::warn(
            "All skills are already installed (no [skills] filter active). \
             Use 'aibox kit skill uninstall' to exclude a skill, then \
             install to re-include it."
        );
        return Ok(());
    }

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("failed to write {}", path.display()))?;

    output::ok(&format!(
        "Skill '{}' added to active set. Run 'aibox sync --no-build' to apply.",
        name
    ));
    Ok(())
}

/// `aibox kit skill uninstall <name>`
///
/// Excludes a skill from the active set by modifying `[skills].include` /
/// `[skills].exclude` in `aibox.toml`.
pub fn cmd_kit_skill_uninstall(config_path: &Option<String>, name: &str) -> Result<()> {
    let path = toml_path(config_path);
    if !path.exists() {
        bail!("No aibox.toml found. Run 'aibox init' first.");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", path.display()))?;

    let config = AiboxConfig::from_cli_option(config_path)?;

    if !config.skills.include.is_empty() {
        // In include-mode: remove name from include list
        if !config.skills.include.contains(&name.to_string()) {
            output::warn(&format!("Skill '{}' is not in [skills].include — already excluded.", name));
            return Ok(());
        }
        let new_include: Vec<&str> = config.skills.include.iter()
            .filter(|s| s.as_str() != name)
            .map(|s| s.as_str())
            .collect();
        set_skills_array(&mut doc, "include", &new_include);
    } else {
        // In exclude-mode or no-filter mode: add name to exclude list
        if config.skills.exclude.contains(&name.to_string()) {
            output::warn(&format!("Skill '{}' is already in [skills].exclude.", name));
            return Ok(());
        }
        let mut new_exclude: Vec<&str> = config.skills.exclude.iter().map(|s| s.as_str()).collect();
        new_exclude.push(name);
        new_exclude.sort();
        set_skills_array(&mut doc, "exclude", &new_exclude);
    }

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("failed to write {}", path.display()))?;

    output::ok(&format!(
        "Skill '{}' removed from active set. Run 'aibox sync --no-build' to apply.",
        name
    ));
    Ok(())
}

/// Write a `[skills].<key>` TOML array, creating `[skills]` if needed.
fn set_skills_array(doc: &mut toml_edit::DocumentMut, key: &str, values: &[&str]) {
    if doc.get("skills").is_none() {
        doc.insert("skills", toml_edit::Item::Table(toml_edit::Table::new()));
    }
    let mut arr = toml_edit::Array::new();
    for v in values {
        arr.push(*v);
    }
    doc["skills"][key] = toml_edit::value(arr);
}

/// `aibox kit process list [--all] [--format]`
pub fn cmd_kit_process_list(
    config_path: &Option<String>,
    all: bool,
    format: OutputFormat,
) -> Result<()> {
    let root = project_root(config_path);
    let installed_names = installed_process_names(&root);

    let processes: Vec<ProcessEntry> = if all {
        match templates_processes_dir(&root) {
            Some(ref tmpl_dir) => walk_processes_dir(tmpl_dir, &installed_names),
            None => {
                output::warn(
                    "No templates mirror found. Run 'aibox init' or 'aibox sync' with a \
                     pinned processkit version first.",
                );
                walk_processes_dir(&root.join("context/processes"), &installed_names)
            }
        }
    } else {
        walk_processes_dir(&root.join("context/processes"), &installed_names)
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&processes)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&processes)?),
        OutputFormat::Table => {
            if processes.is_empty() {
                output::warn("No processes installed. Run 'aibox init' or 'aibox sync' to install processkit content.");
                return Ok(());
            }
            let name_w = processes.iter().map(|p| p.name.len()).max().unwrap_or(5).max(5);
            let desc_w = processes.iter().map(|p| p.description.len().min(60)).max().unwrap_or(11).max(11);
            if all {
                println!("  {:<nw$}  {:<dw$}  STATUS", "PROCESS", "DESCRIPTION", nw = name_w, dw = desc_w);
            } else {
                println!("  {:<nw$}  {:<dw$}", "PROCESS", "DESCRIPTION", nw = name_w, dw = desc_w);
            }
            for p in &processes {
                let desc_trunc: String = p.description.chars().take(60).collect();
                let desc_display = if p.description.len() > 60 { format!("{}…", desc_trunc) } else { p.description.clone() };
                if all {
                    let status = if p.installed { "installed" } else { "available" };
                    println!("  {:<nw$}  {:<dw$}  {}", p.name, desc_display, status, nw = name_w, dw = desc_w);
                } else {
                    println!("  {:<nw$}  {:<dw$}", p.name, desc_display, nw = name_w, dw = desc_w);
                }
            }
        }
    }

    Ok(())
}

/// `aibox kit process info <name> [--format]`
pub fn cmd_kit_process_info(config_path: &Option<String>, name: &str, format: OutputFormat) -> Result<()> {
    let root = project_root(config_path);

    let live_file = root.join("context/processes").join(format!("{}.md", name));
    let tmpl_file = templates_processes_dir(&root).map(|d| d.join(format!("{}.md", name)));

    let (path, installed) = if live_file.exists() {
        (live_file, true)
    } else if let Some(ref t) = tmpl_file {
        if t.exists() {
            (t.clone(), false)
        } else {
            bail!("Process '{}' not found. Run 'aibox kit process list' to see available processes.", name);
        }
    } else {
        bail!("Process '{}' not found. Run 'aibox kit process list' to see available processes.", name);
    };

    let description = extract_process_description(&path)?;

    #[derive(Serialize)]
    struct ProcessDetail {
        name: String,
        description: String,
        installed: bool,
    }

    let detail = ProcessDetail {
        name: name.to_string(),
        description,
        installed,
    };

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&detail)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&detail)?),
        OutputFormat::Table => {
            println!("  Process:    {}", detail.name);
            println!("  Installed:  {}", if detail.installed { "yes" } else { "no" });
            if !detail.description.is_empty() {
                println!();
                println!("  {}", detail.description);
            }
        }
    }

    Ok(())
}
