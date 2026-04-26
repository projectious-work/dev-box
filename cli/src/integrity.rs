//! Install-integrity check + self-heal decision logic for `aibox sync`.
//!
//! WS-1 introduced this module so `aibox sync` can detect a "broken"
//! processkit install (skills missing, mirror missing, version drift)
//! and trigger a reinstall instead of silently doing nothing.
//!
//! ## Pieces
//!
//! - [`IntegrityStatus`] — variant of "what's wrong with the live
//!   install" emitted by [`verify_install_integrity`]. The
//!   `cmd_doctor_integrity` and `decide_sync` callers use it to decide
//!   what to print and whether to reinstall.
//! - [`SyncDecision`] — `Skip` / `Install` / `Reinstall` decision used
//!   by `cmd_sync` to dispatch the install path.
//! - [`LiveProvenance`] — schema for `context/.processkit-provenance.toml`,
//!   the live install marker. Sibling to `aibox.lock`; written by
//!   [`crate::content_init::install_content_source`] immediately before
//!   the lock bump so a corrupted/missing marker is the only state where
//!   "lock says installed but nothing was installed" can occur (and the
//!   integrity check catches that).
//! - [`cmd_doctor_integrity`] — `aibox doctor --integrity` entry point.
//!
//! ## Invariants (do not break)
//!
//! - The provenance file is **written before the lock**, mirroring the
//!   existing `lock-write-last` invariant. A failure between the two
//!   writes leaves the lock untouched, so the next sync sees the prior
//!   state.
//! - The provenance schema is `schema_version = 1`. Readers that see a
//!   different `schema_version` return `Ok(None)` (treated as
//!   `MissingProvenance`); the next sync rewrites the file. Never error.
//! - `verify_install_integrity` is **read-only**. It performs at most
//!   two TOML parses + one bounded directory walk over
//!   `context/skills/processkit/`. Never fetches, never writes.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{AiboxConfig, PROCESSKIT_VERSION_LATEST, PROCESSKIT_VERSION_UNSET};
use crate::lock::AiboxLock;
use crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR;

/// Live install marker filename (relative to `<project_root>/context/`).
pub const LIVE_PROVENANCE_FILENAME: &str = ".processkit-provenance.toml";

/// Current schema version for [`LiveProvenance`].
pub const LIVE_PROVENANCE_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// IntegrityStatus
// ---------------------------------------------------------------------------

/// Outcome of [`verify_install_integrity`]. The `decide_sync` caller
/// distinguishes `Healthy` / `NotInstalled` (no reinstall) from every
/// other variant (reinstall).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityStatus {
    /// Live install matches the lock, the templates mirror, and the
    /// live provenance marker.
    Healthy,
    /// `aibox.lock` claims one version but the mirror's PROVENANCE.toml
    /// (or live `.processkit-provenance.toml`) reports another.
    MismatchedVersion { claimed: String, observed: String },
    /// The lock says version X is installed but
    /// `context/templates/processkit/X/` does not exist.
    MissingTemplateMirror { version: String },
    /// The mirror is present but `context/.processkit-provenance.toml`
    /// is missing or its schema version is unknown.
    MissingProvenance { version: String },
    /// Counts/hashes from the live tree do not match what the live
    /// provenance file recorded at install time.
    Stale {
        version: String,
        reason: String,
        observed_hash: Option<String>,
        expected_hash: Option<String>,
    },
    /// No `aibox.lock` (or the lock has no `[processkit]`, or the
    /// version is the `unset` sentinel). This is intentional state, not
    /// an error.
    NotInstalled,
}

impl IntegrityStatus {
    /// True when the install is in known-good state.
    #[allow(dead_code)] // public surface — used by callers outside cmd_doctor_integrity / decide_sync
    pub fn is_healthy(&self) -> bool {
        matches!(self, IntegrityStatus::Healthy)
    }

    /// True when `decide_sync` should treat this as
    /// `SyncDecision::Reinstall`.
    #[allow(dead_code)] // public surface — covers future call sites (kit, status)
    pub fn needs_reinstall(&self) -> bool {
        !matches!(
            self,
            IntegrityStatus::Healthy | IntegrityStatus::NotInstalled
        )
    }

    /// Stable, machine-readable variant tag — used by the
    /// `--integrity --json` output and downstream tests.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::MismatchedVersion { .. } => "MismatchedVersion",
            Self::MissingTemplateMirror { .. } => "MissingTemplateMirror",
            Self::MissingProvenance { .. } => "MissingProvenance",
            Self::Stale { .. } => "Stale",
            Self::NotInstalled => "NotInstalled",
        }
    }
}

impl std::fmt::Display for IntegrityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::NotInstalled => write!(f, "NotInstalled"),
            Self::MismatchedVersion { claimed, observed } => write!(
                f,
                "MismatchedVersion (claimed {}, observed {})",
                claimed, observed
            ),
            Self::MissingTemplateMirror { version } => {
                write!(f, "MissingTemplateMirror (claimed {})", version)
            }
            Self::MissingProvenance { version } => {
                write!(f, "MissingProvenance (claimed {})", version)
            }
            Self::Stale {
                version, reason, ..
            } => write!(f, "Stale (version {}, reason {})", version, reason),
        }
    }
}

// ---------------------------------------------------------------------------
// SyncDecision
// ---------------------------------------------------------------------------

/// Result of [`decide_sync`]. `cmd_sync` dispatches on this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncDecision {
    /// Don't run the install path. Either the user opted out of pinning
    /// (`unset` / `latest`), or the lock matches the config and the live
    /// tree integrity-checks healthy.
    Skip,
    /// Lock missing or out of sync with the config; install fresh.
    Install { reason: String },
    /// Lock matches the config but the live tree fails the integrity
    /// check; reinstall to self-heal.
    Reinstall {
        reason: String,
        prior_state: IntegrityStatus,
    },
}

// ---------------------------------------------------------------------------
// LiveProvenance schema
// ---------------------------------------------------------------------------

/// `<project_root>/context/.processkit-provenance.toml` contents.
///
/// Written by `install_content_source` immediately before the lock
/// bump. Records the manifest counts so a future sync can detect
/// "skills/schemas/etc. went missing on disk".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveProvenance {
    pub schema_version: u32,
    pub install: LiveProvenanceInstall,
    pub manifest: LiveProvenanceManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveProvenanceInstall {
    pub processkit_version: String,
    pub processkit_source: String,
    pub installed_at: String,
    pub cli_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveProvenanceManifest {
    /// Count of `context/skills/processkit/<skill>/SKILL.md` files at
    /// install time. **Processkit-only**: aibox does not track skills
    /// under other categories (`<other-category>/` is user-authored and
    /// outside the integrity contract). The verify-time live count
    /// (see `count_live_processkit_skills`) is taken from the same
    /// `processkit/` subtree, so the two are directly comparable.
    pub skill_count: u32,
    /// Count of all files under `context/schemas/` (entire dir is
    /// processkit-owned).
    pub schema_count: u32,
    /// Count of all files under `context/processes/` (entire dir is
    /// processkit-owned).
    pub process_count: u32,
    /// Count of all files under `context/state-machines/` (entire dir is
    /// processkit-owned).
    pub state_machine_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_asset_sha256: Option<String>,
    /// Mirrors `processkit_install_hash` from `aibox.lock` —
    /// `compute_processkit_install_fingerprint` (WS-7) over the broad
    /// processkit-shipped install payload (skill source, schemas,
    /// processes, state-machines, `_lib`) at install time.
    /// `None` when no processkit content was installed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_hash: Option<String>,
}

// ---------------------------------------------------------------------------
// Mirror counts (for content_init)
// ---------------------------------------------------------------------------

/// File-tree counts read from a templates mirror. Used at install time
/// to populate [`LiveProvenanceManifest`], and at verify time as the
/// "expected" baseline against which the live tree is compared.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MirrorCounts {
    pub skills: u32,
    pub schemas: u32,
    pub processes: u32,
    pub state_machines: u32,
}

/// Walk the templates mirror at `context/templates/processkit/<version>/`
/// and count manifest entries by category.
///
/// - **skills**: number of `context/skills/processkit/<skill>/SKILL.md`
///   files **only**. aibox only owns the `processkit` category;
///   non-processkit skills under `context/skills/<other-category>/` are
///   user-authored and not part of the integrity contract. Counting them
///   here would mismatch the live count produced by
///   [`count_live_processkit_skills`], which is also processkit-only.
/// - **schemas / state_machines**: number of files under
///   `context/schemas/` / `context/state-machines/` (any type — these
///   directories are entirely processkit-owned).
/// - **processes**: number of files under `context/processes/` (also
///   entirely processkit-owned).
///
/// Used only by `install_content_source`; kept `pub(crate)` so it isn't
/// part of the public surface.
pub(crate) fn count_from_mirror(project_root: &Path, version: &str) -> Result<MirrorCounts> {
    let mirror_dir = project_root.join(TEMPLATES_PROCESSKIT_DIR).join(version);
    if !mirror_dir.is_dir() {
        anyhow::bail!(
            "templates mirror not found at {} (was install_content_source called?)",
            mirror_dir.display()
        );
    }

    let mut counts = MirrorCounts::default();

    // Skills under context/skills/processkit/<skill>/SKILL.md only.
    // The integrity contract is symmetric: live counts are taken from
    // the same `processkit` subset (see count_live_processkit_skills).
    let pk_skills_root = mirror_dir.join("context").join("skills").join("processkit");
    if let Ok(skills) = fs::read_dir(&pk_skills_root) {
        for skill in skills.flatten() {
            let skill_path = skill.path();
            if !skill_path.is_dir() {
                continue;
            }
            if skill_path.join("SKILL.md").is_file() {
                counts.skills += 1;
            }
        }
    }

    counts.schemas = count_files_in(&mirror_dir.join("context").join("schemas"));
    counts.processes = count_files_in(&mirror_dir.join("context").join("processes"));
    counts.state_machines = count_files_in(&mirror_dir.join("context").join("state-machines"));

    Ok(counts)
}

/// Count regular files (non-recursive) under a directory. Returns 0 if
/// the directory is missing.
fn count_files_in(dir: &Path) -> u32 {
    let mut n = 0u32;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                n += 1;
            }
        }
    }
    n
}

/// Count live skills under `context/skills/<category>/<skill>/SKILL.md`.
/// Used by `verify_install_integrity` as the cheap surrogate for
/// "everything still on disk".
fn count_live_processkit_skills(project_root: &Path) -> u32 {
    let mut n = 0u32;
    let pk_skills = project_root
        .join("context")
        .join("skills")
        .join("processkit");
    if let Ok(skills) = fs::read_dir(&pk_skills) {
        for skill in skills.flatten() {
            let skill_path = skill.path();
            if !skill_path.is_dir() {
                continue;
            }
            if skill_path.join("SKILL.md").is_file() {
                n += 1;
            }
        }
    }
    n
}

// ---------------------------------------------------------------------------
// LiveProvenance read / write
// ---------------------------------------------------------------------------

/// `<project_root>/context/.processkit-provenance.toml`.
pub fn live_provenance_path(project_root: &Path) -> PathBuf {
    project_root.join("context").join(LIVE_PROVENANCE_FILENAME)
}

/// Read the live provenance marker.
///
/// Returns `Ok(None)` when:
/// - the file does not exist, or
/// - the file's `schema_version` is not the current
///   [`LIVE_PROVENANCE_SCHEMA_VERSION`] (forward/back-compat: a future
///   schema bump should not cause `aibox sync` to error).
///
/// Returns `Err` only on I/O errors and TOML parse errors that aren't a
/// known-future schema. The next sync rewrites the file with the
/// current schema, so an unparseable file is recoverable.
pub fn read_live_provenance(project_root: &Path) -> Result<Option<LiveProvenance>> {
    let path = live_provenance_path(project_root);
    if !path.exists() {
        return Ok(None);
    }
    let body =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    // Peek at the schema_version first so a future bump doesn't trigger
    // a full-blown parse error against the v1 struct.
    #[derive(Deserialize)]
    struct SchemaPeek {
        #[serde(default)]
        schema_version: u32,
    }
    let peek: SchemaPeek = match toml::from_str(&body) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    if peek.schema_version != LIVE_PROVENANCE_SCHEMA_VERSION {
        // Treat unknown/future schema as MissingProvenance so the next
        // sync rewrites it with our current schema.
        return Ok(None);
    }

    let parsed: LiveProvenance = toml::from_str(&body).with_context(|| {
        format!(
            "failed to parse {} as LiveProvenance schema v{}",
            path.display(),
            LIVE_PROVENANCE_SCHEMA_VERSION
        )
    })?;
    Ok(Some(parsed))
}

/// Write the live provenance marker. Creates `context/` if needed.
pub fn write_live_provenance(project_root: &Path, lp: &LiveProvenance) -> Result<()> {
    let path = live_provenance_path(project_root);
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut body = String::new();
    body.push_str("# Aibox-managed live install marker. DO NOT EDIT.\n");
    body.push_str("# Written by `aibox sync` immediately before aibox.lock is bumped.\n");
    let serialized = toml::to_string_pretty(lp)
        .with_context(|| "failed to serialize LiveProvenance to TOML".to_string())?;
    body.push_str(&serialized);
    fs::write(&path, body).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// verify_install_integrity
// ---------------------------------------------------------------------------

/// Mirror PROVENANCE.toml shape — `[source].generated_for_tag` is the
/// only field we read here.
#[derive(Debug, Deserialize)]
struct MirrorProvenance {
    source: MirrorProvenanceSource,
}

#[derive(Debug, Deserialize)]
struct MirrorProvenanceSource {
    generated_for_tag: String,
}

/// Verify the integrity of the currently-installed processkit.
///
/// See module docs for the algorithm and the 5-or-6 short-circuit
/// branches; the function is intentionally a flat ladder so each branch
/// is easy to read against the spec.
pub fn verify_install_integrity(
    project_root: &Path,
    lock: &Option<AiboxLock>,
) -> Result<IntegrityStatus> {
    let pk = match lock.as_ref().and_then(|l| l.processkit.as_ref()) {
        Some(p) => p,
        None => return Ok(IntegrityStatus::NotInstalled),
    };
    if pk.version == PROCESSKIT_VERSION_UNSET {
        return Ok(IntegrityStatus::NotInstalled);
    }
    let claimed = pk.version.clone();

    // 1. Mirror dir + PROVENANCE.toml exist?
    let mirror_dir = project_root.join(TEMPLATES_PROCESSKIT_DIR).join(&claimed);
    let mirror_prov_path = mirror_dir.join("PROVENANCE.toml");
    if !mirror_prov_path.is_file() {
        return Ok(IntegrityStatus::MissingTemplateMirror { version: claimed });
    }

    // 2. Mirror PROVENANCE.toml is parseable? If its
    //    `[source].generated_for_tag` disagrees with the lock, log a
    //    warning but DO NOT fail — the mirror PROVENANCE is upstream-
    //    controlled (written by processkit's own release tooling) and
    //    aibox cannot guarantee the upstream stamps it correctly. For
    //    example, processkit v0.22.0 shipped its mirror with
    //    `generated_for_tag = "v0.21.0"` (an upstream stamping bug).
    //    The authoritative source for installed-version assertions is
    //    `aibox.lock` paired with the *live*
    //    `context/.processkit-provenance.toml` (both aibox-written —
    //    see step 4 below). The mirror PROVENANCE is used here only to
    //    detect the *presence* of the mirror, not as a source of truth.
    let mirror_prov_body = fs::read_to_string(&mirror_prov_path).with_context(|| {
        format!(
            "failed to read mirror provenance {}",
            mirror_prov_path.display()
        )
    })?;
    let mirror_prov: MirrorProvenance = match toml::from_str(&mirror_prov_body) {
        Ok(p) => p,
        Err(_) => {
            // Treat an unparseable mirror PROVENANCE as a stale mirror;
            // the sync will overwrite it.
            return Ok(IntegrityStatus::MissingTemplateMirror { version: claimed });
        }
    };
    if mirror_prov.source.generated_for_tag != claimed {
        // Downgraded from a hard `MismatchedVersion` error to a warn:
        // the live provenance check below is authoritative.
        tracing::warn!(
            mirror_path = %mirror_prov_path.display(),
            claimed = %claimed,
            mirror_generated_for_tag = %mirror_prov.source.generated_for_tag,
            "mirror PROVENANCE.toml generated_for_tag disagrees with aibox.lock; \
             this is upstream-controlled and not authoritative — continuing with \
             live `context/.processkit-provenance.toml` as source of truth"
        );
    }

    // 3. Live provenance marker exists?
    let live_prov_path = live_provenance_path(project_root);
    if !live_prov_path.exists() {
        return Ok(IntegrityStatus::MissingProvenance { version: claimed });
    }
    let live = match read_live_provenance(project_root)? {
        Some(lp) => lp,
        None => return Ok(IntegrityStatus::MissingProvenance { version: claimed }),
    };

    // 4. Live provenance agrees on the version?
    if live.install.processkit_version != claimed {
        return Ok(IntegrityStatus::MismatchedVersion {
            claimed,
            observed: live.install.processkit_version,
        });
    }

    // 5. Skill-count tripwire (cheap surrogate). Compares the live
    //    `context/skills/processkit/*/SKILL.md` count against the
    //    `manifest.skill_count` recorded at install time. WS-7 will
    //    replace this with a real per-file hash; until then we allow
    //    one slack to soak up the occasional intentional remove.
    let observed_skills = count_live_processkit_skills(project_root);
    if observed_skills + 1 < live.manifest.skill_count {
        return Ok(IntegrityStatus::Stale {
            version: claimed,
            reason: "skill_count_below_mirror".to_string(),
            observed_hash: None,
            expected_hash: None,
        });
    }

    // 6. install_hash check (mirrors processkit_install_hash). Optional
    //    — if the install didn't write one (e.g. no processkit content
    //    on disk), skip it.
    if let Some(expected) = &live.manifest.install_hash {
        let observed =
            crate::mcp_registration::compute_processkit_install_fingerprint(project_root);
        if observed.as_ref() != Some(expected) {
            return Ok(IntegrityStatus::Stale {
                version: claimed,
                reason: "install_hash_mismatch".to_string(),
                observed_hash: observed,
                expected_hash: Some(expected.clone()),
            });
        }
    }

    Ok(IntegrityStatus::Healthy)
}

// ---------------------------------------------------------------------------
// decide_sync
// ---------------------------------------------------------------------------

/// Pure version-comparison helper. Returns true when the lock disagrees
/// with the config on `(source, version)` (or no lock yet).
///
/// Kept as a private helper so `decide_sync` can short-circuit on the
/// version mismatch case before paying for the integrity check.
fn lock_disagrees(
    config_version: &str,
    config_source: &str,
    lock_pair: Option<(&str, &str)>,
) -> bool {
    if config_version == PROCESSKIT_VERSION_UNSET || config_version == PROCESSKIT_VERSION_LATEST {
        return false;
    }
    match lock_pair {
        None => true,
        Some((src, ver)) => src != config_source || ver != config_version,
    }
}

/// Decide whether `cmd_sync` should install / reinstall / skip.
///
/// 1. If the config version is `unset`/`latest`, always Skip.
/// 2. If the lock is missing or out-of-sync with the config, Install.
/// 3. Otherwise, run [`verify_install_integrity`] and Reinstall on any
///    non-Healthy / non-NotInstalled outcome. Healthy and NotInstalled
///    both Skip.
pub fn decide_sync(
    config: &AiboxConfig,
    project_root: &Path,
    lock: &Option<AiboxLock>,
) -> Result<SyncDecision> {
    if config.processkit.version == PROCESSKIT_VERSION_UNSET
        || config.processkit.version == PROCESSKIT_VERSION_LATEST
    {
        return Ok(SyncDecision::Skip);
    }

    let pair = lock
        .as_ref()
        .and_then(|l| l.processkit.as_ref())
        .map(|p| (p.source.clone(), p.version.clone()));
    let pair_ref = pair.as_ref().map(|(s, v)| (s.as_str(), v.as_str()));

    if lock_disagrees(
        &config.processkit.version,
        &config.processkit.source,
        pair_ref,
    ) {
        let reason = match pair_ref {
            None => "no aibox.lock".to_string(),
            Some((s, v)) => format!(
                "lock {}@{} != config {}@{}",
                s, v, config.processkit.source, config.processkit.version
            ),
        };
        return Ok(SyncDecision::Install { reason });
    }

    let status = verify_install_integrity(project_root, lock)?;
    match status {
        IntegrityStatus::Healthy | IntegrityStatus::NotInstalled => Ok(SyncDecision::Skip),
        other => Ok(SyncDecision::Reinstall {
            reason: format!("integrity check: {}", other),
            prior_state: other,
        }),
    }
}

// ---------------------------------------------------------------------------
// aibox doctor --integrity
// ---------------------------------------------------------------------------

/// Entry point for `aibox doctor --integrity` and `--integrity --json`.
///
/// Exit code: `0` for `Healthy` and `NotInstalled` (the latter is
/// intentional config), `1` for everything else.
pub fn cmd_doctor_integrity(project_root: &Path, json: bool) -> Result<()> {
    let lock = crate::lock::read_lock(project_root).ok().flatten();
    let status = verify_install_integrity(project_root, &lock)?;
    let claimed_version = lock
        .as_ref()
        .and_then(|l| l.processkit.as_ref())
        .map(|p| p.version.clone());

    if json {
        print_status_json(&status, claimed_version.as_deref());
    } else {
        print_status_human(&status, claimed_version.as_deref());
    }

    let exit_code = match &status {
        IntegrityStatus::Healthy | IntegrityStatus::NotInstalled => 0,
        _ => 1,
    };
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

fn print_status_human(status: &IntegrityStatus, claimed_version: Option<&str>) {
    match status {
        IntegrityStatus::Healthy => {
            let v = claimed_version.unwrap_or("?");
            crate::output::ok(&format!("processkit install integrity: Healthy ({})", v));
        }
        IntegrityStatus::NotInstalled => {
            crate::output::info(
                "processkit install integrity: NotInstalled (no aibox.lock or version=unset)",
            );
        }
        IntegrityStatus::MismatchedVersion { claimed, observed } => {
            crate::output::error(&format!(
                "processkit install integrity: MismatchedVersion\n  \
                 claimed (aibox.lock):                {}\n  \
                 observed (live provenance / mirror): {}\n  \
                 Run `aibox sync` to self-heal.",
                claimed, observed
            ));
        }
        IntegrityStatus::MissingTemplateMirror { version } => {
            crate::output::error(&format!(
                "processkit install integrity: MissingTemplateMirror\n  \
                 expected: context/templates/processkit/{}/PROVENANCE.toml\n  \
                 Run `aibox sync` to self-heal.",
                version
            ));
        }
        IntegrityStatus::MissingProvenance { version } => {
            crate::output::error(&format!(
                "processkit install integrity: MissingProvenance\n  \
                 expected: context/.processkit-provenance.toml (version {})\n  \
                 Run `aibox sync` to self-heal.",
                version
            ));
        }
        IntegrityStatus::Stale {
            version,
            reason,
            observed_hash,
            expected_hash,
        } => {
            let mut msg = format!(
                "processkit install integrity: Stale (version {}, reason {})",
                version, reason
            );
            if let Some(eh) = expected_hash {
                msg.push_str(&format!("\n  expected hash: {}", eh));
            }
            if let Some(oh) = observed_hash {
                msg.push_str(&format!("\n  observed hash: {}", oh));
            }
            msg.push_str("\n  Run `aibox sync` to self-heal.");
            crate::output::error(&msg);
        }
    }
}

fn print_status_json(status: &IntegrityStatus, claimed_version: Option<&str>) {
    let mut entries: Vec<(String, String)> =
        vec![("status".to_string(), format!("\"{}\"", status.kind()))];
    match status {
        IntegrityStatus::Healthy | IntegrityStatus::NotInstalled => {
            if let Some(v) = claimed_version {
                entries.push(("version".to_string(), format!("\"{}\"", json_escape(v))));
            }
        }
        IntegrityStatus::MismatchedVersion { claimed, observed } => {
            entries.push((
                "claimed".to_string(),
                format!("\"{}\"", json_escape(claimed)),
            ));
            entries.push((
                "observed".to_string(),
                format!("\"{}\"", json_escape(observed)),
            ));
        }
        IntegrityStatus::MissingTemplateMirror { version }
        | IntegrityStatus::MissingProvenance { version } => {
            entries.push((
                "version".to_string(),
                format!("\"{}\"", json_escape(version)),
            ));
        }
        IntegrityStatus::Stale {
            version,
            reason,
            observed_hash,
            expected_hash,
        } => {
            entries.push((
                "version".to_string(),
                format!("\"{}\"", json_escape(version)),
            ));
            entries.push(("reason".to_string(), format!("\"{}\"", json_escape(reason))));
            entries.push((
                "observed_hash".to_string(),
                match observed_hash {
                    Some(h) => format!("\"{}\"", json_escape(h)),
                    None => "null".to_string(),
                },
            ));
            entries.push((
                "expected_hash".to_string(),
                match expected_hash {
                    Some(h) => format!("\"{}\"", json_escape(h)),
                    None => "null".to_string(),
                },
            ));
        }
    }

    let body: Vec<String> = entries
        .into_iter()
        .map(|(k, v)| format!("\"{}\": {}", k, v))
        .collect();
    println!("{{{}}}", body.join(", "));
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::{AiboxLockSection, ProcessKitLockSection};
    use std::fs;
    use tempfile::TempDir;

    // ── fixtures ───────────────────────────────────────────────────────────

    #[allow(deprecated)]
    fn make_lock_with_version(version: &str) -> AiboxLock {
        AiboxLock {
            aibox: AiboxLockSection {
                cli_version: "0.19.2".to_string(),
                synced_at: "2026-04-25T10:00:00Z".to_string(),
            },
            processkit: Some(ProcessKitLockSection {
                source: "https://github.com/projectious-work/processkit.git".to_string(),
                version: version.to_string(),
                src_path: "src".to_string(),
                branch: None,
                resolved_commit: None,
                release_asset_sha256: None,
                installed_at: "2026-04-25T10:00:00Z".to_string(),
                processkit_install_hash: None,
                mcp_config_hash: None,
            }),
            addons: None,
        }
    }

    fn make_lock_unset() -> AiboxLock {
        let mut lock = make_lock_with_version(PROCESSKIT_VERSION_UNSET);
        lock.processkit.as_mut().unwrap().version = PROCESSKIT_VERSION_UNSET.to_string();
        lock
    }

    fn write_mirror_provenance(project_root: &Path, version: &str, generated_for: &str) {
        let dir = project_root.join(TEMPLATES_PROCESSKIT_DIR).join(version);
        fs::create_dir_all(&dir).unwrap();
        let body = format!(
            "[source]\n\
             project = \"processkit\"\n\
             upstream = \"https://example.com/processkit.git\"\n\
             generated_at = \"2026-04-25T10:00:00Z\"\n\
             generated_for_tag = \"{}\"\n\
             \n\
             [files]\n",
            generated_for
        );
        fs::write(dir.join("PROVENANCE.toml"), body).unwrap();
    }

    fn write_live_provenance_for(
        project_root: &Path,
        version: &str,
        skill_count: u32,
        install_hash: Option<&str>,
    ) {
        let lp = LiveProvenance {
            schema_version: LIVE_PROVENANCE_SCHEMA_VERSION,
            install: LiveProvenanceInstall {
                processkit_version: version.to_string(),
                processkit_source: "https://github.com/projectious-work/processkit.git".to_string(),
                installed_at: "2026-04-25T10:00:00Z".to_string(),
                cli_version: "0.19.2".to_string(),
            },
            manifest: LiveProvenanceManifest {
                skill_count,
                schema_count: 1,
                process_count: 1,
                state_machine_count: 1,
                release_asset_sha256: None,
                install_hash: install_hash.map(|s| s.to_string()),
            },
        };
        write_live_provenance(project_root, &lp).unwrap();
    }

    /// Materialise N processkit skills under
    /// `context/skills/processkit/<i>/SKILL.md`.
    fn materialise_live_skills(project_root: &Path, n: u32) {
        let base = project_root
            .join("context")
            .join("skills")
            .join("processkit");
        for i in 0..n {
            let dir = base.join(format!("skill-{}", i));
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), "---\nname: x\n---\n").unwrap();
        }
    }

    // ── 1. kind() tags ─────────────────────────────────────────────────────

    #[test]
    fn integrity_status_kind_tags_match_json_contract() {
        assert_eq!(IntegrityStatus::Healthy.kind(), "Healthy");
        assert_eq!(IntegrityStatus::NotInstalled.kind(), "NotInstalled");
        assert_eq!(
            IntegrityStatus::MismatchedVersion {
                claimed: "v1".into(),
                observed: "v2".into(),
            }
            .kind(),
            "MismatchedVersion"
        );
        assert_eq!(
            IntegrityStatus::MissingTemplateMirror {
                version: "v1".into(),
            }
            .kind(),
            "MissingTemplateMirror"
        );
        assert_eq!(
            IntegrityStatus::MissingProvenance {
                version: "v1".into(),
            }
            .kind(),
            "MissingProvenance"
        );
        assert_eq!(
            IntegrityStatus::Stale {
                version: "v1".into(),
                reason: "x".into(),
                observed_hash: None,
                expected_hash: None,
            }
            .kind(),
            "Stale"
        );
    }

    // ── 2. Display non-empty per variant ───────────────────────────────────

    #[test]
    fn integrity_status_display_each_variant() {
        let variants = vec![
            IntegrityStatus::Healthy,
            IntegrityStatus::NotInstalled,
            IntegrityStatus::MismatchedVersion {
                claimed: "v1".into(),
                observed: "v2".into(),
            },
            IntegrityStatus::MissingTemplateMirror {
                version: "v1".into(),
            },
            IntegrityStatus::MissingProvenance {
                version: "v1".into(),
            },
            IntegrityStatus::Stale {
                version: "v1".into(),
                reason: "skill_count".into(),
                observed_hash: Some("a".into()),
                expected_hash: Some("b".into()),
            },
        ];
        for v in variants {
            let s = format!("{}", v);
            assert!(!s.is_empty(), "Display empty for {:?}", v);
        }
    }

    // ── 3. NotInstalled when lock absent ───────────────────────────────────

    #[test]
    fn verify_returns_not_installed_when_lock_absent() {
        let tmp = TempDir::new().unwrap();
        let status = verify_install_integrity(tmp.path(), &None).unwrap();
        assert_eq!(status, IntegrityStatus::NotInstalled);
    }

    // ── 4. NotInstalled when version unset ─────────────────────────────────

    #[test]
    fn verify_returns_not_installed_when_version_unset() {
        let tmp = TempDir::new().unwrap();
        let lock = Some(make_lock_unset());
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        assert_eq!(status, IntegrityStatus::NotInstalled);
    }

    // ── 5. MissingTemplateMirror when dir absent ──────────────────────────

    #[test]
    fn verify_returns_missing_template_mirror_when_dir_absent() {
        let tmp = TempDir::new().unwrap();
        let lock = Some(make_lock_with_version("v0.19.1"));
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        assert_eq!(
            status,
            IntegrityStatus::MissingTemplateMirror {
                version: "v0.19.1".to_string()
            }
        );
    }

    // ── 6. MissingProvenance when live marker absent ───────────────────────

    #[test]
    fn verify_returns_missing_provenance_when_live_marker_absent() {
        let tmp = TempDir::new().unwrap();
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.19.1");
        let lock = Some(make_lock_with_version("v0.19.1"));
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        assert_eq!(
            status,
            IntegrityStatus::MissingProvenance {
                version: "v0.19.1".to_string()
            }
        );
    }

    // ── 7. MismatchedVersion when live disagrees ──────────────────────────

    #[test]
    fn verify_returns_mismatched_version_when_live_provenance_disagrees() {
        let tmp = TempDir::new().unwrap();
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.19.1");
        // Live provenance claims a *different* version.
        write_live_provenance_for(tmp.path(), "v0.18.0", 0, None);
        let lock = Some(make_lock_with_version("v0.19.1"));
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        assert_eq!(
            status,
            IntegrityStatus::MismatchedVersion {
                claimed: "v0.19.1".to_string(),
                observed: "v0.18.0".to_string(),
            }
        );
    }

    // ── 8. Mirror PROVENANCE disagreement is non-fatal (only the live
    //       provenance is authoritative for version assertions). ──────────

    #[test]
    fn verify_tolerates_mirror_provenance_disagreement_when_live_agrees() {
        // processkit v0.22.0 shipped its mirror with
        // `generated_for_tag = "v0.21.0"` (an upstream stamping bug).
        // aibox does not control that field — the live
        // `context/.processkit-provenance.toml` is the source of truth.
        // So as long as the lock + live provenance + on-disk skills all
        // agree, integrity must be Healthy regardless of the mirror's
        // generated_for_tag.
        let tmp = TempDir::new().unwrap();
        // Mirror dir is at v0.19.1 but its PROVENANCE stamps v0.18.0.
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.18.0");
        // Live provenance + lock both agree on v0.19.1.
        write_live_provenance_for(tmp.path(), "v0.19.1", 3, None);
        materialise_live_skills(tmp.path(), 3);
        let lock = Some(make_lock_with_version("v0.19.1"));
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        assert_eq!(
            status,
            IntegrityStatus::Healthy,
            "mirror PROVENANCE.toml's generated_for_tag is upstream-controlled \
             and must not trigger MismatchedVersion when the live provenance \
             agrees with the lock"
        );
    }

    // ── 9. Stale when skill_count below threshold ──────────────────────────

    #[test]
    fn verify_returns_stale_when_skill_count_below_threshold() {
        let tmp = TempDir::new().unwrap();
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.19.1");
        // Manifest claims 50 skills, but only 10 are on disk —
        // 10 + 1 < 50 → Stale.
        write_live_provenance_for(tmp.path(), "v0.19.1", 50, None);
        materialise_live_skills(tmp.path(), 10);
        let lock = Some(make_lock_with_version("v0.19.1"));
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        match status {
            IntegrityStatus::Stale { reason, .. } => {
                assert_eq!(reason, "skill_count_below_mirror");
            }
            other => panic!("expected Stale, got {:?}", other),
        }
    }

    // ── 10. Healthy when everything aligns ─────────────────────────────────

    #[test]
    fn verify_returns_healthy_when_all_align() {
        let tmp = TempDir::new().unwrap();
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.19.1");
        // Manifest 5 skills, 5 on disk → not stale.
        write_live_provenance_for(tmp.path(), "v0.19.1", 5, None);
        materialise_live_skills(tmp.path(), 5);
        let lock = Some(make_lock_with_version("v0.19.1"));
        let status = verify_install_integrity(tmp.path(), &lock).unwrap();
        assert_eq!(status, IntegrityStatus::Healthy);
    }

    // ── 11. decide_sync skip when version unset ────────────────────────────

    fn config_with_pk_version(version: &str) -> AiboxConfig {
        let mut cfg = crate::config::test_config();
        cfg.processkit.version = version.to_string();
        cfg.processkit.source = "https://github.com/projectious-work/processkit.git".to_string();
        cfg
    }

    #[test]
    fn decide_sync_skip_when_version_unset() {
        let tmp = TempDir::new().unwrap();
        let cfg = config_with_pk_version(PROCESSKIT_VERSION_UNSET);
        let decision = decide_sync(&cfg, tmp.path(), &None).unwrap();
        assert_eq!(decision, SyncDecision::Skip);
    }

    // ── 12. decide_sync install when no lock ───────────────────────────────

    #[test]
    fn decide_sync_install_when_no_lock() {
        let tmp = TempDir::new().unwrap();
        let cfg = config_with_pk_version("v0.19.1");
        let decision = decide_sync(&cfg, tmp.path(), &None).unwrap();
        match decision {
            SyncDecision::Install { reason } => {
                assert!(reason.contains("no aibox.lock"));
            }
            other => panic!("expected Install, got {:?}", other),
        }
    }

    // ── 13. decide_sync install when version drifts ────────────────────────

    #[test]
    fn decide_sync_install_when_version_drifts() {
        let tmp = TempDir::new().unwrap();
        let cfg = config_with_pk_version("v0.19.1");
        // Lock at older version — sync must install.
        let lock = Some(make_lock_with_version("v0.18.0"));
        let decision = decide_sync(&cfg, tmp.path(), &lock).unwrap();
        match decision {
            SyncDecision::Install { reason } => {
                assert!(
                    reason.contains("v0.18.0") && reason.contains("v0.19.1"),
                    "reason should mention both versions: {}",
                    reason
                );
            }
            other => panic!("expected Install, got {:?}", other),
        }
    }

    // ── 14. decide_sync skip when healthy ──────────────────────────────────

    #[test]
    fn decide_sync_skip_when_healthy() {
        let tmp = TempDir::new().unwrap();
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.19.1");
        write_live_provenance_for(tmp.path(), "v0.19.1", 3, None);
        materialise_live_skills(tmp.path(), 3);
        let cfg = config_with_pk_version("v0.19.1");
        let lock = Some(make_lock_with_version("v0.19.1"));
        let decision = decide_sync(&cfg, tmp.path(), &lock).unwrap();
        assert_eq!(decision, SyncDecision::Skip);
    }

    // ── 15. decide_sync reinstall on integrity failure ─────────────────────

    #[test]
    fn decide_sync_reinstall_when_integrity_fails() {
        let tmp = TempDir::new().unwrap();
        // Mirror present but live marker missing → MissingProvenance →
        // Reinstall.
        write_mirror_provenance(tmp.path(), "v0.19.1", "v0.19.1");
        let cfg = config_with_pk_version("v0.19.1");
        let lock = Some(make_lock_with_version("v0.19.1"));
        let decision = decide_sync(&cfg, tmp.path(), &lock).unwrap();
        match decision {
            SyncDecision::Reinstall {
                reason,
                prior_state,
            } => {
                assert!(reason.contains("integrity check"), "reason: {}", reason);
                assert!(
                    matches!(prior_state, IntegrityStatus::MissingProvenance { .. }),
                    "prior_state: {:?}",
                    prior_state
                );
            }
            other => panic!("expected Reinstall, got {:?}", other),
        }
    }

    // ── 16. live provenance round-trip TOML ────────────────────────────────

    #[test]
    fn live_provenance_round_trip_toml() {
        let tmp = TempDir::new().unwrap();
        let original = LiveProvenance {
            schema_version: LIVE_PROVENANCE_SCHEMA_VERSION,
            install: LiveProvenanceInstall {
                processkit_version: "v0.19.1".to_string(),
                processkit_source: "https://github.com/projectious-work/processkit.git".to_string(),
                installed_at: "2026-04-25T10:00:00Z".to_string(),
                cli_version: "0.19.2".to_string(),
            },
            manifest: LiveProvenanceManifest {
                skill_count: 79,
                schema_count: 23,
                process_count: 5,
                state_machine_count: 7,
                release_asset_sha256: Some("deadbeef".to_string()),
                install_hash: Some("abc123".to_string()),
            },
        };
        write_live_provenance(tmp.path(), &original).unwrap();
        let read_back = read_live_provenance(tmp.path()).unwrap().unwrap();
        assert_eq!(read_back, original);
    }

    // ── 17. count_from_mirror only counts processkit skills ───────────────

    #[test]
    fn count_from_mirror_excludes_non_processkit_skill_categories() {
        // Materialise a fixture mirror that contains both a
        // `processkit` skill category (counted) and a `userland`
        // category (NOT counted — user-authored, outside the integrity
        // contract).
        let tmp = TempDir::new().unwrap();
        let version = "v0.22.0";
        let mirror_skills = tmp
            .path()
            .join(TEMPLATES_PROCESSKIT_DIR)
            .join(version)
            .join("context")
            .join("skills");

        // Two processkit skills.
        for name in ["alpha", "beta"] {
            let dir = mirror_skills.join("processkit").join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), "---\nname: x\n---\n").unwrap();
        }
        // One non-processkit (user-authored) skill — must be ignored.
        let userland = mirror_skills.join("userland").join("gamma");
        fs::create_dir_all(&userland).unwrap();
        fs::write(userland.join("SKILL.md"), "---\nname: x\n---\n").unwrap();

        let counts = count_from_mirror(tmp.path(), version).unwrap();
        assert_eq!(
            counts.skills, 2,
            "count_from_mirror must count only processkit skills (got {}; \
             expected 2 since the userland/gamma skill is user-authored \
             and outside the integrity contract)",
            counts.skills
        );
    }

    // ── 18. unknown schema version treated as missing ──────────────────────

    #[test]
    fn live_provenance_unknown_schema_version_treated_as_missing() {
        let tmp = TempDir::new().unwrap();
        // Hand-write a file with schema_version = 999 (unknown).
        let path = live_provenance_path(tmp.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "schema_version = 999\n\
             [install]\n\
             foo = \"bar\"\n",
        )
        .unwrap();
        // Reader returns Ok(None) — treated as MissingProvenance by the
        // verify path.
        let result = read_live_provenance(tmp.path()).unwrap();
        assert!(result.is_none());
    }
}
