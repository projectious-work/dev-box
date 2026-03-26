use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::AiboxConfig;
use crate::output;

/// Default vimrc content (embedded fallback).
const DEFAULT_VIMRC: &str = r#"" aibox default vimrc
set nocompatible
let mapleader=" "

set number relativenumber
set tabstop=4 shiftwidth=4 expandtab smartindent
set undofile undodir=~/.vim/undo
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

set background=AIBOX_VIM_BG
set termguicolors
colorscheme AIBOX_VIM_COLORSCHEME
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

/// Default zellij config.kdl content. Theme name is replaced at seed time.
const DEFAULT_ZELLIJ_CONFIG: &str = r#"// aibox zellij configuration
theme "AIBOX_THEME"
default_layout "dev"
default_shell "bash"
mouse_mode true
copy_on_select true
scroll_buffer_size 10000
rounded_corners true
simplified_ui false
pane_frames true

// Leader: Ctrl+b (press Ctrl+b, release, then press the action key)
// Quick reference:
//   Ctrl+b → h/j/k/l    Navigate panes
//   Ctrl+b → n/d/r       New pane / split down / split right
//   Ctrl+b → x           Close pane
//   Ctrl+b → f           Toggle fullscreen
//   Ctrl+b → z           Toggle pane frames
//   Ctrl+b → t/w         New tab / close tab
//   Ctrl+b → [/]         Previous/next tab
//   Ctrl+b → 1-5         Jump to tab
//   Ctrl+b → s           Strider file picker
//   Ctrl+b → u           Scroll mode
//   Ctrl+b → /           Search scrollback
//   Ctrl+q               Quit zellij
keybinds clear-defaults=true {
    normal {
        bind "Ctrl b" { SwitchToMode "Tmux"; }
        bind "Ctrl q" { Quit; }
    }
    tmux {
        bind "Ctrl b" { SwitchToMode "Normal"; }
        bind "Esc" { SwitchToMode "Normal"; }
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
        bind "q" { Quit; }
    }
    scroll {
        bind "Ctrl b" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Esc" "q" { SwitchToMode "Normal"; }
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
        bind "Ctrl b" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Esc" { SwitchToMode "Normal"; }
        bind "n"     { Search "down"; }
        bind "N"     { Search "up"; }
        bind "c"     { SearchToggleOption "CaseSensitivity"; }
        bind "w"     { SearchToggleOption "Wrap"; }
        bind "o"     { SearchToggleOption "WholeWord"; }
    }
    entersearch {
        bind "Ctrl c" "Esc" { SwitchToMode "Normal"; }
        bind "Enter" { SwitchToMode "Search"; }
    }
}
"#;

/// Generate the KDL snippet for AI provider panes in a tab.
/// Returns empty string if no providers are configured.
fn ai_pane_kdl(providers: &[crate::config::AiProvider]) -> String {
    if providers.is_empty() {
        return String::new();
    }

    let panes: Vec<String> = providers
        .iter()
        .map(|p| {
            let cmd = p.to_string();
            format!(
                "        pane name=\"{cmd}\" {{\n\
                 \x20           command \"{cmd}\"\n\
                 \x20           cwd \"/workspace\"\n\
                 \x20       }}"
            )
        })
        .collect();

    if panes.len() == 1 {
        panes[0].clone()
    } else {
        format!(
            "        pane stacked=true {{\n{}\n        }}",
            panes.join("\n")
        )
    }
}

fn ai_tabs_kdl(providers: &[crate::config::AiProvider]) -> String {
    if providers.is_empty() {
        return String::new();
    }

    providers
        .iter()
        .map(|p| {
            let cmd = p.to_string();
            format!(
                "    tab name=\"{cmd}\" {{\n\
                 \x20       pane name=\"{cmd}\" {{\n\
                 \x20           command \"{cmd}\"\n\
                 \x20           cwd \"/workspace\"\n\
                 \x20       }}\n\
                 \x20   }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate the zellij dev layout dynamically based on configured AI providers.
fn generate_dev_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_tabs = ai_tabs_kdl(providers);
    let ai_section = if ai_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_tabs)
    };

    format!(
        r#"layout {{
    default_tab_template {{
        children
        pane size=1 borderless=true {{
            plugin location="zellij:status-bar"
        }}
    }}
    tab name="dev" focus=true {{
        pane split_direction="vertical" {{
            pane size="40%" name="files" focus=true {{
                command "yazi"
                cwd "/workspace"
            }}
            pane size="60%" name="editor" {{
                command "vim-loop"
                cwd "/workspace"
            }}
        }}
    }}{ai_section}
    tab name="git" {{
        pane name="lazygit" {{
            command "lazygit"
            cwd "/workspace"
        }}
    }}
    tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
        }}
    }}
}}
"#
    )
}

/// Generate the zellij focus layout dynamically based on configured AI providers.
fn generate_focus_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_tabs = ai_tabs_kdl(providers);
    let ai_section = if ai_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_tabs)
    };

    format!(
        r#"layout {{
    default_tab_template {{
        children
        pane size=1 borderless=true {{
            plugin location="zellij:status-bar"
        }}
    }}
    tab name="files" focus=true {{
        pane name="yazi" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
            cwd "/workspace"
        }}
    }}
    tab name="editor" {{
        pane name="vim" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
        }}
    }}{ai_section}
    tab name="git" {{
        pane name="lazygit" {{
            command "lazygit"
            cwd "/workspace"
        }}
    }}
    tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
        }}
    }}
}}
"#
    )
}

/// Generate the zellij cowork layout dynamically based on configured AI providers.
fn generate_cowork_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_pane = ai_pane_kdl(providers);

    if ai_pane.is_empty() {
        // No AI providers — full-width editor layout
        return r#"layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }
    tab name="cowork" focus=true {
        pane split_direction="vertical" {
            pane size="40%" name="files" focus=true {
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=down exec yazi"
                cwd "/workspace"
            }
            pane size="60%" name="editor" {
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=down exec vim-loop"
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
}
"#
        .to_string();
    }

    format!(
        r#"layout {{
    default_tab_template {{
        children
        pane size=1 borderless=true {{
            plugin location="zellij:status-bar"
        }}
    }}
    tab name="cowork" focus=true {{
        pane split_direction="vertical" {{
            pane size="50%" split_direction="horizontal" {{
                pane size="40%" name="files" focus=true {{
                    command "bash"
                    args "-c" "AIBOX_EDITOR_DIR=down exec yazi"
                    cwd "/workspace"
                }}
                pane size="60%" name="editor" {{
                    command "bash"
                    args "-c" "AIBOX_EDITOR_DIR=down exec vim-loop"
                    cwd "/workspace"
                }}
            }}
            pane size="50%" {{
{ai_pane}
            }}
        }}
    }}
    tab name="git" {{
        pane name="lazygit" {{
            command "lazygit"
            cwd "/workspace"
        }}
    }}
    tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
        }}
    }}
}}
"#
    )
}

/// Default yazi config.
const DEFAULT_YAZI_CONFIG: &str = r#"[manager]
ratio = [1, 3, 4]
sort_by = "natural"
sort_sensitive = false
sort_dir_first = true
show_hidden = true
show_symlink = true

[preview]
max_width = 600
max_height = 900
image_delay = 30
image_filter = "nearest"

[plugin]
prepend_previewers = [
    { name = "*.svg",  run = "svg" },
    { name = "*.eps",  run = "eps" },
    { name = "*.jpg",  run = "image" },
    { name = "*.jpeg", run = "image" },
    { name = "*.png",  run = "image" },
    { name = "*.gif",  run = "image" },
    { name = "*.webp", run = "image" },
    { name = "*.bmp",  run = "image" },
    { name = "*.tiff", run = "image" },
    { name = "*.tif",  run = "image" },
    { name = "*.pdf",  run = "pdf" },
]

[opener]
edit = [
    { run = '${EDITOR:-vim} "$@"', desc = "Edit in-place", block = true },
]
edit-pane = [
    { run = 'open-in-editor "$1"', desc = "Open in vim pane", block = false },
]

[open]
rules = [
    { mime = "text/*", use = "edit" },
    { name = "*", use = "edit" },
]
"#;

/// EPS previewer plugin — converts EPS to PNG via ghostscript.
const DEFAULT_YAZI_PLUGIN_EPS: &str = r#"-- eps.yazi — EPS previewer for yazi
-- Converts EPS to PNG using ghostscript, then renders via the built-in image previewer.
-- Requires: ghostscript (gs) in PATH.

local function fail(msg)
	return Err(msg)
end

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return fail("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		local ok, err, code = Command("gs")
			:args({
				"-q",
				"-dNOPAUSE",
				"-dBATCH",
				"-dSAFER",
				"-sDEVICE=png16m",
				"-r150",
				"-dEPSCrop",
				"-sOutputFile=" .. tostring(cache),
				tostring(job.file.url),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if not ok then
			return fail("gs not found or failed (code " .. tostring(code) .. "): " .. tostring(err))
		end

		return Image:new(job, cache):show()
	end,
}
"#;

/// SVG previewer plugin — converts SVG to PNG via resvg.
const DEFAULT_YAZI_PLUGIN_SVG: &str = r#"-- svg.yazi — SVG previewer for yazi
-- Converts SVG to PNG using resvg, then renders via the built-in image previewer.
-- Requires: resvg in PATH.

local function fail(msg)
	return Err(msg)
end

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return fail("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		local ok, err, code = Command("resvg")
			:args({
				"--width",
				tostring(job.area.w * 4),
				"--height",
				tostring(job.area.h * 4),
				tostring(job.file.url),
				tostring(cache),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if not ok then
			return fail("resvg not found or failed (code " .. tostring(code) .. "): " .. tostring(err))
		end

		return Image:new(job, cache):show()
	end,
}
"#;

/// Default yazi keymap.
const DEFAULT_YAZI_KEYMAP: &str = r#"[mgr]
prepend_keymap = [
    { on = "<Enter>", run = "open", desc = "Edit in-place" },
    { on = "e", run = "shell 'open-in-editor \"$1\"'", desc = "Open in vim pane" },
    { on = "O", run = "open --interactive", desc = "Open interactively" },
]
"#;

/// Quick reference cheatsheet.
const DEFAULT_CHEATSHEET: &str = r#"  aibox Quick Reference
  ───────────────────────────────────────────────
  ZELLIJ (leader: Ctrl+b)    YAZI (file manager)
  Ctrl+b h/j/k/l  Move       h/j/k/l  Navigate
  Ctrl+b [/]       Prev/next  Enter    Open in vim
  Ctrl+b 1-5       Jump tab   q        Quit yazi
  Ctrl+b f         Fullscreen /        Search
  Ctrl+b x         Close pane .        Hidden files
  Ctrl+b n/d/r     New pane   Space    Select
  Ctrl+b t/w       Tab +/-
  Ctrl+b s         Strider
  Ctrl+b u         Scroll
  Ctrl+b /         Search
  Ctrl+b q         QUIT (or Ctrl+q)

  LAYOUTS: aibox start --layout dev|focus|cowork
  TABS: Ctrl+b 1 dev  2 git  3 shell  4 help
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
pub fn seed_root_dir(config: &AiboxConfig) -> Result<()> {
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
        root.join(".config").join("yazi").join("plugins").join("eps.yazi"),
        root.join(".config").join("yazi").join("plugins").join("svg.yazi"),
        root.join(".config").join("git"),
        root.join(".config").join("lazygit"),
    ];

    // AI provider directories — only create what's configured
    for provider in &config.ai.providers {
        match provider {
            crate::config::AiProvider::Claude => {
                dirs.push(root.join(".claude"));
            }
            crate::config::AiProvider::Aider => {
                // Aider uses ~/.aider for config
                dirs.push(root.join(".aider"));
            }
            crate::config::AiProvider::Gemini => {
                // Gemini CLI uses ~/.gemini for config
                dirs.push(root.join(".gemini"));
            }
            crate::config::AiProvider::Mistral => {
                dirs.push(root.join(".mistral"));
            }
        }
    }

    for dir in &dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    // Seed config files (never overwrite)
    let theme = &config.appearance.theme;
    let vimrc = DEFAULT_VIMRC
        .replace("AIBOX_VIM_COLORSCHEME", crate::themes::vim_colorscheme(theme))
        .replace("AIBOX_VIM_BG", crate::themes::vim_background(theme));
    seed_file(&root.join(".vim").join("vimrc"), &vimrc)?;
    seed_file(
        &root.join(".config").join("git").join("config"),
        DEFAULT_GITCONFIG,
    )?;

    // Zellij config — apply selected theme
    let zellij_config = DEFAULT_ZELLIJ_CONFIG
        .replace("AIBOX_THEME", &theme.to_string());
    seed_file(
        &root.join(".config").join("zellij").join("config.kdl"),
        &zellij_config,
    )?;

    // Zellij theme file — seed the selected theme
    let theme_filename = format!("{}.kdl", &theme.to_string());
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("themes")
            .join(&theme_filename),
        crate::themes::zellij_theme(theme),
    )?;
    // Zellij layouts — generated dynamically based on AI providers
    let providers = &config.ai.providers;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("dev.kdl"),
        &generate_dev_layout(providers),
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("focus.kdl"),
        &generate_focus_layout(providers),
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("cowork.kdl"),
        &generate_cowork_layout(providers),
    )?;

    // Yazi config + theme
    seed_file(
        &root.join(".config").join("yazi").join("yazi.toml"),
        DEFAULT_YAZI_CONFIG,
    )?;
    seed_file(
        &root.join(".config").join("yazi").join("keymap.toml"),
        DEFAULT_YAZI_KEYMAP,
    )?;
    seed_file(
        &root.join(".config").join("yazi").join("theme.toml"),
        crate::themes::yazi_theme(theme),
    )?;
    // Yazi plugins — custom previewers for EPS and SVG
    seed_file(
        &root
            .join(".config")
            .join("yazi")
            .join("plugins")
            .join("eps.yazi")
            .join("init.lua"),
        DEFAULT_YAZI_PLUGIN_EPS,
    )?;
    seed_file(
        &root
            .join(".config")
            .join("yazi")
            .join("plugins")
            .join("svg.yazi")
            .join("init.lua"),
        DEFAULT_YAZI_PLUGIN_SVG,
    )?;

    // Cheatsheet
    seed_file(
        &root.join(".config").join("cheatsheet.txt"),
        DEFAULT_CHEATSHEET,
    )?;

    // Starship prompt config
    let prompt = &config.appearance.prompt;
    let starship_content = crate::themes::starship_config(prompt, theme);
    seed_file(
        &root.join(".config").join("starship.toml"),
        &starship_content,
    )?;

    // lazygit theme config
    seed_file(
        &root
            .join(".config")
            .join("lazygit")
            .join("config.yml"),
        crate::themes::lazygit_theme(theme),
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

fn seed_file(path: &Path, content: &str) -> Result<()> {
    crate::context::write_if_missing(path, content)
}

/// Write content to a file, overwriting if content differs.
/// Returns true if the file changed, false if content was already identical.
pub fn force_seed_file(path: &Path, content: &str) -> Result<bool> {
    crate::context::write_if_changed(path, content)
}

/// Force-seed all theme-dependent and AI-provider-dependent config files.
/// Overwrites existing files when content has changed. Used by `aibox sync`.
pub fn sync_theme_files(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let theme = &config.appearance.theme;
    let providers = &config.ai.providers;
    let mut updated = Vec::new();

    // vimrc — colorscheme and background
    let vimrc = DEFAULT_VIMRC
        .replace("AIBOX_VIM_COLORSCHEME", crate::themes::vim_colorscheme(theme))
        .replace("AIBOX_VIM_BG", crate::themes::vim_background(theme));
    if force_seed_file(&root.join(".vim").join("vimrc"), &vimrc)? {
        updated.push(".vim/vimrc".to_string());
    }

    // Zellij config — theme name
    let zellij_config = DEFAULT_ZELLIJ_CONFIG
        .replace("AIBOX_THEME", &theme.to_string());
    if force_seed_file(
        &root.join(".config").join("zellij").join("config.kdl"),
        &zellij_config,
    )? {
        updated.push(".config/zellij/config.kdl".to_string());
    }

    // Zellij theme file
    let theme_filename = format!("{}.kdl", &theme.to_string());
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("themes")
            .join(&theme_filename),
        crate::themes::zellij_theme(theme),
    )? {
        updated.push(format!(".config/zellij/themes/{}", theme_filename));
    }

    // Zellij layouts — depend on AI providers
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("dev.kdl"),
        &generate_dev_layout(providers),
    )? {
        updated.push(".config/zellij/layouts/dev.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("focus.kdl"),
        &generate_focus_layout(providers),
    )? {
        updated.push(".config/zellij/layouts/focus.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("cowork.kdl"),
        &generate_cowork_layout(providers),
    )? {
        updated.push(".config/zellij/layouts/cowork.kdl".to_string());
    }

    // lazygit config
    if force_seed_file(
        &root.join(".config").join("lazygit").join("config.yml"),
        crate::themes::lazygit_theme(theme),
    )? {
        updated.push(".config/lazygit/config.yml".to_string());
    }

    // Yazi theme
    if force_seed_file(
        &root.join(".config").join("yazi").join("theme.toml"),
        crate::themes::yazi_theme(theme),
    )? {
        updated.push(".config/yazi/theme.toml".to_string());
    }

    // Starship prompt
    let prompt = &config.appearance.prompt;
    let starship_content = crate::themes::starship_config(prompt, theme);
    if force_seed_file(
        &root.join(".config").join("starship.toml"),
        &starship_content,
    )? {
        updated.push(".config/starship.toml".to_string());
    }

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use serial_test::serial;

    fn make_config(audio_enabled: bool, root_dir: std::path::PathBuf) -> AiboxConfig {
        unsafe {
            std::env::set_var("AIBOX_HOST_ROOT", root_dir.to_str().unwrap());
        }
        let mut config = crate::config::test_config();
        config.container.name = "test".to_string();
        config.container.hostname = "test".to_string();
        config.audio = AudioSection {
            enabled: audio_enabled,
            pulse_server: "tcp:localhost:4714".to_string(),
        };
        config
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
            std::env::remove_var("AIBOX_HOST_ROOT");
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
                .join("gruvbox-dark.kdl")
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
                .join("focus.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("cowork.kdl")
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
            std::env::remove_var("AIBOX_HOST_ROOT");
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
            std::env::remove_var("AIBOX_HOST_ROOT");
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
            std::env::remove_var("AIBOX_HOST_ROOT");
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
            std::env::remove_var("AIBOX_HOST_ROOT");
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

    #[test]
    fn dev_layout_claude_only() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_dev_layout(&providers);
        assert!(layout.contains("tab name=\"claude\""), "should have claude tab");
        assert!(layout.contains("command \"claude\""), "should have claude command");
        assert!(!layout.contains("aider"), "should not have aider");
    }

    #[test]
    fn dev_layout_aider_only() {
        let providers = vec![AiProvider::Aider];
        let layout = generate_dev_layout(&providers);
        assert!(layout.contains("tab name=\"aider\""), "should have aider tab");
        assert!(layout.contains("command \"aider\""), "should have aider command");
        assert!(!layout.contains("claude"), "should not have claude");
    }

    #[test]
    fn dev_layout_multiple_providers() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_dev_layout(&providers);
        assert!(layout.contains("tab name=\"claude\""), "should have claude tab");
        assert!(layout.contains("tab name=\"aider\""), "should have aider tab");
    }

    #[test]
    fn dev_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_dev_layout(&providers);
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(!layout.contains("aider"), "should not have aider");
        assert!(!layout.contains("gemini"), "should not have gemini");
        assert!(layout.contains("tab name=\"dev\""), "should still have dev tab");
        assert!(layout.contains("tab name=\"git\""), "should still have git tab");
    }

    #[test]
    fn focus_layout_gemini() {
        let providers = vec![AiProvider::Gemini];
        let layout = generate_focus_layout(&providers);
        assert!(layout.contains("tab name=\"gemini\""), "should have gemini tab");
        assert!(layout.contains("command \"gemini\""), "should have gemini command");
    }

    #[test]
    fn cowork_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_layout(&providers);
        assert!(layout.contains("command \"claude\""), "should have claude pane");
        assert!(!layout.contains("stacked"), "single provider should not be stacked");
    }

    #[test]
    fn cowork_layout_multiple_providers_stacked() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_cowork_layout(&providers);
        assert!(layout.contains("stacked=true"), "multiple providers should be stacked");
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
    }

    #[test]
    fn cowork_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_cowork_layout(&providers);
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(layout.contains("tab name=\"cowork\""), "should still have cowork tab");
    }

    #[test]
    fn ai_pane_kdl_empty() {
        let result = ai_pane_kdl(&[]);
        assert!(result.is_empty(), "empty providers should produce empty string");
    }

    #[test]
    fn ai_pane_kdl_single() {
        let result = ai_pane_kdl(&[AiProvider::Claude]);
        assert!(result.contains("command \"claude\""));
        assert!(!result.contains("stacked"));
    }

    #[test]
    fn ai_pane_kdl_multiple() {
        let result = ai_pane_kdl(&[AiProvider::Claude, AiProvider::Aider, AiProvider::Gemini]);
        assert!(result.contains("stacked=true"));
        assert!(result.contains("command \"claude\""));
        assert!(result.contains("command \"aider\""));
        assert!(result.contains("command \"gemini\""));
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_aider_dir_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.providers = vec![AiProvider::Aider];
        seed_root_dir(&config).unwrap();

        assert!(root.join(".aider").is_dir(), ".aider directory should be created");
        assert!(!root.join(".claude").exists(), ".claude should not exist");

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_gemini_dir_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.providers = vec![AiProvider::Gemini];
        seed_root_dir(&config).unwrap();

        assert!(root.join(".gemini").is_dir(), ".gemini directory should be created");

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }
}
