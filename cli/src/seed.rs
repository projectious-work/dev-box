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
theme "gruvbox-dark"
default_layout "dev"
default_shell "bash"
pane_frames false
simplified_ui true
copy_on_select true

keybinds clear-defaults=true {
    shared {
        bind "Alt h" { MoveFocusOrTab "Left"; }
        bind "Alt j" { MoveFocus "Down"; }
        bind "Alt k" { MoveFocus "Up"; }
        bind "Alt l" { MoveFocusOrTab "Right"; }
        bind "Alt n" { NewPane; }
        bind "Alt d" { NewPane "Down"; }
        bind "Alt r" { NewPane "Right"; }
        bind "Alt x" { CloseFocus; }
        bind "Alt f" { ToggleFocusFullscreen; }
        bind "Alt t" { NewTab; }
        bind "Alt w" { CloseTab; }
        bind "Alt [" { GoToPreviousTab; }
        bind "Alt ]" { GoToNextTab; }
        bind "Alt 1" { GoToTab 1; }
        bind "Alt 2" { GoToTab 2; }
        bind "Alt 3" { GoToTab 3; }
        bind "Alt 4" { GoToTab 4; }
        bind "Alt 5" { GoToTab 5; }
        bind "Alt u" { SwitchToMode "scroll"; }
        bind "Alt /" { SwitchToMode "entersearch"; }
        bind "Alt s" {
            LaunchOrFocusPlugin "strider" {
                floating true
            }
        }
        bind "Alt m" {
            LaunchOrFocusPlugin "session-manager" {
                floating true
            }
        }
        bind "Ctrl q" { Quit; }
    }
    scroll {
        bind "j" "Down" { ScrollDown; }
        bind "k" "Up" { ScrollUp; }
        bind "d" { HalfPageScrollDown; }
        bind "u" { HalfPageScrollUp; }
        bind "Esc" "q" { SwitchToMode "normal"; }
    }
    entersearch {
        bind "Enter" { SwitchToMode "search"; }
        bind "Esc" { SwitchToMode "normal"; }
    }
    search {
        bind "n" { Search "down"; }
        bind "N" { Search "up"; }
        bind "Esc" "q" { SwitchToMode "normal"; }
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

/// Default zellij dev layout.
const DEFAULT_ZELLIJ_LAYOUT: &str = r#"layout {
    tab name="dev" focus=true {
        pane split_direction="horizontal" {
            pane command="bash" size="15%" {
                args "--login"
            }
            pane split_direction="vertical" {
                pane command="vim" size="80%"
                pane split_direction="horizontal" size="20%" {
                    pane command="bash" {
                        args "--login"
                    }
                    pane command="bash" {
                        args "--login"
                    }
                }
            }
        }
    }
    tab name="git" {
        pane command="lazygit"
    }
    tab name="shell" {
        pane command="bash" {
            args "--login"
        }
    }
}
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

    output::info("Seeding .root/ directory...");

    // Create directory structure
    let dirs = [
        root.join(".ssh"),
        root.join(".vim").join("undo"),
        root.join(".config").join("zellij").join("themes"),
        root.join(".config").join("zellij").join("layouts"),
        root.join(".config").join("git"),
        root.join(".claude"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    // Seed config files (never overwrite)
    seed_file(&root.join(".vim").join("vimrc"), DEFAULT_VIMRC)?;
    seed_file(&root.join(".config").join("git").join("config"), DEFAULT_GITCONFIG)?;
    seed_file(
        &root.join(".config").join("zellij").join("config.kdl"),
        DEFAULT_ZELLIJ_CONFIG,
    )?;
    seed_file(
        &root.join(".config").join("zellij").join("themes").join("gruvbox.kdl"),
        DEFAULT_ZELLIJ_THEME,
    )?;
    seed_file(
        &root.join(".config").join("zellij").join("layouts").join("dev.kdl"),
        DEFAULT_ZELLIJ_LAYOUT,
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
            output::warn("No SSH keys found in .root/.ssh/ — copy your keys manually if needed");
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
        unsafe { std::env::set_var("DEV_BOX_HOST_ROOT", root_dir.to_str().unwrap()); }
        DevBoxConfig {
            dev_box: DevBoxSection {
                version: "0.1.0".to_string(),
                image: ImageFlavor::Base,
                process: ProcessFlavor::Minimal,
            },
            container: ContainerSection {
                name: "test".to_string(),
                hostname: "test".to_string(),
                ports: vec![],
                extra_packages: vec![],
                extra_volumes: vec![],
                environment: HashMap::new(),
            },
            context: ContextSection::default(),
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
        assert!(root.join(".config").join("git").is_dir());
        assert!(root.join(".claude").is_dir());

        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
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
        assert!(root.join(".config").join("zellij").join("config.kdl").exists());
        assert!(root.join(".config").join("zellij").join("themes").join("gruvbox.kdl").exists());
        assert!(root.join(".config").join("zellij").join("layouts").join("dev.kdl").exists());

        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
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
        assert_eq!(content, "custom vimrc", "should not overwrite existing file");

        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_asoundrc_when_audio_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(true, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(root.join(".asoundrc").exists(), ".asoundrc should be created when audio enabled");

        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
    }

    #[test]
    #[serial]
    fn seed_root_dir_no_asoundrc_when_audio_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(!root.join(".asoundrc").exists(), ".asoundrc should not exist when audio disabled");

        unsafe { std::env::remove_var("DEV_BOX_HOST_ROOT"); }
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
