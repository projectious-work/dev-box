use anyhow::Result;

use crate::config::DevBoxConfig;
use crate::output;

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
    let result = ureq::get(url).header("User-Agent", "dev-box-cli").call();

    match result {
        Ok(response) => {
            // Unlikely for GHCR, but handle it.
            let body = response.into_body().read_to_string()?;
            let parsed: T = serde_json::from_str(&body)?;
            Ok(parsed)
        }
        Err(ureq::Error::StatusCode(401)) | Err(ureq::Error::StatusCode(403)) => {
            // Expected: exchange for anonymous Bearer token.
            let token_url = "https://ghcr.io/token?service=ghcr.io&scope=repository:projectious-work/dev-box:pull";
            let token_resp = ureq::get(token_url)
                .header("User-Agent", "dev-box-cli")
                .call()?;
            let token_body = token_resp.into_body().read_to_string()?;
            let token_data: TokenResponse = serde_json::from_str(&token_body)?;

            // Retry with token.
            let authed_resp = ureq::get(url)
                .header("User-Agent", "dev-box-cli")
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
    let url = "https://ghcr.io/v2/projectious-work/dev-box/tags/list";
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
    let url = "https://api.github.com/repos/projectious-work/dev-box/releases/latest";
    let response = ureq::get(url)
        .header("User-Agent", "dev-box-cli")
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

/// Update command implementation.
pub fn cmd_update(config_path: &Option<String>, check: bool) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;

    if check {
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
                         (https://github.com/projectious-work/dev-box/releases/latest)",
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
        let flavor = config.dev_box.image.to_string();
        output::ok(&format!(
            "Current config image version: {} ({})",
            config.dev_box.version, flavor
        ));

        match fetch_latest_image_version(&flavor) {
            Ok(latest) => {
                let current = semver::Version::parse(&config.dev_box.version)
                    .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
                if latest > current {
                    output::warn(&format!(
                        "New image version available for '{}': {} -> {} \
                         (update version in dev-box.toml and run: dev-box generate)",
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
    } else {
        output::info("Update mode");
        output::ok(&format!(
            "Current CLI version: {}",
            env!("CARGO_PKG_VERSION")
        ));
        output::warn("Automatic updates are not yet implemented.");
        output::info(
            "To update manually, change the version in dev-box.toml and run: dev-box generate",
        );
    }

    Ok(())
}
