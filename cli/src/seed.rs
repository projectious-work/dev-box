use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::DevBoxConfig;
use crate::output;

/// Default vimrc content (embedded fallback).
const DEFAULT_VIMRC: &str = r#"" dev-box default vimrc
set nocompatible
let mapleader=" "

set number relativenumber
set tabstop=4 shiftwidth=4 expandtab smartindent
set undofile undodir=/root/.vim/undo
set noswapfile
set colorcolumn=88
set scrolloff=8
set signcolumn=yes
set cursorline
set wildmenu wildmode=longest:full,full
set incsearch hlsearch ignorecase smartcase
set backspace=indent,eol,start
set laststatus=2
set ruler showcmd

" Filetype-specific indentation
autocmd FileType yaml,json,kdl,html,css,javascript setlocal tabstop=2 shiftwidth=2
autocmd FileType markdown setlocal wrap linebreak

" Use ripgrep if available
if executable('rg')
  set grepprg=rg\ --vimgrep\ --smart-case
endif

" Netrw settings
let g:netrw_liststyle=3
let g:netrw_banner=0
let g:netrw_winsize=25

colorscheme desert
syntax on
filetype plugin indent on
"#;

/// Default gitconfig content.
const DEFAULT_GITCONFIG: &str = r#"[core]
    editor = vim
[init]
    defaultBranch = main
[pull]
    rebase = true
"#;

/// Default zellij config.kdl content.
const DEFAULT_ZELLIJ_CONFIG: &str = r#"// dev-box zellij configuration
theme "gruvbox"
default_layout "dev"
default_shell "bash"
mouse_mode true
copy_on_select true
scroll_buffer_size 10000
rounded_corners true
simplified_ui false
pane_frames true

// Leader: Ctrl+g (press Ctrl+g, release, then press the action key)
// Quick reference:
//   Ctrl+g → h/j/k/l    Navigate panes
//   Ctrl+g → n/d/r       New pane / split down / split right
//   Ctrl+g → x           Close pane
//   Ctrl+g → f           Toggle fullscreen
//   Ctrl+g → z           Toggle pane frames
//   Ctrl+g → t/w         New tab / close tab
//   Ctrl+g → [/]         Previous/next tab
//   Ctrl+g → 1-5         Jump to tab
//   Ctrl+g → s           Strider file picker
//   Ctrl+g → u           Scroll mode
//   Ctrl+g → /           Search scrollback
//   Ctrl+q               Quit zellij
keybinds clear-defaults=true {
    normal {
        bind "Ctrl g" { SwitchToMode "Locked"; }
        bind "Ctrl q" { Quit; }
    }
    locked {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Escape" { SwitchToMode "Normal"; }
        bind "h" "Left"  { MoveFocus "Left"; SwitchToMode "Normal"; }
        bind "j" "Down"  { MoveFocus "Down"; SwitchToMode "Normal"; }
        bind "k" "Up"    { MoveFocus "Up"; SwitchToMode "Normal"; }
        bind "l" "Right" { MoveFocus "Right"; SwitchToMode "Normal"; }
        bind "n"     { NewPane; SwitchToMode "Normal"; }
        bind "d"     { NewPane "Down"; SwitchToMode "Normal"; }
        bind "r"     { NewPane "Right"; SwitchToMode "Normal"; }
        bind "x"     { CloseFocus; SwitchToMode "Normal"; }
        bind "f"     { ToggleFocusFullscreen; SwitchToMode "Normal"; }
        bind "z"     { TogglePaneFrames; SwitchToMode "Normal"; }
        bind "e"     { TogglePaneEmbedOrFloating; SwitchToMode "Normal"; }
        bind "=" { Resize "Increase"; }
        bind "-" { Resize "Decrease"; }
        bind "t"     { NewTab; SwitchToMode "Normal"; }
        bind "w"     { CloseTab; SwitchToMode "Normal"; }
        bind "["     { GoToPreviousTab; SwitchToMode "Normal"; }
        bind "]"     { GoToNextTab; SwitchToMode "Normal"; }
        bind "1"     { GoToTab 1; SwitchToMode "Normal"; }
        bind "2"     { GoToTab 2; SwitchToMode "Normal"; }
        bind "3"     { GoToTab 3; SwitchToMode "Normal"; }
        bind "4"     { GoToTab 4; SwitchToMode "Normal"; }
        bind "5"     { GoToTab 5; SwitchToMode "Normal"; }
        bind "i"     { MoveTab "Left"; SwitchToMode "Normal"; }
        bind "o"     { MoveTab "Right"; SwitchToMode "Normal"; }
        bind "s" {
            LaunchOrFocusPlugin "zellij:strider" {
                floating true
                move_to_focused_tab true
            }
            SwitchToMode "Normal"
        }
        bind "m" {
            LaunchOrFocusPlugin "zellij:session-manager" {
                floating true
                move_to_focused_tab true
            }
            SwitchToMode "Normal"
        }
        bind "u" { SwitchToMode "Scroll"; }
        bind "/" { SwitchToMode "EnterSearch"; SearchInput 0; }
    }
    scroll {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Escape" "q" { SwitchToMode "Normal"; }
        bind "j" "Down"  { ScrollDown; }
        bind "k" "Up"    { ScrollUp; }
        bind "d"         { HalfPageScrollDown; }
        bind "u"         { HalfPageScrollUp; }
        bind "f" "PageDown" { PageScrollDown; }
        bind "b" "PageUp"   { PageScrollUp; }
        bind "g"         { ScrollToTop; }
        bind "G"         { ScrollToBottom; }
        bind "/"         { SwitchToMode "EnterSearch"; SearchInput 0; }
    }
    search {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Escape" { SwitchToMode "Normal"; }
        bind "n"     { Search "down"; }
        bind "N"     { Search "up"; }
        bind "c"     { SearchToggleOption "CaseSensitivity"; }
        bind "w"     { SearchToggleOption "Wrap"; }
        bind "o"     { SearchToggleOption "WholeWord"; }
    }
    entersearch {
        bind "Ctrl c" "Escape" { SwitchToMode "Normal"; }
        bind "Enter" { SwitchToMode "Search"; }
    }
}
"#;

/// Default zellij gruvbox theme.
const DEFAULT_ZELLIJ_THEME: &str = r##"themes {
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
"##;

/// Default zellij dev layout — VS Code-like.
const DEFAULT_ZELLIJ_LAYOUT: &str = r#"layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }
    tab name="dev" focus=true {
        pane split_direction="vertical" {
            pane size="20%" name="files" {
                command "yazi"
                cwd "/workspace"
            }
            pane split_direction="horizontal" {
                pane size="60%" name="editor" focus=true {
                    command "vim"
                    cwd "/workspace"
                }
                pane stacked=true size="40%" {
                    pane name="terminal" {
                        command "bash"
                        cwd "/workspace"
                    }
                    pane name="claude" {
                        command "claude"
                        cwd "/workspace"
                    }
                }
            }
        }
    }
    tab name="git" {
        pane name="lazygit" {
            command "lazygit"
            cwd "/workspace"
        }
    }
    tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
        }
    }
    tab name="help" {
        pane name="cheatsheet" {
            command "less"
            args "-R" "/root/.config/cheatsheet.txt"
        }
    }
}
"#;

/// Zellij assist layout — Claude-focused.
const DEFAULT_ZELLIJ_ASSIST_LAYOUT: &str = r#"layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }
    tab name="assist" focus=true {
        pane split_direction="vertical" {
            pane size="20%" name="files" {
                command "yazi"
                cwd "/workspace"
            }
            pane stacked=true size="40%" {
                pane name="claude" focus=true {
                    command "claude"
                    cwd "/workspace"
                }
                pane name="terminal" {
                    command "bash"
                    cwd "/workspace"
                }
            }
            pane name="editor" {
                command "vim"
                cwd "/workspace"
            }
        }
    }
    tab name="git" {
        pane name="lazygit" {
            command "lazygit"
            cwd "/workspace"
        }
    }
    tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
        }
    }
    tab name="help" {
        pane name="cheatsheet" {
            command "less"
            args "-R" "/root/.config/cheatsheet.txt"
        }
    }
}
"#;

/// Zellij focus layout — minimal, stacked main panes.
const DEFAULT_ZELLIJ_FOCUS_LAYOUT: &str = r#"layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }
    tab name="focus" focus=true {
        pane split_direction="vertical" {
            pane size="20%" name="files" {
                command "yazi"
                cwd "/workspace"
            }
            pane stacked=true {
                pane name="terminal" focus=true {
                    command "bash"
                    cwd "/workspace"
                }
                pane name="claude" {
                    command "claude"
                    cwd "/workspace"
                }
                pane name="editor" {
                    command "vim"
                    cwd "/workspace"
                }
            }
        }
    }
    tab name="git" {
        pane name="lazygit" {
            command "lazygit"
            cwd "/workspace"
        }
    }
    tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
        }
    }
    tab name="help" {
        pane name="cheatsheet" {
            command "less"
            args "-R" "/root/.config/cheatsheet.txt"
        }
    }
}
"#;

/// Default yazi config.
const DEFAULT_YAZI_CONFIG: &str = r#"[manager]
ratio = [0, 1, 0]
sort_by = "natural"
sort_sensitive = false
sort_dir_first = true
show_hidden = true
show_symlink = true

[preview]
max_width = 600
max_height = 900

[opener]
edit = [
    { run = 'open-in-editor "$1"', desc = "Open in editor pane", block = false },
]
edit-here = [
    { run = '${EDITOR:-vim} "$@"', desc = "Edit in-place", block = true },
]

[open]
rules = [
    { mime = "text/*", use = "edit" },
    { name = "*", use = "edit" },
]
"#;

/// Default yazi keymap.
const DEFAULT_YAZI_KEYMAP: &str = r#"[manager]
prepend_keymap = [
    { on = "<Enter>", run = "open", desc = "Open in editor pane" },
    { on = "O", run = "open --interactive", desc = "Open interactively" },
]
"#;

/// Quick reference cheatsheet.
const DEFAULT_CHEATSHEET: &str = r#"  dev-box Quick Reference
  ───────────────────────────────────────────────
  ZELLIJ (leader: Ctrl+g)    YAZI (file manager)
  Ctrl+g h/j/k/l  Move       h/j/k/l  Navigate
  Ctrl+g [/]       Prev/next  Enter    Open in vim
  Ctrl+g 1-5       Jump tab   q        Quit yazi
  Ctrl+g f         Fullscreen /        Search
  Ctrl+g x         Close pane .        Hidden files
  Ctrl+g n/d/r     New pane   Space    Select
  Ctrl+g t/w       Tab +/-
  Ctrl+g s         Strider
  Ctrl+g u         Scroll
  Ctrl+g /         Search
  Ctrl+q           QUIT ALL

  LAYOUTS: dev-box start --layout dev|assist|focus
  TABS: Ctrl+g 1 dev  2 git  3 shell  4 help
"#;

/// Default .asoundrc for PulseAudio over TCP.
const DEFAULT_ASOUNDRC: &str = r#"pcm.!default {
    type pulse
}
ctl.!default {
    type pulse
}
"#;

/// Seed the .root/ directory structure and default config files.
/// Never overwrites existing files.
pub fn seed_root_dir(config: &DevBoxConfig) -> Result<()> {
    let root = config.host_root_dir();

    let root_display = root.display();
    output::info(&format!("Seeding {} directory...", root_display));

    // Create directory structure — base dirs always needed
    let mut dirs = vec![
        root.join(".ssh"),
        root.join(".vim").join("undo"),
        root.join(".config").join("zellij").join("themes"),
        root.join(".config").join("zellij").join("layouts"),
        root.join(".config").join("yazi"),
        root.join(".config").join("git"),
    ];

    // AI provider directories — only create what's configured
    for provider in &config.ai.providers {
        match provider {
            crate::config::AiProvider::Claude => {
                dirs.push(root.join(".claude"));
            }
        }
    }

    for dir in &dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    // Seed config files (never overwrite)
    seed_file(&root.join(".vim").join("vimrc"), DEFAULT_VIMRC)?;
    seed_file(
        &root.join(".config").join("git").join("config"),
        DEFAULT_GITCONFIG,
    )?;
    seed_file(
        &root.join(".config").join("zellij").join("config.kdl"),
        DEFAULT_ZELLIJ_CONFIG,
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("themes")
            .join("gruvbox.kdl"),
        DEFAULT_ZELLIJ_THEME,
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("dev.kdl"),
        DEFAULT_ZELLIJ_LAYOUT,
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("assist.kdl"),
        DEFAULT_ZELLIJ_ASSIST_LAYOUT,
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("focus.kdl"),
        DEFAULT_ZELLIJ_FOCUS_LAYOUT,
    )?;

    // Yazi config
    seed_file(
        &root.join(".config").join("yazi").join("yazi.toml"),
        DEFAULT_YAZI_CONFIG,
    )?;
    seed_file(
        &root.join(".config").join("yazi").join("keymap.toml"),
        DEFAULT_YAZI_KEYMAP,
    )?;

    // Cheatsheet
    seed_file(
        &root.join(".config").join("cheatsheet.txt"),
        DEFAULT_CHEATSHEET,
    )?;

    // Audio config
    if config.audio.enabled {
        seed_file(&root.join(".asoundrc"), DEFAULT_ASOUNDRC)?;
    }

    // Warn if .ssh/ is empty
    let ssh_dir = root.join(".ssh");
    if ssh_dir.exists() {
        let entries = fs::read_dir(&ssh_dir)
            .with_context(|| format!("Failed to read .ssh directory: {}", ssh_dir.display()))?;
        if entries.count() == 0 {
            output::warn(&format!(
                "No SSH keys found in {}/.ssh/ — copy your keys manually if needed",
                root_display
            ));
        }
    }

    output::ok("Directory seeding complete");
    Ok(())
}

/// Write content to a file only if it doesn't already exist.
/// Delegates to the shared `write_if_missing` helper in context.rs.
fn seed_file(path: &Path, content: &str) -> Result<()> {
    crate::context::write_if_missing(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use serial_test::serial;
    use std::collections::HashMap;

    fn make_config(audio_enabled: bool, root_dir: std::path::PathBuf) -> DevBoxConfig {
        // We override DEV_BOX_HOST_ROOT to point to our temp root
        unsafe {
            std::env::set_var("DEV_BOX_HOST_ROOT", root_dir.to_str().unwrap());
        }
        DevBoxConfig {
            dev_box: DevBoxSection {
                version: "0.1.0".to_string(),
                image: ImageFlavor::Base,
                process: ProcessFlavor::Minimal,
            },
            container: ContainerSection {
                name: "test".to_string(),
                hostname: "test".to_string(),
                user: "root".to_string(),
                ports: vec![],
                extra_packages: vec![],
                extra_volumes: vec![],
                environment: HashMap::new(),
                post_create_command: None,
                vscode_extensions: vec![],
            },
            context: ContextSection::default(),
            ai: crate::config::AiSection::default(),
            audio: AudioSection {
                enabled: audio_enabled,
                pulse_server: "tcp:localhost:4714".to_string(),
            },
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_directories() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(root.join(".ssh").is_dir());
        assert!(root.join(".vim").join("undo").is_dir());
        assert!(root.join(".config").join("zellij").join("themes").is_dir());
        assert!(root.join(".config").join("zellij").join("layouts").is_dir());
        assert!(root.join(".config").join("yazi").is_dir());
        assert!(root.join(".config").join("git").is_dir());
        assert!(root.join(".claude").is_dir());

        unsafe {
            std::env::remove_var("DEV_BOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_config_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(root.join(".vim").join("vimrc").exists());
        assert!(root.join(".config").join("git").join("config").exists());
        assert!(
            root.join(".config")
                .join("zellij")
                .join("config.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("themes")
                .join("gruvbox.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("dev.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("assist.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("focus.kdl")
                .exists()
        );
        assert!(root.join(".config").join("yazi").join("yazi.toml").exists());
        assert!(
            root.join(".config")
                .join("yazi")
                .join("keymap.toml")
                .exists()
        );
        assert!(root.join(".config").join("cheatsheet.txt").exists());

        unsafe {
            std::env::remove_var("DEV_BOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_does_not_overwrite_existing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(root.join(".vim")).unwrap();
        fs::write(root.join(".vim").join("vimrc"), "custom vimrc").unwrap();

        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        let content = fs::read_to_string(root.join(".vim").join("vimrc")).unwrap();
        assert_eq!(
            content, "custom vimrc",
            "should not overwrite existing file"
        );

        unsafe {
            std::env::remove_var("DEV_BOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_asoundrc_when_audio_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(true, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(
            root.join(".asoundrc").exists(),
            ".asoundrc should be created when audio enabled"
        );

        unsafe {
            std::env::remove_var("DEV_BOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_no_asoundrc_when_audio_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(
            !root.join(".asoundrc").exists(),
            ".asoundrc should not exist when audio disabled"
        );

        unsafe {
            std::env::remove_var("DEV_BOX_HOST_ROOT");
        }
    }

    #[test]
    fn seed_file_creates_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        seed_file(&path, "hello world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn seed_file_skips_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();
        seed_file(&path, "new content").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    }
}
