//! Three-way comparison between the project's installed content-source
//! payload (live working tree), a freshly-fetched cache, and the
//! immutable reference templates dir written by [`crate::content_init`].
//!
//! Used by `aibox sync` to detect what changed upstream and what changed
//! locally, and to write Migration documents for the user to review.
//! Never overwrites anything — always proposes.
//!
//! ## Three-way truth table
//!
//! For each file we compute up to three SHAs: the **reference SHA** (what
//! was installed last time, read from
//! `context/templates/processkit/<lock.version>/<rel_path>`), the **cache
//! SHA** (what upstream ships now), and the **live SHA** (what the project
//! has on disk right now). The classification follows:
//!
//! | reference vs cache | reference vs live    | classification         |
//! |--------------------|----------------------|------------------------|
//! | equal              | equal                | Unchanged              |
//! | equal              | different (or missing)| ChangedLocallyOnly    |
//! | different          | equal                | ChangedUpstreamOnly    |
//! | different          | different            | Conflict               |
//! | (in cache, not in reference)  | n/a       | NewUpstream            |
//! | (in reference, not in cache)  | n/a       | RemovedUpstream        |
//!
//! Files whose install-action is `Skip` (processkit-internal, not
//! user-facing) are excluded from the diff entirely — they live in the
//! templates dir as part of the full upstream snapshot but are never
//! reported in the diff because they have no live counterpart.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lock::{AiboxLock, group_for_path, sha256_of_file, should_skip_entry};
use crate::content_init::templates_dir_for_version;
use crate::content_install::{InstallAction, install_action_for};

// ---------------------------------------------------------------------------
// Per-file classification
// ---------------------------------------------------------------------------

/// Per-file classification from the three-way comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileClassification {
    /// Reference, cache, and live all match — nothing to do.
    Unchanged,
    /// Reference matches cache but not live — user has edited it locally;
    /// upstream has not changed. No-op for this migration but worth noting.
    ChangedLocallyOnly,
    /// Reference matches live but not cache — upstream has changed; user
    /// has not touched it. Safe to take with one approval.
    ChangedUpstreamOnly,
    /// Reference matches neither — both sides changed. Conflict, must be
    /// resolved by hand.
    Conflict,
    /// File exists in cache but not in reference (i.e. wasn't in the
    /// previous version of upstream). New addition.
    NewUpstream,
    /// File exists in reference but not in cache (i.e. removed from
    /// upstream). Decide whether to drop locally or keep as a project fork.
    RemovedUpstream,
}

impl FileClassification {
    /// Short human-readable label used in migration documents.
    pub fn label(self) -> &'static str {
        match self {
            FileClassification::Unchanged => "unchanged",
            FileClassification::ChangedLocallyOnly => "changed-locally-only",
            FileClassification::ChangedUpstreamOnly => "changed-upstream-only",
            FileClassification::Conflict => "conflict",
            FileClassification::NewUpstream => "new-upstream",
            FileClassification::RemovedUpstream => "removed-upstream",
        }
    }
}

/// One file's worth of comparison data.
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Path relative to the cache `<src_path>/`. The same key used to
    /// look up the file in the templates dir.
    pub cache_rel_path: String,
    /// Where the file would be installed in the project (from `content_install`).
    pub project_path: Option<PathBuf>,
    /// Logical group from `lock::group_for_path`.
    pub group: Option<String>,
    /// Classification.
    pub classification: FileClassification,
}

/// Groups of file diffs, keyed by group name. Files with no group are
/// collected under the empty-string key.
pub type GroupedDiff = BTreeMap<String, Vec<FileDiff>>;

// ---------------------------------------------------------------------------
// Classification helper
// ---------------------------------------------------------------------------

/// Classify a single file given the three SHAs (reference / cache / live).
///
/// - If `reference_sha` is `None` and `cache_sha` is `Some` → `NewUpstream`.
/// - If `reference_sha` is `Some` and `cache_sha` is `None` → `RemovedUpstream`.
/// - Otherwise consult the three-way truth table using `live_sha`.
pub fn classify(
    reference_sha: Option<&str>,
    cache_sha: Option<&str>,
    live_sha: Option<&str>,
) -> FileClassification {
    match (reference_sha, cache_sha) {
        (None, Some(_)) => FileClassification::NewUpstream,
        (Some(_), None) => FileClassification::RemovedUpstream,
        (None, None) => FileClassification::Unchanged, // should not happen in practice
        (Some(r), Some(c)) => {
            let cache_eq = r == c;
            let live_eq = live_sha.map(|l| l == r).unwrap_or(false);
            match (cache_eq, live_eq) {
                (true, true) => FileClassification::Unchanged,
                (true, false) => FileClassification::ChangedLocallyOnly,
                (false, true) => FileClassification::ChangedUpstreamOnly,
                (false, false) => FileClassification::Conflict,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Three-way diff
// ---------------------------------------------------------------------------

/// Run the three-way comparison.
///
/// Inputs:
///   - `project_root` — where the project lives (used to resolve install paths)
///   - `cache_src_path` — fetched cache `<src_path>/` directory
///   - `templates_src_path` — the immutable reference dir, normally
///     `<project_root>/context/templates/processkit/<lock.version>/`. Must
///     contain a verbatim mirror of the cache `<src_path>/` from the
///     previous install (this is what `content_init::copy_templates_from_cache`
///     writes).
///
/// Returns the full per-file diff plus a grouped view.
pub fn three_way_diff(
    project_root: &Path,
    cache_src_path: &Path,
    templates_src_path: &Path,
) -> Result<(Vec<FileDiff>, GroupedDiff)> {
    if !cache_src_path.is_dir() {
        anyhow::bail!(
            "three_way_diff: cache src_path {} is not a directory",
            cache_src_path.display()
        );
    }

    let mut diffs: Vec<FileDiff> = Vec::new();
    let mut seen_cache_keys: BTreeSet<String> = BTreeSet::new();

    // Walk the cache to find every installable file.
    walk_tree(cache_src_path, cache_src_path, &mut |rel_path| {
        let action = install_action_for(rel_path);
        let project_install = match action {
            InstallAction::Skip => return Ok(()),
            InstallAction::Install(p) => p,
            // Templated files (e.g. scaffolding/AGENTS.md) are skipped
            // by the v0.16.4 diff because the templates mirror holds
            // the unrendered cache content while the live file holds
            // the rendered output — SHA comparison would always
            // false-positive as "ChangedLocally". The install path
            // uses write_if_missing semantics for these, so user edits
            // are safe. v0.16.5+ will fix this by rendering templated
            // files into the templates mirror too. See DEC-032.
            InstallAction::InstallTemplated(_) => return Ok(()),
        };
        let rel_str = path_to_forward_slash(rel_path);
        seen_cache_keys.insert(rel_str.clone());

        let cache_abs = cache_src_path.join(rel_path);
        let cache_sha = sha256_of_file(&cache_abs)
            .with_context(|| format!("failed to hash cache file {}", cache_abs.display()))?;

        let live_abs = project_root.join(&project_install);
        let live_sha_opt = if live_abs.is_file() {
            Some(sha256_of_file(&live_abs).with_context(|| {
                format!("failed to hash live file {}", live_abs.display())
            })?)
        } else {
            None
        };

        let reference_abs = templates_src_path.join(rel_path);
        let reference_sha_opt = if reference_abs.is_file() {
            Some(sha256_of_file(&reference_abs).with_context(|| {
                format!(
                    "failed to hash reference file {}",
                    reference_abs.display()
                )
            })?)
        } else {
            None
        };

        let classification = classify(
            reference_sha_opt.as_deref(),
            Some(&cache_sha),
            live_sha_opt.as_deref(),
        );

        diffs.push(FileDiff {
            cache_rel_path: rel_str,
            project_path: Some(project_install),
            group: group_for_path(rel_path),
            classification,
        });
        Ok(())
    })?;

    // Walk the templates dir to find removed-upstream files (in reference,
    // not in cache). Skip files that wouldn't be installable anyway.
    if templates_src_path.is_dir() {
        walk_tree(templates_src_path, templates_src_path, &mut |rel_path| {
            let project_install = match install_action_for(rel_path) {
                InstallAction::Skip => return Ok(()),
                InstallAction::Install(p) => p,
                // Templated files: same skip rationale as the cache
                // walk above. v0.16.5+ will handle these properly.
                InstallAction::InstallTemplated(_) => return Ok(()),
            };
            let rel_str = path_to_forward_slash(rel_path);
            if seen_cache_keys.contains(&rel_str) {
                return Ok(());
            }
            diffs.push(FileDiff {
                cache_rel_path: rel_str,
                project_path: Some(project_install),
                group: group_for_path(rel_path),
                classification: FileClassification::RemovedUpstream,
            });
            Ok(())
        })?;
    }

    // Build the grouped view.
    let mut groups: GroupedDiff = BTreeMap::new();
    for d in &diffs {
        let key = d.group.clone().unwrap_or_default();
        groups.entry(key).or_default().push(d.clone());
    }

    Ok((diffs, groups))
}

/// Recursively walk a directory, calling `cb` with each file's path
/// relative to `root`. Honours [`should_skip_entry`] so the diff and the
/// init walker agree on which files exist.
fn walk_tree(
    root: &Path,
    dir: &Path,
    cb: &mut dyn FnMut(&Path) -> Result<()>,
) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", path.display()))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();

        if should_skip_entry(&name_str) {
            continue;
        }
        if ft.is_dir() {
            walk_tree(root, &path, cb)?;
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let rel = path.strip_prefix(root).with_context(|| {
            format!(
                "failed to relativize {} against {}",
                path.display(),
                root.display()
            )
        })?;
        cb(rel)?;
    }
    Ok(())
}

fn path_to_forward_slash(rel: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in rel.components() {
        if let std::path::Component::Normal(os) = c {
            parts.push(os.to_string_lossy().to_string());
        }
    }
    parts.join("/")
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

/// Summary counts useful for sync output.
#[derive(Debug, Default, Clone, Copy)]
pub struct DiffSummary {
    pub unchanged: usize,
    pub changed_locally_only: usize,
    pub changed_upstream_only: usize,
    pub conflict: usize,
    pub new_upstream: usize,
    pub removed_upstream: usize,
}

impl DiffSummary {
    pub fn from_diffs(diffs: &[FileDiff]) -> Self {
        let mut s = DiffSummary::default();
        for d in diffs {
            match d.classification {
                FileClassification::Unchanged => s.unchanged += 1,
                FileClassification::ChangedLocallyOnly => s.changed_locally_only += 1,
                FileClassification::ChangedUpstreamOnly => s.changed_upstream_only += 1,
                FileClassification::Conflict => s.conflict += 1,
                FileClassification::NewUpstream => s.new_upstream += 1,
                FileClassification::RemovedUpstream => s.removed_upstream += 1,
            }
        }
        s
    }

    pub fn has_user_relevant_changes(&self) -> bool {
        self.changed_upstream_only > 0
            || self.conflict > 0
            || self.new_upstream > 0
            || self.removed_upstream > 0
    }
}

// ---------------------------------------------------------------------------
// Migration document generation
// ---------------------------------------------------------------------------

/// Result of a full sync-diff run.
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub summary: DiffSummary,
    pub migration_document_path: Option<PathBuf>,
}

/// Write a Migration document for a single sync run. Outputs to
/// `<project_root>/context/migrations/pending/MIG-<id>.md`. Returns the
/// path of the written file, or `Ok(None)` if an existing matching
/// document already exists in `pending/` or `in-progress/`.
pub fn write_migration_document(
    project_root: &Path,
    lock_before: &AiboxLock,
    cache_version: &str,
    cache_resolved_commit: Option<&str>,
    summary: &DiffSummary,
    diffs: &[FileDiff],
) -> Result<Option<PathBuf>> {
    let pending_dir = project_root.join("context/migrations/pending");
    let in_progress_dir = project_root.join("context/migrations/in-progress");

    // Idempotency: skip if an existing migration document covers the
    // same (source, from_version, to_version).
    if existing_migration_matches(
        &pending_dir,
        &lock_before.source,
        &lock_before.version,
        cache_version,
    )? || existing_migration_matches(
        &in_progress_dir,
        &lock_before.source,
        &lock_before.version,
        cache_version,
    )? {
        return Ok(None);
    }

    fs::create_dir_all(&pending_dir).with_context(|| {
        format!("failed to create {}", pending_dir.display())
    })?;

    let now = chrono::Utc::now();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id_ts = now.format("%Y%m%dT%H%M%S").to_string();
    let id = format!("MIG-{}", id_ts);
    let out_path = pending_dir.join(format!("{}.md", id));

    // Determine affected groups (groups with at least one non-Unchanged entry).
    let mut affected_groups: BTreeSet<String> = BTreeSet::new();
    for d in diffs {
        if d.classification != FileClassification::Unchanged {
            affected_groups.insert(d.group.clone().unwrap_or_default());
        }
    }

    let summary_line = format!(
        "{} changed upstream, {} conflicts, {} new, {} removed ({} groups affected)",
        summary.changed_upstream_only,
        summary.conflict,
        summary.new_upstream,
        summary.removed_upstream,
        affected_groups.len(),
    );

    // YAML frontmatter. We assemble this by hand to keep it readable and
    // to avoid pulling in a second YAML serializer; we already use
    // serde_yaml elsewhere but this document is short and our values are
    // simple.
    let mut body = String::new();
    body.push_str("---\n");
    body.push_str("apiVersion: processkit.projectious.work/v1\n");
    body.push_str("kind: Migration\n");
    body.push_str("metadata:\n");
    body.push_str(&format!("  id: {}\n", id));
    body.push_str(&format!("  created: {}\n", now_iso));
    body.push_str("spec:\n");
    body.push_str("  source: processkit\n");
    body.push_str(&format!(
        "  source_url: {}\n",
        yaml_scalar(&lock_before.source)
    ));
    body.push_str(&format!(
        "  from_version: {}\n",
        yaml_scalar(&lock_before.version)
    ));
    body.push_str(&format!("  to_version: {}\n", yaml_scalar(cache_version)));
    if let Some(commit) = cache_resolved_commit {
        body.push_str(&format!("  to_resolved_commit: {}\n", yaml_scalar(commit)));
    }
    body.push_str("  state: pending\n");
    body.push_str("  generated_by: aibox sync\n");
    body.push_str(&format!("  generated_at: {}\n", now_iso));
    body.push_str(&format!("  summary: {}\n", yaml_scalar(&summary_line)));
    body.push_str("  affected_groups:\n");
    if affected_groups.is_empty() {
        body.push_str("    []\n");
    } else {
        for g in &affected_groups {
            body.push_str(&format!("    - {}\n", yaml_scalar(g)));
        }
    }
    body.push_str("---\n\n");

    // Human-readable markdown body.
    body.push_str(&format!("# Migration {}\n\n", id));
    body.push_str(&format!(
        "From `{}` to `{}` (source: `{}`).\n\n",
        lock_before.version, cache_version, lock_before.source
    ));
    body.push_str(&format!("{}\n\n", summary_line));
    body.push_str("## Counts\n\n");
    body.push_str(&format!("- unchanged: {}\n", summary.unchanged));
    body.push_str(&format!(
        "- changed-locally-only: {}\n",
        summary.changed_locally_only
    ));
    body.push_str(&format!(
        "- changed-upstream-only: {}\n",
        summary.changed_upstream_only
    ));
    body.push_str(&format!("- conflict: {}\n", summary.conflict));
    body.push_str(&format!("- new-upstream: {}\n", summary.new_upstream));
    body.push_str(&format!(
        "- removed-upstream: {}\n\n",
        summary.removed_upstream
    ));

    // Group by group → classification → files.
    let mut by_group: BTreeMap<String, BTreeMap<&'static str, Vec<&FileDiff>>> = BTreeMap::new();
    for d in diffs {
        if d.classification == FileClassification::Unchanged {
            continue;
        }
        by_group
            .entry(d.group.clone().unwrap_or_default())
            .or_default()
            .entry(d.classification.label())
            .or_default()
            .push(d);
    }

    if by_group.is_empty() {
        body.push_str("_No user-relevant changes._\n");
    } else {
        body.push_str("## Changes by group\n\n");
        for (group, by_class) in &by_group {
            let label = if group.is_empty() {
                "(ungrouped)".to_string()
            } else {
                group.clone()
            };
            body.push_str(&format!("### {}\n\n", label));
            for (cls, entries) in by_class {
                body.push_str(&format!("**{}**\n\n", cls));
                for d in entries {
                    let proj = d
                        .project_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "-".to_string());
                    body.push_str(&format!("- `{}` → `{}`\n", d.cache_rel_path, proj));
                }
                body.push('\n');
            }
        }
    }

    fs::write(&out_path, body)
        .with_context(|| format!("failed to write {}", out_path.display()))?;
    Ok(Some(out_path))
}

/// Minimal YAML scalar encoder. If the value contains any YAML-unsafe
/// characters, wrap it in double quotes and escape backslashes / quotes.
fn yaml_scalar(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('"')
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.starts_with('-')
        || s.starts_with('[')
        || s.starts_with('{');
    if !needs_quote {
        return s.to_string();
    }
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// Return true if any `MIG-*.md` file in `dir` has YAML frontmatter whose
/// `spec` block matches the given `(source, from, to)`. Missing directory
/// is Ok(false).
fn existing_migration_matches(
    dir: &Path,
    source: &str,
    from_version: &str,
    to_version: &str,
) -> Result<bool> {
    if !dir.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.starts_with("MIG-") || !name.ends_with(".md") {
            continue;
        }
        let body = match fs::read_to_string(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if let Some((s, f, t)) = extract_migration_pair(&body)
            && s == source
            && f == from_version
            && t == to_version
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Pull `(source_url, from_version, to_version)` out of a migration
/// document's YAML frontmatter. Returns None if the document doesn't
/// have the expected shape.
fn extract_migration_pair(body: &str) -> Option<(String, String, String)> {
    let rest = body.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    let mut source_url: Option<String> = None;
    let mut from_version: Option<String> = None;
    let mut to_version: Option<String> = None;
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        if let Some(v) = trimmed.strip_prefix("source_url:") {
            source_url = Some(parse_yaml_scalar_value(v.trim()));
        } else if let Some(v) = trimmed.strip_prefix("from_version:") {
            from_version = Some(parse_yaml_scalar_value(v.trim()));
        } else if let Some(v) = trimmed.strip_prefix("to_version:") {
            to_version = Some(parse_yaml_scalar_value(v.trim()));
        }
    }
    Some((source_url?, from_version?, to_version?))
}

fn parse_yaml_scalar_value(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    s.to_string()
}

// ---------------------------------------------------------------------------
// Top-level run entry used by `cmd_sync`
// ---------------------------------------------------------------------------

/// Run the full content-source sync-diff flow:
///
/// 1. Fetch the cache for the version pinned in the lock (idempotent).
/// 2. Resolve the templates reference dir for that version.
/// 3. Three-way diff against cache + templates + live.
/// 4. If there are user-relevant changes, write a Migration document.
/// 5. Return a `SyncReport` summarizing the outcome.
///
/// `config` is read for the current `release_asset_url_template` so a
/// user who has updated their template (e.g. switched from a fork URL
/// to a different host) gets the new template applied immediately
/// without having to re-init.
pub fn run_content_sync(
    project_root: &Path,
    lock: &AiboxLock,
    config: &crate::config::AiboxConfig,
) -> Result<SyncReport> {
    let fetched = crate::content_source::fetch(
        &lock.source,
        &lock.version,
        lock.branch.as_deref(),
        &lock.src_path,
        config.processkit.release_asset_url_template.as_deref(),
    )
    .with_context(|| "failed to fetch content-source cache".to_string())?;

    let templates_dir = templates_dir_for_version(project_root, &lock.version);

    let (diffs, _groups) = three_way_diff(project_root, &fetched.src_path, &templates_dir)?;
    let summary = DiffSummary::from_diffs(&diffs);

    let migration_document_path = if summary.has_user_relevant_changes() {
        write_migration_document(
            project_root,
            lock,
            &fetched.version,
            fetched.resolved_commit.as_deref(),
            &summary,
            &diffs,
        )?
    } else {
        None
    };

    Ok(SyncReport {
        summary,
        migration_document_path,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_init::{copy_templates_from_cache, install_files_from_cache};
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    // -- classify() ---------------------------------------------------------

    #[test]
    fn classify_unchanged() {
        assert_eq!(
            classify(Some("a"), Some("a"), Some("a")),
            FileClassification::Unchanged
        );
    }

    #[test]
    fn classify_changed_locally_only() {
        assert_eq!(
            classify(Some("a"), Some("a"), Some("b")),
            FileClassification::ChangedLocallyOnly
        );
        // Missing live file is also a local deviation.
        assert_eq!(
            classify(Some("a"), Some("a"), None),
            FileClassification::ChangedLocallyOnly
        );
    }

    #[test]
    fn classify_changed_upstream_only() {
        assert_eq!(
            classify(Some("a"), Some("b"), Some("a")),
            FileClassification::ChangedUpstreamOnly
        );
    }

    #[test]
    fn classify_conflict() {
        assert_eq!(
            classify(Some("a"), Some("b"), Some("c")),
            FileClassification::Conflict
        );
        // Missing live file with upstream changed is also a conflict —
        // the local tree can't be considered "unchanged-from-reference".
        assert_eq!(
            classify(Some("a"), Some("b"), None),
            FileClassification::Conflict
        );
    }

    #[test]
    fn classify_new_upstream() {
        assert_eq!(
            classify(None, Some("c"), None),
            FileClassification::NewUpstream
        );
        assert_eq!(
            classify(None, Some("c"), Some("c")),
            FileClassification::NewUpstream
        );
    }

    #[test]
    fn classify_removed_upstream() {
        assert_eq!(
            classify(Some("a"), None, Some("a")),
            FileClassification::RemovedUpstream
        );
    }

    // -- Synthetic tree scaffolding -----------------------------------------

    /// Build a synthetic cache at `<tmp>/cache/src` with a handful of
    /// processkit-shaped files. Returns the src_path.
    fn build_cache(tmp: &Path) -> PathBuf {
        let src = tmp.join("cache/src");
        fs::create_dir_all(src.join("skills/event-log/templates")).unwrap();
        fs::create_dir_all(src.join("primitives/schemas")).unwrap();
        fs::create_dir_all(src.join("lib/processkit")).unwrap();
        fs::write(src.join("skills/event-log/SKILL.md"), "# skill v1\n").unwrap();
        fs::write(
            src.join("skills/event-log/templates/entry.yaml"),
            "name: entry\n",
        )
        .unwrap();
        fs::write(
            src.join("primitives/schemas/workitem.yaml"),
            "name: workitem\n",
        )
        .unwrap();
        fs::write(src.join("lib/processkit/entity.py"), "print(1)\n").unwrap();
        // A file that install_action_for will Skip. PROVENANCE.toml is
        // always skipped (aibox reads it from the cache directly). Note:
        // INDEX.md is NOT skipped any more as of v0.16.4 (BACK-116) — it
        // installs at context/INDEX.md.
        fs::write(src.join("PROVENANCE.toml"), "version = \"v1.0.0\"\n").unwrap();
        src
    }

    /// Install the cache into `project_root` AND populate the templates dir
    /// at `templates_dir`. Returns the templates_dir for use by tests.
    fn install_and_snapshot(cache_src: &Path, project_root: &Path) -> PathBuf {
        install_files_from_cache(cache_src, project_root).unwrap();
        // Stash the templates dir at a fixed version label so the diff
        // can find it. Use a free-standing path that does not depend on
        // copy_templates_from_cache writing into project_root — but it
        // does, so just call the helper.
        copy_templates_from_cache(cache_src, project_root, "v1.0.0").unwrap();
        templates_dir_for_version(project_root, "v1.0.0")
    }

    // -- three_way_diff unit tests -----------------------------------------

    #[test]
    fn three_way_diff_synthetic_tree_all_unchanged() {
        let tmp = TempDir::new().unwrap();
        let cache_src = build_cache(tmp.path());
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let templates = install_and_snapshot(&cache_src, &project);

        let (diffs, _groups) = three_way_diff(&project, &cache_src, &templates).unwrap();
        assert!(!diffs.is_empty());
        for d in &diffs {
            assert_eq!(
                d.classification,
                FileClassification::Unchanged,
                "expected Unchanged for {}, got {:?}",
                d.cache_rel_path,
                d.classification
            );
        }
    }

    #[test]
    fn three_way_diff_synthetic_tree_with_each_classification() {
        let tmp = TempDir::new().unwrap();
        let cache_src = build_cache(tmp.path());
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let templates = install_and_snapshot(&cache_src, &project);

        // 1. ChangedUpstreamOnly: perturb cache file only — schema.
        fs::write(
            cache_src.join("primitives/schemas/workitem.yaml"),
            "name: workitem-v2\n",
        )
        .unwrap();

        // 2. ChangedLocallyOnly: perturb live file only — SKILL.md.
        let live_skill = project.join("context/skills/event-log/SKILL.md");
        fs::write(&live_skill, "# locally edited\n").unwrap();

        // 3. Conflict: perturb both cache and live for entry.yaml.
        let conflict_cache = cache_src.join("skills/event-log/templates/entry.yaml");
        let conflict_live =
            project.join("context/skills/event-log/templates/entry.yaml");
        fs::write(&conflict_cache, "name: upstream-edit\n").unwrap();
        fs::write(&conflict_live, "name: local-edit\n").unwrap();

        // 4. NewUpstream: add a new cache file that was not in templates.
        let new_file = cache_src.join("skills/event-log/NEW.md");
        fs::write(&new_file, "# brand new\n").unwrap();

        // 5. RemovedUpstream: drop a file from the cache; templates still has it.
        let removed_cache = cache_src.join("lib/processkit/entity.py");
        fs::remove_file(&removed_cache).unwrap();

        let (diffs, _groups) = three_way_diff(&project, &cache_src, &templates).unwrap();
        let by_path: BTreeMap<&str, FileClassification> = diffs
            .iter()
            .map(|d| (d.cache_rel_path.as_str(), d.classification))
            .collect();

        assert_eq!(
            by_path.get("primitives/schemas/workitem.yaml"),
            Some(&FileClassification::ChangedUpstreamOnly),
        );
        assert_eq!(
            by_path.get("skills/event-log/SKILL.md"),
            Some(&FileClassification::ChangedLocallyOnly),
        );
        assert_eq!(
            by_path.get("skills/event-log/templates/entry.yaml"),
            Some(&FileClassification::Conflict),
        );
        assert_eq!(
            by_path.get("skills/event-log/NEW.md"),
            Some(&FileClassification::NewUpstream),
        );
        assert_eq!(
            by_path.get("lib/processkit/entity.py"),
            Some(&FileClassification::RemovedUpstream),
        );
    }

    #[test]
    fn three_way_diff_groups_files_by_group() {
        let tmp = TempDir::new().unwrap();
        let cache_src = build_cache(tmp.path());
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let templates = install_and_snapshot(&cache_src, &project);
        let (_diffs, groups) = three_way_diff(&project, &cache_src, &templates).unwrap();

        assert!(groups.contains_key("skills/event-log"));
        assert!(groups.contains_key("lib"));
        assert!(groups.contains_key("primitives/schemas/workitem"));

        // Every skills/event-log file should be in that bucket.
        let skill_bucket = &groups["skills/event-log"];
        for d in skill_bucket {
            assert_eq!(d.group.as_deref(), Some("skills/event-log"));
        }
    }

    #[test]
    fn three_way_diff_skips_uninstallable_files() {
        let tmp = TempDir::new().unwrap();
        let cache_src = build_cache(tmp.path());
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let templates = install_and_snapshot(&cache_src, &project);
        let (diffs, _) = three_way_diff(&project, &cache_src, &templates).unwrap();

        // PROVENANCE.toml is Skip per install_action_for and must not
        // appear in the diff, even though it lives in the templates
        // dir. (Until v0.16.3 this test used INDEX.md as the canonical
        // skipped file; v0.16.4 / BACK-116 routes INDEX.md to its
        // per-directory destinations, so we use PROVENANCE.toml here
        // — it remains the unconditional skip target.)
        assert!(
            diffs.iter().all(|d| d.cache_rel_path != "PROVENANCE.toml"),
            "PROVENANCE.toml should not appear in diff"
        );
    }

    // -- DiffSummary --------------------------------------------------------

    fn mk_diff(rel: &str, cls: FileClassification) -> FileDiff {
        FileDiff {
            cache_rel_path: rel.to_string(),
            project_path: None,
            group: None,
            classification: cls,
        }
    }

    #[test]
    fn summary_from_diffs_counts_correctly() {
        let diffs = vec![
            mk_diff("a", FileClassification::Unchanged),
            mk_diff("b", FileClassification::Unchanged),
            mk_diff("c", FileClassification::ChangedLocallyOnly),
            mk_diff("d", FileClassification::ChangedUpstreamOnly),
            mk_diff("e", FileClassification::ChangedUpstreamOnly),
            mk_diff("f", FileClassification::Conflict),
            mk_diff("g", FileClassification::NewUpstream),
            mk_diff("h", FileClassification::RemovedUpstream),
        ];
        let s = DiffSummary::from_diffs(&diffs);
        assert_eq!(s.unchanged, 2);
        assert_eq!(s.changed_locally_only, 1);
        assert_eq!(s.changed_upstream_only, 2);
        assert_eq!(s.conflict, 1);
        assert_eq!(s.new_upstream, 1);
        assert_eq!(s.removed_upstream, 1);
    }

    #[test]
    fn summary_has_user_relevant_changes_returns_true_when_any_change() {
        let empty = DiffSummary::default();
        assert!(!empty.has_user_relevant_changes());

        // Locally-only is NOT user-relevant from sync's perspective.
        let locally_only = DiffSummary {
            changed_locally_only: 5,
            ..Default::default()
        };
        assert!(!locally_only.has_user_relevant_changes());

        let upstream_only = DiffSummary {
            changed_upstream_only: 1,
            ..Default::default()
        };
        assert!(upstream_only.has_user_relevant_changes());

        let conflict = DiffSummary {
            conflict: 1,
            ..Default::default()
        };
        assert!(conflict.has_user_relevant_changes());

        let new_upstream = DiffSummary {
            new_upstream: 1,
            ..Default::default()
        };
        assert!(new_upstream.has_user_relevant_changes());

        let removed_upstream = DiffSummary {
            removed_upstream: 1,
            ..Default::default()
        };
        assert!(removed_upstream.has_user_relevant_changes());
    }

    // -- write_migration_document ------------------------------------------

    fn sample_lock() -> AiboxLock {
        AiboxLock {
            source: "https://github.com/example/processkit.git".to_string(),
            version: "v1.0.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: Some("dead".to_string()),
            release_asset_sha256: None,
            installed_at: "2026-04-06T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn write_migration_document_creates_file_with_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let lock = sample_lock();
        let diffs = vec![
            FileDiff {
                cache_rel_path: "primitives/schemas/workitem.yaml".to_string(),
                project_path: Some(PathBuf::from("context/schemas/workitem.yaml")),
                group: Some("primitives/schemas/workitem".to_string()),
                classification: FileClassification::ChangedUpstreamOnly,
            },
            FileDiff {
                cache_rel_path: "skills/event-log/NEW.md".to_string(),
                project_path: Some(PathBuf::from("context/skills/event-log/NEW.md")),
                group: Some("skills/event-log".to_string()),
                classification: FileClassification::NewUpstream,
            },
        ];
        let summary = DiffSummary::from_diffs(&diffs);

        let written =
            write_migration_document(tmp.path(), &lock, "v1.0.1", Some("beef"), &summary, &diffs)
                .unwrap()
                .expect("should write a document");

        assert!(written.exists());
        let body = fs::read_to_string(&written).unwrap();
        assert!(body.starts_with("---\n"));
        assert!(body.contains("kind: Migration"));
        assert!(body.contains("from_version: v1.0.0"));
        assert!(body.contains("to_version: v1.0.1"));
        assert!(body.contains("to_resolved_commit: beef"));
        assert!(body.contains("source: processkit"));
        assert!(body.contains("state: pending"));
        assert!(body.contains("generated_by: aibox sync"));
        assert!(body.contains("skills/event-log"));
        assert!(body.contains("primitives/schemas/workitem.yaml"));
        assert!(body.contains("changed-upstream-only"));
        assert!(body.contains("new-upstream"));

        // Frontmatter parser should round-trip the identifying pair.
        let pair = extract_migration_pair(&body).expect("should parse frontmatter");
        assert_eq!(pair.0, "https://github.com/example/processkit.git");
        assert_eq!(pair.1, "v1.0.0");
        assert_eq!(pair.2, "v1.0.1");
    }

    #[test]
    fn write_migration_document_idempotent() {
        let tmp = TempDir::new().unwrap();
        let lock = sample_lock();
        let diffs = vec![FileDiff {
            cache_rel_path: "primitives/schemas/workitem.yaml".to_string(),
            project_path: Some(PathBuf::from("context/schemas/workitem.yaml")),
            group: Some("primitives/schemas/workitem".to_string()),
            classification: FileClassification::ChangedUpstreamOnly,
        }];
        let summary = DiffSummary::from_diffs(&diffs);

        let first =
            write_migration_document(tmp.path(), &lock, "v1.0.1", None, &summary, &diffs)
                .unwrap();
        assert!(first.is_some());

        let second =
            write_migration_document(tmp.path(), &lock, "v1.0.1", None, &summary, &diffs)
                .unwrap();
        assert!(
            second.is_none(),
            "second call should be a no-op because a matching document already exists"
        );
    }

    #[test]
    fn write_migration_document_detects_in_progress_copy() {
        let tmp = TempDir::new().unwrap();
        let lock = sample_lock();
        // Pre-place an in-progress migration document for the same pair.
        let in_progress = tmp.path().join("context/migrations/in-progress");
        fs::create_dir_all(&in_progress).unwrap();
        let pre = "---\napiVersion: processkit.projectious.work/v1\nkind: Migration\nspec:\n  source_url: https://github.com/example/processkit.git\n  from_version: v1.0.0\n  to_version: v1.0.1\n---\n\nbody\n";
        fs::write(in_progress.join("MIG-existing.md"), pre).unwrap();

        let diffs = vec![FileDiff {
            cache_rel_path: "primitives/schemas/workitem.yaml".to_string(),
            project_path: Some(PathBuf::from("context/schemas/workitem.yaml")),
            group: Some("primitives/schemas/workitem".to_string()),
            classification: FileClassification::ChangedUpstreamOnly,
        }];
        let summary = DiffSummary::from_diffs(&diffs);
        let out =
            write_migration_document(tmp.path(), &lock, "v1.0.1", None, &summary, &diffs)
                .unwrap();
        assert!(out.is_none(), "should be no-op due to in-progress match");
    }
}
