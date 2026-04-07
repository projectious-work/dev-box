//! Fetches a tagged version of a processkit-compatible source into a local
//! cache directory. Supports tarball download (preferred, for github.com and
//! gitlab-flavored hosts) and a `git clone` fallback for arbitrary git URLs.
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
//! Cache entries are immutable once written. Re-fetching the same
//! `(source, version)` is a no-op: the function detects the
//! `.fetch-complete` marker and returns the existing path.
//!
//! ## Authentication
//!
//! A3 only does anonymous fetches. Tarball downloads use plain HTTPS;
//! the git-clone fallback inherits whatever credential helper the user
//! has configured for git (e.g. `gh auth setup-git`). Private repos
//! work if and only if the user's environment is already authenticated
//! out-of-band — A3 does not prompt or manage tokens itself.
//!
//! ## Verification
//!
//! After fetching, [`fetch`] calls [`validate_cache`] to ensure the
//! result looks like processkit (specifically, that
//! `<src_path>/PROVENANCE.toml` exists). If validation fails, the
//! incomplete cache is left in place WITHOUT the `.fetch-complete`
//! marker, so a subsequent fetch will retry from scratch.

use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::PROCESSKIT_VERSION_UNSET;
use crate::output;

/// Result of a successful fetch.
#[derive(Debug, Clone)]
pub struct FetchedSource {
    /// `~/.cache/aibox/processkit/<host>/<org>/<name>/<version>/`
    pub cache_root: PathBuf,
    /// `cache_root.join(src_path)` — the directory whose contents
    /// represent the "shipped" processkit payload.
    pub src_path: PathBuf,
    /// Resolved git commit, when known. Always populated for the
    /// git-clone path; `None` for the tarball path because GitHub /
    /// GitLab tag tarballs do not encode the commit sha in the URL.
    pub resolved_commit: Option<String>,
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
// Public fetch entry point
// ---------------------------------------------------------------------------

/// Fetch the given `source@version` into the cache.
///
/// If `branch` is `Some`, the moving-branch fallback path is used and a
/// git clone of that branch is performed (the discouraged "track HEAD"
/// mode from A1). Otherwise the function tries the tarball fast path
/// for known hosts, then falls back to a tag-pinned `git clone`.
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
        let src = cache_root.join(src_path);
        // Defensive validation in case someone hand-edited the cache.
        validate_cache(&cache_root, src_path).with_context(|| {
            format!(
                "cached fetch at {} failed validation; remove the cache and retry",
                cache_root.display()
            )
        })?;
        return Ok(FetchedSource {
            cache_root,
            src_path: src,
            resolved_commit: read_commit_marker(&marker),
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

    // Strategy:
    //   1. If `branch` is set, always git-clone that branch.
    //   2. Otherwise, try host-specific tarball fast path.
    //   3. Otherwise, fall back to tag-pinned git clone.
    let used_tarball = if let Some(b) = branch {
        output::info(&format!(
            "Cloning processkit branch '{}' from {} (moving-branch mode)",
            b, source
        ));
        resolved_commit = Some(git_clone(source, Some(b), &cache_root)?);
        false
    } else if let Some(url) = host_tarball_url(&parsed, version) {
        output::info(&format!(
            "Downloading processkit tarball {} -> {}",
            url,
            cache_root.display()
        ));
        match download_and_extract_tarball(&url, &cache_root) {
            Ok(()) => true,
            Err(tar_err) => {
                output::warn(&format!(
                    "tarball download failed ({}); falling back to git clone",
                    tar_err
                ));
                // Reset cache_root and retry via git clone.
                if cache_root.exists() {
                    fs::remove_dir_all(&cache_root)?;
                }
                fs::create_dir_all(&cache_root)?;
                resolved_commit = Some(git_clone(source, Some(version), &cache_root)?);
                false
            }
        }
    } else {
        output::info(&format!(
            "Cloning processkit tag '{}' from {}",
            version, source
        ));
        resolved_commit = Some(git_clone(source, Some(version), &cache_root)?);
        false
    };

    // Verify result before declaring success.
    validate_cache(&cache_root, src_path).with_context(|| {
        format!(
            "fetched source at {} does not look like processkit",
            cache_root.display()
        )
    })?;

    // Write the marker LAST. Body is the resolved commit (if any) so we
    // can echo it on subsequent idempotent calls.
    let marker_body = resolved_commit.clone().unwrap_or_default();
    let mut f = fs::File::create(&marker)
        .with_context(|| format!("failed to write marker {}", marker.display()))?;
    f.write_all(marker_body.as_bytes())?;

    let strategy = if used_tarball { "tarball" } else { "git clone" };
    output::ok(&format!(
        "Fetched processkit {} via {} into {}",
        version,
        strategy,
        cache_root.display()
    ));

    Ok(FetchedSource {
        cache_root: cache_root.clone(),
        src_path: cache_root.join(src_path),
        resolved_commit,
        source: source.to_string(),
        version: version.to_string(),
    })
}

/// Read the resolved commit recorded in the `.fetch-complete` marker, if any.
fn read_commit_marker(marker: &Path) -> Option<String> {
    fs::read_to_string(marker)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Cache validation
// ---------------------------------------------------------------------------

/// Sanity-check that a fetched cache contains the processkit shape.
/// Returns `Ok(())` if `<cache_root>/<src_path>/PROVENANCE.toml` exists.
pub fn validate_cache(cache_root: &Path, src_path: &str) -> Result<()> {
    let provenance = cache_root.join(src_path).join("PROVENANCE.toml");
    if !provenance.exists() {
        bail!(
            "fetched source does not look like processkit (missing {}). \
             If this is a pre-v0.4.0 processkit, upgrade the version pin to v0.4.0+.",
            provenance.display()
        );
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

    // -- URL parsing --------------------------------------------------------

    #[test]
    fn cache_dir_parses_github_https_url() {
        let p = cache_dir(
            "https://github.com/projectious-work/processkit.git",
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

    // -- Sentinel rejection -------------------------------------------------

    #[test]
    fn fetch_refuses_unset_sentinel() {
        let err = fetch(
            "https://github.com/foo/bar.git",
            PROCESSKIT_VERSION_UNSET,
            None,
            "src",
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
        fs::write(src.join("PROVENANCE.toml"), "version = \"v0.4.0\"\n").unwrap();
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
            "https://github.com/projectious-work/processkit.git",
            "v0.4.0",
        )
        .unwrap();
        // Synthesize the cache contents.
        let src = cache_root.join("src");
        fs::create_dir_all(src.join("skills/event-log")).unwrap();
        fs::write(src.join("PROVENANCE.toml"), "version = \"v0.4.0\"\n").unwrap();
        fs::write(src.join("skills/event-log/SKILL.md"), "# event log\n").unwrap();
        fs::write(cache_root.join(".fetch-complete"), "deadbeef").unwrap();

        let result = fetch(
            "https://github.com/projectious-work/processkit.git",
            "v0.4.0",
            None,
            "src",
        )
        .expect("idempotent fetch should succeed without network");
        assert_eq!(result.cache_root, cache_root);
        assert_eq!(result.src_path, src);
        assert_eq!(result.resolved_commit.as_deref(), Some("deadbeef"));

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
