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
            "--name",
            "appearance-test",
            "--base",
            "debian",
            "--process",
            "managed",
            "--theme",
            theme,
            "--prompt",
            prompt,
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

/// Replace the customization section in aibox.toml and re-sync.
fn change_appearance(dir: &std::path::Path, theme: &str, prompt: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Replace [customization] section — search for header at start of line to avoid matching comments.
    let section_header = "[customization]";
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
                    content
                        .lines()
                        .find(|l| l.contains(placeholder))
                        .unwrap_or("???")
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
        vimrc
            .lines()
            .find(|l| l.contains("colorscheme"))
            .unwrap_or("no colorscheme line")
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
fn theme_change_writes_runtime_migration_without_overwriting_live_files() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let aibox_home = dir.path().join(".aibox-home");

    let zellij_before = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
    assert!(zellij_before.contains("gruvbox-dark"));
    let vimrc_before = fs::read_to_string(aibox_home.join(".vim/vimrc")).unwrap();
    assert!(vimrc_before.contains("gruvbox"));
    let yazi_before = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
    assert!(!yazi_before.is_empty(), "yazi theme should not be empty");

    change_appearance(dir.path(), "dracula", "default");

    // ChangedUpstreamOnly files are now auto-applied (the user hasn't
    // touched them, only the config changed), so the live files should
    // already reflect the new theme.
    let zellij_after = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
    assert!(
        zellij_after.contains("dracula"),
        "zellij config should be auto-updated to the new theme"
    );
    let vimrc_after = fs::read_to_string(aibox_home.join(".vim/vimrc")).unwrap();
    assert!(
        vimrc_after.contains("dracula"),
        "vimrc should be auto-updated to the new colorscheme"
    );
    let yazi_after = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
    assert_ne!(
        yazi_after, yazi_before,
        "yazi theme should be auto-updated to the new theme"
    );

    let pending_dir = dir.path().join("context/migrations/pending");
    let docs: Vec<_> = fs::read_dir(&pending_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    assert!(
        !docs.is_empty(),
        "expected at least one pending migration document after theme change"
    );

    let runtime_doc = docs
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(|name| name.starts_with("MIG-RUNTIME-"))
                .unwrap_or(false)
        })
        .expect("expected a runtime migration document with MIG-RUNTIME- prefix");

    let migration = fs::read_to_string(runtime_doc).unwrap();
    assert!(
        migration.contains("runtime-zellij")
            || migration.contains("runtime-vim")
            || migration.contains("runtime-yazi"),
        "runtime migration should mention at least one themed runtime group"
    );
    assert!(
        migration.contains(".aibox-home/.config/zellij/config.kdl")
            || migration.contains(".aibox-home/.vim/vimrc")
    );
}

/// Verify that each theme produces distinct config for all themed tools.
#[test]
fn theme_alignment_all_tools_match_selected_theme() {
    let themes = [
        ("gruvbox-dark", "gruvbox", "#D79921"),
        ("catppuccin-mocha", "catppuccin_mocha", "#89B4FA"),
        ("catppuccin-latte", "catppuccin_latte", "#1E66F5"),
        ("dracula", "dracula", "#BD93F9"),
        ("tokyo-night", "tokyonight", "#7AA2F7"),
        ("nord", "nord", "#88C0D0"),
        ("projectious", "projectious", "#E05232"),
    ];

    for (theme, vim_scheme, accent) in &themes {
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
            vimrc
                .lines()
                .find(|l| l.contains("colorscheme"))
                .unwrap_or("no colorscheme line")
        );

        // Zellij must reference the theme name
        let zellij = fs::read_to_string(aibox_home.join(".config/zellij/config.kdl")).unwrap();
        assert!(
            zellij.contains(theme),
            "theme '{}': zellij config should reference theme name",
            theme
        );

        // Yazi theme must use the current schema and include git styling.
        let yazi = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
        assert!(
            !yazi.is_empty(),
            "theme '{}': yazi theme.toml should not be empty",
            theme
        );
        for required in [
            "[tabs]",
            "[mode]",
            "[status]",
            "[git]",
            "normal_main",
            "overall =",
        ] {
            assert!(
                yazi.contains(required),
                "theme '{}': yazi theme.toml should contain '{}'",
                theme,
                required
            );
        }
        for removed in [
            "tab_active",
            "mode_normal",
            "separator_open",
            "permissions_t",
            "[select]",
            "[completion]",
        ] {
            assert!(
                !yazi.contains(removed),
                "theme '{}': yazi theme.toml should not contain legacy key '{}'",
                theme,
                removed
            );
        }

        // Lazygit must be non-empty
        let lazygit = fs::read_to_string(aibox_home.join(".config/lazygit/config.yml")).unwrap();
        assert!(
            !lazygit.is_empty(),
            "theme '{}': lazygit config should not be empty",
            theme
        );

        // Starship must contain theme-specific colors, not Gruvbox-only fallbacks.
        let starship = fs::read_to_string(aibox_home.join(".config/starship.toml")).unwrap();
        assert!(
            starship.contains("[palette"),
            "theme '{}': starship config should contain palette section",
            theme
        );
        assert!(
            starship.contains(accent),
            "theme '{}': starship config should contain accent color {}",
            theme,
            accent
        );
        if *theme != "gruvbox-dark" {
            for hardcoded in ["#D79921", "#D65D0E", "#689D6A", "#928374"] {
                assert!(
                    !starship.contains(hardcoded),
                    "theme '{}': starship config should not contain Gruvbox-specific color {}",
                    theme,
                    hardcoded
                );
            }
        }
    }
}

// ─── Keymap Tests ────────────────────────────────────────────────────────────

/// Verify that seeded yazi keymap includes the "e" binding for open-in-editor.
#[test]
fn yazi_keymap_includes_edit_in_pane_binding() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let keymap =
        fs::read_to_string(dir.path().join(".aibox-home/.config/yazi/keymap.toml")).unwrap();

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
        "arrow",
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

    let content = fs::read_to_string(dir.path().join(".aibox-home/.config/starship.toml")).unwrap();

    assert!(
        content.contains("directory") && content.contains("git_branch"),
        "default starship config should include directory and git_branch modules"
    );
}

#[test]
fn prompt_plain_no_nerd_font() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "plain");

    let content = fs::read_to_string(dir.path().join(".aibox-home/.config/starship.toml")).unwrap();

    // Plain preset should mention it's ASCII-only or not have Nerd Font symbols
    assert!(
        content.contains("plain") || content.contains("ASCII") || !content.contains("\u{e0b0}"),
        "plain starship config should use ASCII-only symbols"
    );
}
