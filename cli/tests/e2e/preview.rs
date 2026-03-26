//! File preview tests — verify yazi previewer configuration is correctly seeded.
//!
//! Checks that after `aibox init`:
//!   - The yazi plugin files for SVG and EPS are present in .aibox-home
//!   - The yazi.toml contains the expected [plugin] prepend_previewers entries
//!
//! These are Tier 1 tests: no running container needed.

use std::fs;
use std::process::Command;

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox")
}

/// Initialize a project in a temp dir with default settings.
fn init_project(dir: &std::path::Path, name: &str) {
    let output = run_in(
        dir,
        &["init", "--name", name, "--base", "debian", "--process", "core"],
    );
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ─── Plugin File Presence Tests ───────────────────────────────────────────────

/// After `aibox init`, svg.yazi/init.lua must be seeded into .aibox-home.
#[test]
fn svg_yazi_plugin_seeded() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-svg");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/svg.yazi/init.lua");

    assert!(
        plugin_path.exists(),
        "svg.yazi/init.lua should be seeded at {}",
        plugin_path.display()
    );
}

/// After `aibox init`, eps.yazi/init.lua must be seeded into .aibox-home.
#[test]
fn eps_yazi_plugin_seeded() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-eps");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/eps.yazi/init.lua");

    assert!(
        plugin_path.exists(),
        "eps.yazi/init.lua should be seeded at {}",
        plugin_path.display()
    );
}

/// The svg.yazi plugin must reference resvg for SVG → PNG conversion.
#[test]
fn svg_yazi_plugin_uses_resvg() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-svg-content");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/svg.yazi/init.lua");
    let content = fs::read_to_string(&plugin_path)
        .unwrap_or_else(|e| panic!("failed to read svg.yazi/init.lua: {}", e));

    assert!(
        content.contains("resvg"),
        "svg.yazi/init.lua should invoke resvg for SVG conversion"
    );
}

/// The eps.yazi plugin must reference ghostscript (gs) for EPS → PNG conversion.
#[test]
fn eps_yazi_plugin_uses_ghostscript() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-eps-content");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/eps.yazi/init.lua");
    let content = fs::read_to_string(&plugin_path)
        .unwrap_or_else(|e| panic!("failed to read eps.yazi/init.lua: {}", e));

    assert!(
        content.contains("\"gs\"") || content.contains("'gs'"),
        "eps.yazi/init.lua should invoke gs (ghostscript) for EPS conversion"
    );
}

// ─── yazi.toml [plugin] Section Tests ────────────────────────────────────────

/// yazi.toml must have a [plugin] section with prepend_previewers after init.
#[test]
fn yazi_toml_has_plugin_section() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-plugin-section");

    let yazi_toml = fs::read_to_string(
        dir.path().join(".aibox-home/.config/yazi/yazi.toml"),
    )
    .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    assert!(
        yazi_toml.contains("[plugin]"),
        "yazi.toml should contain a [plugin] section"
    );
    assert!(
        yazi_toml.contains("prepend_previewers"),
        "yazi.toml [plugin] section should define prepend_previewers"
    );
}

/// yazi.toml must route *.svg files through the svg previewer plugin.
#[test]
fn yazi_toml_svg_previewer_entry() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-svg-entry");

    let yazi_toml = fs::read_to_string(
        dir.path().join(".aibox-home/.config/yazi/yazi.toml"),
    )
    .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    assert!(
        yazi_toml.contains("\"*.svg\"") || yazi_toml.contains("'*.svg'"),
        "yazi.toml should contain a prepend_previewers entry matching *.svg"
    );
    // The svg entry must invoke the "svg" plugin run target
    assert!(
        yazi_toml.contains(r#"run = "svg""#),
        "yazi.toml svg entry should set run = \"svg\""
    );
}

/// yazi.toml must route *.eps files through the eps previewer plugin.
#[test]
fn yazi_toml_eps_previewer_entry() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-eps-entry");

    let yazi_toml = fs::read_to_string(
        dir.path().join(".aibox-home/.config/yazi/yazi.toml"),
    )
    .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    assert!(
        yazi_toml.contains("\"*.eps\"") || yazi_toml.contains("'*.eps'"),
        "yazi.toml should contain a prepend_previewers entry matching *.eps"
    );
    // The eps entry must invoke the "eps" plugin run target
    assert!(
        yazi_toml.contains(r#"run = "eps""#),
        "yazi.toml eps entry should set run = \"eps\""
    );
}

/// SVG and EPS entries must appear before the built-in image/pdf entries
/// (prepend_previewers semantics: first match wins).
#[test]
fn yazi_toml_svg_and_eps_precede_builtin_previewers() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-order");

    let yazi_toml = fs::read_to_string(
        dir.path().join(".aibox-home/.config/yazi/yazi.toml"),
    )
    .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    let svg_pos = yazi_toml
        .find("\"*.svg\"")
        .or_else(|| yazi_toml.find("'*.svg'"))
        .expect("*.svg entry not found in yazi.toml");
    let eps_pos = yazi_toml
        .find("\"*.eps\"")
        .or_else(|| yazi_toml.find("'*.eps'"))
        .expect("*.eps entry not found in yazi.toml");
    let jpg_pos = yazi_toml
        .find("\"*.jpg\"")
        .or_else(|| yazi_toml.find("'*.jpg'"))
        .expect("*.jpg entry not found in yazi.toml");

    assert!(
        svg_pos < jpg_pos,
        "*.svg entry (pos {}) should appear before *.jpg entry (pos {}) in prepend_previewers",
        svg_pos,
        jpg_pos
    );
    assert!(
        eps_pos < jpg_pos,
        "*.eps entry (pos {}) should appear before *.jpg entry (pos {}) in prepend_previewers",
        eps_pos,
        jpg_pos
    );
}

// ─── Fixture File Sanity Tests ────────────────────────────────────────────────
//
// These tests verify the sample files in tests/e2e/fixtures/ are readable
// and have the expected content markers. They serve as a baseline to confirm
// the fixture files copied from assets/placeholder-package/ are intact.

/// The sample SVG fixture starts with an <svg> or <?xml ...> declaration.
#[test]
fn fixture_sample_svg_is_valid_xml() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e/fixtures/sample.svg");

    assert!(fixture.exists(), "sample.svg fixture should exist at {}", fixture.display());

    let content = fs::read_to_string(&fixture)
        .unwrap_or_else(|e| panic!("failed to read sample.svg fixture: {}", e));

    assert!(
        content.contains("<svg") || content.contains("<?xml"),
        "sample.svg should contain SVG/XML markup"
    );
}

/// The sample EPS fixture starts with the standard %!PS-Adobe EPS header.
#[test]
fn fixture_sample_eps_has_eps_header() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e/fixtures/sample.eps");

    assert!(fixture.exists(), "sample.eps fixture should exist at {}", fixture.display());

    let content = fs::read_to_string(&fixture)
        .unwrap_or_else(|e| panic!("failed to read sample.eps fixture: {}", e));

    assert!(
        content.starts_with("%!PS-Adobe") || content.contains("%%BoundingBox"),
        "sample.eps should start with a valid PostScript/EPS header"
    );
}
