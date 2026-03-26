use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::AiboxConfig;
use crate::{generate, output};

// --- Response structs for JSON deserialization ---

#[derive(serde::Deserialize)]
struct TagsList {
    tags: Vec<String>,
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(serde::Deserialize)]
struct GhRelease {
    tag_name: String,
}

// --- Helper: authenticated GET against GHCR (OCI token exchange) ---

/// Perform a GET request against GHCR, handling anonymous token exchange.
///
/// GHCR returns 401 on unauthenticated requests. We exchange for an anonymous
/// Bearer token via the token endpoint, then retry with the token.
fn ghcr_get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    // First, try the request directly — it will almost certainly 401.
    let result = ureq::get(url).header("User-Agent", "aibox-cli").call();

    match result {
        Ok(response) => {
            // Unlikely for GHCR, but handle it.
            let body = response.into_body().read_to_string()?;
            let parsed: T = serde_json::from_str(&body)?;
            Ok(parsed)
        }
        Err(ureq::Error::StatusCode(401)) | Err(ureq::Error::StatusCode(403)) => {
            // Expected: exchange for anonymous Bearer token.
            let token_url = "https://ghcr.io/token?service=ghcr.io&scope=repository:projectious-work/aibox:pull";
            let token_resp = ureq::get(token_url)
                .header("User-Agent", "aibox-cli")
                .call()?;
            let token_body = token_resp.into_body().read_to_string()?;
            let token_data: TokenResponse = serde_json::from_str(&token_body)?;

            // Retry with token.
            let authed_resp = ureq::get(url)
                .header("User-Agent", "aibox-cli")
                .header("Authorization", &format!("Bearer {}", token_data.token))
                .call()?;
            let authed_body = authed_resp.into_body().read_to_string()?;
            let parsed: T = serde_json::from_str(&authed_body)?;
            Ok(parsed)
        }
        Err(e) => Err(e.into()),
    }
}

// --- Fetch latest image version from GHCR tags ---

/// Query the GHCR tags list for the given image flavor and return the highest
/// semver version found.
fn fetch_latest_image_version(flavor: &str) -> Result<semver::Version> {
    let url = "https://ghcr.io/v2/projectious-work/aibox/tags/list";
    let tags_list: TagsList = ghcr_get_json(url)?;

    let prefix = format!("{}-v", flavor);
    let mut versions: Vec<semver::Version> = tags_list
        .tags
        .iter()
        .filter_map(|tag| {
            tag.strip_prefix(&prefix)
                .and_then(|v| semver::Version::parse(v).ok())
        })
        .collect();

    if versions.is_empty() {
        anyhow::bail!("No published tags found for flavor '{}'", flavor);
    }

    versions.sort();
    Ok(versions.pop().unwrap())
}

// --- Fetch latest CLI version from GitHub releases ---

/// Query the GitHub releases API for the latest release tag and parse it as
/// a semver version.
fn fetch_latest_cli_version() -> Result<semver::Version> {
    let url = "https://api.github.com/repos/projectious-work/aibox/releases/latest";
    let response = ureq::get(url)
        .header("User-Agent", "aibox-cli")
        .header("Accept", "application/vnd.github+json")
        .call()?;
    let body = response.into_body().read_to_string()?;
    let release: GhRelease = serde_json::from_str(&body)?;

    let version_str = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let version = semver::Version::parse(version_str)?;
    Ok(version)
}

/// Check for available updates (CLI + image versions).
fn check_updates(config: &AiboxConfig) -> Result<()> {
    output::info("Checking for updates...");

    // --- CLI version ---
    let current_cli = env!("CARGO_PKG_VERSION");
    output::ok(&format!("Current CLI version: {}", current_cli));

    match fetch_latest_cli_version() {
        Ok(latest) => {
            let current = semver::Version::parse(current_cli)
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
            if latest > current {
                output::warn(&format!(
                    "New CLI version available: {} -> {} \
                     (https://github.com/projectious-work/aibox/releases/latest)",
                    current, latest
                ));
            } else {
                output::ok("CLI is up to date");
            }
        }
        Err(e) => {
            output::warn(&format!("Could not check latest CLI version: {}", e));
        }
    }

    // --- Image version ---
    let flavor = config.aibox.base.to_string();
    output::ok(&format!(
        "Current config image version: {} ({})",
        config.aibox.version, flavor
    ));

    match fetch_latest_image_version(&flavor) {
        Ok(latest) => {
            let current = semver::Version::parse(&config.aibox.version)
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
            if latest > current {
                output::warn(&format!(
                    "New image version available for '{}': {} -> {} \
                     (run 'aibox update' to upgrade)",
                    flavor, current, latest
                ));
            } else {
                output::ok(&format!("Image '{}' is up to date", flavor));
            }
        }
        Err(e) => {
            output::warn(&format!(
                "Could not check latest image version for '{}': {}",
                flavor, e
            ));
        }
    }

    // --- Schema version (informational) ---
    output::ok(&format!(
        "Schema version: {}",
        config.context.schema_version
    ));

    Ok(())
}

/// Resolve the config file path, preferring the CLI option, then default.
fn resolve_config_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

/// Update the version field in aibox.toml using string replacement to preserve comments.
fn update_toml_version(toml_path: &Path, old_version: &str, new_version: &str) -> Result<()> {
    let content =
        std::fs::read_to_string(toml_path).context("Failed to read aibox.toml for upgrade")?;

    // Replace the version value in the [aibox] section.
    // Match `version = "X.Y.Z"` pattern — only the first occurrence (in [aibox]).
    let old_pattern = format!("version = \"{}\"", old_version);
    let new_pattern = format!("version = \"{}\"", new_version);

    if !content.contains(&old_pattern) {
        anyhow::bail!(
            "Could not find '{}' in aibox.toml — manual edit may be needed",
            old_pattern
        );
    }

    let updated = content.replacen(&old_pattern, &new_pattern, 1);
    std::fs::write(toml_path, updated).context("Failed to write updated aibox.toml")?;

    Ok(())
}

/// Perform the upgrade: fetch latest image version, update aibox.toml, regenerate files.
fn do_upgrade(config_path: &Option<String>, dry_run: bool) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let flavor = config.aibox.base.to_string();
    let current_version = &config.aibox.version;

    output::info(&format!(
        "Current image version: {} ({})",
        current_version, flavor
    ));

    // Fetch latest image version from GHCR
    output::info("Fetching latest image version from registry...");
    let latest = match fetch_latest_image_version(&flavor) {
        Ok(v) => v,
        Err(e) => {
            output::warn(&format!(
                "Could not fetch latest image version from registry: {}\n\
                 If the registry requires authentication, try: docker login ghcr.io",
                e
            ));
            return Ok(());
        }
    };
    let current = semver::Version::parse(current_version)
        .unwrap_or_else(|_| semver::Version::new(0, 0, 0));

    if latest <= current {
        output::ok(&format!(
            "Image '{}' is already at the latest version ({})",
            flavor, current
        ));
        return Ok(());
    }

    let latest_str = latest.to_string();
    output::ok(&format!(
        "New version available: {} -> {}",
        current, latest_str
    ));

    if dry_run {
        output::info("[dry-run] Would update version in aibox.toml");
        output::info("[dry-run] Would regenerate .devcontainer/ files");
        output::info("[dry-run] Would update .aibox-version");
        return Ok(());
    }

    // 1. Update version in aibox.toml
    let toml_path = resolve_config_path(config_path);
    update_toml_version(&toml_path, current_version, &latest_str)?;
    output::ok(&format!(
        "Updated version in {} ({} -> {})",
        toml_path.display(),
        current_version,
        latest_str
    ));

    // 2. Reload config with updated version and regenerate container files
    let updated_config = AiboxConfig::from_cli_option(config_path)?;
    generate::generate_all(&updated_config)?;

    // 3. Update .aibox-version
    let version_file = Path::new(".aibox-version");
    std::fs::write(version_file, &latest_str).context("Failed to update .aibox-version")?;
    output::ok("Updated .aibox-version");

    output::ok(&format!(
        "Upgrade complete: {} -> {} — rebuild your container to apply changes",
        current, latest_str
    ));

    Ok(())
}

/// Update command implementation.
pub fn cmd_update(config_path: &Option<String>, check: bool, dry_run: bool) -> Result<()> {
    if check {
        let config = AiboxConfig::from_cli_option(config_path)?;
        check_updates(&config)
    } else {
        do_upgrade(config_path, dry_run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_toml_version_replaces_version() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        let content = r#"[aibox]
version = "0.3.5"
image = "python"
process = "research"

[container]
name = "my-project"
"#;
        std::fs::write(&toml_path, content).unwrap();

        update_toml_version(&toml_path, "0.3.5", "0.3.7").unwrap();

        let updated = std::fs::read_to_string(&toml_path).unwrap();
        assert!(updated.contains("version = \"0.3.7\""));
        assert!(!updated.contains("version = \"0.3.5\""));
        // Ensure the rest is preserved
        assert!(updated.contains("image = \"python\""));
        assert!(updated.contains("name = \"my-project\""));
    }

    #[test]
    fn update_toml_version_preserves_comments() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        let content = r#"# My project config
[aibox]
# Image version from GHCR
version = "0.2.1"
image = "base"
process = "minimal"
"#;
        std::fs::write(&toml_path, content).unwrap();

        update_toml_version(&toml_path, "0.2.1", "0.3.0").unwrap();

        let updated = std::fs::read_to_string(&toml_path).unwrap();
        assert!(updated.contains("# My project config"));
        assert!(updated.contains("# Image version from GHCR"));
        assert!(updated.contains("version = \"0.3.0\""));
    }

    #[test]
    fn update_toml_version_fails_on_missing_version() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        std::fs::write(&toml_path, "[aibox]\nimage = \"base\"\n").unwrap();

        let result = update_toml_version(&toml_path, "0.3.5", "0.3.7");
        assert!(result.is_err());
    }

    #[test]
    fn update_toml_version_only_replaces_first_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        // Hypothetical: version appears in a comment too
        let content = r#"[aibox]
version = "0.3.5"
image = "base"
process = "minimal"

# Note: schema version = "0.3.5" was also used elsewhere
"#;
        std::fs::write(&toml_path, content).unwrap();

        update_toml_version(&toml_path, "0.3.5", "0.4.0").unwrap();

        let updated = std::fs::read_to_string(&toml_path).unwrap();
        assert!(updated.contains("version = \"0.4.0\""));
        // The comment text should still contain the old version string
        assert!(updated.contains("0.3.5"));
    }
}
