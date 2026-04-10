//! aibox ↔ processkit compatibility table.
//!
//! Each entry maps an exact aibox CLI version to the processkit version it
//! was released with and tested against. This is the MINIMUM compatible
//! processkit version for that aibox release.
//!
//! When a project's `[processkit].version` in `aibox.toml` is older than
//! the minimum for the running aibox binary, `aibox sync` emits a warning.
//!
//! Update this table with every aibox release that changes processkit
//! compatibility. Keep entries in ascending version order.

/// One entry in the compatibility table.
pub struct CompatEntry {
    /// The exact aibox release version.
    pub aibox_version: &'static str,
    /// The processkit version this aibox was released with (minimum compatible).
    pub processkit_version: &'static str,
    /// Brief note on what changed in processkit at this boundary.
    pub note: &'static str,
}

/// Compatibility table: aibox version → minimum processkit version.
///
/// If your aibox version is not listed, use the entry for the closest
/// older listed version.
pub static COMPAT_TABLE: &[CompatEntry] = &[
    CompatEntry {
        aibox_version: "0.16.0",
        processkit_version: "v0.4.0",
        note: "initial processkit integration",
    },
    CompatEntry {
        aibox_version: "0.16.1",
        processkit_version: "v0.4.0",
        note: "sync auto-install added",
    },
    CompatEntry {
        aibox_version: "0.17.0",
        processkit_version: "v0.5.0",
        note: "aibox.lock sectioned format (DEC-037)",
    },
    CompatEntry {
        aibox_version: "0.17.2",
        processkit_version: "v0.6.0",
        note: "core skill enforcement, processkit v0.6.0 compat",
    },
    CompatEntry {
        aibox_version: "0.17.3",
        processkit_version: "v0.6.0",
        note: "Claude Code slash-command adapters (aibox#37)",
    },
    CompatEntry {
        aibox_version: "0.17.4",
        processkit_version: "v0.6.0",
        note: "content migration documents (pending/in-progress/applied)",
    },
    CompatEntry {
        aibox_version: "0.17.5",
        processkit_version: "v0.8.0",
        note: "processkit v0.8.0 GrandLily src/ restructure",
    },
    CompatEntry {
        aibox_version: "0.17.6",
        processkit_version: "v0.8.0",
        note: "migration briefing overhaul, structured logging, compat matrix",
    },
    CompatEntry {
        aibox_version: "0.17.7",
        processkit_version: "v0.8.0",
        note: "migration briefing accuracy fixes, version in help header",
    },
];

/// Find the minimum compatible processkit version for the given aibox version.
/// Returns `None` if the aibox version is older than any entry in the table.
pub fn min_processkit_for(aibox_version: &str) -> Option<&'static CompatEntry> {
    // Find the entry with the highest aibox_version that is <= aibox_version.
    // Versions are semver strings — parse them for comparison.
    let target = parse_semver(aibox_version)?;

    COMPAT_TABLE.iter().rfind(|e| {
        parse_semver(e.aibox_version)
            .map(|v| v <= target)
            .unwrap_or(false)
    })
}

/// Parse a semver string like "0.17.5" or "v0.17.5" into (major, minor, patch).
fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim_start_matches('v');
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Check if a processkit version string meets the minimum requirement.
/// Both strings should be like "v0.8.0" or "0.8.0".
pub fn processkit_meets_minimum(installed: &str, minimum: &str) -> bool {
    match (parse_semver(installed), parse_semver(minimum)) {
        (Some(inst), Some(min)) => inst >= min,
        _ => true, // if we can't parse, don't warn
    }
}
