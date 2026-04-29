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
//! | reference vs cache | live vs reference | live vs cache | classification         |
//! |--------------------|-------------------|---------------|------------------------|
//! | equal              | equal             | (= ref)       | Unchanged              |
//! | equal              | differ / missing  | (= cache)     | ChangedLocallyOnly     |
//! | differ             | equal             | differ        | ChangedUpstreamOnly    |
//! | differ             | differ            | equal         | Unchanged (live already at new upstream) |
//! | differ             | differ            | differ        | Conflict               |
//! | (in cache, not in reference)  | n/a   | n/a           | NewUpstream            |
//! | (in reference, not in cache)  | n/a   | n/a           | RemovedUpstream        |
//!
//! The "live already at new upstream" row covers the stale-lock case:
//! `aibox.lock` says the project is at version *N*, the templates mirror at
//! `context/templates/processkit/N/` is therefore the reference, but the
//! user's live tree has independently been brought to version *N+1* (e.g.,
//! a fresh checkout where `context/` was committed at N+1 while the lock
//! still records N). Without the `live == cache` check the classifier
//! would report every such file as a Conflict (BACK-TrueRaven, 2026-04-26).
//!
//! Files whose install-action is `Skip` (processkit-internal, not
//! user-facing) are excluded from the diff entirely — they live in the
//! templates dir as part of the full upstream snapshot but are never
//! reported in the diff because they have no live counterpart.
//!
//! ## Upstream-removed-long-ago (`RemovedUpstreamStale`)
//!
//! The truth table above is "two-template" — it only consults the *most
//! recent* reference snapshot (`template_old`, i.e. the mirror of the
//! version recorded in `aibox.lock`). That misclassifies a real-world
//! case the user encounters when they skip versions: a skill that was
//! present in, say, v0.18.x but removed upstream in v0.19.0. If the user
//! never ran a sync against an in-between version, neither
//! `template_old` (v0.19.x) nor the new cache (v0.21.x) contains the
//! file — yet the live tree still has it. The two-template diff would
//! tag it `ChangedLocallyOnly` ("user-added"), which is wrong: it's
//! upstream cruft, not local content.
//!
//! After the primary three-way diff runs, every `ChangedLocallyOnly`
//! result is re-checked against *all* older mirrors under
//! `context/templates/processkit/<v>/`. If any older mirror contains
//! the file, the classification is rewritten to
//! `RemovedUpstreamStale { last_seen_in: <newest older version> }`. The
//! migration document then proposes (suggests, not auto-applies) the
//! cleanup. Files absent from every mirror remain `ChangedLocallyOnly`.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::content_init::templates_dir_for_version;
use crate::content_install::{InstallAction, install_action_for};
use crate::lock::{group_for_path, sha256_of_file, should_skip_entry};

// ---------------------------------------------------------------------------
// Per-file classification
// ---------------------------------------------------------------------------

/// Per-file classification from the three-way comparison.
///
/// `RemovedUpstreamStale` carries a `String` (the older mirror version
/// that last contained the file) so this enum is `Clone` rather than
/// `Copy`. Pattern matching that previously held copies must now hold
/// borrows or clones; see callers for the change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileClassification {
    /// Reference, cache, and live all match — nothing to do. Also covers
    /// the stale-lock case where `live == cache` but the on-disk
    /// reference (templates mirror at `lock.version`) is older: the live
    /// tree is already at the new upstream, so there is nothing to apply
    /// for this file. (See the "live already at new upstream" row of the
    /// truth table at the top of this module.)
    Unchanged,
    /// Reference matches cache but not live — user has edited it locally;
    /// upstream has not changed. No-op for this migration but worth noting.
    ChangedLocallyOnly,
    /// Reference matches live but not cache — upstream has changed; user
    /// has not touched it. Safe to take with one approval.
    ChangedUpstreamOnly,
    /// Reference, cache, and live are all distinct — three-way divergence.
    /// Must be resolved by hand. (When `live == cache` but `cache !=
    /// reference`, the file is reported as `Unchanged` instead — see that
    /// variant's docs.)
    Conflict,
    /// File exists in cache but not in reference (i.e. wasn't in the
    /// previous version of upstream). New addition.
    NewUpstream,
    /// File exists in reference but not in cache (i.e. removed from
    /// upstream). Decide whether to drop locally or keep as a project fork.
    RemovedUpstream,
    /// File is missing from both `template_old` and the new cache, but
    /// is present in an *older* mirror under
    /// `context/templates/processkit/`. Indicates upstream removed it
    /// in a version the user skipped over, so the live copy is dead
    /// content that the user never explicitly authored. The migration
    /// document *suggests* removal — the apply step never auto-deletes.
    /// `last_seen_in` is the version label of the newest older mirror
    /// that still contained the file (used for the migration doc
    /// diagnostic).
    RemovedUpstreamStale { last_seen_in: String },
}

impl FileClassification {
    /// Short human-readable label used in migration documents.
    pub fn label(&self) -> &'static str {
        match self {
            FileClassification::Unchanged => "unchanged",
            FileClassification::ChangedLocallyOnly => "changed-locally-only",
            FileClassification::ChangedUpstreamOnly => "changed-upstream-only",
            FileClassification::Conflict => "conflict",
            FileClassification::NewUpstream => "new-upstream",
            FileClassification::RemovedUpstream => "removed-upstream",
            FileClassification::RemovedUpstreamStale { .. } => "removed-upstream-stale",
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
/// - Otherwise consult the three-way truth table at the top of this module.
///
/// The classifier is "stale-lock-aware": when `cache != reference` but
/// `live == cache`, the live tree is already at the new upstream and the
/// file is classified `Unchanged`, not `Conflict`. Without this guard a
/// fresh checkout (where `context/` was committed ahead of `aibox.lock`)
/// produces a flood of false-positive conflicts during `aibox sync`
/// (BACK-TrueRaven, 2026-04-26).
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
            let cache_eq_ref = r == c;
            let live_eq_ref = live_sha.map(|l| l == r).unwrap_or(false);
            let live_eq_cache = live_sha.map(|l| l == c).unwrap_or(false);
            match (cache_eq_ref, live_eq_ref, live_eq_cache) {
                // Upstream hasn't moved (cache == reference). live_eq_cache
                // is implied by live_eq_ref in this branch, so the third
                // bit is redundant.
                (true, true, _) => FileClassification::Unchanged,
                (true, false, _) => FileClassification::ChangedLocallyOnly,
                // Upstream moved.
                (false, true, _) => FileClassification::ChangedUpstreamOnly,
                // Live already matches the new upstream — nothing to apply.
                // Reference is stale relative to the live tree (e.g. fresh
                // checkout where `context/` shipped at version N+1 but the
                // lock still records N). Treat as Unchanged so the
                // migration document doesn't cry "conflict" on every file.
                (false, false, true) => FileClassification::Unchanged,
                // Genuine three-way divergence.
                (false, false, false) => FileClassification::Conflict,
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
    // Side-table of live SHAs keyed by cache_rel_path. Used by the
    // RemovedUpstreamStale reclassification pass below to distinguish
    // "live is byte-identical to the kept-old-upstream baseline"
    // (genuine stale upstream cruft) from "user has extended/modified
    // the file on top of that baseline" (real local content). Without
    // this guard we silently propose deletion of user work — Bug #57.
    let mut live_shas: BTreeMap<String, String> = BTreeMap::new();

    // Walk the cache to find every installable file.
    walk_tree(cache_src_path, cache_src_path, &mut |rel_path| {
        let action = install_action_for(rel_path);
        let project_install = match action {
            InstallAction::Skip => return Ok(()),
            InstallAction::Install(p) => p,
            // Templated files (e.g. scaffolding/AGENTS.md) are now
            // treated like Install for diff purposes. The v0.16.5
            // rendered-mirror change (DEC-034) makes this correct:
            // the templates mirror holds the SAME rendered content
            // as the live file (both pass through render() with the
            // Class A vocabulary), so SHA comparison no longer
            // false-positives.
            InstallAction::InstallTemplated(p) => p,
        };
        let rel_str = path_to_forward_slash(rel_path);
        seen_cache_keys.insert(rel_str.clone());

        let cache_abs = cache_src_path.join(rel_path);
        let cache_sha = sha256_of_file(&cache_abs)
            .with_context(|| format!("failed to hash cache file {}", cache_abs.display()))?;

        let live_abs = project_root.join(&project_install);
        let live_sha_opt = if live_abs.is_file() {
            Some(
                sha256_of_file(&live_abs)
                    .with_context(|| format!("failed to hash live file {}", live_abs.display()))?,
            )
        } else {
            None
        };
        if let Some(s) = &live_sha_opt {
            live_shas.insert(rel_str.clone(), s.clone());
        }

        let reference_abs = templates_src_path.join(rel_path);
        let reference_sha_opt = if reference_abs.is_file() {
            Some(sha256_of_file(&reference_abs).with_context(|| {
                format!("failed to hash reference file {}", reference_abs.display())
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
                // Templated files: rendered mirror, treated as Install.
                // See DEC-034.
                InstallAction::InstallTemplated(p) => p,
            };
            let rel_str = path_to_forward_slash(rel_path);
            if seen_cache_keys.contains(&rel_str) {
                return Ok(());
            }
            // Capture the live SHA (if any) so the stale-reclassification
            // pass below can compare it to older mirrors. RemovedUpstream
            // entries don't currently get reclassified, but recording the
            // live SHA here keeps the side-table consistent for any future
            // caller that wants it.
            let live_abs = project_root.join(&project_install);
            if live_abs.is_file()
                && let Ok(s) = sha256_of_file(&live_abs)
            {
                live_shas.insert(rel_str.clone(), s);
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

    // Reclassify any ChangedLocallyOnly entries that originate from a
    // skill removed upstream long ago. See `RemovedUpstreamStale` in
    // FileClassification for the rationale. We consult every mirror
    // under context/templates/processkit/ EXCLUDING `templates_src_path`
    // (which we've already considered as the "old" reference).
    //
    // Critical guard (Bug #57): only convert to RemovedUpstreamStale
    // when the LIVE file is byte-identical to the older mirror's copy.
    // If the user has modified or extended the file on top of the
    // older-mirror baseline, the live content represents real local
    // work — emitting a "suggested cleanup" migration for it is silent
    // data loss. In that case leave the classification as
    // ChangedLocallyOnly so the user's edits are preserved.
    let older_mirrors = list_older_mirrors(project_root, templates_src_path).unwrap_or_default();
    if !older_mirrors.is_empty() {
        for diff in &mut diffs {
            if !matches!(diff.classification, FileClassification::ChangedLocallyOnly) {
                continue;
            }
            // Walk older mirrors NEWEST-first so `last_seen_in` records
            // the latest pre-removal version (most informative for the
            // migration doc).
            for mirror in older_mirrors.iter().rev() {
                let candidate = mirror.path.join(&diff.cache_rel_path);
                if !candidate.is_file() {
                    continue;
                }
                // Hash the older-mirror file and compare to the live SHA
                // captured during the cache walk. If the live file is
                // missing entirely, also treat as a non-match — there's
                // no live copy to confirm the file is upstream cruft, so
                // leave the classification alone.
                let mirror_sha = match sha256_of_file(&candidate) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let live_sha = match live_shas.get(&diff.cache_rel_path) {
                    Some(s) => s,
                    None => break,
                };
                if live_sha != &mirror_sha {
                    // Live diverges from the kept-old-upstream baseline:
                    // user has modified or extended the file. Preserve
                    // the ChangedLocallyOnly classification.
                    break;
                }
                diff.classification = FileClassification::RemovedUpstreamStale {
                    last_seen_in: mirror.version.clone(),
                };
                break;
            }
        }
    }

    // Build the grouped view.
    let mut groups: GroupedDiff = BTreeMap::new();
    for d in &diffs {
        let key = d.group.clone().unwrap_or_default();
        groups.entry(key).or_default().push(d.clone());
    }

    Ok((diffs, groups))
}

/// One older-mirror entry: the version label and the absolute path to
/// `<project_root>/context/templates/processkit/<version>/`.
#[derive(Debug, Clone)]
struct OlderMirror {
    version: String,
    path: PathBuf,
}

/// Enumerate every mirror directory under
/// `<project_root>/context/templates/processkit/` *except* the one at
/// `current_mirror` (which is the `template_old` already consulted by the
/// primary three-way diff).
///
/// Sorted by version label using semver semantics when both labels parse
/// (with an optional leading `v`); otherwise lexicographic. Hidden
/// entries (e.g. `.aibox`) and non-directories are skipped. The result
/// is sorted oldest-first; callers iterate `.rev()` to walk newest-first
/// when they want the most-recent containing mirror.
fn list_older_mirrors(project_root: &Path, current_mirror: &Path) -> Result<Vec<OlderMirror>> {
    let templates_root = project_root.join("context/templates/processkit");
    if !templates_root.is_dir() {
        return Ok(Vec::new());
    }
    // Canonicalize the current mirror so we can compare it to each
    // candidate directory regardless of trailing slashes / `..` parts.
    let current_canon = current_mirror.canonicalize().ok();

    let mut mirrors: Vec<OlderMirror> = Vec::new();
    for entry in fs::read_dir(&templates_root)
        .with_context(|| format!("failed to read {}", templates_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", path.display()))?;
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        // Skip hidden entries (e.g. `.aibox`), and the live "current"
        // mirror that's already the diff's reference baseline.
        if name_str.starts_with('.') {
            continue;
        }
        if let Some(ref cur) = current_canon
            && let Ok(this) = path.canonicalize()
            && &this == cur
        {
            continue;
        }
        mirrors.push(OlderMirror {
            version: name_str,
            path,
        });
    }

    mirrors.sort_by(|a, b| compare_version_labels(&a.version, &b.version));
    Ok(mirrors)
}

/// Compare two version labels for ordering older→newer.
///
/// Strips an optional leading `v` and tries `semver::Version::parse` on
/// each side. If both parse, uses semver ordering; otherwise falls back
/// to lexicographic ordering of the raw labels (acceptable for the
/// `v0.X.Y` scheme per WS-6).
fn compare_version_labels(a: &str, b: &str) -> std::cmp::Ordering {
    let strip = |s: &str| s.strip_prefix('v').unwrap_or(s).to_string();
    let pa = semver::Version::parse(&strip(a)).ok();
    let pb = semver::Version::parse(&strip(b)).ok();
    match (pa, pb) {
        (Some(va), Some(vb)) => va.cmp(&vb),
        _ => a.cmp(b),
    }
}

/// Recursively walk a directory, calling `cb` with each file's path
/// relative to `root`. Honours [`should_skip_entry`] so the diff and the
/// init walker agree on which files exist.
fn walk_tree(root: &Path, dir: &Path, cb: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?
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
    /// Count of `RemovedUpstreamStale` entries — files present in the
    /// live tree that were removed upstream in a version the user
    /// skipped over. Surfaced separately so the sync migration doc can
    /// propose suggested cleanup without bumping the strict
    /// upstream-side counters.
    pub removed_upstream_stale: usize,
}

impl DiffSummary {
    pub fn from_diffs(diffs: &[FileDiff]) -> Self {
        let mut s = DiffSummary::default();
        for d in diffs {
            match &d.classification {
                FileClassification::Unchanged => s.unchanged += 1,
                FileClassification::ChangedLocallyOnly => s.changed_locally_only += 1,
                FileClassification::ChangedUpstreamOnly => s.changed_upstream_only += 1,
                FileClassification::Conflict => s.conflict += 1,
                FileClassification::NewUpstream => s.new_upstream += 1,
                FileClassification::RemovedUpstream => s.removed_upstream += 1,
                FileClassification::RemovedUpstreamStale { .. } => s.removed_upstream_stale += 1,
            }
        }
        s
    }

    pub fn has_user_relevant_changes(&self) -> bool {
        self.changed_upstream_only > 0
            || self.conflict > 0
            || self.new_upstream > 0
            || self.removed_upstream > 0
            || self.removed_upstream_stale > 0
    }

    /// True when upstream itself introduced at least one file-level change
    /// (added, removed, or modified-upstream-only). A `conflict` is NOT an
    /// upstream-side change — it means upstream is unchanged but the local
    /// copy diverged.
    ///
    /// Used to decide whether a `from == to` sync can safely skip emitting
    /// a migration document: if upstream didn't move, every conflict is a
    /// local-only edit that no migration can resolve.
    pub fn has_upstream_side_changes(&self) -> bool {
        self.changed_upstream_only > 0 || self.new_upstream > 0 || self.removed_upstream > 0
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
///
/// Takes the processkit section directly (not the full `AiboxLock`)
/// because the migration document only describes processkit content
/// changes — the `[aibox]` section's CLI version is unrelated.
pub fn write_migration_document(
    project_root: &Path,
    lock_before: &crate::lock::ProcessKitLockSection,
    cache_version: &str,
    cache_resolved_commit: Option<&str>,
    summary: &DiffSummary,
    diffs: &[FileDiff],
) -> Result<Option<PathBuf>> {
    // No-op guard: at `from == to`, every "conflict" is a local-only edit
    // that upstream never touched, so no migration is actually needed.
    // Only write when upstream itself moved something — or when we
    // discovered an upstream-removed-long-ago skill that needs
    // suggested cleanup (RemovedUpstreamStale; WS-6).
    if lock_before.version == cache_version
        && !summary.has_upstream_side_changes()
        && summary.removed_upstream_stale == 0
    {
        return Ok(None);
    }

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

    fs::create_dir_all(&pending_dir)
        .with_context(|| format!("failed to create {}", pending_dir.display()))?;

    let now = chrono::Utc::now();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id_ts = now.format("%Y%m%dT%H%M%S").to_string();
    let id = format!("MIG-{}", id_ts);
    let out_path = pending_dir.join(format!("{}.md", id));

    // Determine affected groups (groups with at least one non-Unchanged entry).
    let mut affected_groups: BTreeSet<String> = BTreeSet::new();
    for d in diffs {
        if !matches!(d.classification, FileClassification::Unchanged) {
            affected_groups.insert(d.group.clone().unwrap_or_default());
        }
    }

    let summary_line = format!(
        "{} changed upstream, {} conflicts, {} new, {} removed, {} stale-removed ({} groups affected)",
        summary.changed_upstream_only,
        summary.conflict,
        summary.new_upstream,
        summary.removed_upstream,
        summary.removed_upstream_stale,
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
        "- removed-upstream: {}\n",
        summary.removed_upstream
    ));
    body.push_str(&format!(
        "- removed-upstream-stale: {}\n\n",
        summary.removed_upstream_stale
    ));

    // RemovedUpstreamStale gets its own dedicated section above the
    // group-by-group breakdown so the user can see at a glance which
    // local skills are upstream cruft. Listed by project_path with the
    // version that last contained the file. The action is *suggested*
    // — apply never auto-deletes user content (WS-6).
    let stale_entries: Vec<(&FileDiff, &str)> = diffs
        .iter()
        .filter_map(|d| match &d.classification {
            FileClassification::RemovedUpstreamStale { last_seen_in } => {
                Some((d, last_seen_in.as_str()))
            }
            _ => None,
        })
        .collect();
    if !stale_entries.is_empty() {
        body.push_str("## Skills removed upstream (suggested cleanup)\n\n");
        body.push_str("The following local skills were present in older processkit versions but\n");
        body.push_str("have been removed upstream. Review and delete if no longer needed:\n\n");
        for (d, last_seen) in &stale_entries {
            let proj = d
                .project_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| d.cache_rel_path.clone());
            body.push_str(&format!(
                "- `{}/`  (last present in {}, removed in {})\n",
                proj, last_seen, lock_before.version,
            ));
        }
        body.push_str("\n_Apply with care — `aibox migrate apply` only marks the document as\n");
        body.push_str("applied; deletion is a manual step that you should only take after\n");
        body.push_str("confirming the skill is no longer wanted._\n\n");
    }

    // Group by group → classification → files.
    let mut by_group: BTreeMap<String, BTreeMap<&'static str, Vec<&FileDiff>>> = BTreeMap::new();
    for d in diffs {
        if matches!(d.classification, FileClassification::Unchanged) {
            continue;
        }
        // RemovedUpstreamStale already shown above in its dedicated
        // section; don't duplicate it in the per-group breakdown.
        if matches!(
            d.classification,
            FileClassification::RemovedUpstreamStale { .. }
        ) {
            continue;
        }
        by_group
            .entry(d.group.clone().unwrap_or_default())
            .or_default()
            .entry(d.classification.label())
            .or_default()
            .push(d);
    }

    if by_group.is_empty() && stale_entries.is_empty() {
        body.push_str("_No user-relevant changes._\n");
    } else if !by_group.is_empty() {
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
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
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
/// 1. Fetch the cache for `config.processkit.version` (idempotent; the
///    install step that precedes this call has already populated the cache).
/// 2. Use `from_pk.version` to locate the on-disk reference snapshot
///    (`context/templates/processkit/<from_pk.version>/`).
/// 3. Three-way diff against cache + reference snapshot + live.
/// 4. If there are user-relevant changes, write a Migration document.
/// 5. Return a `SyncReport` summarizing the outcome.
///
/// `from_pk` must be the lock section captured **before** the install step.
/// Passing the post-install lock would make both the fetch version and the
/// reference dir point at the same new snapshot, yielding an empty diff and
/// a migration with `from_version == to_version` (BACK-20260415_0938).
///
/// `config` is read for `release_asset_url_template` so a user who has
/// updated their template gets the new template applied immediately.
pub fn run_content_sync(
    project_root: &Path,
    from_pk: &crate::lock::ProcessKitLockSection,
    config: &crate::config::AiboxConfig,
) -> Result<SyncReport> {
    // Bug #56 short-circuit: when the lock and config target the same
    // processkit version, the "old" templates mirror IS the just-fetched
    // cache. Running the diff in that case produces nonsensical "new
    // upstream" / `RemovedUpstreamStale` reports because the reference,
    // cache, and (possibly) older-mirror sets all point at the same
    // snapshot. Skip the diff entirely and return an empty report.
    //
    // This is content-sync-only. Install-integrity self-heal lives in a
    // separate code path (`container.rs`) and is unaffected.
    if from_pk.version == config.processkit.version {
        tracing::info!(
            "skipping content sync: already at version {}",
            config.processkit.version
        );
        return Ok(SyncReport {
            summary: DiffSummary::default(),
            migration_document_path: None,
        });
    }

    // Fetch the version that config (and the just-updated lock) targets.
    // The cache is already populated by install_content_source; this fetch
    // is idempotent (returns the cached entry without a network round-trip).
    let fetched = crate::content_source::fetch(
        &from_pk.source,
        &config.processkit.version,
        from_pk.branch.as_deref(),
        &from_pk.src_path,
        config.processkit.release_asset_url_template.as_deref(),
    )
    .with_context(|| "failed to fetch content-source cache".to_string())?;

    // Reference dir is the OLD on-disk snapshot so the diff sees real changes.
    let templates_dir = templates_dir_for_version(project_root, &from_pk.version);

    let (diffs, _groups) = three_way_diff(project_root, &fetched.src_path, &templates_dir)?;
    let summary = DiffSummary::from_diffs(&diffs);

    let migration_document_path = if summary.has_user_relevant_changes() {
        write_migration_document(
            project_root,
            from_pk,
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
    fn classify_live_already_at_new_upstream_is_unchanged() {
        // BACK-TrueRaven: stale-lock case. cache != reference (upstream
        // moved), but live == cache (the project's content/ tree was
        // committed at the new upstream version even though aibox.lock
        // still records the old version). There is nothing to apply for
        // this file; the previous behaviour was to misclassify it as
        // Conflict, which crashed every fresh checkout's migration plan.
        assert_eq!(
            classify(Some("ref"), Some("new"), Some("new")),
            FileClassification::Unchanged
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
        let conflict_live = project.join("context/skills/event-log/templates/entry.yaml");
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
            .map(|d| (d.cache_rel_path.as_str(), d.classification.clone()))
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

    #[test]
    fn summary_has_upstream_side_changes_excludes_conflict_and_locally_only() {
        // Conflict alone is NOT an upstream-side change — upstream didn't
        // move; only the local copy diverged.
        let conflict_only = DiffSummary {
            conflict: 3,
            ..Default::default()
        };
        assert!(!conflict_only.has_upstream_side_changes());

        let locally_only = DiffSummary {
            changed_locally_only: 10,
            ..Default::default()
        };
        assert!(!locally_only.has_upstream_side_changes());

        // Any upstream-side count triggers true.
        assert!(
            DiffSummary {
                changed_upstream_only: 1,
                ..Default::default()
            }
            .has_upstream_side_changes()
        );
        assert!(
            DiffSummary {
                new_upstream: 1,
                ..Default::default()
            }
            .has_upstream_side_changes()
        );
        assert!(
            DiffSummary {
                removed_upstream: 1,
                ..Default::default()
            }
            .has_upstream_side_changes()
        );
    }

    // -- write_migration_document ------------------------------------------

    #[allow(deprecated)]
    fn sample_lock() -> crate::lock::ProcessKitLockSection {
        crate::lock::ProcessKitLockSection {
            source: "https://github.com/example/processkit.git".to_string(),
            version: "v1.0.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: Some("dead".to_string()),
            release_asset_sha256: None,
            installed_at: "2026-04-06T00:00:00Z".to_string(),
            processkit_install_hash: None,
            mcp_config_hash: None,
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
    fn write_migration_document_skips_when_same_version_and_only_conflicts() {
        // Regression guard for the walker false-positive: when the project
        // syncs against the same version it is already on, AGENTS.md (and
        // any other locally-owned file) will classify as Conflict against
        // the unchanged upstream. That is NOT a migration.
        let tmp = TempDir::new().unwrap();
        let lock = sample_lock(); // version: v1.0.0
        let diffs = vec![FileDiff {
            cache_rel_path: "AGENTS.md".to_string(),
            project_path: Some(PathBuf::from("AGENTS.md")),
            group: Some("AGENTS".to_string()),
            classification: FileClassification::Conflict,
        }];
        let summary = DiffSummary::from_diffs(&diffs);

        // from_version == cache_version ("v1.0.0")
        let written =
            write_migration_document(tmp.path(), &lock, "v1.0.0", Some("dead"), &summary, &diffs)
                .unwrap();

        assert!(
            written.is_none(),
            "no migration doc should be written when from == to and every change is a conflict"
        );
        assert!(
            !tmp.path().join("context/migrations/pending").exists()
                || tmp
                    .path()
                    .join("context/migrations/pending")
                    .read_dir()
                    .map(|mut r| r.next().is_none())
                    .unwrap_or(true),
            "no file should land in pending/"
        );
    }

    #[test]
    fn write_migration_document_still_writes_when_same_version_but_upstream_moved() {
        // If upstream truly has deltas (a new file, removed file, or
        // modified-upstream-only entry), we still want the migration doc
        // — even at the same from/to version string.
        let tmp = TempDir::new().unwrap();
        let lock = sample_lock();
        let diffs = vec![FileDiff {
            cache_rel_path: "skills/new-skill/SKILL.md".to_string(),
            project_path: Some(PathBuf::from("context/skills/new-skill/SKILL.md")),
            group: Some("skills/new-skill".to_string()),
            classification: FileClassification::NewUpstream,
        }];
        let summary = DiffSummary::from_diffs(&diffs);

        let written =
            write_migration_document(tmp.path(), &lock, "v1.0.0", Some("dead"), &summary, &diffs)
                .unwrap();

        assert!(
            written.is_some(),
            "upstream-side change must still produce a migration doc"
        );
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
            write_migration_document(tmp.path(), &lock, "v1.0.1", None, &summary, &diffs).unwrap();
        assert!(first.is_some());

        let second =
            write_migration_document(tmp.path(), &lock, "v1.0.1", None, &summary, &diffs).unwrap();
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
            write_migration_document(tmp.path(), &lock, "v1.0.1", None, &summary, &diffs).unwrap();
        assert!(out.is_none(), "should be no-op due to in-progress match");
    }

    // -- Regression: upgrade scenario records correct from_version ----------
    //
    // Simulates the BACK-20260415_0938 bug: sync vA → vB must record
    // from_version: vA (not vB) and must detect non-zero changed files
    // when vA and vB have different content.

    /// Build a vA cache with one content file, install it into project,
    /// snapshot it as the vA reference, then build a vB cache where that
    /// file has changed.  Run the three-way diff (vB cache vs vA reference)
    /// and write the migration, asserting:
    ///   - from_version is the vA tag, not the vB tag.
    ///   - At least one file is classified as ChangedUpstreamOnly.
    #[test]
    fn upgrade_migration_records_from_version_correctly() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();

        // Build vA cache: one schema file.
        let cache_va = tmp.path().join("cache-va/src");
        fs::create_dir_all(cache_va.join("primitives/schemas")).unwrap();
        fs::write(
            cache_va.join("primitives/schemas/workitem.yaml"),
            "name: workitem-va\n",
        )
        .unwrap();

        // Install vA and snapshot as the reference baseline.
        install_files_from_cache(&cache_va, &project).unwrap();
        copy_templates_from_cache(&cache_va, &project, "va").unwrap();
        let reference_dir = templates_dir_for_version(&project, "va");

        // Build vB cache: the same schema file with changed content.
        let cache_vb = tmp.path().join("cache-vb/src");
        fs::create_dir_all(cache_vb.join("primitives/schemas")).unwrap();
        fs::write(
            cache_vb.join("primitives/schemas/workitem.yaml"),
            "name: workitem-vb\n",
        )
        .unwrap();

        // Diff vB cache against vA reference (as run_content_sync does after fix).
        let (diffs, _) = three_way_diff(&project, &cache_vb, &reference_dir).unwrap();
        let summary = DiffSummary::from_diffs(&diffs);

        assert!(
            summary.changed_upstream_only > 0,
            "expected at least one changed-upstream-only file when vA != vB content; got {:?}",
            summary,
        );

        // Write the migration using vA as from_pk (the pre-install lock).
        #[allow(deprecated)]
        let from_lock = crate::lock::ProcessKitLockSection {
            source: "https://github.com/example/processkit.git".to_string(),
            version: "va".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: None,
            release_asset_sha256: None,
            installed_at: "2026-04-15T00:00:00Z".to_string(),
            processkit_install_hash: None,
            mcp_config_hash: None,
        };
        let written = write_migration_document(&project, &from_lock, "vb", None, &summary, &diffs)
            .unwrap()
            .expect("expected a migration document to be written");

        let body = fs::read_to_string(&written).unwrap();
        assert!(
            body.contains("from_version: va"),
            "migration must record from_version: va but got:\n{}",
            body
        );
        assert!(
            body.contains("to_version: vb"),
            "migration must record to_version: vb but got:\n{}",
            body
        );
    }

    // -- Regression: Bug #57 — extended-locally must NOT reclassify ---------
    //
    // The reclassification pass that converts ChangedLocallyOnly →
    // RemovedUpstreamStale used to fire whenever an older mirror still had
    // the file, regardless of whether the live content matched the older
    // mirror or had been extended on top of it. That silently proposed
    // deletion of the user's added content. The fix only converts when the
    // live SHA equals the older-mirror SHA (i.e. the live copy is true
    // upstream cruft that the user never customised).

    /// Negative test: live extends the older-mirror baseline → must stay
    /// `ChangedLocallyOnly`. Without the fix this would flip to
    /// `RemovedUpstreamStale` and the migration doc would propose deletion
    /// of the user's added content.
    #[test]
    fn three_way_diff_extended_locally_stays_changed_locally_only() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();

        // The "new" cache + matching template_old: file present, same
        // bytes both sides. Reference == cache satisfies the precondition
        // for ChangedLocallyOnly.
        let cache_src = tmp.path().join("cache/src");
        fs::create_dir_all(cache_src.join("skills/processkit/foo")).unwrap();
        fs::write(
            cache_src.join("skills/processkit/foo/SKILL.md"),
            "A\nB\nC\n",
        )
        .unwrap();
        install_files_from_cache(&cache_src, &project).unwrap();
        copy_templates_from_cache(&cache_src, &project, "v1.0.0").unwrap();
        let templates = templates_dir_for_version(&project, "v1.0.0");

        // Older mirror at v0.9.0 contains the same file (so the
        // reclassification candidate exists).
        let older = project.join("context/templates/processkit/v0.9.0/skills/processkit/foo");
        fs::create_dir_all(&older).unwrap();
        fs::write(older.join("SKILL.md"), "A\nB\nC\n").unwrap();

        // User has extended the live file with their own content.
        let live = project.join("context/skills/processkit/foo/SKILL.md");
        fs::write(&live, "A\nB\nC\nLOCAL_EXTENSION\n").unwrap();

        let (diffs, _) = three_way_diff(&project, &cache_src, &templates).unwrap();
        let cls = diffs
            .iter()
            .find(|d| d.cache_rel_path == "skills/processkit/foo/SKILL.md")
            .map(|d| d.classification.clone())
            .expect("SKILL.md should appear in diff");

        assert_eq!(
            cls,
            FileClassification::ChangedLocallyOnly,
            "user-extended file must NOT be reclassified to RemovedUpstreamStale; got {:?}",
            cls
        );
    }

    /// Regression: BACK-TrueRaven — when the live tree has been brought
    /// to the new upstream content but `aibox.lock` still records the old
    /// version, the three-way diff would report every modified file as a
    /// Conflict because reference (templates mirror at lock.version) ≠
    /// live and reference ≠ cache. The fix in `classify` checks
    /// `live == cache` and reports `Unchanged` for those files.
    ///
    /// Reproduces the v0.22.0 → v0.23.0 case captured in
    /// MIG-20260426T155754 where 50 of 50 "conflicts" were actually
    /// stale-lock false-positives.
    #[test]
    fn three_way_diff_live_already_at_new_cache_is_unchanged_not_conflict() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();

        // Cache holds the NEW upstream content.
        let cache_src = tmp.path().join("cache/src");
        fs::create_dir_all(cache_src.join("skills/event-log")).unwrap();
        fs::write(cache_src.join("skills/event-log/SKILL.md"), "# v0.23.0\n").unwrap();

        // Live tree already matches the new cache (fresh checkout where
        // context/ shipped at v0.23.0).
        install_files_from_cache(&cache_src, &project).unwrap();

        // Reference (templates mirror) is the OLD v0.22.0 snapshot — its
        // SKILL.md content differs from cache.
        let templates = templates_dir_for_version(&project, "v0.22.0");
        let templates_skill = templates.join("skills/event-log/SKILL.md");
        fs::create_dir_all(templates_skill.parent().unwrap()).unwrap();
        fs::write(&templates_skill, "# v0.22.0\n").unwrap();

        let (diffs, _) = three_way_diff(&project, &cache_src, &templates).unwrap();
        let cls = diffs
            .iter()
            .find(|d| d.cache_rel_path == "skills/event-log/SKILL.md")
            .map(|d| d.classification.clone())
            .expect("SKILL.md should appear in diff");

        assert_eq!(
            cls,
            FileClassification::Unchanged,
            "live==cache (stale-lock case) must be Unchanged, not Conflict"
        );
    }

    /// Positive paired test: live is byte-identical to the older-mirror
    /// baseline → must remain `RemovedUpstreamStale`. Proves the fix does
    /// not regress the original kept-old-stale behaviour.
    #[test]
    fn three_way_diff_byte_identical_to_old_mirror_still_stale() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();

        // Cache + template_old hold the same NEW content for the file.
        // (We need cache == reference to land in ChangedLocallyOnly.)
        let cache_src = tmp.path().join("cache/src");
        fs::create_dir_all(cache_src.join("skills/processkit/foo")).unwrap();
        fs::write(
            cache_src.join("skills/processkit/foo/SKILL.md"),
            "NEW\nUPSTREAM\n",
        )
        .unwrap();
        install_files_from_cache(&cache_src, &project).unwrap();
        copy_templates_from_cache(&cache_src, &project, "v1.0.0").unwrap();
        let templates = templates_dir_for_version(&project, "v1.0.0");

        // Older mirror has the OLD bytes — `"A\nB\nC\n"`.
        let older = project.join("context/templates/processkit/v0.9.0/skills/processkit/foo");
        fs::create_dir_all(&older).unwrap();
        fs::write(older.join("SKILL.md"), "A\nB\nC\n").unwrap();

        // Overwrite the live file with the OLD bytes — byte-identical to
        // the older mirror, so the file IS upstream cruft from a skipped
        // version.
        let live = project.join("context/skills/processkit/foo/SKILL.md");
        fs::write(&live, "A\nB\nC\n").unwrap();

        let (diffs, _) = three_way_diff(&project, &cache_src, &templates).unwrap();
        let cls = diffs
            .iter()
            .find(|d| d.cache_rel_path == "skills/processkit/foo/SKILL.md")
            .map(|d| d.classification.clone())
            .expect("SKILL.md should appear in diff");

        match &cls {
            FileClassification::RemovedUpstreamStale { last_seen_in } => {
                assert_eq!(last_seen_in, "v0.9.0");
            }
            other => panic!(
                "byte-identical-to-old-mirror file should be RemovedUpstreamStale; got {:?}",
                other
            ),
        }
    }

    // -- Regression: Bug #56 — same-version sync writes no migration -------

    /// Bug #56: when `from_pk.version == config.processkit.version` the
    /// "old" templates mirror is the just-fetched cache, so the diff sees
    /// identical trees and produces noisy/incorrect entries. The fix
    /// short-circuits `run_content_sync` before touching the cache or the
    /// diff — the returned `SyncReport` is empty and no migration document
    /// is written.
    #[test]
    fn run_content_sync_short_circuits_when_versions_match() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        fs::create_dir_all(&project).unwrap();

        // Build a cache and snapshot as the v0.21.0 reference. Install the
        // files into the live tree, then extend one of them so the diff
        // (if it ran) would surface a ChangedLocallyOnly entry.
        let cache_src = tmp.path().join("cache/src");
        fs::create_dir_all(cache_src.join("primitives/schemas")).unwrap();
        fs::create_dir_all(cache_src.join("skills/event-log")).unwrap();
        fs::write(
            cache_src.join("primitives/schemas/workitem.yaml"),
            "name: workitem\n",
        )
        .unwrap();
        fs::write(cache_src.join("skills/event-log/SKILL.md"), "# v1\n").unwrap();
        install_files_from_cache(&cache_src, &project).unwrap();
        copy_templates_from_cache(&cache_src, &project, "v0.21.0").unwrap();

        // User extends the live SKILL.md.
        let live_skill = project.join("context/skills/event-log/SKILL.md");
        fs::write(&live_skill, "# v1\nLOCAL_EXTENSION\n").unwrap();

        // from_pk and config both at v0.21.0.
        #[allow(deprecated)]
        let from_pk = crate::lock::ProcessKitLockSection {
            source: "https://github.com/example/processkit.git".to_string(),
            version: "v0.21.0".to_string(),
            src_path: "src".to_string(),
            branch: None,
            resolved_commit: None,
            release_asset_sha256: None,
            installed_at: "2026-04-26T00:00:00Z".to_string(),
            processkit_install_hash: None,
            mcp_config_hash: None,
        };
        let mut config = crate::config::test_config();
        config.processkit.version = "v0.21.0".to_string();
        config.processkit.source = from_pk.source.clone();

        let report = run_content_sync(&project, &from_pk, &config)
            .expect("same-version sync should succeed");

        assert!(
            report.migration_document_path.is_none(),
            "no migration document should be written for same-version sync"
        );
        assert_eq!(report.summary.removed_upstream_stale, 0);
        assert_eq!(report.summary.new_upstream, 0);
        assert_eq!(report.summary.removed_upstream, 0);
        assert_eq!(report.summary.changed_upstream_only, 0);
        assert_eq!(report.summary.conflict, 0);
        assert_eq!(report.summary.changed_locally_only, 0);

        // Belt-and-suspenders: pending/ should be empty (or absent).
        let pending = project.join("context/migrations/pending");
        if pending.is_dir() {
            let count = fs::read_dir(&pending).unwrap().count();
            assert_eq!(
                count, 0,
                "no migration files should land in pending/ for same-version sync"
            );
        }
    }
}
