//! Visual tests — verify themed terminal output via asciinema recordings.
//!
//! Records headless zellij sessions on the e2e companion and asserts
//! theme-specific ANSI RGB color sequences appear in the cast files.
//!
//! Requires: e2e-runner with asciinema, zellij, yazi, vim, starship installed.

use serial_test::serial;

use super::runner::E2eRunner;

/// RGB values that uniquely identify each theme in zellij's ANSI output.
/// Sourced from the `green` field in each theme's KDL definition (cli/src/themes.rs).
const THEME_SIGNATURES: &[(&str, u8, u8, u8)] = &[
    ("gruvbox-dark", 152, 151, 26),
    ("catppuccin-mocha", 166, 227, 161),
    ("catppuccin-latte", 64, 160, 43),
    ("dracula", 80, 250, 123),
    ("tokyo-night", 158, 206, 106),
    ("nord", 163, 190, 140),
];

/// Extract all output event data from an asciicast v2 file.
///
/// Cast format: first line is a JSON header, subsequent lines are
/// `[timestamp, "o", "data"]` JSON arrays. We concatenate all "o" data.
fn extract_cast_output(cast_content: &str) -> String {
    cast_content
        .lines()
        .skip(1) // skip header
        .filter_map(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
            let arr = parsed.as_array()?;
            if arr.len() >= 3 && arr[1].as_str() == Some("o") {
                arr[2].as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Check if an ANSI truecolor RGB sequence appears in the output.
/// Matches both foreground (`38;2;R;G;B`) and background (`48;2;R;G;B`).
fn contains_rgb(output: &str, r: u8, g: u8, b: u8) -> bool {
    let fg = format!("38;2;{};{};{}", r, g, b);
    let bg = format!("48;2;{};{};{}", r, g, b);
    output.contains(&fg) || output.contains(&bg)
}

/// Record a themed zellij session on the companion and return the cast content.
///
/// Steps: init project with theme → create driver script → record via asciinema.
/// Uses a minimal zellij layout (single pane) so the test depends only on
/// zellij's status bar and frame rendering, not on specific pane commands.
fn record_themed_session(runner: &E2eRunner, test_name: &str, theme: &str) -> String {
    runner.cleanup(test_name);

    // Init project with the specified theme
    let output = runner.aibox(
        test_name,
        &[
            "init",
            "--name",
            test_name,
            "--base",
            "debian",
            "--process",
            "core",
            "--theme",
            theme,
        ],
    );
    assert!(
        output.status.success(),
        "init with theme={} failed: {}",
        theme,
        String::from_utf8_lossy(&output.stderr)
    );

    let workspace = format!("/workspaces/{}", test_name);

    // Clean up any leftover zellij sessions
    runner.exec(
        "pkill -9 -x zellij 2>/dev/null; rm -rf /tmp/zellij-* 2>/dev/null; sleep 0.5",
    );

    // Write driver script that launches zellij with the generated config
    let driver_path = format!("{}/driver.sh", workspace);
    let cast_path = format!("{}/recording.cast", workspace);

    runner.write_file(
        test_name,
        "driver.sh",
        &format!(
            r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={workspace}/.aibox-home
(sleep 3 && pkill -x zellij 2>/dev/null) &
zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" 2>/dev/null
true
"#
        ),
    );
    runner.exec(&format!("chmod +x {}", driver_path));

    // Record (LC_ALL=C.UTF-8: companion image uses C.UTF-8, belt-and-suspenders
    // in case client locale bleeds in via SSH despite AcceptEnv being cleared)
    let rec_cmd = format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite -c {} {} 2>/dev/null; true",
        driver_path, cast_path
    );
    runner.exec(&rec_cmd);

    // Read back the cast file
    runner.read_file(test_name, "recording.cast")
}

// ─── Theme color tests ──────────────────────────────────────────────────────

/// Verify that each theme's signature color (green) appears in the
/// zellij rendering output as an ANSI truecolor RGB sequence.
#[test]
#[serial]
#[ntest::timeout(180_000)] // 3 minutes for all themes
fn visual_themes_produce_signature_colors() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    for &(theme, r, g, b) in THEME_SIGNATURES {
        let test_name = format!("visual-theme-{}", theme);
        let cast = record_themed_session(&runner, &test_name, theme);

        // Basic cast validation
        let line_count = cast.lines().count();
        assert!(
            line_count > 10,
            "theme '{}': cast too small ({} lines)",
            theme,
            line_count
        );

        let output = extract_cast_output(&cast);
        assert!(
            !output.is_empty(),
            "theme '{}': no output events in cast",
            theme
        );

        // Assert the theme's signature green color appears
        assert!(
            contains_rgb(&output, r, g, b),
            "theme '{}': expected RGB({},{},{}) in ANSI output but not found",
            theme,
            r,
            g,
            b
        );

        runner.cleanup(&test_name);
    }
}

/// Verify that each theme produces DISTINCT colors — no two themes
/// share the same signature, proving the theme config is actually loaded.
#[test]
#[serial]
#[ntest::timeout(180_000)]
fn visual_themes_are_distinct() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    // Record one theme and verify other themes' signatures are absent
    let test_name = "visual-theme-distinct";
    let cast = record_themed_session(&runner, test_name, "nord");
    let output = extract_cast_output(&cast);

    // Nord's green MUST be present
    assert!(
        contains_rgb(&output, 163, 190, 140),
        "nord: own signature color should be present"
    );

    // Other themes' greens should NOT be present
    let non_nord = [
        ("gruvbox-dark", 152, 151, 26),
        ("dracula", 80, 250, 123),
        ("catppuccin-mocha", 166, 227, 161),
        ("tokyo-night", 158, 206, 106),
    ];
    for (other, r, g, b) in non_nord {
        assert!(
            !contains_rgb(&output, r, g, b),
            "nord recording should NOT contain {}'s green RGB({},{},{})",
            other,
            r,
            g,
            b
        );
    }

    runner.cleanup(test_name);
}

// ─── Yazi keymap test ───────────────────────────────────────────────────────

/// Verify that yazi launches inside zellij and renders its UI.
/// Checks for yazi-specific strings in the cast output (file list chrome).
#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_yazi_renders_in_zellij() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-yazi";
    runner.cleanup(test_name);

    // Init project
    runner.aibox(
        test_name,
        &[
            "init",
            "--name",
            test_name,
            "--base",
            "debian",
            "--process",
            "core",
        ],
    );

    let workspace = format!("/workspaces/{}", test_name);

    // Create some files for yazi to display
    runner.write_file(test_name, "project-files/README.md", "# Test\n");
    runner.write_file(test_name, "project-files/main.rs", "fn main() {}\n");

    runner.exec(
        "pkill -9 -x zellij 2>/dev/null; rm -rf /tmp/zellij-* 2>/dev/null; sleep 0.5",
    );

    // Create a layout that runs yazi pointing at the test files
    runner.write_file(
        test_name,
        "yazi-layout.kdl",
        &format!(
            r#"layout {{
    pane {{
        command "yazi"
        args "{workspace}/project-files"
    }}
}}
"#
        ),
    );

    // Driver script
    runner.write_file(
        test_name,
        "driver.sh",
        &format!(
            r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={workspace}/.aibox-home
(sleep 3 && pkill -x zellij 2>/dev/null) &
zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout {workspace}/yazi-layout.kdl 2>/dev/null
true
"#
        ),
    );
    runner.exec(&format!("chmod +x {}/driver.sh", workspace));

    let cast_path = format!("{}/recording.cast", workspace);
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite -c {workspace}/driver.sh {cast_path} 2>/dev/null; true"
    ));

    let cast = runner.read_file(test_name, "recording.cast");
    let output = extract_cast_output(&cast);

    // Yazi should render the file names we created
    assert!(
        output.contains("README") || output.contains("main.rs"),
        "yazi should display file names from the test directory"
    );

    runner.cleanup(test_name);
}
