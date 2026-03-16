use anyhow::Result;

use crate::config::DevBoxConfig;
use crate::output;

/// Update command implementation.
pub fn cmd_update(config_path: &Option<String>, check: bool) -> Result<()> {
    let config = DevBoxConfig::from_cli_option(config_path)?;

    if check {
        output::info("Checking for updates...");
        output::ok(&format!(
            "Current CLI version: {}",
            env!("CARGO_PKG_VERSION")
        ));
        output::ok(&format!(
            "Current config version: {}",
            config.dev_box.version
        ));
        output::ok(&format!(
            "Schema version: {}",
            config.context.schema_version
        ));
        output::warn(
            "Registry checking is not yet implemented. \
             Check https://github.com/projectious-work/dev-box/releases for new versions.",
        );
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
