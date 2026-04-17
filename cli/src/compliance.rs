//! Compliance-contract helpers — surface the processkit skill-gate compliance
//! contract to each AI harness at sync time.
//!
//! Three things happen on every `aibox sync`:
//!
//! 1. **Drift detection** (Issue #46): compare the
//!    `<!-- pk-compliance-contract v1 BEGIN -->…END` block embedded in
//!    `AGENTS.md` against the canonical source file
//!    `context/skills/processkit/skill-gate/assets/compliance-contract.md`.
//!    If they differ, emit a warning.
//!
//! 2. **Cursor rules** (Issue #47): when `cursor` is in `[ai].harnesses`,
//!    write `.cursor/rules/processkit-compliance.md` from the canonical
//!    source.
//!
//! 3. **Aider conf** (Issue #48): when `aider` is in `[ai].harnesses`,
//!    ensure `.aider.conf.yml` contains `read:` entries for `AGENTS.md`
//!    and `context/skills/processkit/skill-gate/assets/compliance-contract.md`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::{AiHarness, AiboxConfig};
use crate::output;

// ---------------------------------------------------------------------------
// Canonical paths
// ---------------------------------------------------------------------------

/// Project-root-relative path to the canonical compliance contract.
const COMPLIANCE_CONTRACT_REL: &str =
    "context/skills/processkit/skill-gate/assets/compliance-contract.md";

/// BEGIN marker for the embedded block in AGENTS.md (v1).
const AGENTS_BLOCK_BEGIN: &str = "<!-- pk-compliance-contract v1 BEGIN -->";
/// END marker for the embedded block in AGENTS.md (v1).
const AGENTS_BLOCK_END: &str = "<!-- pk-compliance-contract v1 END -->";

// ---------------------------------------------------------------------------
// Issue #46 — drift detection
// ---------------------------------------------------------------------------

/// Compare the compliance-contract block in `AGENTS.md` against the canonical
/// source. Prints a warning if they differ; returns `Ok(())` in all cases
/// (drift is non-fatal).
pub fn check_compliance_contract_drift(project_root: &Path) -> Result<()> {
    let canonical_path = project_root.join(COMPLIANCE_CONTRACT_REL);
    let agents_path = project_root.join("AGENTS.md");

    // If either file is absent we can't compare — skip silently.
    if !canonical_path.is_file() || !agents_path.is_file() {
        return Ok(());
    }

    let canonical = fs::read_to_string(&canonical_path).with_context(|| {
        format!(
            "reading canonical compliance contract: {}",
            canonical_path.display()
        )
    })?;

    let agents_content = fs::read_to_string(&agents_path)
        .with_context(|| format!("reading AGENTS.md: {}", agents_path.display()))?;

    let embedded = extract_block(&agents_content, AGENTS_BLOCK_BEGIN, AGENTS_BLOCK_END);

    match embedded {
        None => {
            // Block markers not present — nothing to compare.
        }
        Some(block) => {
            if block.trim() != canonical.trim() {
                output::warn(
                    "Compliance contract in AGENTS.md differs from the canonical source at \
                     context/skills/processkit/skill-gate/assets/compliance-contract.md.",
                );
                output::warn("Run `aibox sync --fix-compliance-contract` to update AGENTS.md.");
            }
        }
    }

    Ok(())
}

/// Extract the content between `begin_marker` and `end_marker` (exclusive).
/// Returns `None` if either marker is absent.
fn extract_block<'a>(text: &'a str, begin_marker: &str, end_marker: &str) -> Option<&'a str> {
    let start = text.find(begin_marker).map(|i| i + begin_marker.len())?;
    let end = text[start..].find(end_marker).map(|i| start + i)?;
    Some(&text[start..end])
}

// ---------------------------------------------------------------------------
// Issue #47 — Cursor rules
// ---------------------------------------------------------------------------

/// Write `.cursor/rules/processkit-compliance.md` from the canonical
/// compliance contract. Only runs when `cursor` is in `[ai].harnesses`.
pub fn write_cursor_compliance_rules(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    if !config.ai.harnesses.contains(&AiHarness::Cursor) {
        return Ok(());
    }

    let canonical_path = project_root.join(COMPLIANCE_CONTRACT_REL);
    if !canonical_path.is_file() {
        // Canonical file not yet installed — skip silently.
        return Ok(());
    }

    let content = fs::read_to_string(&canonical_path).with_context(|| {
        format!(
            "reading canonical compliance contract: {}",
            canonical_path.display()
        )
    })?;

    let rules_dir = project_root.join(".cursor").join("rules");
    fs::create_dir_all(&rules_dir).with_context(|| format!("creating {}", rules_dir.display()))?;

    let dest = rules_dir.join("processkit-compliance.md");
    fs::write(&dest, &content).with_context(|| format!("writing {}", dest.display()))?;

    output::ok(&format!("Wrote compliance contract to {}", dest.display()));

    Ok(())
}

// ---------------------------------------------------------------------------
// Issue #48 — Aider conf
// ---------------------------------------------------------------------------

/// Ensure `.aider.conf.yml` has `read:` entries for `AGENTS.md` and the
/// compliance contract. Only runs when `aider` is in `[ai].harnesses`.
/// Preserves all existing entries.
pub fn write_aider_compliance_conf(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    if !config.ai.harnesses.contains(&AiHarness::Aider) {
        return Ok(());
    }

    let conf_path = project_root.join(".aider.conf.yml");

    // Read existing YAML (if any).
    let mut doc: serde_yaml::Value = if conf_path.is_file() {
        let raw = fs::read_to_string(&conf_path)
            .with_context(|| format!("reading {}", conf_path.display()))?;
        serde_yaml::from_str(&raw).unwrap_or(serde_yaml::Value::Mapping(Default::default()))
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };

    // Ensure the root is a mapping.
    if !doc.is_mapping() {
        doc = serde_yaml::Value::Mapping(Default::default());
    }

    let required: &[&str] = &["AGENTS.md", COMPLIANCE_CONTRACT_REL];

    let mapping = doc.as_mapping_mut().expect("root is a mapping");
    let read_key = serde_yaml::Value::String("read".to_string());

    // Get or create the `read:` sequence.
    let read_entry = mapping
        .entry(read_key)
        .or_insert_with(|| serde_yaml::Value::Sequence(vec![]));

    if let serde_yaml::Value::Sequence(seq) = read_entry {
        for &entry in required {
            let v = serde_yaml::Value::String(entry.to_string());
            if !seq.contains(&v) {
                seq.push(v);
            }
        }
    }

    let yaml_str = serde_yaml::to_string(&doc).with_context(|| "serialising .aider.conf.yml")?;

    fs::write(&conf_path, &yaml_str).with_context(|| format!("writing {}", conf_path.display()))?;

    output::ok(&format!(
        "Updated .aider.conf.yml with compliance contract read: entries"
    ));

    Ok(())
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run all three compliance-contract sync steps. Best-effort: failures in
/// individual steps are warned-and-continued so they do not abort the sync.
pub fn regenerate_compliance_configs(config: &AiboxConfig, project_root: &Path) -> Result<()> {
    if let Err(e) = check_compliance_contract_drift(project_root) {
        output::warn(&format!("Compliance drift check failed: {}", e));
    }
    if let Err(e) = write_cursor_compliance_rules(config, project_root) {
        output::warn(&format!("Cursor compliance rules failed: {}", e));
    }
    if let Err(e) = write_aider_compliance_conf(config, project_root) {
        output::warn(&format!("Aider compliance conf failed: {}", e));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: build a minimal AiboxConfig with the given harnesses.
    fn config_with_harnesses(harnesses: Vec<AiHarness>) -> AiboxConfig {
        let toml_str = format!(
            "[aibox]\nversion = \"0.18.2\"\n\n[container]\nname = \"test\"\n\n[ai]\nharnesses = [{harnesses}]\n",
            harnesses = harnesses
                .iter()
                .map(|h| format!("\"{}\"", h))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut config: AiboxConfig = toml::from_str(&toml_str).expect("parse test config");
        config.ai.migrate_legacy();
        config
    }

    fn write_canonical(root: &Path) {
        let dir = root
            .join("context")
            .join("skills")
            .join("processkit")
            .join("skill-gate")
            .join("assets");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("compliance-contract.md"), "canonical content\n").unwrap();
    }

    // -----------------------------------------------------------------------
    // Issue #46 — drift detection
    // -----------------------------------------------------------------------

    #[test]
    fn drift_detected_when_blocks_differ() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        // Write AGENTS.md with a stale embedded block.
        let agents = format!(
            "{}\nstale content\n{}",
            AGENTS_BLOCK_BEGIN, AGENTS_BLOCK_END
        );
        fs::write(root.join("AGENTS.md"), agents).unwrap();

        // Should complete without error (drift is non-fatal).
        check_compliance_contract_drift(root).expect("should not error");
        // The warning is emitted to stderr — we verify the function runs
        // without returning Err and that the comparison logic executes.
    }

    #[test]
    fn no_drift_when_blocks_match() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let agents = format!(
            "{}\ncanonical content\n{}",
            AGENTS_BLOCK_BEGIN, AGENTS_BLOCK_END
        );
        fs::write(root.join("AGENTS.md"), agents).unwrap();

        check_compliance_contract_drift(root).expect("should not error");
    }

    #[test]
    fn drift_skipped_when_canonical_missing() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // No canonical file — should not error.
        fs::write(root.join("AGENTS.md"), "no markers here").unwrap();
        check_compliance_contract_drift(root).expect("should not error");
    }

    // -----------------------------------------------------------------------
    // Issue #47 — Cursor rules
    // -----------------------------------------------------------------------

    #[test]
    fn cursor_rules_written_when_cursor_in_harnesses() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let config = config_with_harnesses(vec![AiHarness::Cursor]);
        write_cursor_compliance_rules(&config, root).expect("should succeed");

        let dest = root
            .join(".cursor")
            .join("rules")
            .join("processkit-compliance.md");
        assert!(
            dest.is_file(),
            ".cursor/rules/processkit-compliance.md should exist"
        );
        assert_eq!(
            fs::read_to_string(&dest).unwrap().trim(),
            "canonical content"
        );
    }

    #[test]
    fn cursor_rules_skipped_when_cursor_not_in_harnesses() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        write_canonical(root);

        let config = config_with_harnesses(vec![AiHarness::Claude]);
        write_cursor_compliance_rules(&config, root).expect("should succeed");

        let dest = root
            .join(".cursor")
            .join("rules")
            .join("processkit-compliance.md");
        assert!(
            !dest.exists(),
            "file should not be written for non-cursor harness"
        );
    }

    // -----------------------------------------------------------------------
    // Issue #48 — Aider conf
    // -----------------------------------------------------------------------

    #[test]
    fn aider_conf_created_with_required_entries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let config = config_with_harnesses(vec![AiHarness::Aider]);
        write_aider_compliance_conf(&config, root).expect("should succeed");

        let conf_path = root.join(".aider.conf.yml");
        assert!(conf_path.is_file());

        let raw = fs::read_to_string(&conf_path).unwrap();
        assert!(raw.contains("AGENTS.md"), "should contain AGENTS.md");
        assert!(
            raw.contains(COMPLIANCE_CONTRACT_REL),
            "should contain compliance contract path"
        );
    }

    #[test]
    fn aider_conf_preserves_existing_entries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Pre-existing conf with a custom read entry.
        let existing = "read:\n  - some-custom-file.md\n";
        fs::write(root.join(".aider.conf.yml"), existing).unwrap();

        let config = config_with_harnesses(vec![AiHarness::Aider]);
        write_aider_compliance_conf(&config, root).expect("should succeed");

        let raw = fs::read_to_string(root.join(".aider.conf.yml")).unwrap();
        assert!(
            raw.contains("some-custom-file.md"),
            "custom entry should be preserved"
        );
        assert!(raw.contains("AGENTS.md"), "AGENTS.md should be added");
        assert!(
            raw.contains(COMPLIANCE_CONTRACT_REL),
            "compliance path should be added"
        );
    }

    #[test]
    fn aider_conf_idempotent_does_not_duplicate() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let config = config_with_harnesses(vec![AiHarness::Aider]);

        // Run twice.
        write_aider_compliance_conf(&config, root).expect("first run");
        write_aider_compliance_conf(&config, root).expect("second run");

        let raw = fs::read_to_string(root.join(".aider.conf.yml")).unwrap();
        // Each required entry should appear exactly once.
        assert_eq!(raw.matches("AGENTS.md").count(), 1, "AGENTS.md duplicated");
        assert_eq!(
            raw.matches(COMPLIANCE_CONTRACT_REL).count(),
            1,
            "compliance path duplicated"
        );
    }

    #[test]
    fn aider_conf_skipped_when_aider_not_in_harnesses() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let config = config_with_harnesses(vec![AiHarness::Claude]);
        write_aider_compliance_conf(&config, root).expect("should succeed");

        assert!(
            !root.join(".aider.conf.yml").exists(),
            ".aider.conf.yml should not be created for non-aider harness"
        );
    }
}
