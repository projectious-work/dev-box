//! Theme data for all supported color themes.
//!
//! Each theme provides config snippets for Zellij, Vim, Yazi, and lazygit.

use crate::config::Theme;

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
    }
}

/// Returns the Zellij theme name to use in config.kdl.
pub fn zellij_theme_name(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => "gruvbox-dark",
        Theme::CatppuccinMocha => "catppuccin-mocha",
        Theme::CatppuccinLatte => "catppuccin-latte",
        Theme::Dracula => "dracula",
        Theme::TokyoNight => "tokyo-night",
        Theme::Nord => "nord",
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
    }
}

/// Returns the Vim background setting (dark/light).
pub fn vim_background(theme: &Theme) -> &'static str {
    match theme {
        Theme::CatppuccinLatte => "light",
        _ => "dark",
    }
}

/// Returns the Yazi flavor directory name.
/// Flavors are bundled in the image at /root/.config/yazi/flavors/<name>.yazi/
pub fn yazi_flavor(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => "gruvbox-dark",
        Theme::CatppuccinMocha => "catppuccin-mocha",
        Theme::CatppuccinLatte => "catppuccin-latte",
        Theme::Dracula => "dracula",
        Theme::TokyoNight => "tokyo-night",
        Theme::Nord => "nord",
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
    }
}
