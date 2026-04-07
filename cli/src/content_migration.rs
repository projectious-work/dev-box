//! Lifecycle management for processkit Migration documents.
//!
//! Migration documents live under `context/migrations/{pending,in-progress,applied}/`.
//! The state subdirectory mirrors the entity's `spec.state`. State transitions
//! move files between those subdirectories. "Rejected" is a terminal state
//! whose files live in `applied/` (since it's their permanent home — see the
//! processkit migration-management skill SKILL.md).
//!
//! This module provides:
//! - [`MigrationState`] — the state enum
//! - [`MigrationDocument`] — parsed Migration document (YAML frontmatter + body)
//! - [`list_migrations`] / [`list_all_migrations`] / [`find_migration`]
//! - [`transition_migration`] — state transition (move file + rewrite frontmatter)
//! - [`update_index`] — regenerate `context/migrations/INDEX.md`
//! - `cmd_migrate_*` entry points wired up from `main.rs`
//!
//! ## YAML comment preservation
//!
//! Frontmatter is round-tripped through `serde_yaml::Value`, which does not
//! preserve comments. Body markdown (everything after the closing `---`)
//! is preserved byte-for-byte. Agents write these documents; comments inside
//! the frontmatter are rare in practice.

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::output;

// ---------------------------------------------------------------------------
// MigrationState
// ---------------------------------------------------------------------------

/// Migration lifecycle state. Mirrors the directory the file lives in
/// (with the caveat that `Rejected` files live under `applied/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationState {
    Pending,
    InProgress,
    Applied,
    Rejected,
}

impl MigrationState {
    /// The subdirectory under `context/migrations/` where files in this
    /// state are physically stored. `Rejected` shares `applied/` with
    /// `Applied` because both are terminal.
    pub fn dir_name(self) -> &'static str {
        match self {
            MigrationState::Pending => "pending",
            MigrationState::InProgress => "in-progress",
            MigrationState::Applied => "applied",
            MigrationState::Rejected => "applied",
        }
    }

    /// Parse a subdirectory name back into a state. Note this is lossy for
    /// `applied` (which maps to `Applied` — `Rejected` documents are
    /// distinguished by their frontmatter `state` field, not their path).
    #[allow(dead_code)]
    pub fn from_dir_name(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(MigrationState::Pending),
            "in-progress" => Some(MigrationState::InProgress),
            "applied" => Some(MigrationState::Applied),
            _ => None,
        }
    }

    /// The kebab-case string used in the YAML `spec.state` field.
    pub fn as_yaml_str(self) -> &'static str {
        match self {
            MigrationState::Pending => "pending",
            MigrationState::InProgress => "in-progress",
            MigrationState::Applied => "applied",
            MigrationState::Rejected => "rejected",
        }
    }

    pub fn from_yaml_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(MigrationState::Pending),
            "in-progress" => Some(MigrationState::InProgress),
            "applied" => Some(MigrationState::Applied),
            "rejected" => Some(MigrationState::Rejected),
            _ => None,
        }
    }

    /// States that this state is allowed to transition INTO.
    pub fn allowed_transitions(self) -> &'static [MigrationState] {
        match self {
            MigrationState::Pending => &[MigrationState::InProgress, MigrationState::Rejected],
            MigrationState::InProgress => &[
                MigrationState::Applied,
                MigrationState::Pending,
                MigrationState::Rejected,
            ],
            MigrationState::Applied => &[],
            MigrationState::Rejected => &[],
        }
    }
}

// ---------------------------------------------------------------------------
// MigrationDocument
// ---------------------------------------------------------------------------

/// A parsed Migration document. Frontmatter fields are extracted into named
/// fields; the full YAML is kept in `raw_yaml` for round-tripping unknown
/// fields. The markdown body is preserved verbatim in `body`.
#[derive(Debug, Clone)]
pub struct MigrationDocument {
    pub path: PathBuf,
    pub id: String,
    pub source: String,
    pub from_version: String,
    pub to_version: String,
    pub state: MigrationState,
    pub generated_at: Option<String>,
    pub summary: Option<String>,
    pub raw_yaml: serde_yaml::Value,
    pub body: String,
}

impl MigrationDocument {
    /// Parse a Migration document from a file on disk.
    pub fn parse_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read migration document {}", path.display()))?;
        Self::parse_from_str(path.to_path_buf(), &content)
    }

    /// Parse a Migration document from raw file contents.
    pub fn parse_from_str(path: PathBuf, content: &str) -> Result<Self> {
        let (frontmatter, body) = split_frontmatter(content).ok_or_else(|| {
            anyhow!(
                "Migration document {} is missing YAML frontmatter (must start with `---`)",
                path.display()
            )
        })?;

        let raw_yaml: serde_yaml::Value = serde_yaml::from_str(frontmatter)
            .with_context(|| format!("Failed to parse YAML frontmatter in {}", path.display()))?;

        let id = yaml_get_str(&raw_yaml, &["metadata", "id"])
            .ok_or_else(|| anyhow!("Migration document {} missing metadata.id", path.display()))?
            .to_string();

        let spec = raw_yaml
            .get("spec")
            .ok_or_else(|| anyhow!("Migration document {} missing spec block", path.display()))?;

        let state_str = spec
            .get("state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Migration document {} missing spec.state", path.display()))?;
        let state = MigrationState::from_yaml_str(state_str).ok_or_else(|| {
            anyhow!(
                "Migration document {} has unknown spec.state `{}`",
                path.display(),
                state_str
            )
        })?;

        let source = spec
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Migration document {} missing spec.source", path.display()))?
            .to_string();

        let from_version = spec
            .get("from_version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!(
                    "Migration document {} missing spec.from_version",
                    path.display()
                )
            })?
            .to_string();

        let to_version = spec
            .get("to_version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!(
                    "Migration document {} missing spec.to_version",
                    path.display()
                )
            })?
            .to_string();

        let generated_at = spec
            .get("generated_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let summary = spec
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(MigrationDocument {
            path,
            id,
            source,
            from_version,
            to_version,
            state,
            generated_at,
            summary,
            raw_yaml,
            body: body.to_string(),
        })
    }

    /// Rewrite the file at `self.path` using the current `raw_yaml` and
    /// `body`. The frontmatter is re-serialized from `raw_yaml`.
    pub fn write_to_disk(&self) -> Result<()> {
        let yaml = serde_yaml::to_string(&self.raw_yaml)
            .with_context(|| format!("Failed to serialize YAML for {}", self.path.display()))?;
        let mut out = String::with_capacity(yaml.len() + self.body.len() + 16);
        out.push_str("---\n");
        out.push_str(&yaml);
        if !yaml.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("---\n");
        out.push_str(&self.body);
        fs::write(&self.path, out).with_context(|| {
            format!("Failed to write migration document {}", self.path.display())
        })?;
        Ok(())
    }

    /// Update the `spec.state` field in `raw_yaml` to match `self.state`.
    fn sync_state_into_yaml(&mut self) {
        if let Some(spec) = self.raw_yaml.get_mut("spec")
            && let Some(map) = spec.as_mapping_mut()
        {
            map.insert(
                serde_yaml::Value::String("state".to_string()),
                serde_yaml::Value::String(self.state.as_yaml_str().to_string()),
            );
        }
    }
}

/// Split a markdown document into `(frontmatter_yaml, body)`. Returns `None`
/// if the document does not start with a `---` delimited frontmatter block.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    for (line_start, line) in line_offsets(rest) {
        if line.trim_end() == "---" {
            let frontmatter = &rest[..line_start];
            let after = line_start + line.len();
            let body_start = if rest[after..].starts_with('\n') {
                after + 1
            } else if rest[after..].starts_with("\r\n") {
                after + 2
            } else {
                after
            };
            return Some((frontmatter, &rest[body_start..]));
        }
    }
    None
}

/// Iterate over (offset, line_without_trailing_newline).
fn line_offsets(s: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut pos = 0;
    std::iter::from_fn(move || {
        if pos >= s.len() {
            return None;
        }
        let rest = &s[pos..];
        let newline = rest.find('\n');
        let len = match newline {
            Some(n) => n + 1,
            None => rest.len(),
        };
        let line_no_nl = match newline {
            Some(n) => &rest[..n],
            None => rest,
        };
        let start = pos;
        pos += len;
        Some((start, line_no_nl))
    })
}

fn yaml_get_str<'a>(v: &'a serde_yaml::Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str()
}

// ---------------------------------------------------------------------------
// Listing / finding
// ---------------------------------------------------------------------------

fn migrations_root(project_root: &Path) -> PathBuf {
    project_root.join("context").join("migrations")
}

/// List Migration documents whose *frontmatter state* matches `state`.
/// Files are looked up under the corresponding subdirectory (for `Rejected`,
/// this is `applied/`, and we filter by frontmatter state).
pub fn list_migrations(
    project_root: &Path,
    state: MigrationState,
) -> Result<Vec<MigrationDocument>> {
    let dir = migrations_root(project_root).join(state.dir_name());
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in
        fs::read_dir(&dir).with_context(|| format!("Failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if path.file_name().and_then(|s| s.to_str()) == Some("INDEX.md") {
            continue;
        }
        match MigrationDocument::parse_from_file(&path) {
            Ok(doc) => {
                if doc.state == state {
                    out.push(doc);
                }
            }
            Err(e) => {
                output::warn(&format!(
                    "Skipping unparseable migration {}: {:#}",
                    path.display(),
                    e
                ));
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// List every Migration document across all state subdirectories.
pub fn list_all_migrations(project_root: &Path) -> Result<Vec<MigrationDocument>> {
    let root = migrations_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for subdir in &["pending", "in-progress", "applied"] {
        let dir = root.join(subdir);
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file()
                || path.extension().and_then(|s| s.to_str()) != Some("md")
                || path.file_name().and_then(|s| s.to_str()) == Some("INDEX.md")
            {
                continue;
            }
            match MigrationDocument::parse_from_file(&path) {
                Ok(doc) => out.push(doc),
                Err(e) => output::warn(&format!(
                    "Skipping unparseable migration {}: {:#}",
                    path.display(),
                    e
                )),
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Find a Migration document by its `metadata.id` across all state
/// subdirectories. Returns `Ok(None)` if no match is found.
pub fn find_migration(project_root: &Path, id: &str) -> Result<Option<MigrationDocument>> {
    for doc in list_all_migrations(project_root)? {
        if doc.id == id {
            return Ok(Some(doc));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Transition
// ---------------------------------------------------------------------------

/// Transition a Migration document to a new state. Validates the transition,
/// rewrites the frontmatter state field, moves the file to the matching
/// subdirectory, and regenerates `INDEX.md`.
pub fn transition_migration(
    project_root: &Path,
    id: &str,
    to_state: MigrationState,
) -> Result<MigrationDocument> {
    let mut doc = find_migration(project_root, id)?
        .ok_or_else(|| anyhow!("No migration found with id `{}`", id))?;

    if doc.state == to_state {
        return Ok(doc);
    }

    if !doc.state.allowed_transitions().contains(&to_state) {
        bail!(
            "Cannot transition migration `{}` from `{}` to `{}`. Allowed: {:?}",
            id,
            doc.state.as_yaml_str(),
            to_state.as_yaml_str(),
            doc.state
                .allowed_transitions()
                .iter()
                .map(|s| s.as_yaml_str())
                .collect::<Vec<_>>()
        );
    }

    let old_path = doc.path.clone();
    let file_name = old_path
        .file_name()
        .ok_or_else(|| anyhow!("Migration path has no file name: {}", old_path.display()))?
        .to_os_string();

    let new_dir = migrations_root(project_root).join(to_state.dir_name());
    fs::create_dir_all(&new_dir)
        .with_context(|| format!("Failed to create {}", new_dir.display()))?;
    let new_path = new_dir.join(&file_name);

    doc.state = to_state;
    doc.path = new_path.clone();
    doc.sync_state_into_yaml();
    doc.write_to_disk()?;

    if old_path != new_path && old_path.exists() {
        fs::remove_file(&old_path)
            .with_context(|| format!("Failed to remove old file {}", old_path.display()))?;
    }

    update_index(project_root)?;
    Ok(doc)
}

// ---------------------------------------------------------------------------
// INDEX.md
// ---------------------------------------------------------------------------

/// Regenerate `context/migrations/INDEX.md` from the live state of the
/// `pending/`, `in-progress/`, and `applied/` directories. Idempotent:
/// writes the same content for the same inputs.
pub fn update_index(project_root: &Path) -> Result<()> {
    let root = migrations_root(project_root);
    fs::create_dir_all(&root).with_context(|| format!("Failed to create {}", root.display()))?;

    let pending = list_migrations(project_root, MigrationState::Pending)?;
    let in_progress = list_migrations(project_root, MigrationState::InProgress)?;
    // Applied subdirectory mingles Applied and Rejected — split by frontmatter state.
    let mut applied = Vec::new();
    let mut rejected = Vec::new();
    let applied_dir = root.join("applied");
    if applied_dir.exists() {
        for entry in fs::read_dir(&applied_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file()
                || path.extension().and_then(|s| s.to_str()) != Some("md")
                || path.file_name().and_then(|s| s.to_str()) == Some("INDEX.md")
            {
                continue;
            }
            if let Ok(doc) = MigrationDocument::parse_from_file(&path) {
                match doc.state {
                    MigrationState::Applied => applied.push(doc),
                    MigrationState::Rejected => rejected.push(doc),
                    _ => {}
                }
            }
        }
    }
    applied.sort_by(|a, b| a.id.cmp(&b.id));
    rejected.sort_by(|a, b| a.id.cmp(&b.id));

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let content = format_index(&now, &pending, &in_progress, &applied, &rejected);

    let index_path = root.join("INDEX.md");
    // Only write if content changed (idempotence + cleaner git diffs).
    let needs_write = match fs::read_to_string(&index_path) {
        Ok(existing) => strip_generated_line(&existing) != strip_generated_line(&content),
        Err(_) => true,
    };
    if needs_write {
        fs::write(&index_path, content)
            .with_context(|| format!("Failed to write {}", index_path.display()))?;
    }
    Ok(())
}

/// Drop the line containing "Generated by" so idempotence checks ignore the
/// timestamp that changes every call.
fn strip_generated_line(s: &str) -> String {
    s.lines()
        .filter(|l| !l.contains("Generated by `aibox` on"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_index(
    now_iso: &str,
    pending: &[MigrationDocument],
    in_progress: &[MigrationDocument],
    applied: &[MigrationDocument],
    rejected: &[MigrationDocument],
) -> String {
    let mut out = String::new();
    out.push_str("# Migrations Index\n\n");
    out.push_str(&format!(
        "_Generated by `aibox` on {}. Edit at your own risk — this file is regenerated by every migration state change._\n\n",
        now_iso
    ));

    out.push_str(&format!("## Pending ({})\n\n", pending.len()));
    if pending.is_empty() {
        out.push_str("(none)\n\n");
    } else {
        for doc in pending {
            out.push_str(&format_index_bullet(doc));
        }
        out.push('\n');
    }

    out.push_str(&format!("## In progress ({})\n\n", in_progress.len()));
    if in_progress.is_empty() {
        out.push_str("(none)\n\n");
    } else {
        for doc in in_progress {
            out.push_str(&format_index_bullet(doc));
        }
        out.push('\n');
    }

    out.push_str(&format!("## Applied ({})\n\n", applied.len()));
    if applied.is_empty() {
        out.push_str("(none)\n\n");
    } else {
        out.push_str("| Date       | Migration |\n");
        out.push_str("|------------|-----------|\n");
        for doc in applied {
            let date = doc.generated_at.as_deref().unwrap_or("unknown");
            let date = date.get(..10).unwrap_or(date);
            out.push_str(&format!(
                "| {:<10} | {} — {} v{} → v{} |\n",
                date, doc.id, doc.source, doc.from_version, doc.to_version
            ));
        }
        out.push('\n');
    }

    out.push_str(&format!("## Rejected ({})\n\n", rejected.len()));
    if rejected.is_empty() {
        out.push_str("(none)\n");
    } else {
        for doc in rejected {
            out.push_str(&format_index_bullet(doc));
        }
    }

    out
}

fn format_index_bullet(doc: &MigrationDocument) -> String {
    let gen_str = doc
        .generated_at
        .as_deref()
        .map(|s| format!(" _(generated {})_", s.get(..10).unwrap_or(s)))
        .unwrap_or_default();
    let mut s = format!(
        "- **{}** — {} v{} → v{}{}\n",
        doc.id, doc.source, doc.from_version, doc.to_version, gen_str
    );
    if let Some(summary) = &doc.summary
        && !summary.is_empty()
    {
        s.push_str(&format!("  {}\n", summary));
    }
    s
}

// ---------------------------------------------------------------------------
// Command entry points
// ---------------------------------------------------------------------------

/// `aibox migrate continue` — show pending and in-progress migrations and
/// suggest the next one to work on. Read-only with respect to migration
/// documents; regenerates INDEX.md as a side effect.
pub fn cmd_migrate_continue(project_root: &Path) -> Result<()> {
    if migrations_root(project_root).exists() {
        update_index(project_root)?;
    }

    let in_progress = list_migrations(project_root, MigrationState::InProgress)?;
    let pending = list_migrations(project_root, MigrationState::Pending)?;

    if in_progress.is_empty() && pending.is_empty() {
        output::ok("No pending or in-progress migrations. You're up to date.");
        return Ok(());
    }

    if !in_progress.is_empty() {
        output::info(&format!("In-progress migrations ({})", in_progress.len()));
        for doc in &in_progress {
            print_migration_brief(doc);
        }
    }

    if !pending.is_empty() {
        output::info(&format!("Pending migrations ({})", pending.len()));
        for doc in &pending {
            print_migration_brief(doc);
        }
    }

    // Suggest next: prefer first in-progress, else first pending.
    let next = in_progress.first().or_else(|| pending.first());
    if let Some(next) = next {
        output::info(&format!("Suggested next: {}", next.id));
        println!("\n--- {} ---", next.path.display());
        println!("{}", next.body);
    }

    Ok(())
}

/// `aibox migrate start <id>` — transition pending → in-progress.
pub fn cmd_migrate_start(project_root: &Path, id: &str) -> Result<()> {
    let doc = transition_migration(project_root, id, MigrationState::InProgress)?;
    output::ok(&format!(
        "Migration `{}` is now in-progress ({})",
        doc.id,
        doc.path.display()
    ));
    Ok(())
}

/// `aibox migrate apply <id>` — transition pending or in-progress → applied.
pub fn cmd_migrate_apply(project_root: &Path, id: &str) -> Result<()> {
    // Allow apply from either pending or in-progress: if pending, first
    // move it through in-progress to preserve the state machine semantics.
    let doc = find_migration(project_root, id)?
        .ok_or_else(|| anyhow!("No migration found with id `{}`", id))?;
    if doc.state == MigrationState::Pending {
        transition_migration(project_root, id, MigrationState::InProgress)?;
    }
    let doc = transition_migration(project_root, id, MigrationState::Applied)?;
    output::ok(&format!(
        "Migration `{}` marked as applied ({})",
        doc.id,
        doc.path.display()
    ));
    Ok(())
}

/// `aibox migrate reject <id> --reason ...` — transition to rejected.
pub fn cmd_migrate_reject(project_root: &Path, id: &str, reason: &str) -> Result<()> {
    // Attach the reason to the frontmatter before transitioning.
    let mut doc = find_migration(project_root, id)?
        .ok_or_else(|| anyhow!("No migration found with id `{}`", id))?;

    if let Some(spec) = doc.raw_yaml.get_mut("spec")
        && let Some(map) = spec.as_mapping_mut()
    {
        map.insert(
            serde_yaml::Value::String("rejection_reason".to_string()),
            serde_yaml::Value::String(reason.to_string()),
        );
    }
    doc.write_to_disk()?;

    let doc = transition_migration(project_root, id, MigrationState::Rejected)?;
    output::ok(&format!(
        "Migration `{}` rejected: {} ({})",
        doc.id,
        reason,
        doc.path.display()
    ));
    Ok(())
}

fn print_migration_brief(doc: &MigrationDocument) {
    let gen_str = doc
        .generated_at
        .as_deref()
        .map(|s| format!(" [{}]", s.get(..10).unwrap_or(s)))
        .unwrap_or_default();
    println!(
        "  - {} — {} v{} → v{}{}",
        doc.id, doc.source, doc.from_version, doc.to_version, gen_str
    );
    if let Some(summary) = &doc.summary
        && !summary.is_empty()
    {
        println!("    {}", summary);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SAMPLE_DOC: &str = "---
metadata:
  id: MIG-bright-owl
spec:
  source: processkit
  from_version: \"0.3.0\"
  to_version: \"0.4.0\"
  state: pending
  generated_at: \"2026-04-15T10:00:00Z\"
  summary: \"4 affected skills, 2 affected processes\"
---
# Migration body

Some markdown here.
";

    fn write_sample(project_root: &Path, subdir: &str, file_name: &str, content: &str) -> PathBuf {
        let dir = project_root.join("context/migrations").join(subdir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(file_name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn state_machine_allowed_transitions_table() {
        assert_eq!(
            MigrationState::Pending.allowed_transitions(),
            &[MigrationState::InProgress, MigrationState::Rejected]
        );
        assert_eq!(
            MigrationState::InProgress.allowed_transitions(),
            &[
                MigrationState::Applied,
                MigrationState::Pending,
                MigrationState::Rejected
            ]
        );
        assert!(MigrationState::Applied.allowed_transitions().is_empty());
        assert!(MigrationState::Rejected.allowed_transitions().is_empty());
    }

    #[test]
    fn state_dir_name_rules() {
        assert_eq!(MigrationState::Pending.dir_name(), "pending");
        assert_eq!(MigrationState::InProgress.dir_name(), "in-progress");
        assert_eq!(MigrationState::Applied.dir_name(), "applied");
        // Rejected shares "applied" as its terminal home.
        assert_eq!(MigrationState::Rejected.dir_name(), "applied");
        assert_eq!(
            MigrationState::from_dir_name("in-progress"),
            Some(MigrationState::InProgress)
        );
        assert_eq!(MigrationState::from_dir_name("nope"), None);
    }

    #[test]
    fn parse_migration_document_from_file() {
        let tmp = TempDir::new().unwrap();
        let path = write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);

        let doc = MigrationDocument::parse_from_file(&path).unwrap();
        assert_eq!(doc.id, "MIG-bright-owl");
        assert_eq!(doc.source, "processkit");
        assert_eq!(doc.from_version, "0.3.0");
        assert_eq!(doc.to_version, "0.4.0");
        assert_eq!(doc.state, MigrationState::Pending);
        assert_eq!(doc.generated_at.as_deref(), Some("2026-04-15T10:00:00Z"));
        assert_eq!(
            doc.summary.as_deref(),
            Some("4 affected skills, 2 affected processes")
        );
        assert!(doc.body.contains("# Migration body"));
    }

    #[test]
    fn parse_fails_without_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.md");
        fs::write(&path, "# No frontmatter here\n").unwrap();
        assert!(MigrationDocument::parse_from_file(&path).is_err());
    }

    #[test]
    fn list_migrations_in_pending_dir() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);

        let doc2 = SAMPLE_DOC.replace("MIG-bright-owl", "MIG-calm-river");
        write_sample(tmp.path(), "pending", "MIG-calm-river.md", &doc2);

        let docs = list_migrations(tmp.path(), MigrationState::Pending).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].id, "MIG-bright-owl");
        assert_eq!(docs[1].id, "MIG-calm-river");
    }

    #[test]
    fn list_migrations_returns_empty_for_nonexistent_dir() {
        let tmp = TempDir::new().unwrap();
        let docs = list_migrations(tmp.path(), MigrationState::Pending).unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn find_migration_searches_all_state_dirs() {
        let tmp = TempDir::new().unwrap();
        let content = SAMPLE_DOC.replace("state: pending", "state: in-progress");
        write_sample(tmp.path(), "in-progress", "MIG-bright-owl.md", &content);

        let found = find_migration(tmp.path(), "MIG-bright-owl").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().state, MigrationState::InProgress);

        let missing = find_migration(tmp.path(), "MIG-ghost").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn transition_migration_pending_to_in_progress() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);

        let doc = transition_migration(tmp.path(), "MIG-bright-owl", MigrationState::InProgress)
            .unwrap();
        assert_eq!(doc.state, MigrationState::InProgress);
        assert!(doc.path.to_string_lossy().contains("in-progress"));
        assert!(
            !tmp.path()
                .join("context/migrations/pending/MIG-bright-owl.md")
                .exists()
        );
        assert!(
            tmp.path()
                .join("context/migrations/in-progress/MIG-bright-owl.md")
                .exists()
        );

        let reread = MigrationDocument::parse_from_file(&doc.path).unwrap();
        assert_eq!(reread.state, MigrationState::InProgress);

        assert!(tmp.path().join("context/migrations/INDEX.md").exists());
    }

    #[test]
    fn transition_migration_in_progress_to_applied() {
        let tmp = TempDir::new().unwrap();
        let content = SAMPLE_DOC.replace("state: pending", "state: in-progress");
        write_sample(tmp.path(), "in-progress", "MIG-bright-owl.md", &content);

        let doc =
            transition_migration(tmp.path(), "MIG-bright-owl", MigrationState::Applied).unwrap();
        assert_eq!(doc.state, MigrationState::Applied);
        assert!(
            tmp.path()
                .join("context/migrations/applied/MIG-bright-owl.md")
                .exists()
        );
    }

    #[test]
    fn transition_migration_pending_to_rejected_lands_in_applied_dir() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);

        let doc =
            transition_migration(tmp.path(), "MIG-bright-owl", MigrationState::Rejected).unwrap();
        assert_eq!(doc.state, MigrationState::Rejected);
        // Rejected files live in applied/.
        assert!(doc.path.to_string_lossy().contains("applied"));
        assert!(
            tmp.path()
                .join("context/migrations/applied/MIG-bright-owl.md")
                .exists()
        );
        assert!(
            !tmp.path()
                .join("context/migrations/pending/MIG-bright-owl.md")
                .exists()
        );
    }

    #[test]
    fn transition_migration_invalid_transition_errors() {
        let tmp = TempDir::new().unwrap();
        let content = SAMPLE_DOC.replace("state: pending", "state: applied");
        write_sample(tmp.path(), "applied", "MIG-bright-owl.md", &content);

        let err = transition_migration(tmp.path(), "MIG-bright-owl", MigrationState::Pending)
            .unwrap_err();
        assert!(format!("{:#}", err).contains("Cannot transition"));
    }

    #[test]
    fn transition_migration_unknown_id_errors() {
        let tmp = TempDir::new().unwrap();
        let err = transition_migration(tmp.path(), "MIG-ghost", MigrationState::InProgress)
            .unwrap_err();
        assert!(format!("{:#}", err).contains("No migration found"));
    }

    #[test]
    fn update_index_writes_all_four_sections() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);
        let inp = SAMPLE_DOC
            .replace("MIG-bright-owl", "MIG-quick-hawk")
            .replace("state: pending", "state: in-progress");
        write_sample(tmp.path(), "in-progress", "MIG-quick-hawk.md", &inp);
        let app = SAMPLE_DOC
            .replace("MIG-bright-owl", "MIG-old-star")
            .replace("state: pending", "state: applied");
        write_sample(tmp.path(), "applied", "MIG-old-star.md", &app);
        let rej = SAMPLE_DOC
            .replace("MIG-bright-owl", "MIG-lost-fox")
            .replace("state: pending", "state: rejected");
        write_sample(tmp.path(), "applied", "MIG-lost-fox.md", &rej);

        update_index(tmp.path()).unwrap();
        let idx = fs::read_to_string(tmp.path().join("context/migrations/INDEX.md")).unwrap();
        assert!(idx.contains("# Migrations Index"));
        assert!(idx.contains("## Pending (1)"));
        assert!(idx.contains("MIG-bright-owl"));
        assert!(idx.contains("## In progress (1)"));
        assert!(idx.contains("MIG-quick-hawk"));
        assert!(idx.contains("## Applied (1)"));
        assert!(idx.contains("MIG-old-star"));
        assert!(idx.contains("## Rejected (1)"));
        assert!(idx.contains("MIG-lost-fox"));
    }

    #[test]
    fn update_index_handles_empty_state() {
        let tmp = TempDir::new().unwrap();
        update_index(tmp.path()).unwrap();
        let idx = fs::read_to_string(tmp.path().join("context/migrations/INDEX.md")).unwrap();
        assert!(idx.contains("## Pending (0)"));
        assert!(idx.contains("## In progress (0)"));
        assert!(idx.contains("## Applied (0)"));
        assert!(idx.contains("## Rejected (0)"));
        assert!(idx.matches("(none)").count() >= 4);
    }

    #[test]
    fn update_index_is_idempotent_across_calls() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);
        update_index(tmp.path()).unwrap();
        let first = fs::read_to_string(tmp.path().join("context/migrations/INDEX.md")).unwrap();
        update_index(tmp.path()).unwrap();
        let second = fs::read_to_string(tmp.path().join("context/migrations/INDEX.md")).unwrap();
        // Timestamp may differ on the `Generated by` line; strip it for comparison.
        assert_eq!(strip_generated_line(&first), strip_generated_line(&second));
    }

    #[test]
    fn cmd_migrate_continue_with_no_pending_or_in_progress_says_so() {
        let tmp = TempDir::new().unwrap();
        // No migrations dir at all — should succeed, not error.
        cmd_migrate_continue(tmp.path()).unwrap();
    }

    #[test]
    fn cmd_migrate_continue_with_pending_lists_them() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);
        cmd_migrate_continue(tmp.path()).unwrap();
        // Continue refreshes INDEX.md — verify that as a side-effect.
        assert!(tmp.path().join("context/migrations/INDEX.md").exists());
    }

    #[test]
    fn cmd_migrate_apply_from_pending_walks_through_in_progress() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);
        cmd_migrate_apply(tmp.path(), "MIG-bright-owl").unwrap();
        assert!(
            tmp.path()
                .join("context/migrations/applied/MIG-bright-owl.md")
                .exists()
        );
    }

    #[test]
    fn cmd_migrate_reject_records_reason() {
        let tmp = TempDir::new().unwrap();
        write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);
        cmd_migrate_reject(tmp.path(), "MIG-bright-owl", "not needed for this project").unwrap();

        let path = tmp
            .path()
            .join("context/migrations/applied/MIG-bright-owl.md");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("rejection_reason"));
        assert!(content.contains("not needed for this project"));
        assert!(content.contains("state: rejected"));
    }

    #[test]
    fn write_to_disk_preserves_body() {
        let tmp = TempDir::new().unwrap();
        let path = write_sample(tmp.path(), "pending", "MIG-bright-owl.md", SAMPLE_DOC);
        let mut doc = MigrationDocument::parse_from_file(&path).unwrap();
        doc.state = MigrationState::InProgress;
        doc.sync_state_into_yaml();
        doc.write_to_disk().unwrap();

        let reread = fs::read_to_string(&path).unwrap();
        assert!(reread.contains("# Migration body"));
        assert!(reread.contains("Some markdown here."));
        assert!(reread.contains("state: in-progress"));
    }
}
