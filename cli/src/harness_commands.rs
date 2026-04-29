//! Harness slash-command registration — sync `commands/` adapter files from
//! installed processkit skills to per-harness target directories so each
//! harness can tab-complete them as slash commands.
//!
//! ## What this does
//!
//! processkit v0.7.0 introduced a `commands/` convention: skills that expose
//! user-invocable workflows ship thin adapter files at
//! `commands/<skill>-<workflow>.md` containing harness-agnostic frontmatter
//! and a one-line invocation body. Different harnesses discover these files
//! from different project-relative directories (and, for Gemini, in a
//! different file format).
//!
//! This module generalises the original `.claude/commands/` sync to a
//! per-harness "profile" — each enabled harness with a profile gets its own
//! install/cleanup pass. The Claude profile is always-on (matches the
//! historical behaviour); other harness profiles are gated on
//! `config.ai.harnesses.contains(...)`.
//!
//! ## Per-harness mapping
//!
//! | Harness  | Target layout                              | Format |
//! |----------|--------------------------------------------|--------|
//! | Claude   | `.claude/commands/<name>.md`               | md verbatim |
//! | Codex    | `.agents/skills/<name>/SKILL.md`           | Codex Skill |
//! | Cursor   | `.cursor/commands/<name>.md`               | md verbatim |
//! | Gemini   | `.gemini/commands/<name>.toml`             | TOML (converted) |
//! | OpenCode | `.opencode/commands/<name>.md`             | md verbatim |
//!
//! Codex CLI 0.125.0 does not surface arbitrary `~/.codex/prompts/<name>.md`
//! files as `/<name>` slash commands; the supported customization mechanism
//! is **Codex Skills** at `<workspace>/.agents/skills/<name>/SKILL.md`. See
//! DEC-20260426_1636-MightySky. Aibox v0.21.1 incorrectly wrote to the
//! legacy prompts dir; v0.21.2+ writes Codex Skills and cleans up the
//! orphaned prompt files at sync time.
//!
//! Universe / wanted / cleanup semantics carry over verbatim from the
//! original `claude_commands.rs` — see comments below.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::config::{AiHarness, AiboxConfig, PROCESSKIT_VERSION_UNSET};
use crate::output;
use crate::processkit_vocab::{mirror_skills_dir, parse_skill_frontmatter};

/// Output format for a harness profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandFormat {
    /// Copy the source markdown verbatim.
    MarkdownVerbatim,
    /// Convert the source markdown to a Gemini-style TOML wrapper.
    GeminiToml,
    /// Convert the source markdown to a Codex Skill (`SKILL.md` with
    /// `name` + `description` front-matter).
    CodexSkill,
}

/// Per-harness scaffolding profile.
struct HarnessCommandProfile {
    harness: AiHarness,
    /// Project-relative target directory for the deployed command files.
    target_dir: PathBuf,
    /// File extension for the deployed files (`md` or `toml`).
    file_extension: &'static str,
    /// How to convert a source markdown body into the deployed file content.
    format: CommandFormat,
    /// If true, each command is deployed at `target_dir/<stem>/SKILL.<ext>`
    /// (one subdirectory per command). Currently used only for Codex
    /// Skills. If false, flat layout `target_dir/<stem>.<ext>`.
    subdir_per_command: bool,
}

impl HarnessCommandProfile {
    /// Translate a source filename (e.g. `pk-resume.md`) into the deployed
    /// path **relative to `target_dir`** (e.g. `pk-resume.md`,
    /// `pk-resume.toml`, or `pk-resume/SKILL.md`).
    fn deployed_relpath(&self, source_md_filename: &str) -> Option<PathBuf> {
        let stem = source_md_filename.strip_suffix(".md")?;
        if self.subdir_per_command {
            Some(PathBuf::from(stem).join(format!("SKILL.{ext}", ext = self.file_extension)))
        } else {
            Some(PathBuf::from(format!(
                "{stem}.{ext}",
                ext = self.file_extension
            )))
        }
    }

    /// Render the deployed file content for this profile, given a source
    /// markdown filename (e.g. `pk-resume.md`) and its bytes.
    fn render(&self, source_md_filename: &str, source_bytes: &[u8]) -> Result<Vec<u8>> {
        match self.format {
            CommandFormat::MarkdownVerbatim => Ok(source_bytes.to_vec()),
            CommandFormat::GeminiToml => {
                Ok(render_gemini_toml(source_md_filename, source_bytes)?.into_bytes())
            }
            CommandFormat::CodexSkill => {
                Ok(render_codex_skill(source_md_filename, source_bytes)?.into_bytes())
            }
        }
    }
}

/// Look up the scaffolding profile for a given harness, if any. Returns
/// `None` for harnesses we explicitly do not scaffold (Aider, Continue,
/// Copilot, Hermes, Mistral).
fn profile_for(harness: AiHarness, project_root: &Path) -> Option<HarnessCommandProfile> {
    match harness {
        AiHarness::Claude => Some(HarnessCommandProfile {
            harness,
            target_dir: project_root.join(".claude").join("commands"),
            file_extension: "md",
            format: CommandFormat::MarkdownVerbatim,
            subdir_per_command: false,
        }),
        AiHarness::Codex => Some(HarnessCommandProfile {
            harness,
            // Codex Skills layout: <workspace>/.agents/skills/<name>/SKILL.md.
            // The legacy ~/.codex/prompts/<name>.md mechanism (used by
            // aibox v0.21.1) is not surfaced as slash commands by Codex
            // CLI 0.125.0; see DEC-20260426_1636-MightySky.
            target_dir: project_root.join(".agents").join("skills"),
            file_extension: "md",
            format: CommandFormat::CodexSkill,
            subdir_per_command: true,
        }),
        AiHarness::Cursor => Some(HarnessCommandProfile {
            harness,
            target_dir: project_root.join(".cursor").join("commands"),
            file_extension: "md",
            format: CommandFormat::MarkdownVerbatim,
            subdir_per_command: false,
        }),
        AiHarness::Gemini => Some(HarnessCommandProfile {
            harness,
            target_dir: project_root.join(".gemini").join("commands"),
            file_extension: "toml",
            format: CommandFormat::GeminiToml,
            subdir_per_command: false,
        }),
        AiHarness::OpenCode => Some(HarnessCommandProfile {
            harness,
            target_dir: project_root.join(".opencode").join("commands"),
            file_extension: "md",
            format: CommandFormat::MarkdownVerbatim,
            subdir_per_command: false,
        }),
        // No project-scoped slash-command surface (or pending upstream).
        AiHarness::Aider
        | AiHarness::Continue
        | AiHarness::Copilot
        | AiHarness::Hermes
        | AiHarness::Mistral => None,
    }
}

/// Returns `true` if the given harness profile should run for this config.
/// Claude is always-on (preserves pre-v0.20.x behaviour). All other
/// harnesses must be explicitly enabled via `[ai].harnesses`.
fn profile_enabled(profile: &HarnessCommandProfile, config: &AiboxConfig) -> bool {
    if profile.harness == AiHarness::Claude {
        return true;
    }
    config.ai.harnesses.contains(&profile.harness)
}

/// All harnesses we know how to scaffold. Claude leads the list so it
/// runs first when summarising counts; the rest are alphabetical.
const SCAFFOLDABLE_HARNESSES: &[AiHarness] = &[
    AiHarness::Claude,
    AiHarness::Codex,
    AiHarness::Cursor,
    AiHarness::Gemini,
    AiHarness::OpenCode,
];

/// Sync processkit command adapter files to every enabled harness target.
///
/// Idempotent — re-running on a stable (version, skills, harnesses) set
/// produces byte-identical output. Best-effort callers should warn-and-
/// continue on error rather than aborting the rest of sync.
pub fn sync_harness_commands(project_root: &Path, config: &AiboxConfig) -> Result<()> {
    let pk_version = &config.processkit.version;
    if pk_version == PROCESSKIT_VERSION_UNSET {
        return Ok(());
    }

    let mirror_skills_dir = mirror_skills_dir(project_root, pk_version);
    let live_skills_dir = project_root.join("context").join("skills");

    if mirror_skills_dir.is_none() && !live_skills_dir.is_dir() {
        return Ok(());
    }

    // Generate any missing command files from SKILL.md declarations before collecting.
    generate_missing_command_files(&live_skills_dir);

    // Universe of all known processkit command basenames (md filenames).
    let empty_dir = PathBuf::new();
    let mirror_dir_ref = mirror_skills_dir.as_deref().unwrap_or(&empty_dir);
    let universe = collect_command_filenames(mirror_dir_ref);

    // Wanted set: filename → source md path for currently-installed skills.
    let wanted = collect_live_commands(&live_skills_dir)?;

    if universe.is_empty() && wanted.is_empty() {
        return Ok(());
    }

    for harness in SCAFFOLDABLE_HARNESSES {
        let Some(profile) = profile_for(harness.clone(), project_root) else {
            continue;
        };
        if !profile_enabled(&profile, config) {
            continue;
        }
        sync_one_profile(&profile, &universe, &wanted)?;
    }

    // Sweep up legacy Codex prompt files left behind by aibox v0.21.1.
    // Runs unconditionally (independent of whether Codex is currently
    // enabled) because the legacy files were written in v0.21.1 even for
    // configs that have since dropped Codex from `[ai].harnesses`.
    cleanup_legacy_codex_prompts(project_root, &universe);

    Ok(())
}

/// Run install + cleanup for a single harness profile.
fn sync_one_profile(
    profile: &HarnessCommandProfile,
    universe: &HashSet<String>,
    wanted: &HashMap<String, PathBuf>,
) -> Result<()> {
    fs::create_dir_all(&profile.target_dir)
        .with_context(|| format!("failed to create {}", profile.target_dir.display()))?;

    let mut added = 0usize;
    let mut removed = 0usize;

    // Install the wanted set (skip if byte-identical already).
    for (md_filename, source_path) in wanted {
        let Some(relpath) = profile.deployed_relpath(md_filename) else {
            continue;
        };
        let dest = profile.target_dir.join(&relpath);
        let source_bytes = fs::read(source_path)
            .with_context(|| format!("failed to read {}", source_path.display()))?;
        let new_content = profile.render(md_filename, &source_bytes)?;
        if dest.exists() && fs::read(&dest).ok().as_deref() == Some(new_content.as_slice()) {
            continue; // already up-to-date
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&dest, &new_content)
            .with_context(|| format!("failed to write {}", dest.display()))?;
        added += 1;
    }

    // Cleanup: remove deployed files that are in the universe but not in
    // the wanted set. User-authored files (names not in the universe) are
    // never touched. For per-command-subdir layouts (Codex Skills), prune
    // an emptied per-command directory after the file removal.
    for source_md_name in universe {
        if wanted.contains_key(source_md_name) {
            continue;
        }
        let Some(relpath) = profile.deployed_relpath(source_md_name) else {
            continue;
        };
        let path = profile.target_dir.join(&relpath);
        if path.is_file() {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove stale command {}", path.display()))?;
            removed += 1;
            if profile.subdir_per_command
                && let Some(parent) = path.parent()
                && parent != profile.target_dir.as_path()
            {
                let parent_empty = fs::read_dir(parent)
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false);
                if parent_empty {
                    let _ = fs::remove_dir(parent);
                }
            }
        }
    }

    if added > 0 || removed > 0 {
        let rel_target = profile
            .target_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| profile.target_dir.display().to_string());
        output::ok(&format!(
            "{} commands: {} added/updated, {} removed → {}",
            profile.harness, added, removed, rel_target
        ));
    }

    Ok(())
}

/// Remove every aibox-managed command file from every scaffolded harness
/// target directory. User-authored files are preserved. Empty target
/// directories are removed afterwards.
pub fn remove_managed_commands_all(project_root: &Path, config: &AiboxConfig) -> Result<()> {
    let pk_version = &config.processkit.version;
    if pk_version == PROCESSKIT_VERSION_UNSET {
        return Ok(());
    }

    let mirror_dir = mirror_skills_dir(project_root, pk_version);
    let empty_dir = PathBuf::new();
    let mirror_dir_ref = mirror_dir.as_deref().unwrap_or(&empty_dir);
    let universe = collect_command_filenames(mirror_dir_ref);
    if universe.is_empty() {
        return Ok(());
    }

    for harness in SCAFFOLDABLE_HARNESSES {
        let Some(profile) = profile_for(harness.clone(), project_root) else {
            continue;
        };
        // Cleanup is gated on the same profile_enabled rule used during
        // sync — that way a user who never enabled (say) Cursor doesn't
        // suddenly get its commands directory deleted on `aibox reset`.
        if !profile_enabled(&profile, config) {
            continue;
        }
        remove_managed_for_profile(&profile, &universe)?;
    }
    Ok(())
}

fn remove_managed_for_profile(
    profile: &HarnessCommandProfile,
    universe: &HashSet<String>,
) -> Result<()> {
    if !profile.target_dir.is_dir() {
        return Ok(());
    }

    let mut removed = 0usize;
    for source_md_name in universe {
        let Some(relpath) = profile.deployed_relpath(source_md_name) else {
            continue;
        };
        let path = profile.target_dir.join(&relpath);
        if path.is_file() {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            removed += 1;
            if profile.subdir_per_command
                && let Some(parent) = path.parent()
                && parent != profile.target_dir.as_path()
            {
                let parent_empty = fs::read_dir(parent)
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false);
                if parent_empty {
                    let _ = fs::remove_dir(parent);
                }
            }
        }
    }

    let is_empty = fs::read_dir(&profile.target_dir)
        .map(|mut d| d.next().is_none())
        .unwrap_or(false);
    if is_empty {
        fs::remove_dir(&profile.target_dir)
            .with_context(|| format!("failed to remove {}", profile.target_dir.display()))?;
    }

    if removed > 0 {
        output::ok(&format!(
            "Removed {} managed command file(s) from {}",
            removed,
            profile.target_dir.display()
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Source scanning (universe + wanted)
// ---------------------------------------------------------------------------

/// Generate missing command adapter files from SKILL.md declarations.
///
/// When a skill declares commands in its SKILL.md `metadata.processkit.commands`
/// but the corresponding `commands/<name>.md` file is absent, this function
/// generates the file with the standard adapter template. Used to handle
/// incomplete skill distributions (e.g., processkit v0.19.1 pk-doctor).
///
/// Best-effort: logs warnings for individual failures but does not abort.
fn generate_missing_command_files(live_skills_dir: &Path) {
    if !live_skills_dir.is_dir() {
        return;
    }

    let Ok(categories) = fs::read_dir(live_skills_dir) else {
        return;
    };

    for category in categories.flatten() {
        if !category.path().is_dir() {
            continue;
        }
        let Ok(skills) = fs::read_dir(category.path()) else {
            continue;
        };

        for skill in skills.flatten() {
            let skill_path = skill.path();
            if !skill_path.is_dir() {
                continue;
            }

            let skill_md = skill_path.join("SKILL.md");
            let Ok(fm) = parse_skill_frontmatter(&skill_md) else {
                continue;
            };

            let Some(meta) = fm.processkit_meta() else {
                continue;
            };

            for cmd in &meta.commands {
                let cmd_filename = format!("{}.md", cmd.name);
                let cmd_file = skill_path.join("commands").join(&cmd_filename);

                if cmd_file.exists() {
                    continue;
                }

                let content = format!(
                    "---\nargument-hint: \"{}\"\nallowed-tools: []\n---\n\n{}\n",
                    cmd.args, cmd.description
                );

                if let Some(parent) = cmd_file.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                match fs::write(&cmd_file, content) {
                    Ok(_) => {
                        output::warn(&format!(
                            "Generated missing command file {} from SKILL.md declaration",
                            cmd_file.display()
                        ));
                    }
                    Err(e) => {
                        output::warn(&format!(
                            "Failed to generate command file {}: {}",
                            cmd_file.display(),
                            e
                        ));
                    }
                }
            }
        }
    }
}

/// Walk `skills_dir/<category>/<skill>/commands/*.md` and return a set of
/// all command filenames (basenames only). Used to build the universe from
/// the templates mirror.
fn collect_command_filenames(skills_dir: &Path) -> HashSet<String> {
    let mut set = HashSet::new();
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    let Ok(categories) = fs::read_dir(skills_dir) else {
        return set;
    };
    for category in categories.flatten() {
        if !category.path().is_dir() {
            continue;
        }
        let Ok(skills) = fs::read_dir(category.path()) else {
            continue;
        };
        for skill in skills.flatten() {
            let commands_dir = skill.path().join("commands");
            let Ok(cmd_entries) = fs::read_dir(&commands_dir) else {
                continue;
            };
            for cmd in cmd_entries.flatten() {
                let name = cmd.file_name();
                let Some(s) = name.to_str() else { continue };
                if s.ends_with(".md") {
                    if let Some(prev) = seen.get(s)
                        && prev != &skill.path()
                    {
                        crate::output::warn(&format!(
                            "duplicate command filename '{s}' found in \
                             '{prev}' and '{cur}' — last-wins; \
                             '{cur}' takes precedence. \
                             Disambiguate upstream to silence this warning.",
                            prev = prev.display(),
                            cur = skill.path().display(),
                        ));
                    }
                    seen.insert(s.to_string(), skill.path());
                    set.insert(s.to_string());
                }
            }
        }
    }
    set
}

/// Walk `skills_dir/<category>/<skill>/commands/*.md` and return a map of
/// filename → source path. Returns Err on slash-command name collision
/// between two skills. Iteration order is deterministic
/// (category, skill_name) lexicographic.
fn collect_live_commands(skills_dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut map: HashMap<String, PathBuf> = HashMap::new();
    let mut seen_skill: HashMap<String, PathBuf> = HashMap::new();

    let Ok(category_entries) = fs::read_dir(skills_dir) else {
        return Ok(map);
    };

    let mut categories: Vec<PathBuf> = category_entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    categories.sort();

    for category in categories {
        let Ok(skill_entries) = fs::read_dir(&category) else {
            continue;
        };

        let mut skills: Vec<PathBuf> = skill_entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        skills.sort();

        for skill_path in skills {
            let commands_dir = skill_path.join("commands");
            let Ok(cmd_entries) = fs::read_dir(&commands_dir) else {
                continue;
            };

            let mut cmds: Vec<PathBuf> = cmd_entries.flatten().map(|e| e.path()).collect();
            cmds.sort();

            for cmd_path in cmds {
                let Some(s) = cmd_path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !s.ends_with(".md") {
                    continue;
                }

                if let Some(prev) = seen_skill.get(s)
                    && prev != &skill_path
                {
                    let prev_cmd = prev.join("commands").join(s);
                    return Err(anyhow!(
                        "Slash command name collision: {name} is shipped by both\n  \
                         - {prev}\n  - {cur}\n\
                         This blocks .claude/commands/{name} deployment.\n\
                         Resolution: file an upstream issue with the offending skill, \
                         or set [skills].exclude in aibox.toml to drop one of the \
                         conflicting skills.",
                        name = s,
                        prev = prev_cmd.display(),
                        cur = cmd_path.display(),
                    ));
                }
                seen_skill.insert(s.to_string(), skill_path.clone());
                map.insert(s.to_string(), cmd_path);
            }
        }
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// Gemini TOML conversion
// ---------------------------------------------------------------------------

/// Convert a processkit command markdown source into a Gemini custom-command
/// TOML wrapper.
///
/// Schema (per https://geminicli.com/docs/cli/custom-commands/):
/// ```toml
/// description = "..."
/// prompt = """
/// <body>
/// """
/// ```
///
/// `description` is sourced from (in order of preference):
///   1. The frontmatter `description:` field, if present.
///   2. The first non-empty body line, with a leading `# ` stripped if present.
///   3. The empty string as a last resort.
///
/// `prompt` is the body of the source file (post-frontmatter) when
/// frontmatter is present, otherwise the entire source file verbatim.
fn render_gemini_toml(source_filename: &str, source_bytes: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(source_bytes)
        .with_context(|| format!("source file {source_filename} is not valid UTF-8"))?;

    let (frontmatter_yaml, body) = split_frontmatter(text);

    let frontmatter_description = frontmatter_yaml
        .as_ref()
        .and_then(|yaml| extract_yaml_description(yaml));

    let description = frontmatter_description.unwrap_or_else(|| {
        body.lines()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim_start_matches("# ").trim().to_string())
            .unwrap_or_default()
    });

    let prompt_body = body.trim_end_matches('\n');

    Ok(format!(
        "description = {desc}\nprompt = \"\"\"\n{body}\n\"\"\"\n",
        desc = toml_basic_string(&description),
        body = prompt_body,
    ))
}

/// Convert a processkit command markdown source into a Codex Skill
/// (`SKILL.md`) body. The result has YAML front-matter with `name` and
/// `description` only — Claude-Code conventions like `argument-hint` and
/// `allowed-tools` are dropped because Codex does not consume them.
///
/// `name` is the source filename's stem (e.g. `pk-resume.md` → `pk-resume`).
///
/// `description` is sourced from (in order of preference):
///   1. The frontmatter `description:` field, if present.
///   2. The first non-empty body line, with a leading `# ` stripped if present.
///   3. The empty string as a last resort.
///
/// Body is the source body (post-frontmatter) when frontmatter is present,
/// otherwise the source verbatim.
fn render_codex_skill(source_filename: &str, source_bytes: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(source_bytes)
        .with_context(|| format!("source file {source_filename} is not valid UTF-8"))?;

    let stem = source_filename
        .strip_suffix(".md")
        .unwrap_or(source_filename);

    let (frontmatter_yaml, body) = split_frontmatter(text);

    let frontmatter_description = frontmatter_yaml
        .as_ref()
        .and_then(|yaml| extract_yaml_description(yaml));

    let description = frontmatter_description.unwrap_or_else(|| {
        body.lines()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim_start_matches("# ").trim().to_string())
            .unwrap_or_default()
    });

    let body_trimmed = body.trim_start_matches('\n');

    Ok(format!(
        "---\nname: {name}\ndescription: {desc}\n---\n\n{body}",
        name = yaml_scalar(stem),
        desc = yaml_scalar(&description),
        body = body_trimmed,
    ))
}

/// Render a string as a YAML scalar suitable for inline front-matter use.
/// Quotes the value with double quotes when it contains characters that
/// would otherwise change YAML parsing (`:`, `#`, `'`, `"`, leading/trailing
/// whitespace, …); otherwise emits it bare.
fn yaml_scalar(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.chars()
            .any(|c| matches!(c, ':' | '#' | '"' | '\'' | '\n' | '\r' | '\t' | '\\'));
    if !needs_quote {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// One-shot migration: remove orphaned `pk-*` prompt files that aibox
/// v0.21.1 wrote to `.aibox-home/.codex/prompts/` before v0.21.2 moved
/// the Codex scaffold to `.agents/skills/`. Idempotent — a no-op when
/// the legacy directory is absent or contains no managed files.
///
/// Only files whose source-md-name appears in `universe` (the set of
/// known pk-* command names) are touched. User-authored files in the
/// legacy directory are preserved.
fn cleanup_legacy_codex_prompts(project_root: &Path, universe: &HashSet<String>) {
    let legacy_dir = project_root
        .join(".aibox-home")
        .join(".codex")
        .join("prompts");
    if !legacy_dir.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(&legacy_dir) else {
        return;
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        if !name_str.ends_with(".md") {
            continue;
        }
        if !universe.contains(name_str) {
            continue;
        }
        if fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    let dir_now_empty = fs::read_dir(&legacy_dir)
        .map(|mut d| d.next().is_none())
        .unwrap_or(false);
    if dir_now_empty {
        let _ = fs::remove_dir(&legacy_dir);
    }
    if removed > 0 {
        output::ok(&format!(
            "Removed {removed} orphaned legacy Codex prompt file(s) (pre-v0.21.2 layout) from .aibox-home/.codex/prompts/"
        ));
    }
}

/// Split `---\n...\n---\n` frontmatter off the front of a markdown
/// document. Returns `(Some(yaml_block), body)` if frontmatter is present,
/// otherwise `(None, full_text)`.
fn split_frontmatter(text: &str) -> (Option<String>, &str) {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (None, text);
    }
    let mut yaml = String::new();
    let mut body_start_byte = None;
    let mut consumed = "---\n".len();
    for line in text.lines().skip(1) {
        if line.trim() == "---" {
            // end of frontmatter — body starts after this line + its newline
            body_start_byte = Some(consumed + line.len() + 1);
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
        consumed += line.len() + 1;
    }
    match body_start_byte {
        Some(idx) if idx <= text.len() => (Some(yaml), &text[idx..]),
        _ => (None, text),
    }
}

/// Pull a `description:` field out of a YAML frontmatter block, in a
/// minimal hand-rolled way (we don't depend on serde_yaml here because the
/// frontmatter shape varies across processkit-generated and hand-authored
/// command files). Returns None when absent.
fn extract_yaml_description(yaml: &str) -> Option<String> {
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("description:") {
            let v = rest.trim();
            // strip surrounding quotes if any
            let v = v
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(v);
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Render a string as a TOML basic string literal (with surrounding
/// double quotes and minimal escaping).
fn toml_basic_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_skill_commands(
        skills_dir: &Path,
        category: &str,
        skill: &str,
        commands: &[&str],
        content: &str,
    ) {
        let cmd_dir = skills_dir.join(category).join(skill).join("commands");
        fs::create_dir_all(&cmd_dir).unwrap();
        for name in commands {
            fs::write(cmd_dir.join(name), content).unwrap();
        }
    }

    fn config_with(version: &str, harnesses: Vec<AiHarness>) -> AiboxConfig {
        use crate::config::{
            AddonsSection, AiSection, AiboxConfig, AiboxSection, AudioSection, ContainerSection,
            ContextSection, CustomizationSection, ProcessKitSection, SkillsSection,
        };
        AiboxConfig {
            aibox: AiboxSection {
                version: "0.20.0".to_string(),
                base: crate::config::BaseImage::Debian,
            },
            container: ContainerSection {
                name: "t".to_string(),
                hostname: "t".to_string(),
                user: "aibox".to_string(),
                post_create_command: None,
                keepalive: false,
                environment: std::collections::HashMap::new(),
                extra_volumes: vec![],
            },
            context: ContextSection::default(),
            ai: AiSection {
                harnesses,
                ..AiSection::default()
            },
            process: None,
            addons: AddonsSection::default(),
            skills: SkillsSection::default(),
            processkit: ProcessKitSection {
                version: version.to_string(),
                ..ProcessKitSection::default()
            },
            agents: crate::config::AgentsSection::default(),
            customization: CustomizationSection::default(),
            audio: AudioSection::default(),
            mcp: crate::config::McpSection::default(),
            local_env: std::collections::HashMap::new(),
            local_mcp_servers: vec![],
        }
    }

    /// Helper: set up a minimal mirror+live tree with a single `pk-resume.md`
    /// command that contains a leading "# Title" line.
    fn fixture_with_pk_resume(project: &Path) {
        let mirror = project.join("context/templates/processkit/v0.20.0/context/skills");
        make_skill_commands(
            &mirror,
            "processkit",
            "status-briefing",
            &["pk-resume.md"],
            "# Resume the session\n\nDo the thing.\n",
        );
        let live = project.join("context/skills");
        make_skill_commands(
            &live,
            "processkit",
            "status-briefing",
            &["pk-resume.md"],
            "# Resume the session\n\nDo the thing.\n",
        );
    }

    // ----- Claude profile (regression-test parity with prior module) -----

    #[test]
    fn claude_profile_writes_md_to_claude_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::Claude]);
        sync_harness_commands(project, &config).unwrap();

        let dest = project.join(".claude/commands/pk-resume.md");
        assert!(dest.exists(), "claude target should exist");
        let content = fs::read_to_string(&dest).unwrap();
        assert!(content.contains("Resume the session"));
    }

    #[test]
    fn claude_profile_runs_even_when_not_in_harnesses_list() {
        // Always-on semantic for Claude. (Pre-v0.20.0 behaviour.)
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        // Empty harnesses; Claude must still scaffold.
        let config = config_with("v0.20.0", vec![]);
        sync_harness_commands(project, &config).unwrap();

        assert!(project.join(".claude/commands/pk-resume.md").exists());
    }

    // ----- Codex profile (Codex Skills layout, v0.21.2+) -----

    #[test]
    fn codex_profile_writes_skill_to_agents_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::Codex]);
        sync_harness_commands(project, &config).unwrap();

        let dest = project.join(".agents/skills/pk-resume/SKILL.md");
        assert!(dest.exists(), "codex skill target should exist");

        let content = fs::read_to_string(&dest).unwrap();
        // Front-matter has only Codex-supported keys.
        assert!(content.starts_with("---\n"));
        assert!(content.contains("\nname: pk-resume\n"));
        assert!(
            content.contains("\ndescription:"),
            "expected description in front-matter; got:\n{content}"
        );
        assert!(
            !content.contains("argument-hint"),
            "Claude-only key argument-hint must be stripped; got:\n{content}"
        );
        assert!(
            !content.contains("allowed-tools"),
            "Claude-only key allowed-tools must be stripped; got:\n{content}"
        );
        // Body is preserved.
        assert!(content.contains("Do the thing."));
    }

    #[test]
    fn codex_profile_skipped_when_not_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::Claude]);
        sync_harness_commands(project, &config).unwrap();

        assert!(!project.join(".agents/skills").exists());
        assert!(!project.join(".aibox-home/.codex/prompts").exists());
    }

    #[test]
    fn codex_skill_pulls_description_from_frontmatter_first() {
        let src = "---\ndescription: \"Frontmatter wins\"\nargument-hint: \"[x]\"\n---\n\n# Body header should not be used\n\nBody.\n";
        let out = render_codex_skill("pk-thing.md", src.as_bytes()).unwrap();
        // We round-trip the description through `yaml_scalar`, which
        // emits unquoted scalars when safe; both forms are valid YAML.
        assert!(
            out.contains("description: Frontmatter wins\n")
                || out.contains("description: \"Frontmatter wins\"\n"),
            "expected frontmatter description; got:\n{out}"
        );
        assert!(
            !out.contains("argument-hint"),
            "frontmatter must be reduced to name+description; got:\n{out}"
        );
        assert!(out.contains("Body header should not be used"));
        assert!(out.contains("name: pk-thing\n"));
    }

    #[test]
    fn codex_skill_falls_back_to_first_body_line() {
        let src = "# Resume the session\n\nDo the thing.\n";
        let out = render_codex_skill("pk-resume.md", src.as_bytes()).unwrap();
        assert!(
            out.contains("description: Resume the session\n")
                || out.contains("description: \"Resume the session\"\n"),
            "expected description from leading '# ' line; got:\n{out}"
        );
        assert!(out.contains("name: pk-resume\n"));
        assert!(out.contains("Do the thing."));
    }

    #[test]
    fn codex_legacy_prompts_are_cleaned_up_on_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        // Simulate aibox v0.21.1 leftovers: a managed file in the legacy
        // location, plus a user-authored file that must be preserved.
        let legacy_dir = project.join(".aibox-home/.codex/prompts");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::write(legacy_dir.join("pk-resume.md"), "stale-v0.21.1").unwrap();
        fs::write(legacy_dir.join("user-thing.md"), "user wrote this").unwrap();

        let config = config_with("v0.20.0", vec![AiHarness::Codex]);
        sync_harness_commands(project, &config).unwrap();

        // New location populated.
        assert!(project.join(".agents/skills/pk-resume/SKILL.md").exists());
        // Managed legacy file removed.
        assert!(!legacy_dir.join("pk-resume.md").exists());
        // User file preserved (its name is not in the universe).
        assert_eq!(
            fs::read_to_string(legacy_dir.join("user-thing.md")).unwrap(),
            "user wrote this"
        );
    }

    #[test]
    fn codex_legacy_prompts_cleanup_runs_even_when_codex_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        // User upgraded from v0.21.1 to v0.21.2 and has since disabled
        // Codex. The orphan still must be removed so an aibox doctor
        // run isn't confused by a stale managed file.
        let legacy_dir = project.join(".aibox-home/.codex/prompts");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::write(legacy_dir.join("pk-resume.md"), "stale-v0.21.1").unwrap();

        let config = config_with("v0.20.0", vec![AiHarness::Claude]);
        sync_harness_commands(project, &config).unwrap();

        assert!(!legacy_dir.join("pk-resume.md").exists());
        // .agents/skills/ should not have been written (Codex disabled).
        assert!(!project.join(".agents/skills/pk-resume").exists());
    }

    // ----- Cursor profile -----

    #[test]
    fn cursor_profile_writes_md_to_cursor_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::Cursor]);
        sync_harness_commands(project, &config).unwrap();

        assert!(project.join(".cursor/commands/pk-resume.md").exists());
    }

    // ----- OpenCode profile -----

    #[test]
    fn opencode_profile_writes_md_to_opencode_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::OpenCode]);
        sync_harness_commands(project, &config).unwrap();

        assert!(project.join(".opencode/commands/pk-resume.md").exists());
    }

    // ----- Gemini profile -----

    #[test]
    fn gemini_profile_writes_toml_with_description_and_prompt() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::Gemini]);
        sync_harness_commands(project, &config).unwrap();

        let dest = project.join(".gemini/commands/pk-resume.toml");
        assert!(dest.exists(), "gemini toml target should exist");
        let content = fs::read_to_string(&dest).unwrap();

        assert!(
            content.contains("description = \"Resume the session\""),
            "expected description from leading '# ' line; got:\n{content}"
        );
        assert!(content.contains("prompt = \"\"\""));
        assert!(
            content.contains("Do the thing."),
            "expected body in prompt; got:\n{content}"
        );
        // Verify it parses as TOML.
        let parsed: toml::Value =
            toml::from_str(&content).expect("rendered output must parse as TOML");
        assert_eq!(
            parsed.get("description").and_then(|v| v.as_str()),
            Some("Resume the session")
        );
        assert!(parsed.get("prompt").and_then(|v| v.as_str()).is_some());
    }

    #[test]
    fn gemini_render_prefers_frontmatter_description() {
        let src = "---\ndescription: \"Frontmatter wins\"\nargument-hint: \"[x]\"\n---\n\n# Body header should not be used\n\nBody.\n";
        let toml_out = render_gemini_toml("foo.md", src.as_bytes()).unwrap();
        assert!(
            toml_out.contains("description = \"Frontmatter wins\""),
            "expected frontmatter description; got:\n{toml_out}"
        );
        // Body section should NOT include the YAML frontmatter.
        assert!(
            !toml_out.contains("argument-hint"),
            "frontmatter must be stripped from prompt; got:\n{toml_out}"
        );
    }

    // ----- Multi-harness fan-out -----

    #[test]
    fn multi_harness_fans_out_to_all_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with(
            "v0.20.0",
            vec![
                AiHarness::Claude,
                AiHarness::Codex,
                AiHarness::Cursor,
                AiHarness::Gemini,
                AiHarness::OpenCode,
            ],
        );
        sync_harness_commands(project, &config).unwrap();

        assert!(project.join(".claude/commands/pk-resume.md").exists());
        assert!(project.join(".agents/skills/pk-resume/SKILL.md").exists());
        assert!(project.join(".cursor/commands/pk-resume.md").exists());
        assert!(project.join(".gemini/commands/pk-resume.toml").exists());
        assert!(project.join(".opencode/commands/pk-resume.md").exists());
    }

    // ----- Idempotency: byte-compare, no mtime churn -----

    #[test]
    fn idempotent_resync_does_not_rewrite_identical_files() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with(
            "v0.20.0",
            vec![AiHarness::Claude, AiHarness::Gemini, AiHarness::Cursor],
        );

        sync_harness_commands(project, &config).unwrap();

        let claude_dest = project.join(".claude/commands/pk-resume.md");
        let gemini_dest = project.join(".gemini/commands/pk-resume.toml");
        let cursor_dest = project.join(".cursor/commands/pk-resume.md");

        let mt_claude = fs::metadata(&claude_dest).unwrap().modified().unwrap();
        let mt_gemini = fs::metadata(&gemini_dest).unwrap().modified().unwrap();
        let mt_cursor = fs::metadata(&cursor_dest).unwrap().modified().unwrap();

        // Capture bytes for byte-compare safety.
        let bytes_claude = fs::read(&claude_dest).unwrap();
        let bytes_gemini = fs::read(&gemini_dest).unwrap();
        let bytes_cursor = fs::read(&cursor_dest).unwrap();

        // Re-run; should be a no-op.
        sync_harness_commands(project, &config).unwrap();

        assert_eq!(
            mt_claude,
            fs::metadata(&claude_dest).unwrap().modified().unwrap()
        );
        assert_eq!(
            mt_gemini,
            fs::metadata(&gemini_dest).unwrap().modified().unwrap()
        );
        assert_eq!(
            mt_cursor,
            fs::metadata(&cursor_dest).unwrap().modified().unwrap()
        );

        assert_eq!(bytes_claude, fs::read(&claude_dest).unwrap());
        assert_eq!(bytes_gemini, fs::read(&gemini_dest).unwrap());
        assert_eq!(bytes_cursor, fs::read(&cursor_dest).unwrap());
    }

    // ----- User-authored files preserved (not in universe) -----

    #[test]
    fn user_authored_files_not_in_universe_are_preserved() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        // Pre-create a user file in each enabled harness target dir.
        fs::create_dir_all(project.join(".claude/commands")).unwrap();
        fs::write(project.join(".claude/commands/my-thing.md"), "user-claude").unwrap();
        fs::create_dir_all(project.join(".cursor/commands")).unwrap();
        fs::write(project.join(".cursor/commands/my-thing.md"), "user-cursor").unwrap();
        fs::create_dir_all(project.join(".gemini/commands")).unwrap();
        fs::write(
            project.join(".gemini/commands/my-thing.toml"),
            "description = \"u\"\nprompt = \"\"\"x\"\"\"\n",
        )
        .unwrap();

        let config = config_with(
            "v0.20.0",
            vec![AiHarness::Claude, AiHarness::Cursor, AiHarness::Gemini],
        );

        // Sync once (install), then sync again (cleanup pass).
        sync_harness_commands(project, &config).unwrap();
        sync_harness_commands(project, &config).unwrap();

        assert_eq!(
            fs::read_to_string(project.join(".claude/commands/my-thing.md")).unwrap(),
            "user-claude"
        );
        assert_eq!(
            fs::read_to_string(project.join(".cursor/commands/my-thing.md")).unwrap(),
            "user-cursor"
        );
        assert!(project.join(".gemini/commands/my-thing.toml").exists());
    }

    // ----- Stale managed commands removed when wanted set shrinks -----

    #[test]
    fn dropped_managed_commands_removed_on_resync() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        // Mirror knows about pk-foo AND pk-bar; live ships only pk-foo.
        let mirror = project.join("context/templates/processkit/v0.20.0/context/skills");
        make_skill_commands(
            &mirror,
            "processkit",
            "skill-a",
            &["pk-foo.md", "pk-bar.md"],
            "# pk-foo\n",
        );
        let live = project.join("context/skills");
        make_skill_commands(&live, "processkit", "skill-a", &["pk-foo.md"], "# pk-foo\n");

        // Pre-place stale pk-bar in each harness target.
        fs::create_dir_all(project.join(".claude/commands")).unwrap();
        fs::write(project.join(".claude/commands/pk-bar.md"), "stale").unwrap();
        fs::create_dir_all(project.join(".cursor/commands")).unwrap();
        fs::write(project.join(".cursor/commands/pk-bar.md"), "stale").unwrap();
        fs::create_dir_all(project.join(".gemini/commands")).unwrap();
        fs::write(
            project.join(".gemini/commands/pk-bar.toml"),
            "description = \"old\"\nprompt = \"\"\"x\"\"\"\n",
        )
        .unwrap();

        let config = config_with(
            "v0.20.0",
            vec![AiHarness::Claude, AiHarness::Cursor, AiHarness::Gemini],
        );
        sync_harness_commands(project, &config).unwrap();

        // pk-bar removed everywhere…
        assert!(!project.join(".claude/commands/pk-bar.md").exists());
        assert!(!project.join(".cursor/commands/pk-bar.md").exists());
        assert!(!project.join(".gemini/commands/pk-bar.toml").exists());

        // …but pk-foo installed everywhere.
        assert!(project.join(".claude/commands/pk-foo.md").exists());
        assert!(project.join(".cursor/commands/pk-foo.md").exists());
        assert!(project.join(".gemini/commands/pk-foo.toml").exists());
    }

    // ----- Cleanup (remove_managed_commands_all) -----

    #[test]
    fn cleanup_removes_managed_files_only_for_enabled_profiles() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        fixture_with_pk_resume(project);

        let config = config_with("v0.20.0", vec![AiHarness::Claude, AiHarness::Cursor]);
        sync_harness_commands(project, &config).unwrap();

        // Pre-place a user file in cursor dir to confirm it's preserved.
        fs::write(project.join(".cursor/commands/user.md"), "u").unwrap();

        remove_managed_commands_all(project, &config).unwrap();

        // Managed files gone.
        assert!(!project.join(".claude/commands/pk-resume.md").exists());
        assert!(!project.join(".cursor/commands/pk-resume.md").exists());

        // .claude/commands removed (was empty); .cursor/commands kept (user file).
        assert!(!project.join(".claude/commands").exists());
        assert!(project.join(".cursor/commands/user.md").exists());
    }

    // ----- Pure renderer / split tests -----

    #[test]
    fn split_frontmatter_handles_no_frontmatter() {
        let (yaml, body) = split_frontmatter("# Title\n\nbody.\n");
        assert!(yaml.is_none());
        assert_eq!(body, "# Title\n\nbody.\n");
    }

    #[test]
    fn split_frontmatter_extracts_yaml_and_body() {
        let src = "---\nfoo: bar\n---\n# Title\nbody\n";
        let (yaml, body) = split_frontmatter(src);
        assert_eq!(yaml.as_deref(), Some("foo: bar\n"));
        assert_eq!(body, "# Title\nbody\n");
    }

    #[test]
    fn extract_yaml_description_handles_quoted_and_unquoted() {
        assert_eq!(
            extract_yaml_description("description: hello\n").as_deref(),
            Some("hello")
        );
        assert_eq!(
            extract_yaml_description("description: \"quoted\"\n").as_deref(),
            Some("quoted")
        );
        assert_eq!(extract_yaml_description("other: x\n").as_deref(), None);
    }

    #[test]
    fn toml_basic_string_escapes_special_chars() {
        assert_eq!(toml_basic_string("a"), "\"a\"");
        assert_eq!(toml_basic_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(toml_basic_string("a\\b"), "\"a\\\\b\"");
        assert_eq!(toml_basic_string("a\nb"), "\"a\\nb\"");
    }

    // ----- collision detection (regression of existing claude_commands) -----

    #[test]
    fn collect_live_commands_hard_fails_on_collision() {
        let tmp = tempfile::tempdir().unwrap();
        let skills = tmp.path().join("skills");
        make_skill_commands(&skills, "engineering", "bar", &["pk-foo.md"], "# e");
        make_skill_commands(&skills, "devops", "baz", &["pk-foo.md"], "# d");

        let result = collect_live_commands(&skills);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("pk-foo.md"));
        assert!(err.contains("engineering") && err.contains("bar"));
        assert!(err.contains("devops") && err.contains("baz"));
    }

    #[test]
    fn sync_no_ops_when_version_unset() {
        let tmp = tempfile::tempdir().unwrap();
        let config = config_with(crate::config::PROCESSKIT_VERSION_UNSET, vec![]);
        sync_harness_commands(tmp.path(), &config).unwrap();
        assert!(!tmp.path().join(".claude/commands").exists());
    }

    #[test]
    fn generate_missing_command_files_from_skill_md_decl() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();

        let skill_dir = project.join("context/skills/processkit/test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            r#"---
name: test-skill
metadata:
  processkit:
    commands:
      - name: test-skill-run
        args: "[--verbose]"
        description: "Execute the test skill with optional verbosity"
---
# Test Skill
"#,
        )
        .unwrap();
        assert!(!skill_dir.join("commands").exists());

        generate_missing_command_files(&project.join("context/skills"));

        let cmd_file = skill_dir.join("commands/test-skill-run.md");
        assert!(cmd_file.exists());
        let content = fs::read_to_string(&cmd_file).unwrap();
        assert!(content.contains("argument-hint: \"[--verbose]\""));
        assert!(content.contains("Execute the test skill with optional verbosity"));
    }
}
