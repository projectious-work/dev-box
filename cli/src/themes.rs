//! Theme data for all supported color themes.
//!
//! Each theme provides config snippets for Zellij, Vim, Yazi, and lazygit.

use crate::config::{StarshipPreset, Theme};

/// Returns the Zellij theme KDL content for the given theme.
pub fn zellij_theme(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => r##"themes {
    gruvbox-dark {
        fg "#D5C4A1"
        bg "#282828"
        black "#3C3836"
        red "#CC241D"
        green "#98971A"
        yellow "#D79921"
        blue "#458588"
        magenta "#B16286"
        cyan "#689D6A"
        white "#FBF1C7"
        orange "#D65D0E"
    }
}
"##,
        Theme::CatppuccinMocha => r##"themes {
    catppuccin-mocha {
        fg "#CDD6F4"
        bg "#1E1E2E"
        black "#181825"
        red "#F38BA8"
        green "#A6E3A1"
        yellow "#F9E2AF"
        blue "#89B4FA"
        magenta "#F5C2E7"
        cyan "#94E2D5"
        white "#BAC2DE"
        orange "#FAB387"
    }
}
"##,
        Theme::CatppuccinLatte => r##"themes {
    catppuccin-latte {
        fg "#4C4F69"
        bg "#EFF1F5"
        black "#DCE0E8"
        red "#D20F39"
        green "#40A02B"
        yellow "#DF8E1D"
        blue "#1E66F5"
        magenta "#EA76CB"
        cyan "#179299"
        white "#5C5F77"
        orange "#FE640B"
    }
}
"##,
        Theme::Dracula => r##"themes {
    dracula {
        fg "#F8F8F2"
        bg "#282A36"
        black "#21222C"
        red "#FF5555"
        green "#50FA7B"
        yellow "#F1FA8C"
        blue "#BD93F9"
        magenta "#FF79C6"
        cyan "#8BE9FD"
        white "#F8F8F2"
        orange "#FFB86C"
    }
}
"##,
        Theme::TokyoNight => r##"themes {
    tokyo-night {
        fg "#C0CAF5"
        bg "#1A1B26"
        black "#15161E"
        red "#F7768E"
        green "#9ECE6A"
        yellow "#E0AF68"
        blue "#7AA2F7"
        magenta "#BB9AF7"
        cyan "#7DCFFF"
        white "#A9B1D6"
        orange "#FF9E64"
    }
}
"##,
        Theme::Nord => r##"themes {
    nord {
        fg "#D8DEE9"
        bg "#2E3440"
        black "#3B4252"
        red "#BF616A"
        green "#A3BE8C"
        yellow "#EBCB8B"
        blue "#81A1C1"
        magenta "#B48EAD"
        cyan "#88C0D0"
        white "#E5E9F0"
        orange "#D08770"
    }
}
"##,
        Theme::Projectious => r##"themes {
    projectious {
        fg "#C5DAF0"
        bg "#0E1720"
        black "#131E2B"
        red "#A32D2D"
        green "#2D6A4F"
        yellow "#8B6508"
        blue "#2B4D78"
        magenta "#546A82"
        cyan "#8AACC8"
        white "#E2E9F2"
        orange "#E05232"
    }
}
"##,
    }
}

/// Returns the Vim colorscheme name for the given theme.
/// These are single-file .vim colorschemes bundled in the image.
pub fn vim_colorscheme(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => "gruvbox",
        Theme::CatppuccinMocha => "catppuccin_mocha",
        Theme::CatppuccinLatte => "catppuccin_latte",
        Theme::Dracula => "dracula",
        Theme::TokyoNight => "tokyonight",
        Theme::Nord => "nord",
        Theme::Projectious => "projectious",
    }
}

/// Returns the Vim background setting (dark/light).
pub fn vim_background(theme: &Theme) -> &'static str {
    match theme {
        Theme::CatppuccinLatte => "light",
        _ => "dark",
    }
}

/// Returns the Yazi theme.toml content for the given theme.
/// Gruvbox uses the default theme.toml; others are bundled from images/base-debian/config/yazi/themes/.
pub fn yazi_theme(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => include_str!("../../images/base-debian/config/yazi/theme.toml"),
        Theme::CatppuccinMocha => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-mocha.toml")
        }
        Theme::CatppuccinLatte => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-latte.toml")
        }
        Theme::Dracula => include_str!("../../images/base-debian/config/yazi/themes/dracula.toml"),
        Theme::TokyoNight => {
            include_str!("../../images/base-debian/config/yazi/themes/tokyo-night.toml")
        }
        Theme::Nord => include_str!("../../images/base-debian/config/yazi/themes/nord.toml"),
        Theme::Projectious => {
            include_str!("../../images/base-debian/config/yazi/themes/projectious.toml")
        }
    }
}

/// Returns the lazygit theme YAML snippet (gui.theme section).
pub fn lazygit_theme(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => r#"gui:
  theme:
    activeBorderColor:
      - '#D79921'
      - bold
    inactiveBorderColor:
      - '#665C54'
    optionsTextColor:
      - '#458588'
    selectedLineBgColor:
      - '#3C3836'
    cherryPickedCommitBgColor:
      - '#504945'
    cherryPickedCommitFgColor:
      - '#D79921'
    unstagedChangesColor:
      - '#CC241D'
    defaultFgColor:
      - '#D5C4A1'
    searchingActiveBorderColor:
      - '#FABD2F'
"#,
        Theme::CatppuccinMocha => r#"gui:
  theme:
    activeBorderColor:
      - '#89B4FA'
      - bold
    inactiveBorderColor:
      - '#A6ADC8'
    optionsTextColor:
      - '#89B4FA'
    selectedLineBgColor:
      - '#313244'
    cherryPickedCommitBgColor:
      - '#45475A'
    cherryPickedCommitFgColor:
      - '#89B4FA'
    unstagedChangesColor:
      - '#F38BA8'
    defaultFgColor:
      - '#CDD6F4'
    searchingActiveBorderColor:
      - '#F9E2AF'
"#,
        Theme::CatppuccinLatte => r#"gui:
  theme:
    activeBorderColor:
      - '#1E66F5'
      - bold
    inactiveBorderColor:
      - '#6C6F85'
    optionsTextColor:
      - '#1E66F5'
    selectedLineBgColor:
      - '#CCD0DA'
    cherryPickedCommitBgColor:
      - '#BCC0CC'
    cherryPickedCommitFgColor:
      - '#1E66F5'
    unstagedChangesColor:
      - '#D20F39'
    defaultFgColor:
      - '#4C4F69'
    searchingActiveBorderColor:
      - '#DF8E1D'
"#,
        Theme::Dracula => r#"gui:
  theme:
    activeBorderColor:
      - '#FF79C6'
      - bold
    inactiveBorderColor:
      - '#BD93F9'
    optionsTextColor:
      - '#6272A4'
    selectedLineBgColor:
      - '#6272A4'
    cherryPickedCommitBgColor:
      - '#8BE9FD'
    cherryPickedCommitFgColor:
      - '#6272A4'
    unstagedChangesColor:
      - '#FF5555'
    defaultFgColor:
      - '#F8F8F2'
    searchingActiveBorderColor:
      - '#8BE9FD'
      - bold
"#,
        Theme::TokyoNight => r#"gui:
  theme:
    activeBorderColor:
      - '#7AA2F7'
      - bold
    inactiveBorderColor:
      - '#3B4261'
    optionsTextColor:
      - '#7AA2F7'
    selectedLineBgColor:
      - '#283457'
    cherryPickedCommitBgColor:
      - '#3B4261'
    cherryPickedCommitFgColor:
      - '#7AA2F7'
    unstagedChangesColor:
      - '#F7768E'
    defaultFgColor:
      - '#C0CAF5'
    searchingActiveBorderColor:
      - '#E0AF68'
"#,
        Theme::Nord => r#"gui:
  theme:
    activeBorderColor:
      - '#88C0D0'
      - bold
    inactiveBorderColor:
      - '#4C566A'
    optionsTextColor:
      - '#81A1C1'
    selectedLineBgColor:
      - '#3B4252'
    cherryPickedCommitBgColor:
      - '#434C5E'
    cherryPickedCommitFgColor:
      - '#88C0D0'
    unstagedChangesColor:
      - '#BF616A'
    defaultFgColor:
      - '#D8DEE9'
    searchingActiveBorderColor:
      - '#EBCB8B'
"#,
        Theme::Projectious => r#"gui:
  theme:
    activeBorderColor:
      - '#E05232'
      - bold
    inactiveBorderColor:
      - '#546A82'
    optionsTextColor:
      - '#8AACC8'
    selectedLineBgColor:
      - '#1d3352'
    cherryPickedCommitBgColor:
      - '#2B4D78'
    cherryPickedCommitFgColor:
      - '#E05232'
    unstagedChangesColor:
      - '#A32D2D'
    defaultFgColor:
      - '#C5DAF0'
    searchingActiveBorderColor:
      - '#8B6508'
"#,
    }
}

/// Color palette values for Starship prompt theming.
fn theme_palette(theme: &Theme) -> (&str, &str, &str, &str, &str) {
    // Returns (bg, fg, accent, green, red)
    match theme {
        Theme::GruvboxDark => ("#282828", "#D5C4A1", "#D79921", "#98971A", "#CC241D"),
        Theme::CatppuccinMocha => ("#1E1E2E", "#CDD6F4", "#89B4FA", "#A6E3A1", "#F38BA8"),
        Theme::CatppuccinLatte => ("#EFF1F5", "#4C4F69", "#1E66F5", "#40A02B", "#D20F39"),
        Theme::Dracula => ("#282A36", "#F8F8F2", "#BD93F9", "#50FA7B", "#FF5555"),
        Theme::TokyoNight => ("#1A1B26", "#C0CAF5", "#7AA2F7", "#9ECE6A", "#F7768E"),
        Theme::Nord => ("#2E3440", "#D8DEE9", "#88C0D0", "#A3BE8C", "#BF616A"),
        Theme::Projectious => ("#0E1720", "#C5DAF0", "#E05232", "#2D6A4F", "#A32D2D"),
    }
}

/// Generate starship.toml content for the given preset and theme.
pub fn starship_config(preset: &StarshipPreset, theme: &Theme) -> String {
    let (bg, fg, accent, green, _red) = theme_palette(theme);

    match preset {
        StarshipPreset::Default => format!(
r#"# aibox starship config — default preset
palette = "aibox"

format = "$directory$git_branch$git_status$python$rust$nodejs$golang$cmd_duration$line_break$character"

[directory]
style = "bold fg:{accent}"
truncation_length = 3

[git_branch]
style = "fg:{green}"

[git_status]
style = "fg:{accent}"

[python]
style = "fg:#D79921"
format = "[$symbol$version]($style) "

[rust]
style = "fg:#D65D0E"
format = "[$symbol$version]($style) "

[nodejs]
style = "fg:#98971A"
format = "[$symbol$version]($style) "

[golang]
style = "fg:#689D6A"
format = "[$symbol$version]($style) "

[cmd_duration]
style = "fg:#928374"
min_time = 2_000

[character]
success_symbol = "[❯](bold fg:{green})"
error_symbol = "[❯](bold fg:red)"

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
"#),

        StarshipPreset::Plain => format!(
r#"# aibox starship config — plain preset (no Nerd Font needed)
format = "$directory$git_branch$git_status$cmd_duration$line_break$character"

[directory]
style = "bold fg:{accent}"

[git_branch]
symbol = ""
style = "fg:{green}"

[git_status]
style = "fg:{accent}"

[character]
success_symbol = "[>](bold fg:{green})"
error_symbol = "[>](bold fg:red)"

[python]
symbol = "py "
[rust]
symbol = "rs "
[nodejs]
symbol = "js "
[golang]
symbol = "go "
"#),

        StarshipPreset::Minimal => format!(
r#"# aibox starship config — minimal preset
format = "$directory$git_branch$line_break$character"

[directory]
style = "bold fg:{accent}"
truncation_length = 2

[git_branch]
style = "fg:{green}"
format = " [$branch]($style)"

[character]
success_symbol = "[❯](fg:{accent})"
error_symbol = "[❯](bold fg:red)"
"#),

        StarshipPreset::NerdFont => format!(
r#"# aibox starship config — nerd-font preset
palette = "aibox"

format = "$os$directory$git_branch$git_status$python$rust$nodejs$golang$docker_context$cmd_duration$line_break$character"

[os]
disabled = false
style = "fg:{fg}"

[directory]
style = "bold fg:{accent}"
read_only = " 󰌾"

[git_branch]
symbol = " "
style = "fg:{green}"

[git_status]
style = "fg:{accent}"

[python]
symbol = " "
[rust]
symbol = " "
[nodejs]
symbol = " "
[golang]
symbol = " "
[docker_context]
symbol = " "

[cmd_duration]
style = "fg:#928374"

[character]
success_symbol = "[❯](bold fg:{green})"
error_symbol = "[❯](bold fg:red)"

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
"#),

        StarshipPreset::Pastel => format!(
r#"# aibox starship config — pastel powerline preset
palette = "aibox"

format = """
[](fg:{accent})\
$directory\
[](fg:{accent} bg:{green})\
$git_branch\
$git_status\
[](fg:{green} bg:{bg})\
$python$rust$nodejs$golang\
$cmd_duration\
$line_break$character"""

[directory]
style = "bold bg:{accent} fg:{bg}"
truncation_length = 3

[git_branch]
style = "bg:{green} fg:{bg}"
symbol = " "

[git_status]
style = "bg:{green} fg:{bg}"

[character]
success_symbol = "[❯](bold fg:{accent})"
error_symbol = "[❯](bold fg:red)"

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
"#),

        StarshipPreset::Bracketed => format!(
r#"# aibox starship config — bracketed segments preset
format = "$directory$git_branch$git_status$python$rust$nodejs$golang$cmd_duration$line_break$character"

[directory]
style = "fg:{accent}"
format = "[$path]($style)[$read_only]($read_only_style) "

[git_branch]
style = "fg:{green}"
format = "[\\[$branch\\]]($style) "

[git_status]
style = "fg:{accent}"
format = "[\\[$all_status$ahead_behind\\]]($style) "

[python]
format = "[\\[$symbol$version\\]](fg:#D79921) "
[rust]
format = "[\\[$symbol$version\\]](fg:#D65D0E) "
[nodejs]
format = "[\\[$symbol$version\\]](fg:#98971A) "
[golang]
format = "[\\[$symbol$version\\]](fg:#689D6A) "

[cmd_duration]
format = "[\\[$duration\\]](fg:#928374) "

[character]
success_symbol = "[❯](bold fg:{green})"
error_symbol = "[❯](bold fg:red)"
"#),

        StarshipPreset::Arrow => format!(
r#"# aibox starship config — arrow preset (powerline chevron/airline style)
# Requires a Nerd Font or Powerline-patched font for the arrow separators (e0b0/e0b2).
palette = "aibox"

format = """
[](fg:{accent})\
$directory\
[](fg:{accent} bg:{green})\
$git_branch\
$git_status\
[](fg:{green} bg:{bg})\
 $cmd_duration\
$line_break\
$character"""

[directory]
style = "bold bg:{accent} fg:{bg}"
format = "[ $path ]($style)"
truncation_length = 3
truncate_to_repo = true

[git_branch]
style = "bg:{green} fg:{bg}"
symbol = " "
format = "[ $symbol$branch ]($style)"

[git_status]
style = "bg:{green} fg:{bg}"
format = "[$all_status$ahead_behind]($style)"
ahead = "⇡$count"
behind = "⇣$count"
diverged = "⇕⇡$ahead_count⇣$behind_count"
modified = "!$count"
staged = "+$count"
untracked = "?$count"

[cmd_duration]
style = "fg:#928374"
min_time = 2_000
format = "[ $duration]($style)"

[character]
success_symbol = "[❯](bold fg:{accent})"
error_symbol = "[❯](bold fg:red)"

[python]
style = "fg:#D79921"
format = "[$symbol$version]($style) "
[rust]
style = "fg:#D65D0E"
format = "[$symbol$version]($style) "
[nodejs]
style = "fg:#98971A"
format = "[$symbol$version]($style) "
[golang]
style = "fg:#689D6A"
format = "[$symbol$version]($style) "

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
"#),
    }
}
