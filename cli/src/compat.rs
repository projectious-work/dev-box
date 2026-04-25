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
    CompatEntry {
        aibox_version: "0.17.8",
        processkit_version: "v0.8.0",
        note: "migration briefing: distinguish sequential vs duplicate migrations",
    },
    CompatEntry {
        aibox_version: "0.17.9",
        processkit_version: "v0.8.0",
        note: "\"latest\" sentinel for aibox and processkit version fields",
    },
    CompatEntry {
        aibox_version: "0.17.10",
        processkit_version: "v0.8.0",
        note: "fix: validate() rejected \"latest\" in [aibox].version (regression from v0.17.9)",
    },
    CompatEntry {
        aibox_version: "0.17.11",
        processkit_version: "v0.8.0",
        note: "fix: [aibox].version = \"latest\" resolved to concrete image tag before Dockerfile generation",
    },
    CompatEntry {
        aibox_version: "0.17.12",
        processkit_version: "v0.8.0",
        note: "yazi git.yazi plugin; Linux/Windows gitignore entries; template-snapshot diff guidance in migration docs",
    },
    CompatEntry {
        aibox_version: "0.17.13",
        processkit_version: "v0.8.0",
        note: "fix: mandatory MCP server enforcement (closes #40); Rust addon linker + x86_64 cross-compile support; Zellij leader Ctrl+g; yazi git status indicators; zellij scratch pad",
    },
    CompatEntry {
        aibox_version: "0.17.14",
        processkit_version: "v0.8.0",
        note: "fix: docs-docusaurus addon installs @docusaurus/core (closes #41); pin default version to 3.8 (closes #42)",
    },
    CompatEntry {
        aibox_version: "0.17.15",
        processkit_version: "v0.13.0",
        note: "fix: gitignore OS patterns + .aibox/; gitignore + scaffold generated MCP client configs; [mcp] section in aibox.toml + .aibox-local.toml; Zellij leader hints via zjstatus; remove dangerous Ctrl+q from normal mode; restore deleted schemas/v1.0.0; docs updated",
    },
    CompatEntry {
        aibox_version: "0.17.16",
        processkit_version: "v0.13.0",
        note: "fix: zellij --layout flag position; Rust x86_64 target added in builder stage; rename ai provider 'codex' → 'openai' (BREAKING: update providers = [\"openai\"] in aibox.toml); add ai-openai addon to install.sh",
    },
    CompatEntry {
        aibox_version: "0.17.17",
        processkit_version: "v0.13.0",
        note: "aibox.toml inline addon documentation; ai-openai addon dep fix",
    },
    CompatEntry {
        aibox_version: "0.17.18",
        processkit_version: "v0.13.0",
        note: "fix: ai-openai addon npm install -g ran as USER aibox causing EACCES; fix: broken ai-codex link in ai-mistral docs",
    },
    CompatEntry {
        aibox_version: "0.17.19",
        processkit_version: "v0.13.0",
        note: "fix: rust addon COPY --from=rust-builder left .cargo/.rustup owned by root; add chown before USER aibox switch",
    },
    CompatEntry {
        aibox_version: "0.17.20",
        processkit_version: "v0.13.0",
        note: "runtime migration id collisions fix; codex auth persistence; preserve .aibox-home via runtime migrations; narrow reset backups; yazi/lazygit theme fixes",
    },
    CompatEntry {
        aibox_version: "0.18.0",
        processkit_version: "v0.13.0",
        note: "harness/provider split ([ai].harnesses + [ai].model_providers); theme auto-apply + WCAG audit; version resolution fixes; backward-compat for legacy [ai].providers",
    },
    CompatEntry {
        aibox_version: "0.18.1",
        processkit_version: "v0.13.0",
        note: "fix: rename ai-openai addon → ai-codex to match AiHarness::Codex addon_name(); add backward compat migration for [addons.ai-openai.tools]",
    },
    CompatEntry {
        aibox_version: "0.18.2",
        processkit_version: "v0.14.0",
        note: "yazi dir preview, git status signs, status bar, scratch pad removal",
    },
    CompatEntry {
        aibox_version: "0.18.3",
        processkit_version: "v0.17.0",
        note: "bump default processkit to v0.17.0; sync baseline-snapshot ordering fix; restore v0.14.0 baseline; 8-role AI-agent team scaffolding",
    },
    CompatEntry {
        aibox_version: "0.18.4",
        processkit_version: "v0.17.0",
        note: "INCOMPLETE RELEASE — tag cut before the multi-version-upgrade fixes landed; Cargo.toml was also not bumped so the shipped binary self-reports as 0.18.3. Skip this version; use 0.18.5 or later.",
    },
    CompatEntry {
        aibox_version: "0.18.5",
        processkit_version: "v0.18.1",
        note: "catch-up release: completes the 0.18.4 work (multi-version upgrade gaps closed); bumps default processkit to v0.18.1 (hookEventName hotfix + src↔context parity); hook commands now use $CLAUDE_PROJECT_DIR so they work regardless of Claude Code's launch cwd; maintain.sh release now writes Cargo.toml + refreshes Cargo.lock before tagging.",
    },
    CompatEntry {
        aibox_version: "0.18.6",
        processkit_version: "v0.18.1",
        note: "MCP-merge release: fixes the flat one-level walker bug in mcp_registration.rs and claude_commands.rs that prevented .mcp.json and .claude/commands/ from being populated against the category-nested skills tree (aibox#53); promotes skill-gate to MANDATORY_MCP_SKILLS so acknowledge_contract() is reachable on every harness session and the PreToolUse compliance gate is satisfiable out of the box; adds collision guard for duplicate skill basenames across categories; repairs cmd_docs_deploy (gh-pages worktree git identity + tmpdir unbound trap).",
    },
    CompatEntry {
        aibox_version: "0.18.7",
        processkit_version: "v0.18.2",
        note: "MCP safety + ergonomics release: hard-fail safety rail in mcp_registration.rs validates every merged MCP script path exists on disk (caught the 12 stale processkit-side mcp-config.json paths reported as processkit#8, fixed upstream in processkit v0.18.2); compliance contract drift checker now tolerant of v1 OR v2 markers (Option C — bridges the transitional state where AGENTS.md template ships v2 but skill-gate's contract source is still v1); devcontainer drift fix — generated file headers no longer stamp the live CLI version, and aibox.lock preserves prior synced_at / installed_at when nothing else changed (clean container rebuild is now a true no-op for git status); gh-pages auto-config probe-first (eliminates the spurious 'Could not configure Pages automatically' warning on every release when Pages is already managed); ships .opencode/plugins/processkit-gate.ts to enforce the compliance contract on OpenCode sessions (closes aibox#51, requires upstream sst/opencode#2319 and #5894 — both shipped).",
    },
    CompatEntry {
        aibox_version: "0.19.0",
        processkit_version: "v0.21.0",
        note: "Minor release: integrates processkit v0.21.0 (major upstream update with enhanced content structure); ships global MCP permission configuration across 8 harnesses (Claude Code, OpenCode, Continue, Cursor, Gemini, Copilot, Aider, Codex) via [mcp.permissions] in aibox.toml with glob pattern matching and deny-precedence semantics; completed backlog grooming with 90-day focus established; all tests passing (597 unit + 41 E2E + 16 integration).",
    },
    CompatEntry {
        aibox_version: "0.19.1",
        processkit_version: "v0.21.0",
        note: "Patch release: applies processkit v0.21.0 migration (564 new files: skills, schemas, roles, bindings, models); updates aibox runtime templates (27 new .aibox-home/* files); all 654 tests passing (unit + integration + E2E).",
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

/// Return the slice of `COMPAT_TABLE` entries whose `aibox_version` is
/// strictly greater than `from_excl` and less than or equal to `to_incl`.
/// Used by the migration document generator to enumerate every released
/// intermediate when a project jumps across multiple CLI versions.
///
/// If either bound fails to parse as semver, falls back to `&[]` (callers
/// downgrade to the generic target-only rendering).
pub fn entries_in_range(from_excl: &str, to_incl: &str) -> Vec<&'static CompatEntry> {
    let (Some(from_v), Some(to_v)) = (parse_semver(from_excl), parse_semver(to_incl)) else {
        return Vec::new();
    };
    if from_v >= to_v {
        return Vec::new();
    }
    COMPAT_TABLE
        .iter()
        .filter(|e| match parse_semver(e.aibox_version) {
            Some(v) => v > from_v && v <= to_v,
            None => false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Release hygiene: every time the CLI version bumps, a corresponding
    /// `COMPAT_TABLE` entry must be added. This test fails the release if
    /// the table is out of date, so the omission is caught before ship.
    #[test]
    fn cargo_pkg_version_has_compat_entry() {
        let cargo = env!("CARGO_PKG_VERSION");
        let found = COMPAT_TABLE.iter().any(|e| e.aibox_version == cargo);
        assert!(
            found,
            "CARGO_PKG_VERSION = {cargo} has no entry in COMPAT_TABLE              (cli/src/compat.rs) — add one alongside the version bump"
        );
    }

    #[test]
    fn entries_in_range_basic() {
        let got: Vec<&str> = entries_in_range("0.17.9", "0.17.12")
            .iter()
            .map(|e| e.aibox_version)
            .collect();
        assert_eq!(got, vec!["0.17.10", "0.17.11", "0.17.12"]);
    }

    #[test]
    fn entries_in_range_cross_minor() {
        let got: Vec<&str> = entries_in_range("0.17.20", "0.18.2")
            .iter()
            .map(|e| e.aibox_version)
            .collect();
        // 0.18.0, 0.18.1, 0.18.2 all must appear.
        assert_eq!(got, vec!["0.18.0", "0.18.1", "0.18.2"]);
    }

    #[test]
    fn entries_in_range_same_version_is_empty() {
        assert!(entries_in_range("0.18.3", "0.18.3").is_empty());
    }

    #[test]
    fn entries_in_range_descending_is_empty() {
        assert!(entries_in_range("0.18.3", "0.17.10").is_empty());
    }

    #[test]
    fn entries_in_range_bad_input_is_empty() {
        assert!(entries_in_range("bogus", "0.18.3").is_empty());
    }
}
