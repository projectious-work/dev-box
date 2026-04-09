//! Fetches a tagged version of a content source into a local cache
//! directory. The fetcher is content-source-neutral: it doesn't know or
//! care that the only currently-configured content source is processkit.
//!
//! ## Fetch strategy (priority order)
//!
//! 1. **Branch override** (`branch != None`) → `git clone --branch <name>`.
//!    Used for the discouraged "track HEAD of a branch" mode.
//! 2. **Release-asset tarball** → download a purpose-built `.tar.gz`
//!    attached to the release (e.g.
//!    `https://github.com/<org>/<name>/releases/download/<version>/<name>-<version>.tar.gz`).
//!    Verified against a sibling `<asset>.sha256` file when present.
//!    This is the preferred path: smaller, explicit shippable contract,
//!    bit-exact reproducibility via the recorded SHA256. See DEC-025 +
//!    BACK-106. The URL template is configurable via the consumer-side
//!    `[processkit] release_asset_url_template` field for non-GitHub
//!    hosts (Gitea, GitLab, …).
//! 3. **Host auto-tarball** → the GitHub / GitLab auto-generated
//!    `archive/refs/tags/<version>.tar.gz`. Used when no release asset
//!    is available (404 on the asset URL).
//! 4. **Git clone** of the tag. Used when neither tarball strategy works
//!    (e.g. self-hosted git over SSH, mid-stream branch testing).
//!
//! Cache layout:
//!
//! ```text
//! ~/.cache/aibox/processkit/<host>/<org>/<name>/<version>/
//!     <src_path>/PROVENANCE.toml
//!     <src_path>/skills/...
//!     <src_path>/primitives/...
//!     ...
//!     .fetch-complete   <-- idempotency marker, written last
//! ```
//!
//! When the release-asset strategy is used and the tarball unpacks to a
//! flat layout (PROVENANCE.toml at the top level instead of under
//! `src/`), the cache layout is auto-detected and `FetchedSource.src_path`
//! points at the cache root directly. This lets a producer ship a
//! src-only release artifact without forcing consumers to re-encode the
//! `src_path` step.
//!
//! Cache entries are immutable once written. Re-fetching the same
//! `(source, version)` is a no-op: the function detects the
//! `.fetch-complete` marker and returns the existing path.
//!
//! ## Authentication
//!
//! Anonymous fetches only. Tarball downloads use plain HTTPS; the
//! git-clone fallback inherits whatever credential helper the user has
//! configured for git (e.g. `gh auth setup-git`). Private repos work if
//! and only if the user's environment is already authenticated
//! out-of-band — this module does not prompt or manage tokens itself.
//!
//! ## Verification
//!
//! After fetching, [`fetch`] calls [`validate_cache`] to ensure the
//! result has the expected shape (`PROVENANCE.toml` reachable from the
//! resolved src path). When the release-asset strategy is used and a
//! sibling `.sha256` checksum file is available, the downloaded tarball
//! bytes are verified against that checksum BEFORE extraction; a
//! mismatch aborts the fetch. The verified SHA256 is recorded in
//! `aibox.lock` as `release_asset_sha256` for bit-exact reproducibility.
//! If validation fails, the incomplete cache is left in place WITHOUT
//! the `.fetch-complete` marker, so a subsequent fetch will retry from
//! scratch.

use anyhow::{Context, Result, anyhow, bail};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::PROCESSKIT_VERSION_UNSET;
use crate::output;
use crate::processkit_vocab::PROVENANCE_FILENAME;

/// Result of a successful fetch.
#[derive(Debug, Clone)]
pub struct FetchedSource {
    /// `~/.cache/aibox/processkit/<host>/<org>/<name>/<version>/`
    pub cache_root: PathBuf,
    /// The directory whose contents represent the "shipped" payload.
    /// Usually `cache_root.join(<src_path>)`, but when the release-asset
    /// strategy is used and the tarball unpacks to a flat layout, this
    /// is `cache_root` itself.
    pub src_path: PathBuf,
    /// Resolved git commit, when known. Always populated for the
    /// git-clone path; `None` for the tarball paths because tag
    /// archives do not encode the commit sha in the URL.
    pub resolved_commit: Option<String>,
    /// SHA256 hex digest of the release-asset tarball, populated only
    /// when the release-asset strategy was used AND a sibling `.sha256`
    /// checksum file was successfully downloaded and verified.
    pub release_asset_sha256: Option<String>,
    /// Echo of the input source URL.
    pub source: String,
    /// Echo of the input version tag.
    pub version: String,
}

// ---------------------------------------------------------------------------
// URL parsing
// ---------------------------------------------------------------------------

/// Parsed components of a processkit source URL, used to derive the
/// cache subdirectory.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSource {
    host: String,
    org: String,
    name: String,
}

/// Parse a source URL into `(host, org, name)` components.
///
/// Supports:
/// - `https://host/org/name(.git)`
/// - `http://host/org/name(.git)`
/// - `ssh://[user@]host[:port]/org/name(.git)`
/// - `git@host:org/name(.git)` (scp-like)
/// - `file:///abs/path/to/repo`
/// - bare `/abs/path` or `./relative` (treated as file paths, host=`local`)
fn parse_source(source: &str) -> Result<ParsedSource> {
    let s = source.trim();
    if s.is_empty() {
        bail!("source URL is empty");
    }

    // file:// scheme — derive name from the last path component, host=`local`.
    if let Some(rest) = s.strip_prefix("file://") {
        let path = Path::new(rest);
        let name = last_component_name(path)?;
        return Ok(ParsedSource {
            host: "local".to_string(),
            org: "file".to_string(),
            name,
        });
    }

    // Bare local path — same handling as file://.
    if s.starts_with('/') || s.starts_with("./") || s.starts_with("../") {
        let name = last_component_name(Path::new(s))?;
        return Ok(ParsedSource {
            host: "local".to_string(),
            org: "file".to_string(),
            name,
        });
    }

    // scp-like git URL: `git@host:org/name(.git)` (no scheme, has `@` and `:`).
    // Note: must NOT contain `://`.
    if !s.contains("://") {
        if let Some(at_idx) = s.find('@') {
            let after_at = &s[at_idx + 1..];
            if let Some(colon_idx) = after_at.find(':') {
                let host = &after_at[..colon_idx];
                let path = &after_at[colon_idx + 1..];
                return parse_org_name(host, path);
            }
        }
        bail!("unrecognized source URL (no scheme, not scp-like): {}", s);
    }

    // http(s):// or ssh:// — split scheme, then host and path.
    let (_scheme, rest) = s.split_once("://").unwrap();
    // Strip optional `user@` prefix.
    let rest = rest.splitn(2, '@').last().unwrap_or(rest);
    // First `/` separates host[:port] from path.
    let (host_part, path) = rest
        .split_once('/')
        .ok_or_else(|| anyhow!("source URL has no path component: {}", s))?;
    // Drop port from host if present (`host:1234`).
    let host = host_part
        .split_once(':')
        .map(|(h, _)| h)
        .unwrap_or(host_part);
    parse_org_name(host, path)
}

/// Build a `ParsedSource` from a host and a `org/name(.git)` path. Strips
/// `.git`, then takes the last two path segments as `org` and `name`.
fn parse_org_name(host: &str, path: &str) -> Result<ParsedSource> {
    let cleaned = path.trim_start_matches('/').trim_end_matches('/');
    let cleaned = cleaned.strip_suffix(".git").unwrap_or(cleaned);
    let segments: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 2 {
        bail!(
            "source URL path needs at least <org>/<name>, got: {}",
            path
        );
    }
    let name = segments[segments.len() - 1].to_string();
    let org = segments[segments.len() - 2].to_string();
    Ok(ParsedSource {
        host: sanitize_component(host),
        org: sanitize_component(&org),
        name: sanitize_component(&name),
    })
}

/// Extract the last filesystem component as a sanitized name.
fn last_component_name(path: &Path) -> Result<String> {
    let raw = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("path has no final component: {}", path.display()))?;
    let raw = raw.strip_suffix(".git").unwrap_or(raw);
    Ok(sanitize_component(raw))
}

/// Make a string safe to use as a single filesystem path component.
/// Replaces path separators, parent-dir tokens, and leading dots with `_`.
fn sanitize_component(s: &str) -> String {
    if s.is_empty() {
        return "_".to_string();
    }
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '/' | '\\' | '\0' => out.push('_'),
            _ => out.push(ch),
        }
    }
    // Replace `..` (path traversal) entirely.
    if out == ".." || out == "." {
        return "_".to_string();
    }
    // Strip leading dots so the component is never hidden / never `..`.
    while out.starts_with('.') {
        out.replace_range(0..1, "_");
    }
    out
}

// ---------------------------------------------------------------------------
// Cache directory resolution
// ---------------------------------------------------------------------------

/// Compute the cache directory for a `(source, version)` pair without
/// fetching. Useful for `aibox status --processkit` reporting.
pub fn cache_dir(source: &str, version: &str) -> Result<PathBuf> {
    let parsed = parse_source(source)?;
    let version_component = sanitize_component(version);
    let base = base_cache_dir()?;
    Ok(base
        .join(&parsed.host)
        .join(&parsed.org)
        .join(&parsed.name)
        .join(version_component))
}

/// Root of all aibox processkit caches: `~/.cache/aibox/processkit/`.
fn base_cache_dir() -> Result<PathBuf> {
    let cache_home = dirs::cache_dir()
        .ok_or_else(|| anyhow!("could not determine user cache directory"))?;
    Ok(cache_home.join("aibox").join("processkit"))
}

// ---------------------------------------------------------------------------
// Version listing
// ---------------------------------------------------------------------------

/// List the versions available at `source`, newest first.
///
/// Strategy:
///
/// 1. **GitHub-hosted sources** (host == `github.com`): try the GitHub
///    Releases API first (`/repos/<org>/<name>/releases?per_page=100`),
///    which gives explicit released-tag metadata. **On any failure
///    (network, 403 rate limit, JSON parse, empty result) fall through
///    to `git ls-remote`.** The unauthenticated GitHub API is capped at
///    60 requests/hour per IP — a real footgun on shared NATs and CI
///    runners. The git smart-HTTP protocol has a much higher rate limit
///    and works without any API key.
/// 2. **Anything else** (GitLab, Gitea, self-hosted, file://, SSH,
///    scp-like): `git ls-remote --tags --refs <source>` directly.
///
/// Filtering: only tags that parse as semver (with an optional leading
/// `v`) are returned. Sorted descending by semver. Duplicates after the
/// `v` strip are deduplicated. Empty result is `Ok(vec![])`, not an error.
///
/// Used by `aibox init`'s interactive picker (and the
/// `--processkit-version` flag's "default to latest" path).
pub fn list_versions(source: &str) -> Result<Vec<String>> {
    // git ls-remote is always the authoritative source: it sees every pushed
    // tag, including those not yet published as a formal GitHub Release.
    // The GitHub Releases API is used only as a fallback (e.g. if git is
    // unavailable in the environment) because it can miss tags that have been
    // pushed but not formally released.
    match list_git_tags(source) {
        Ok(v) if !v.is_empty() => return Ok(v),
        Ok(_) => {
            tracing::debug!(
                "git ls-remote returned no semver tags for {}; \
                 trying GitHub Releases API",
                source
            );
        }
        Err(e) => {
            tracing::debug!(
                "git ls-remote failed for {}: {:#}; trying GitHub Releases API",
                source,
                e
            );
        }
    }

    let parsed = parse_source(source)?;
    if parsed.host == "github.com" {
        list_github_releases(&parsed.org, &parsed.name)
    } else {
        bail!("Could not list any semver-tagged versions at {}", source)
    }
}

fn list_github_releases(org: &str, name: &str) -> Result<Vec<String>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases?per_page=100",
        org, name
    );
    let resp = ureq::get(&url)
        .header("User-Agent", "aibox-cli")
        .header("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("HTTP GET {} failed", url))?;
    if !resp.status().is_success() {
        bail!("{} returned status {}", url, resp.status());
    }
    let mut body = String::new();
    resp.into_body()
        .into_reader()
        .read_to_string(&mut body)
        .with_context(|| format!("failed to read body from {}", url))?;
    let json: serde_json::Value =
        serde_json::from_str(&body).context("failed to parse GitHub releases JSON")?;
    let arr = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub releases response was not a JSON array"))?;
    let tags: Vec<String> = arr
        .iter()
        .filter_map(|r| r.get("tag_name").and_then(|t| t.as_str()).map(String::from))
        .collect();
    Ok(filter_and_sort_semver_tags(tags))
}

fn list_git_tags(source: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", source])
        .output()
        .with_context(|| format!("failed to spawn `git ls-remote --tags {}`", source))?;
    if !output.status.success() {
        bail!(
            "git ls-remote --tags {} failed: {}",
            source,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .filter_map(|ref_path| ref_path.strip_prefix("refs/tags/"))
        .map(String::from)
        .collect();
    Ok(filter_and_sort_semver_tags(tags))
}

/// Keep only semver-looking tags, sort descending, and dedupe by the
/// stripped semver form (so `v1.2.3` and `1.2.3` collapse to one entry,
/// preferring the input that came first).
fn filter_and_sort_semver_tags(tags: Vec<String>) -> Vec<String> {
    let mut keep: Vec<(semver::Version, String)> = tags
        .into_iter()
        .filter_map(|t| parse_loose_semver(&t).map(|v| (v, t)))
        .collect();
    keep.sort_by(|a, b| b.0.cmp(&a.0));
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(keep.len());
    for (v, raw) in keep {
        if seen.insert(v.to_string()) {
            out.push(raw);
        }
    }
    out
}

/// Parse a tag string as semver, allowing an optional leading `v`.
fn parse_loose_semver(tag: &str) -> Option<semver::Version> {
    let stripped = tag.strip_prefix('v').unwrap_or(tag);
    semver::Version::parse(stripped).ok()
}

// ---------------------------------------------------------------------------
// Public fetch entry point
// ---------------------------------------------------------------------------

/// Fetch the given `source@version` into the cache.
///
/// Strategy:
/// 1. If `branch` is `Some`, the moving-branch fallback path is used
///    and a git clone of that branch is performed (discouraged but
///    supported for testing pre-release work).
/// 2. Otherwise, try the **release-asset tarball** path: download a
///    purpose-built tarball attached to the release, optionally
///    verifying it against a sibling `.sha256` file. URL is built from
///    `release_asset_url_template` (or the GitHub-style default).
/// 3. On 404 (or any other release-asset failure), fall back to the
///    **host auto-tarball** path (the GitHub / GitLab
///    `archive/refs/tags/<version>.tar.gz`).
/// 4. If that fails too, fall back to **`git clone`** of the tag.
///
/// Returns the existing cache without re-fetching if a successful fetch
/// is already on disk (idempotency marker present).
///
/// Refuses to fetch when `version` is the [`PROCESSKIT_VERSION_UNSET`]
/// sentinel.
pub fn fetch(
    source: &str,
    version: &str,
    branch: Option<&str>,
    src_path: &str,
    release_asset_url_template: Option<&str>,
) -> Result<FetchedSource> {
    if version == PROCESSKIT_VERSION_UNSET {
        bail!(
            "processkit.version is '{}' — set a real version in aibox.toml before fetching.",
            PROCESSKIT_VERSION_UNSET
        );
    }
    if version.trim().is_empty() && branch.is_none() {
        bail!("processkit.version is empty and no branch override is set; nothing to fetch.");
    }

    let parsed = parse_source(source)?;
    let cache_root = cache_dir(source, version)?;
    let marker = cache_root.join(".fetch-complete");

    // Idempotency: a complete fetch from a previous run is reused as-is.
    if marker.exists() {
        // Defensive validation in case someone hand-edited the cache.
        let resolved_src = resolve_src_path(&cache_root, src_path).with_context(|| {
            format!(
                "cached fetch at {} failed validation; remove the cache and retry",
                cache_root.display()
            )
        })?;
        let (commit, sha256) = read_marker(&marker);
        return Ok(FetchedSource {
            cache_root,
            src_path: resolved_src,
            resolved_commit: commit,
            release_asset_sha256: sha256,
            source: source.to_string(),
            version: version.to_string(),
        });
    }

    // Make a clean target directory. If a previous attempt left a
    // half-populated directory behind, wipe it before retrying.
    if cache_root.exists() {
        fs::remove_dir_all(&cache_root).with_context(|| {
            format!("failed to clean stale cache dir {}", cache_root.display())
        })?;
    }
    fs::create_dir_all(&cache_root).with_context(|| {
        format!("failed to create cache dir {}", cache_root.display())
    })?;

    let mut resolved_commit: Option<String> = None;
    let mut release_asset_sha256: Option<String> = None;
    let strategy: &str;

    if let Some(b) = branch {
        // 1. Branch override → git clone.
        output::info(&format!(
            "Cloning content source branch '{}' from {} (moving-branch mode)",
            b, source
        ));
        resolved_commit = Some(git_clone(source, Some(b), &cache_root)?);
        strategy = "git clone";
    } else if let Some(asset_url) = build_release_asset_url(
        source,
        &parsed,
        version,
        release_asset_url_template,
    ) {
        // 2. Release-asset tarball.
        output::info(&format!(
            "Downloading release asset {} -> {}",
            asset_url,
            cache_root.display()
        ));
        match download_and_extract_release_asset(&asset_url, &cache_root) {
            Ok(verified_sha) => {
                release_asset_sha256 = verified_sha;
                strategy = "release asset";
            }
            Err(asset_err) => {
                // 3. Fall back to host auto-tarball.
                output::warn(&format!(
                    "release asset fetch failed ({}); falling back to host auto-tarball",
                    asset_err
                ));
                if cache_root.exists() {
                    fs::remove_dir_all(&cache_root)?;
                }
                fs::create_dir_all(&cache_root)?;

                if let Some(host_url) = host_tarball_url(&parsed, version) {
                    output::info(&format!(
                        "Downloading host tarball {} -> {}",
                        host_url,
                        cache_root.display()
                    ));
                    match download_and_extract_tarball(&host_url, &cache_root) {
                        Ok(()) => {
                            strategy = "host tarball";
                        }
                        Err(tar_err) => {
                            // 4. Fall back to git clone.
                            output::warn(&format!(
                                "host tarball download failed ({}); falling back to git clone",
                                tar_err
                            ));
                            if cache_root.exists() {
                                fs::remove_dir_all(&cache_root)?;
                            }
                            fs::create_dir_all(&cache_root)?;
                            resolved_commit =
                                Some(git_clone(source, Some(version), &cache_root)?);
                            strategy = "git clone";
                        }
                    }
                } else {
                    // No host tarball pattern → straight to git clone.
                    resolved_commit = Some(git_clone(source, Some(version), &cache_root)?);
                    strategy = "git clone";
                }
            }
        }
    } else if let Some(host_url) = host_tarball_url(&parsed, version) {
        // No release asset URL was buildable (e.g. caller passed an
        // empty template AND host is not GitHub/GitLab) — host tarball
        // is the next best thing.
        output::info(&format!(
            "Downloading host tarball {} -> {}",
            host_url,
            cache_root.display()
        ));
        match download_and_extract_tarball(&host_url, &cache_root) {
            Ok(()) => {
                strategy = "host tarball";
            }
            Err(tar_err) => {
                output::warn(&format!(
                    "host tarball download failed ({}); falling back to git clone",
                    tar_err
                ));
                if cache_root.exists() {
                    fs::remove_dir_all(&cache_root)?;
                }
                fs::create_dir_all(&cache_root)?;
                resolved_commit = Some(git_clone(source, Some(version), &cache_root)?);
                strategy = "git clone";
            }
        }
    } else {
        // 4. Direct git clone (no tarball strategy applies).
        output::info(&format!(
            "Cloning content source tag '{}' from {}",
            version, source
        ));
        resolved_commit = Some(git_clone(source, Some(version), &cache_root)?);
        strategy = "git clone";
    }

    // Verify result before declaring success. resolve_src_path also
    // auto-detects flat-vs-wrapped tarball layouts.
    let resolved_src = resolve_src_path(&cache_root, src_path).with_context(|| {
        format!(
            "fetched source at {} does not have the expected shape",
            cache_root.display()
        )
    })?;

    // Write the marker LAST. Body encodes the resolved_commit (if any)
    // and the release_asset_sha256 (if any) so subsequent idempotent
    // calls can echo them.
    write_marker(&marker, resolved_commit.as_deref(), release_asset_sha256.as_deref())?;

    output::ok(&format!(
        "Fetched content source {} via {} into {}",
        version,
        strategy,
        cache_root.display()
    ));

    Ok(FetchedSource {
        cache_root: cache_root.clone(),
        src_path: resolved_src,
        resolved_commit,
        release_asset_sha256,
        source: source.to_string(),
        version: version.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Marker file (idempotency)
// ---------------------------------------------------------------------------
//
// Format: two lines.
//   line 1: resolved git commit (may be empty)
//   line 2: release_asset_sha256 (may be empty)
//
// A pre-v0.15 marker is a single line with just the commit; the parser
// is tolerant — missing line 2 is read as None.

/// Write the `.fetch-complete` marker at `path` with the optional
/// resolved commit and optional release-asset SHA256.
fn write_marker(
    path: &Path,
    resolved_commit: Option<&str>,
    release_asset_sha256: Option<&str>,
) -> Result<()> {
    let body = format!(
        "{}\n{}\n",
        resolved_commit.unwrap_or(""),
        release_asset_sha256.unwrap_or("")
    );
    let mut f = fs::File::create(path)
        .with_context(|| format!("failed to write marker {}", path.display()))?;
    f.write_all(body.as_bytes())?;
    Ok(())
}

/// Read the `.fetch-complete` marker. Returns `(resolved_commit, sha256)`.
/// Both are `None` for missing/empty entries.
fn read_marker(path: &Path) -> (Option<String>, Option<String>) {
    let body = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return (None, None),
    };
    let mut lines = body.lines();
    let commit = lines
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let sha256 = lines
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    (commit, sha256)
}

// ---------------------------------------------------------------------------
// Cache validation + src auto-detect
// ---------------------------------------------------------------------------

/// Sanity-check that a fetched cache contains the expected shape.
/// Returns `Ok(())` if `<cache_root>/<src_path>/PROVENANCE.toml` exists.
pub fn validate_cache(cache_root: &Path, src_path: &str) -> Result<()> {
    let provenance = cache_root.join(src_path).join(PROVENANCE_FILENAME);
    if !provenance.exists() {
        bail!(
            "fetched source does not have the expected shape (missing {}). \
             If this is a pre-v0.4.0 processkit, upgrade the version pin to v0.4.0+.",
            provenance.display()
        );
    }
    Ok(())
}

/// Resolve the actual src directory inside `cache_root`, auto-detecting
/// the two valid layouts:
///
/// - **Wrapped** (git tarball or git clone): `<cache_root>/<src_path>/PROVENANCE.toml`
/// - **Flat** (release asset shipping just the src tree):
///   `<cache_root>/PROVENANCE.toml`
///
/// The flat layout lets a producer ship a smaller, src-only release
/// asset without forcing consumers to encode the `src_path` step.
///
/// Returns the path that contains `PROVENANCE.toml`. Errors if neither
/// layout is present.
pub fn resolve_src_path(cache_root: &Path, src_path: &str) -> Result<PathBuf> {
    // Wrapped layout — check first because it's the historical default.
    let wrapped = cache_root.join(src_path);
    if wrapped.join(PROVENANCE_FILENAME).exists() {
        return Ok(wrapped);
    }
    // Flat layout — release asset where the tarball top level IS the src.
    if cache_root.join(PROVENANCE_FILENAME).exists() {
        return Ok(cache_root.to_path_buf());
    }
    bail!(
        "fetched cache at {} has no PROVENANCE.toml at either {} or the cache root. \
         Expected a content source with PROVENANCE.toml at the top of <src_path>/.",
        cache_root.display(),
        wrapped.display(),
    )
}

// ---------------------------------------------------------------------------
// Release-asset URL building
// ---------------------------------------------------------------------------

/// Default URL template for release-asset tarballs (GitHub-style):
/// `{source}/releases/download/{version}/{name}-{version}.tar.gz`
///
/// `{source}` is the source URL with any trailing `.git` stripped.
/// `{name}` and `{org}` come from the parsed source URL. `{version}`
/// is the tag.
const DEFAULT_RELEASE_ASSET_URL_TEMPLATE: &str =
    "{source}/releases/download/{version}/{name}-{version}.tar.gz";

/// Build the release-asset tarball URL for a `(source, version)` pair.
///
/// If `template` is provided, expand it with the placeholders below.
/// Otherwise use [`DEFAULT_RELEASE_ASSET_URL_TEMPLATE`]. Returns `None`
/// only if the source URL has no usable HTTP form (e.g. a bare scp-like
/// `git@host:org/name` with no derivable HTTPS host) AND no template
/// override is set — in that case the caller falls through to the next
/// strategy.
///
/// Placeholders:
///   - `{source}` — the source URL with any trailing `.git` stripped
///   - `{version}` — the version tag
///   - `{org}` — the parsed org segment
///   - `{name}` — the parsed name segment (with `.git` stripped)
fn build_release_asset_url(
    source: &str,
    parsed: &ParsedSource,
    version: &str,
    template: Option<&str>,
) -> Option<String> {
    // Source URL normalization: strip trailing `.git` so the template's
    // `{source}` slot expands to the canonical web URL form.
    let source_for_template = source.strip_suffix(".git").unwrap_or(source).to_string();

    // If neither a template was provided NOR the source has an http(s)
    // scheme, we can't build a release-asset URL — skip the strategy.
    let template = template.unwrap_or(DEFAULT_RELEASE_ASSET_URL_TEMPLATE);
    if template.contains("{source}")
        && !source_for_template.starts_with("http://")
        && !source_for_template.starts_with("https://")
    {
        return None;
    }

    let url = template
        .replace("{source}", &source_for_template)
        .replace("{version}", version)
        .replace("{org}", &parsed.org)
        .replace("{name}", &parsed.name);
    Some(url)
}

// ---------------------------------------------------------------------------
// Release-asset download + verify
// ---------------------------------------------------------------------------

/// Download a release-asset tarball and (optionally) verify its
/// checksum, then extract into `dest`.
///
/// Behavior:
/// 1. HTTP GET the asset URL. A 404 (or any other error) is propagated
///    so the caller can fall back to the next strategy.
/// 2. Try to fetch `{asset_url}.sha256`. If present, parse the first
///    hex token as the expected SHA256. Compute the actual SHA256 of
///    the downloaded bytes. Mismatch → hard error (don't fall back —
///    a checksum mismatch is a security signal, not a 404).
/// 3. If no `.sha256` sibling exists, warn and proceed without
///    verification.
/// 4. Extract the tarball. Auto-detect whether the tarball has a
///    single-directory wrapper (strip it) or is already flat.
///
/// Returns the verified SHA256 (or `None` if no checksum was available).
fn download_and_extract_release_asset(url: &str, dest: &Path) -> Result<Option<String>> {
    let resp = ureq::get(url)
        .header("User-Agent", "aibox-cli")
        .call()
        .with_context(|| format!("HTTP GET {} failed", url))?;
    if resp.status().as_u16() == 404 {
        bail!("release asset not found at {} (HTTP 404)", url);
    }
    if !resp.status().is_success() {
        bail!(
            "release asset request to {} returned status {}",
            url,
            resp.status()
        );
    }
    let mut bytes: Vec<u8> = Vec::new();
    resp.into_body()
        .into_reader()
        .read_to_end(&mut bytes)
        .context("failed to read release asset body")?;

    // Compute SHA256 of the downloaded tarball — used for both
    // verification and the lock-file marker.
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual_sha = format!("{:x}", hasher.finalize());

    // Optional checksum verification.
    let checksum_url = format!("{}.sha256", url);
    let verified_sha: Option<String> = match fetch_text_optional(&checksum_url)? {
        Some(checksum_body) => {
            let expected = parse_sha256_checksum(&checksum_body).with_context(|| {
                format!(
                    "failed to parse SHA256 checksum from {}; expected hex digest, got: {}",
                    checksum_url,
                    checksum_body.lines().next().unwrap_or("(empty)")
                )
            })?;
            if !actual_sha.eq_ignore_ascii_case(&expected) {
                bail!(
                    "release asset SHA256 mismatch for {}: expected {}, got {}",
                    url,
                    expected,
                    actual_sha
                );
            }
            output::ok(&format!(
                "Verified release asset SHA256 against {}",
                checksum_url
            ));
            Some(actual_sha.clone())
        }
        None => {
            output::warn(&format!(
                "no .sha256 sibling at {} — proceeding without checksum verification",
                checksum_url
            ));
            None
        }
    };

    extract_tar_gz_auto(&bytes, dest)?;
    Ok(verified_sha)
}

/// HTTP GET that treats 404 as `Ok(None)` and any other non-success as
/// an error. Used for the `.sha256` sibling probe.
fn fetch_text_optional(url: &str) -> Result<Option<String>> {
    match ureq::get(url).header("User-Agent", "aibox-cli").call() {
        Ok(resp) => {
            if resp.status().as_u16() == 404 {
                return Ok(None);
            }
            if !resp.status().is_success() {
                return Ok(None);
            }
            let mut body = String::new();
            resp.into_body()
                .into_reader()
                .read_to_string(&mut body)
                .with_context(|| format!("failed to read body from {}", url))?;
            Ok(Some(body))
        }
        // Network/DNS error → treat as missing (the caller already
        // succeeded on the asset itself, so the .sha256 not being
        // reachable is non-fatal).
        Err(_) => Ok(None),
    }
}

/// Parse a SHA256 checksum file. Accepts:
///   - bare hex digest (`abc123...`)
///   - `sha256sum`-style line (`abc123...  filename`)
///   - leading/trailing whitespace
fn parse_sha256_checksum(body: &str) -> Result<String> {
    let first_line = body.lines().next().ok_or_else(|| anyhow!("empty checksum file"))?;
    let token = first_line
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("empty checksum line"))?;
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("not a SHA256 hex digest: {}", token);
    }
    Ok(token.to_lowercase())
}

/// Extract a `.tar.gz` from memory into `dest`, auto-detecting whether
/// the archive has a single top-level directory wrapper.
///
/// Heuristic: if every entry in the archive shares the same first path
/// component, treat it as a wrapper and strip it (mirroring `tar
/// --strip-components=1`). Otherwise extract verbatim.
///
/// This handles both shapes a producer might ship:
///   - `<name>-<version>/skills/...`  (wrapped, like GitHub auto-tarball)
///   - `skills/...`                   (flat, more typical for hand-built release tarballs)
fn extract_tar_gz_auto(bytes: &[u8], dest: &Path) -> Result<()> {
    // First pass: collect entry paths to detect a common top-level prefix.
    let common_prefix = {
        let gz = flate2::read::GzDecoder::new(bytes);
        let mut archive = tar::Archive::new(gz);
        let mut prefix: Option<String> = None;
        let mut all_share = true;
        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?.into_owned();
            let mut comps = path.components();
            let first = match comps.next() {
                Some(std::path::Component::Normal(s)) => s.to_string_lossy().to_string(),
                _ => continue,
            };
            match &prefix {
                None => prefix = Some(first),
                Some(p) if p == &first => {}
                Some(_) => {
                    all_share = false;
                    break;
                }
            }
        }
        if all_share { prefix } else { None }
    };

    // Second pass: extract. If a common prefix was detected, strip it.
    let gz = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let raw_path = entry.path()?.into_owned();
        let stripped: PathBuf = match &common_prefix {
            Some(prefix) => {
                let mut comps = raw_path.components();
                let first = comps.next();
                match first {
                    Some(std::path::Component::Normal(s))
                        if s.to_string_lossy() == *prefix =>
                    {
                        comps.collect()
                    }
                    _ => raw_path.clone(),
                }
            }
            None => raw_path.clone(),
        };
        if stripped.as_os_str().is_empty() {
            continue;
        }
        if stripped.is_absolute()
            || stripped
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            bail!("tarball entry escapes destination: {}", stripped.display());
        }
        let out_path = dest.join(&stripped);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        entry
            .unpack(&out_path)
            .with_context(|| format!("failed to unpack {}", out_path.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tarball fast path
// ---------------------------------------------------------------------------

/// Build a tarball URL for known hosts. Returns `None` if the host
/// is not GitHub or GitLab-flavored — caller falls back to git clone.
fn host_tarball_url(parsed: &ParsedSource, version: &str) -> Option<String> {
    if parsed.host == "github.com" {
        return Some(format!(
            "https://github.com/{}/{}/archive/refs/tags/{}.tar.gz",
            parsed.org, parsed.name, version
        ));
    }
    if parsed.host.contains("gitlab") {
        return Some(format!(
            "https://{}/{}/{}/-/archive/{}/{}-{}.tar.gz",
            parsed.host, parsed.org, parsed.name, version, parsed.name, version
        ));
    }
    None
}

/// Download a `.tar.gz` over HTTPS and extract it into `dest`, stripping
/// the single top-level directory wrapper that GitHub / GitLab tag
/// archives include.
fn download_and_extract_tarball(url: &str, dest: &Path) -> Result<()> {
    let resp = ureq::get(url)
        .header("User-Agent", "aibox-cli")
        .call()
        .with_context(|| format!("HTTP GET {} failed", url))?;
    let mut bytes: Vec<u8> = Vec::new();
    resp.into_body()
        .into_reader()
        .read_to_end(&mut bytes)
        .context("failed to read tarball body")?;

    extract_tar_gz_strip_top(&bytes, dest)
}

/// Decode a gzipped tarball from memory and extract it into `dest`,
/// dropping the leading path component of every entry. This mirrors
/// `tar --strip-components=1`.
fn extract_tar_gz_strip_top(bytes: &[u8], dest: &Path) -> Result<()> {
    let gz = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let raw_path = entry.path()?.into_owned();
        let mut components = raw_path.components();
        // Drop the leading top-level directory.
        if components.next().is_none() {
            continue;
        }
        let stripped: PathBuf = components.collect();
        if stripped.as_os_str().is_empty() {
            continue;
        }
        // Reject any absolute or `..` paths defensively.
        if stripped.is_absolute()
            || stripped
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            bail!("tarball entry escapes destination: {}", stripped.display());
        }
        let out_path = dest.join(&stripped);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        entry
            .unpack(&out_path)
            .with_context(|| format!("failed to unpack {}", out_path.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Git clone fallback
// ---------------------------------------------------------------------------

/// Run `git clone --depth=1` and capture the resolved HEAD commit.
/// If `ref_name` is `Some`, it's passed as `--branch` (works for both
/// tags and branches).
fn git_clone(source: &str, ref_name: Option<&str>, dest: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth=1");
    if let Some(r) = ref_name {
        cmd.arg("--branch").arg(r);
    }
    cmd.arg(source).arg(dest);

    let output = cmd
        .output()
        .with_context(|| format!("failed to invoke git clone for {}", source))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git clone failed for {}: {}", source, stderr.trim());
    }

    let rev = Command::new("git")
        .args(["-C"])
        .arg(dest)
        .args(["rev-parse", "HEAD"])
        .output()
        .with_context(|| "failed to invoke git rev-parse HEAD".to_string())?;
    if !rev.status.success() {
        let stderr = String::from_utf8_lossy(&rev.stderr);
        bail!("git rev-parse HEAD failed: {}", stderr.trim());
    }
    let commit = String::from_utf8_lossy(&rev.stdout).trim().to_string();
    Ok(commit)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // -- Version listing helpers (no network) ------------------------------

    #[test]
    fn parse_loose_semver_accepts_v_prefix_and_bare() {
        assert_eq!(
            parse_loose_semver("v0.5.1").unwrap(),
            semver::Version::new(0, 5, 1)
        );
        assert_eq!(
            parse_loose_semver("0.5.1").unwrap(),
            semver::Version::new(0, 5, 1)
        );
    }

    #[test]
    fn parse_loose_semver_rejects_non_semver() {
        assert!(parse_loose_semver("latest").is_none());
        assert!(parse_loose_semver("v0.5").is_none()); // missing patch
        assert!(parse_loose_semver("not-a-tag").is_none());
    }

    #[test]
    fn filter_and_sort_drops_non_semver_tags() {
        let raw = vec![
            "v0.5.1".to_string(),
            "latest".to_string(),
            "v0.4.0".to_string(),
            "release-candidate".to_string(),
        ];
        let out = filter_and_sort_semver_tags(raw);
        assert_eq!(out, vec!["v0.5.1", "v0.4.0"]);
    }

    #[test]
    fn filter_and_sort_returns_descending() {
        let raw = vec![
            "v0.4.0".to_string(),
            "v0.5.1".to_string(),
            "v0.5.0".to_string(),
            "v0.4.1".to_string(),
        ];
        let out = filter_and_sort_semver_tags(raw);
        assert_eq!(out, vec!["v0.5.1", "v0.5.0", "v0.4.1", "v0.4.0"]);
    }

    #[test]
    fn filter_and_sort_dedupes_v_prefixed_and_bare() {
        // GitHub may include both "0.5.1" (rare) and "v0.5.1" tags. After
        // semver normalization they collide; we keep the first occurrence.
        let raw = vec![
            "v0.5.1".to_string(),
            "0.5.1".to_string(),
            "v0.4.0".to_string(),
        ];
        let out = filter_and_sort_semver_tags(raw);
        assert_eq!(out, vec!["v0.5.1", "v0.4.0"]);
    }

    #[test]
    fn filter_and_sort_handles_prerelease() {
        let raw = vec![
            "v1.0.0".to_string(),
            "v1.0.0-rc1".to_string(),
            "v0.9.0".to_string(),
        ];
        let out = filter_and_sort_semver_tags(raw);
        // Per semver, 1.0.0 > 1.0.0-rc1, so the stable comes first.
        assert_eq!(out, vec!["v1.0.0", "v1.0.0-rc1", "v0.9.0"]);
    }

    #[test]
    fn filter_and_sort_empty_in_empty_out() {
        assert!(filter_and_sort_semver_tags(vec![]).is_empty());
    }

    // -- URL parsing --------------------------------------------------------

    #[test]
    fn cache_dir_parses_github_https_url() {
        let p = cache_dir(
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            "v0.4.0",
        )
        .unwrap();
        let s = p.to_string_lossy();
        assert!(
            s.ends_with("aibox/processkit/github.com/projectious-work/processkit/v0.4.0"),
            "unexpected cache path: {}",
            s
        );
    }

    #[test]
    fn cache_dir_parses_github_ssh_url() {
        let scp = parse_source("git@github.com:projectious-work/processkit.git").unwrap();
        assert_eq!(scp.host, "github.com");
        assert_eq!(scp.org, "projectious-work");
        assert_eq!(scp.name, "processkit");

        let ssh = parse_source("ssh://git@github.com/foo/bar.git").unwrap();
        assert_eq!(ssh.host, "github.com");
        assert_eq!(ssh.org, "foo");
        assert_eq!(ssh.name, "bar");
    }

    #[test]
    fn cache_dir_parses_gitlab_https_url() {
        let p = parse_source("https://gitlab.acme.com/platform/processkit-acme.git").unwrap();
        assert_eq!(p.host, "gitlab.acme.com");
        assert_eq!(p.org, "platform");
        assert_eq!(p.name, "processkit-acme");
    }

    #[test]
    fn cache_dir_parses_file_url() {
        let p = parse_source("file:///tmp/local-clone").unwrap();
        assert_eq!(p.host, "local");
        assert_eq!(p.org, "file");
        assert_eq!(p.name, "local-clone");
    }

    #[test]
    fn cache_dir_strips_trailing_dot_git() {
        let p = parse_source("https://github.com/foo/bar.git").unwrap();
        assert_eq!(p.name, "bar");
    }

    #[test]
    fn cache_dir_sanitizes_path_traversal_in_components() {
        // We can't get a `..` through real URL parsing for org/name, but
        // sanitize_component must catch it directly.
        assert_eq!(sanitize_component(".."), "_");
        assert_eq!(sanitize_component("../etc"), "_._etc");
        assert_eq!(sanitize_component(".hidden"), "_hidden");
        assert_eq!(sanitize_component("a/b"), "a_b");
        // Versions with `/` in them get neutralized, so the cache path
        // remains a single component.
        let p = cache_dir(
            "https://github.com/foo/bar.git",
            "release/v1.0",
        )
        .unwrap();
        let last = p.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(last, "release_v1.0");
    }

    // -- Release asset URL building -----------------------------------------

    fn parsed_for(source: &str) -> ParsedSource {
        parse_source(source).unwrap()
    }

    #[test]
    fn release_asset_url_default_template_github() {
        let url = build_release_asset_url(
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            &parsed_for(crate::processkit_vocab::PROCESSKIT_GIT_SOURCE),
            "v0.5.1",
            None,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://github.com/projectious-work/processkit/releases/download/v0.5.1/processkit-v0.5.1.tar.gz"
        );
    }

    #[test]
    fn release_asset_url_default_template_strips_dot_git() {
        // The .git suffix on the source URL must NOT appear in the
        // expanded {source} slot.
        let url = build_release_asset_url(
            "https://github.com/foo/bar.git",
            &parsed_for("https://github.com/foo/bar.git"),
            "v1.0.0",
            None,
        )
        .unwrap();
        assert!(!url.contains(".git/"));
        assert!(url.contains("/foo/bar/releases/download/v1.0.0/bar-v1.0.0.tar.gz"));
    }

    #[test]
    fn release_asset_url_custom_template() {
        let template = "https://gitea.acme.com/{org}/{name}/releases/download/{version}/payload.tar.gz";
        let url = build_release_asset_url(
            "https://gitea.acme.com/platform/processkit-acme.git",
            &parsed_for("https://gitea.acme.com/platform/processkit-acme.git"),
            "v2.0.0",
            Some(template),
        )
        .unwrap();
        assert_eq!(
            url,
            "https://gitea.acme.com/platform/processkit-acme/releases/download/v2.0.0/payload.tar.gz"
        );
    }

    #[test]
    fn release_asset_url_returns_none_for_non_http_source_with_default_template() {
        // The default template uses {source}, which only makes sense
        // for an http(s) source URL. SSH/scp URLs fall through to the
        // next strategy.
        let url = build_release_asset_url(
            "git@github.com:foo/bar.git",
            &parsed_for("git@github.com:foo/bar.git"),
            "v1.0.0",
            None,
        );
        assert!(url.is_none());
    }

    #[test]
    fn release_asset_url_works_for_non_http_source_when_template_avoids_source_placeholder() {
        // A user-provided template that doesn't reference {source} can
        // still build a URL for a scp-like git URL.
        let template = "https://github.com/{org}/{name}/releases/download/{version}/{name}-{version}.tar.gz";
        let url = build_release_asset_url(
            "git@github.com:foo/bar.git",
            &parsed_for("git@github.com:foo/bar.git"),
            "v1.0.0",
            Some(template),
        )
        .unwrap();
        assert_eq!(
            url,
            "https://github.com/foo/bar/releases/download/v1.0.0/bar-v1.0.0.tar.gz"
        );
    }

    // -- SHA256 checksum parsing --------------------------------------------

    #[test]
    fn parse_sha256_bare_hex() {
        let hex = "abc123def456abc123def456abc123def456abc123def456abc123def456abcd";
        assert_eq!(parse_sha256_checksum(hex).unwrap(), hex);
    }

    #[test]
    fn parse_sha256_sha256sum_style() {
        let body =
            "abc123def456abc123def456abc123def456abc123def456abc123def456abcd  processkit-v0.5.1.tar.gz\n";
        let parsed = parse_sha256_checksum(body).unwrap();
        assert_eq!(
            parsed,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abcd"
        );
    }

    #[test]
    fn parse_sha256_uppercase_normalized_to_lower() {
        let hex = "ABC123DEF456ABC123DEF456ABC123DEF456ABC123DEF456ABC123DEF456ABCD";
        let parsed = parse_sha256_checksum(hex).unwrap();
        assert_eq!(
            parsed,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abcd"
        );
    }

    #[test]
    fn parse_sha256_rejects_short_hash() {
        assert!(parse_sha256_checksum("deadbeef").is_err());
    }

    #[test]
    fn parse_sha256_rejects_non_hex() {
        let almost = "Z".repeat(64);
        assert!(parse_sha256_checksum(&almost).is_err());
    }

    #[test]
    fn parse_sha256_rejects_empty() {
        assert!(parse_sha256_checksum("").is_err());
    }

    // -- Marker round trip --------------------------------------------------

    #[test]
    fn marker_round_trip_with_both_fields() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("marker");
        write_marker(&p, Some("deadbeef"), Some("cafebabe")).unwrap();
        let (commit, sha) = read_marker(&p);
        assert_eq!(commit.as_deref(), Some("deadbeef"));
        assert_eq!(sha.as_deref(), Some("cafebabe"));
    }

    #[test]
    fn marker_round_trip_commit_only() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("marker");
        write_marker(&p, Some("deadbeef"), None).unwrap();
        let (commit, sha) = read_marker(&p);
        assert_eq!(commit.as_deref(), Some("deadbeef"));
        assert_eq!(sha, None);
    }

    #[test]
    fn marker_round_trip_sha_only() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("marker");
        write_marker(&p, None, Some("cafebabe")).unwrap();
        let (commit, sha) = read_marker(&p);
        assert_eq!(commit, None);
        assert_eq!(sha.as_deref(), Some("cafebabe"));
    }

    #[test]
    fn marker_legacy_single_line_format_still_parses() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("marker");
        // Pre-v0.15 marker format: single line with just the commit.
        fs::write(&p, "deadbeef").unwrap();
        let (commit, sha) = read_marker(&p);
        assert_eq!(commit.as_deref(), Some("deadbeef"));
        assert_eq!(sha, None);
    }

    // -- resolve_src_path auto-detect ---------------------------------------

    #[test]
    fn resolve_src_path_wrapped_layout() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/PROVENANCE.toml"), "v\n").unwrap();
        let resolved = resolve_src_path(tmp.path(), "src").unwrap();
        assert_eq!(resolved, tmp.path().join("src"));
    }

    #[test]
    fn resolve_src_path_flat_layout() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(PROVENANCE_FILENAME), "v\n").unwrap();
        // Even with src_path = "src" requested, a flat layout is detected.
        let resolved = resolve_src_path(tmp.path(), "src").unwrap();
        assert_eq!(resolved, tmp.path().to_path_buf());
    }

    #[test]
    fn resolve_src_path_prefers_wrapped_when_both_exist() {
        // If somehow both a wrapped and a flat PROVENANCE.toml exist,
        // wrapped wins (it's the historical default).
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/PROVENANCE.toml"), "wrapped\n").unwrap();
        fs::write(tmp.path().join(PROVENANCE_FILENAME), "flat\n").unwrap();
        let resolved = resolve_src_path(tmp.path(), "src").unwrap();
        assert_eq!(resolved, tmp.path().join("src"));
    }

    #[test]
    fn resolve_src_path_errors_when_no_provenance() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        let err = resolve_src_path(tmp.path(), "src").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("PROVENANCE.toml"));
    }

    // -- Tarball auto-extract -----------------------------------------------

    fn make_tar_gz(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = tar::Builder::new(&mut gz);
            for (name, content) in entries {
                let mut header = tar::Header::new_gnu();
                header.set_path(name).unwrap();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append(&header, *content).unwrap();
            }
            builder.finish().unwrap();
        }
        gz.finish().unwrap()
    }

    #[test]
    fn extract_tar_gz_auto_strips_common_top_level_wrapper() {
        let tar_gz = make_tar_gz(&[
            ("processkit-v0.5.1/PROVENANCE.toml", b"v = \"v0.5.1\"\n"),
            ("processkit-v0.5.1/skills/event-log/SKILL.md", b"# skill\n"),
        ]);
        let tmp = TempDir::new().unwrap();
        extract_tar_gz_auto(&tar_gz, tmp.path()).unwrap();
        // Wrapper directory was stripped.
        assert!(tmp.path().join(PROVENANCE_FILENAME).exists());
        assert!(tmp.path().join("skills/event-log/SKILL.md").exists());
        assert!(!tmp.path().join("processkit-v0.5.1").exists());
    }

    #[test]
    fn extract_tar_gz_auto_preserves_flat_archive() {
        // Mixed top-level entries → no common prefix → no stripping.
        let tar_gz = make_tar_gz(&[
            ("PROVENANCE.toml", b"v\n"),
            ("skills/event-log/SKILL.md", b"# skill\n"),
            ("primitives/schemas/workitem.yaml", b"k: v\n"),
        ]);
        let tmp = TempDir::new().unwrap();
        extract_tar_gz_auto(&tar_gz, tmp.path()).unwrap();
        assert!(tmp.path().join(PROVENANCE_FILENAME).exists());
        assert!(tmp.path().join("skills/event-log/SKILL.md").exists());
        assert!(tmp.path().join("primitives/schemas/workitem.yaml").exists());
    }

    #[test]
    fn extract_tar_gz_auto_resolves_to_flat_layout_via_resolve_src_path() {
        // End-to-end: extract a flat tarball, then resolve_src_path
        // auto-detects the flat layout.
        let tar_gz = make_tar_gz(&[
            ("PROVENANCE.toml", b"v\n"),
            ("skills/event-log/SKILL.md", b"# skill\n"),
        ]);
        let tmp = TempDir::new().unwrap();
        extract_tar_gz_auto(&tar_gz, tmp.path()).unwrap();
        let resolved = resolve_src_path(tmp.path(), "src").unwrap();
        assert_eq!(resolved, tmp.path().to_path_buf());
    }

    // -- Sentinel rejection -------------------------------------------------

    #[test]
    fn fetch_refuses_unset_sentinel() {
        let err = fetch(
            "https://github.com/foo/bar.git",
            PROCESSKIT_VERSION_UNSET,
            None,
            "src",
            None,
        )
        .unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("'unset'"),
            "expected sentinel error, got: {}",
            msg
        );
    }

    // -- validate_cache -----------------------------------------------------

    #[test]
    fn validate_cache_accepts_processkit_shaped_dir() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(src.join("primitives/schemas")).unwrap();
        fs::write(src.join(PROVENANCE_FILENAME), "version = \"v0.4.0\"\n").unwrap();
        fs::write(src.join("primitives/schemas/workitem.yaml"), "name: x\n").unwrap();
        validate_cache(tmp.path(), "src").unwrap();
    }

    #[test]
    fn validate_cache_rejects_missing_provenance() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        let err = validate_cache(tmp.path(), "src").unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("PROVENANCE.toml"),
            "expected provenance error, got: {}",
            msg
        );
    }

    // -- Idempotency marker -------------------------------------------------

    /// Build a complete fake cache directory and prove that `fetch()`
    /// short-circuits on the marker without ever touching the network.
    /// We override `dirs::cache_dir()` indirectly by pointing
    /// `XDG_CACHE_HOME` at a temp dir for the duration of this test.
    #[test]
    fn idempotency_marker_skips_refetch() {
        let tmp = TempDir::new().unwrap();
        // Force `dirs::cache_dir()` to land inside our tmp.
        // SAFETY: tests are run serially-enough for this to be safe; the
        // env var is restored when `_guard` drops at the end of the
        // function via the `unsafe { std::env::remove_var }` call.
        // We avoid `serial_test` here because no other test touches
        // XDG_CACHE_HOME.
        let prev = std::env::var_os("XDG_CACHE_HOME");
        // SAFETY: setting an env var is unsafe in 2024 edition Rust.
        unsafe {
            std::env::set_var("XDG_CACHE_HOME", tmp.path());
        }

        let cache_root = cache_dir(
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            "v0.4.0",
        )
        .unwrap();
        // Synthesize the cache contents.
        let src = cache_root.join("src");
        fs::create_dir_all(src.join("skills/event-log")).unwrap();
        fs::write(src.join(PROVENANCE_FILENAME), "version = \"v0.4.0\"\n").unwrap();
        fs::write(src.join("skills/event-log/SKILL.md"), "# event log\n").unwrap();
        // Marker is now two lines: commit, then sha256.
        fs::write(cache_root.join(".fetch-complete"), "deadbeef\n\n").unwrap();

        let result = fetch(
            crate::processkit_vocab::PROCESSKIT_GIT_SOURCE,
            "v0.4.0",
            None,
            "src",
            None,
        )
        .expect("idempotent fetch should succeed without network");
        assert_eq!(result.cache_root, cache_root);
        assert_eq!(result.src_path, src);
        assert_eq!(result.resolved_commit.as_deref(), Some("deadbeef"));
        assert_eq!(result.release_asset_sha256, None);

        // Restore the env var so we don't pollute neighbouring tests.
        // SAFETY: same as the set_var above.
        unsafe {
            match prev {
                Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
                None => std::env::remove_var("XDG_CACHE_HOME"),
            }
        }
    }
}
