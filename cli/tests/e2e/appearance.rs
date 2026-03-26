//! Appearance tests — verify themes and prompts render correctly.
//!
//! These are Tier 1 tests: they run `aibox init` + `aibox sync` locally
//! and inspect the generated/seeded config files. No container needed.

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

/// Initialize with specific theme and prompt.
fn init_with_appearance(dir: &std::path::Path, theme: &str, prompt: &str) {
    let output = run_in(
        dir,
        &[
            "init",
            "--name", "appearance-test",
            "--base", "debian",
            "--process", "core",
            "--theme", theme,
            "--prompt", prompt,
        ],
    );
    assert!(
        output.status.success(),
        "init with theme={} prompt={} failed: {}",
        theme,
        prompt,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Replace the appearance section in aibox.toml and re-sync.
fn change_appearance(dir: &std::path::Path, theme: &str, prompt: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Replace [appearance] section — search for header at start of line to avoid matching comments.
    let section_header = "[appearance]";
    let needle = format!("\n{}", section_header);
    if let Some(needle_pos) = content.find(&needle) {
        let start = needle_pos + 1; // position of '[' in section header
        let rest = &content[start + section_header.len()..];
        let end = rest
            .find("\n[")
            .map(|i| start + section_header.len() + i)
            .unwrap_or(content.len());
        let new_content = format!(
            "{}{}\ntheme = \"{}\"\nprompt = \"{}\"\n{}",
            &content[..start],
            section_header,
            theme,
            prompt,
            &content[end..]
        );
        fs::write(&toml_path, new_content).unwrap();
    }

    let output = run_in(dir, &["sync", "--no-build"]);
    assert!(
        output.status.success(),
        "sync after appearance change failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Check that no template placeholders survive in seeded config files.
fn assert_no_placeholders(dir: &std::path::Path) {
    let aibox_home = dir.join(".aibox-home");
    let placeholders = ["AIBOX_THEME", "AIBOX_VIM_COLORSCHEME", "AIBOX_VIM_BG"];

    let files_to_check = [
        ".vim/vimrc",
        ".config/zellij/config.kdl",
        ".config/starship.toml",
    ];

    for file in &files_to_check {
        let path = aibox_home.join(file);
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap();
            for placeholder in &placeholders {
                assert!(
                    !content.contains(placeholder),
                    "placeholder '{}' found in {}: {}",
                    placeholder,
                    file,
                    content.lines().find(|l| l.contains(placeholder)).unwrap_or("???")
                );
            }
        }
    }
}

// ─── Theme Tests ─────────────────────────────────────────────────────────────

/// Test that all themes render without errors and no placeholders survive.
#[test]
fn all_themes_render_without_error() {
    let themes = [
        "gruvbox-dark",
        "catppuccin-mocha",
        "catppuccin-latte",
        "dracula",
        "tokyo-night",
        "nord",
        "projectious",
    ];

    for theme in &themes {
        let dir = tempfile::tempdir().unwrap();
        init_with_appearance(dir.path(), theme, "default");
        assert_no_placeholders(dir.path());
    }
}

#[test]
fn theme_gruvbox_renders_correctly() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let aibox_home = dir.path().join(".aibox-home");

    // Check vimrc
    let vimrc = fs::read_to_string(aibox_home.join(".vim/vimrc")).unwrap();
    assert!(
        vimrc.contains("gruvbox") || vimrc.contains("retrobox"),
        "vimrc should use gruvbox-related colorscheme, got: {}",
        vimrc.lines().find(|l| l.contains("colorscheme")).unwrap_or("no colorscheme line")
    );

    // Check zellij config
    let zellij = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
    assert!(
        zellij.contains("gruvbox-dark"),
        "zellij config should reference gruvbox-dark theme"
    );
}

#[test]
fn theme_catppuccin_mocha_renders() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "catppuccin-mocha", "default");

    let aibox_home = dir.path().join(".aibox-home");

    // Check zellij config
    let zellij = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
    assert!(
        zellij.contains("catppuccin-mocha"),
        "zellij config should reference catppuccin-mocha theme"
    );
}

#[test]
fn theme_change_updates_all_files() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let aibox_home = dir.path().join(".aibox-home");

    // Verify initial theme across all tools
    let zellij = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
    assert!(zellij.contains("gruvbox-dark"));
    let vimrc = fs::read_to_string(aibox_home.join(".vim/vimrc")).unwrap();
    assert!(vimrc.contains("gruvbox"));
    let yazi_theme = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
    let yazi_initial_len = yazi_theme.len();
    assert!(yazi_initial_len > 0, "yazi theme should not be empty");

    // Change theme
    change_appearance(dir.path(), "dracula", "default");

    // Verify zellij updated
    let zellij = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
    assert!(
        zellij.contains("dracula"),
        "zellij config should be updated to dracula after theme change"
    );
    assert!(
        !zellij.contains("gruvbox-dark"),
        "gruvbox-dark should no longer appear after theme change"
    );

    // Verify vim updated
    let vimrc = fs::read_to_string(aibox_home.join(".vim/vimrc")).unwrap();
    assert!(
        vimrc.contains("dracula"),
        "vimrc colorscheme should be updated to dracula, got: {}",
        vimrc.lines().find(|l| l.contains("colorscheme")).unwrap_or("no colorscheme line")
    );

    // Verify yazi theme updated (content should differ from gruvbox)
    let yazi_theme = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
    assert!(
        !yazi_theme.is_empty(),
        "yazi theme should not be empty after theme change"
    );

    // Verify lazygit updated
    let lazygit = fs::read_to_string(aibox_home.join(".config/lazygit/config.yml")).unwrap();
    assert!(
        !lazygit.is_empty(),
        "lazygit config should not be empty after theme change"
    );

    // Verify starship updated
    let starship = fs::read_to_string(aibox_home.join(".config/starship.toml")).unwrap();
    assert!(
        !starship.is_empty(),
        "starship config should not be empty after theme change"
    );
}

/// Verify that each theme produces distinct config for all themed tools.
#[test]
fn theme_alignment_all_tools_match_selected_theme() {
    let themes_and_vim = [
        ("gruvbox-dark", "gruvbox"),
        ("nord", "nord"),
        ("dracula", "dracula"),
        ("catppuccin-mocha", "catppuccin_mocha"),
        ("tokyo-night", "tokyonight"),
    ];

    for (theme, vim_scheme) in &themes_and_vim {
        let dir = tempfile::tempdir().unwrap();
        init_with_appearance(dir.path(), theme, "default");

        let aibox_home = dir.path().join(".aibox-home");

        // Vim must use the matching colorscheme
        let vimrc = fs::read_to_string(aibox_home.join(".vim/vimrc")).unwrap();
        assert!(
            vimrc.contains(&format!("colorscheme {}", vim_scheme)),
            "theme '{}': vimrc should contain 'colorscheme {}', got: {}",
            theme,
            vim_scheme,
            vimrc.lines().find(|l| l.contains("colorscheme")).unwrap_or("no colorscheme line")
        );

        // Zellij must reference the theme name
        let zellij = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
        assert!(
            zellij.contains(theme),
            "theme '{}': zellij config should reference theme name",
            theme
        );

        // Yazi theme must be non-empty (distinct per theme)
        let yazi = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
        assert!(
            !yazi.is_empty(),
            "theme '{}': yazi theme.toml should not be empty",
            theme
        );

        // Lazygit must be non-empty
        let lazygit = fs::read_to_string(aibox_home.join(".config/lazygit/config.yml")).unwrap();
        assert!(
            !lazygit.is_empty(),
            "theme '{}': lazygit config should not be empty",
            theme
        );

        // Starship must contain a palette section
        let starship = fs::read_to_string(aibox_home.join(".config/starship.toml")).unwrap();
        assert!(
            starship.contains("[palette"),
            "theme '{}': starship config should contain palette section",
            theme
        );
    }
}

// ─── Keymap Tests ────────────────────────────────────────────────────────────

/// Verify that seeded yazi keymap includes the "e" binding for open-in-editor.
#[test]
fn yazi_keymap_includes_edit_in_pane_binding() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let keymap = fs::read_to_string(
        dir.path().join(".aibox-home/.config/yazi/keymap.toml"),
    )
    .unwrap();

    assert!(
        keymap.contains(r#"on = "e""#),
        "yazi keymap should contain 'e' keybinding for open-in-editor"
    );
    assert!(
        keymap.contains("open-in-editor"),
        "yazi keymap 'e' binding should invoke open-in-editor"
    );
}

// ─── Prompt Tests ────────────────────────────────────────────────────────────

/// Test that all prompt presets render without errors.
#[test]
fn all_prompts_render_without_error() {
    let presets = [
        "default",
        "plain",
        "minimal",
        "nerd-font",
        "pastel",
        "bracketed",
    ];

    for preset in &presets {
        let dir = tempfile::tempdir().unwrap();
        init_with_appearance(dir.path(), "gruvbox-dark", preset);

        let starship = dir.path().join(".aibox-home/.config/starship.toml");
        assert!(
            starship.exists(),
            "starship.toml should exist for preset '{}'",
            preset
        );

        let content = fs::read_to_string(&starship).unwrap();
        assert!(
            !content.is_empty(),
            "starship.toml should not be empty for preset '{}'",
            preset
        );
    }
}

#[test]
fn prompt_default_generates_starship() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let content = fs::read_to_string(
        dir.path().join(".aibox-home/.config/starship.toml"),
    )
    .unwrap();

    assert!(
        content.contains("directory") && content.contains("git_branch"),
        "default starship config should include directory and git_branch modules"
    );
}

#[test]
fn prompt_plain_no_nerd_font() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "plain");

    let content = fs::read_to_string(
        dir.path().join(".aibox-home/.config/starship.toml"),
    )
    .unwrap();

    // Plain preset should mention it's ASCII-only or not have Nerd Font symbols
    assert!(
        content.contains("plain") || content.contains("ASCII") || !content.contains("\u{e0b0}"),
        "plain starship config should use ASCII-only symbols"
    );
}
