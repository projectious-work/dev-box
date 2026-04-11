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

" ── VSCode-like cursor movement (v0.16.6+) ──────────────────────────────────
"
" Word and line jumps with the chords macOS / VSCode users expect, in
" insert mode AND in normal/visual modes so the same fingers work
" everywhere.
"
" Reliability:
"   <A-Left>/<A-Right>  → reliable across iTerm2, Ghostty, Alacritty,
"                          WezTerm, and zellij. Most terminals send
"                          ^[[1;3D / ^[[1;3C and vim recognises both.
"   <Home>/<End>        → universally reliable. Use these for line
"                          begin/end. To get macOS-native Cmd+Left /
"                          Cmd+Right behaviour, configure your
"                          terminal (iTerm2: Profiles → Keys; Ghostty:
"                          keybind config) to send Home/End on
"                          Cmd+Left / Cmd+Right.
"
" Insert-mode word movement uses <C-o> (one-shot normal command, then
" back to insert). The <Right> after `e` puts the cursor AFTER the
" word's last character, matching VSCode's "select to next word end"
" semantics; without it the cursor lands ON the last character.
inoremap <A-Left>  <C-o>b
inoremap <A-Right> <C-o>e<Right>
nnoremap <A-Left>  b
nnoremap <A-Right> e
vnoremap <A-Left>  b
vnoremap <A-Right> e

" Smart Home/End. Insert-mode <Home> goes to first non-whitespace
" (matches IDE 'smart home'); a second press goes to column 0.
" Vim's default insert-mode <End> already does the right thing
" (jumps past the last character) so we don't override it, but we
" also map normal/visual mode for consistency.
inoremap <expr> <Home> col('.') == match(getline('.'), '\S') + 1 ? "\<C-o>0" : "\<C-o>^"
nnoremap <Home> ^
nnoremap <End>  $
vnoremap <Home> ^
vnoremap <End>  $

" ── Scratch pad auto-save ───────────────────────────────────────────────────
" When vim opens /home/aibox/.scratch.md (the zellij floating scratch pane),
" save on every text change so `:q!` can never lose edits.
augroup scratch_autosave
  autocmd!
  autocmd TextChanged,TextChangedI /home/aibox/.scratch.md silent! write
augroup END

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

/// Default zellij config.kdl content. Theme name and layout are replaced at seed time.
const DEFAULT_ZELLIJ_CONFIG: &str = r#"// aibox zellij configuration
theme "AIBOX_THEME"
default_layout "AIBOX_LAYOUT"
default_shell "bash"
mouse_mode true
copy_on_select true
scroll_buffer_size 10000
rounded_corners true
simplified_ui false
pane_frames true

// Leader: Ctrl+g (press Ctrl+g, release, then press the action key)
// Quick reference:
//   Alt+h/j/k/l          Navigate panes (no leader needed; always shown in status bar)
//   Ctrl+g → h/j/k/l    Navigate panes (leader variant)
//   Ctrl+g → n/d/r       New pane / split down / split right
//   Ctrl+g → x           Close pane
//   Ctrl+g → f           Toggle fullscreen
//   Ctrl+g → z           Toggle pane frames
//   Ctrl+g → p           Toggle scratch notepad (vim floating pane)
//   Ctrl+g → t/w         New tab / close tab
//   Ctrl+g → [/]         Previous/next tab
//   Ctrl+g → ,/.         Previous/next stacked pane
//   Ctrl+g → 1-5         Jump to tab
//   Ctrl+g → s           Strider file picker
//   Ctrl+g → u           Scroll mode
//   Ctrl+g → /           Search scrollback
//   Ctrl+g → q           Quit zellij (entire session)
keybinds clear-defaults=true {
    normal {
        bind "Ctrl g" { SwitchToMode "Tmux"; }
        // Direct pane navigation — no leader needed; always visible in status bar.
        // Alt+Arrow keys are intentionally NOT bound here: terminal apps (bash
        // readline, vim, Claude Code) rely on Alt+Left/Right for word navigation.
        bind "Alt h" { MoveFocus "Left"; }
        bind "Alt j" { MoveFocus "Down"; }
        bind "Alt k" { MoveFocus "Up"; }
        bind "Alt l" { MoveFocus "Right"; }
    }
    tmux {
        bind "Ctrl g" { SwitchToMode "Normal"; }
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
        bind "p"     { ToggleFloatingPanes; SwitchToMode "Normal"; }
        bind "=" { Resize "Increase"; }
        bind "-" { Resize "Decrease"; }
        bind "t"     { NewTab; SwitchToMode "Normal"; }
        bind "w"     { CloseTab; SwitchToMode "Normal"; }
        bind "["     { GoToPreviousTab; SwitchToMode "Normal"; }
        bind "]"     { GoToNextTab; SwitchToMode "Normal"; }
        bind ","     { PreviousSwapLayout; SwitchToMode "Normal"; }
        bind "."     { NextSwapLayout; SwitchToMode "Normal"; }
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
        bind "Ctrl g" { SwitchToMode "Normal"; }
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
        bind "Ctrl g" { SwitchToMode "Normal"; }
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
            let name = p.to_string();
            let cmd = p.binary_name();
            format!(
                "        pane name=\"{name}\" {{\n\
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
            let name = p.to_string();
            let cmd = p.binary_name();
            format!(
                "    tab name=\"{name}\" {{\n\
                 \x20       pane name=\"{name}\" {{\n\
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
        r##"layout {{
    default_tab_template {{
        children
        floating_panes {{
            pane name="scratch" {{
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }}
        }}
        pane size=1 borderless=true {{
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {{
                format_left  "{{mode}}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }}
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
"##
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
        r##"layout {{
    default_tab_template {{
        children
        floating_panes {{
            pane name="scratch" {{
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }}
        }}
        pane size=1 borderless=true {{
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {{
                format_left  "{{mode}}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }}
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
"##
    )
}

/// Generate the zellij cowork layout dynamically based on configured AI providers.
fn generate_cowork_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_pane = ai_pane_kdl(providers);

    if ai_pane.is_empty() {
        // No AI providers — full-width editor layout
        return r##"layout {
    default_tab_template {
        children
        floating_panes {
            pane name="scratch" {
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }
        }
        pane size=1 borderless=true {
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {
                format_left  "{mode}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }
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
"##
        .to_string();
    }

    format!(
        r##"layout {{
    default_tab_template {{
        children
        floating_panes {{
            pane name="scratch" {{
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }}
        }}
        pane size=1 borderless=true {{
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {{
                format_left  "{{mode}}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }}
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
"##
    )
}

/// Generate the zellij cowork-swap layout dynamically based on configured AI providers.
///
/// cowork-swap is a re-arrangement of `cowork` for users who prefer the editor on
/// the right (the bigger pane). The outer split is 40/60 instead of cowork's
/// 50/50, and the editor and AI panes swap roles:
///
///   Tab 1 ("cowork-swap"):
///     ┌──────────────────┬────────────────────────────────────────┐
///     │  yazi (top, 40%) │                                        │
///     │                  │                                        │
///     ├──────────────────┤  vim editor (60%)                      │
///     │  AI agent (60%)  │                                        │
///     │                  │                                        │
///     └──────────────────┴────────────────────────────────────────┘
///   Tab 2 ("git"):    fullscreen lazygit
///   Tab 3 ("shell"):  fullscreen bash
///
/// When no AI providers are configured, the cowork-swap tab degenerates to
/// the same yazi-left + vim-right shape as `dev` (with the cowork-swap tab
/// name preserved).
///
/// AIBOX_EDITOR_DIR is "right" (the default) on yazi/vim because vim is
/// to the right of yazi geometrically — opening a file from yazi via `e`
/// moves focus right.
fn generate_cowork_swap_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_pane = ai_pane_kdl(providers);

    if ai_pane.is_empty() {
        // No AI providers — fall back to a simple yazi-left + vim-right shape
        // (same as dev, with the cowork-swap tab name preserved).
        return r##"layout {
    default_tab_template {
        children
        floating_panes {
            pane name="scratch" {
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }
        }
        pane size=1 borderless=true {
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {
                format_left  "{mode}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }
        }
    }
    tab name="cowork-swap" focus=true {
        pane split_direction="vertical" {
            pane size="40%" name="files" focus=true {
                command "yazi"
                cwd "/workspace"
            }
            pane size="60%" name="editor" {
                command "vim-loop"
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
"##
        .to_string();
    }

    format!(
        r##"layout {{
    default_tab_template {{
        children
        floating_panes {{
            pane name="scratch" {{
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }}
        }}
        pane size=1 borderless=true {{
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {{
                format_left  "{{mode}}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }}
        }}
    }}
    tab name="cowork-swap" focus=true {{
        pane split_direction="vertical" {{
            pane size="40%" split_direction="horizontal" {{
                pane size="40%" name="files" focus=true {{
                    command "yazi"
                    cwd "/workspace"
                }}
                pane size="60%" {{
{ai_pane}
                }}
            }}
            pane size="60%" name="editor" {{
                command "vim-loop"
                cwd "/workspace"
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
"##
    )
}

/// Generate the zellij ai layout dynamically based on configured AI providers.
///
/// AI layout: yazi-first, AI-first.
///   Tab 1 ("ai"):     left 50% yazi, right 50% AI agent pane (vertical split, no editor)
///   Tab 2 ("editor"): fullscreen vim
///   Tab 3 ("git"):    fullscreen lazygit
///   Tab 4 ("shell"):  fullscreen bash
///
/// When no AI providers are configured, the ai tab is fullscreen yazi (the
/// editor still lives in tab 2; opening files via `e` from yazi works as
/// usual).
fn generate_ai_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_pane = ai_pane_kdl(providers);

    if ai_pane.is_empty() {
        return r##"layout {
    default_tab_template {
        children
        floating_panes {
            pane name="scratch" {
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }
        }
        pane size=1 borderless=true {
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {
                format_left  "{mode}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }
        }
    }
    tab name="ai" focus=true {
        pane name="files" {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
            cwd "/workspace"
        }
    }
    tab name="editor" {
        pane name="vim" {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
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
"##
        .to_string();
    }

    format!(
        r##"layout {{
    default_tab_template {{
        children
        floating_panes {{
            pane name="scratch" {{
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }}
        }}
        pane size=1 borderless=true {{
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {{
                format_left  "{{mode}}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }}
        }}
    }}
    tab name="ai" focus=true {{
        pane split_direction="vertical" {{
            pane size="50%" name="files" focus=true {{
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
                cwd "/workspace"
            }}
            pane size="50%" {{
{ai_pane}
            }}
        }}
    }}
    tab name="editor" {{
        pane name="vim" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
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
"##
    )
}

/// Generate the zellij browse layout dynamically based on configured AI providers.
///
/// Browse layout: yazi-focused with large preview.
///   Tab 1 ("browse"): top 60% yazi, bottom 40% AI agent pane
///   Tab 2 ("editor"): fullscreen vim
///   Tab 3 ("git"):    fullscreen lazygit
///   Tab 4 ("shell"):  fullscreen bash
///
/// When no AI providers are configured, the browse tab is fullscreen yazi.
fn generate_browse_layout(providers: &[crate::config::AiProvider]) -> String {
    let ai_pane = ai_pane_kdl(providers);

    if ai_pane.is_empty() {
        return r##"layout {
    default_tab_template {
        children
        floating_panes {
            pane name="scratch" {
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }
        }
        pane size=1 borderless=true {
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {
                format_left  "{mode}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }
        }
    }
    tab name="browse" focus=true {
        pane name="files" focus=true {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
            cwd "/workspace"
        }
    }
    tab name="editor" {
        pane name="vim" {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
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
"##
        .to_string();
    }

    format!(
        r##"layout {{
    default_tab_template {{
        children
        floating_panes {{
            pane name="scratch" {{
                command "bash"
                args "-c" "while true; do vim -c 'startinsert' /home/aibox/.scratch.md; done"
                cwd "/workspace"
            }}
        }}
        pane size=1 borderless=true {{
            // zjstatus: per-mode key hints. URL version must match Zellij API — see
            // https://github.com/dj95/zjstatus/releases for the compatible release.
            plugin location="https://github.com/dj95/zjstatus/releases/download/v0.21.0/zjstatus.wasm" {{
                format_left  "{{mode}}"
                format_right ""
                format_space ""
                hide_frame_for_single_pane "false"

                mode_normal       "#[bg=#5e81ac,fg=#2e3440,bold] NORMAL #[bg=default,fg=#7b88a1]  ^g LEADER · g→q QUIT · g→n/d/r PANE · g→t/w TAB · Alt+h/j/k/l FOCUS"
                mode_tmux         "#[bg=#ebcb8b,fg=#2e3440,bold] LEADER #[bg=default,fg=#7b88a1]  h/j/k/l FOCUS · n/d/r PANE · t/w TAB · [/] PREV/NEXT TAB · ,/. PREV/NEXT STACK · x CLOSE · q QUIT · Esc CANCEL"
                mode_scroll       "#[bg=#d08770,fg=#2e3440,bold] SCROLL #[bg=default,fg=#7b88a1]  j/k SCROLL · d/u HALF · f/b PAGE · g/G TOP/BTM · / SEARCH · ^g EXIT"
                mode_enter_search "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  Type to search… Enter CONFIRM · Esc CANCEL"
                mode_search       "#[bg=#ebcb8b,fg=#2e3440,bold] SEARCH #[bg=default,fg=#7b88a1]  n/N NEXT/PREV · c CASE · w WRAP · ^g EXIT"
                mode_locked       "#[bg=#bf616a,fg=#eceff4,bold] LOCKED #[bg=default,fg=#7b88a1]  ^g UNLOCK"
                mode_pane         "#[bg=#a3be8c,fg=#2e3440,bold] PANE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_tab          "#[bg=#b48ead,fg=#2e3440,bold] TAB    #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_resize       "#[bg=#a3be8c,fg=#2e3440,bold] RESIZE #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_move         "#[bg=#a3be8c,fg=#2e3440,bold] MOVE   #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_session      "#[bg=#b48ead,fg=#2e3440,bold] SESSION #[bg=default,fg=#7b88a1]  ^g EXIT"
                mode_rename_tab   "#[bg=#b48ead,fg=#2e3440,bold] RENAME TAB #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_rename_pane  "#[bg=#b48ead,fg=#2e3440,bold] RENAME PANE #[bg=default,fg=#7b88a1]  Enter CONFIRM · Esc CANCEL"
                mode_prompt       "#[bg=#81a1c1,fg=#2e3440,bold] PROMPT #[bg=default,fg=#7b88a1]  ^g EXIT"
            }}
        }}
    }}
    tab name="browse" focus=true {{
        pane split_direction="horizontal" {{
            pane size="60%" name="files" focus=true {{
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
                cwd "/workspace"
            }}
            pane size="40%" {{
{ai_pane}
            }}
        }}
    }}
    tab name="editor" {{
        pane name="vim" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
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
"##
    )
}

/// Default yazi config.
///
/// Note on `[mgr]` (formerly `[manager]`):
/// Yazi 25+ renamed the `[manager]` section to `[mgr]`. Files using the old
/// name are silently ignored — `ratio` and friends have no effect. The
/// `migrate_yazi_section` helper rewrites existing host-side files at sync
/// time. Do not change `[mgr]` back to `[manager]`.
const DEFAULT_YAZI_CONFIG: &str = r#"[mgr]
ratio = [3, 5, 18]
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
prepend_fetchers = [
    { id = "git", url = "*",  run = "git" },
    { id = "git", url = "*/", run = "git" },
]
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
-- Converts EPS to PNG using ghostscript (gs).
-- Requires: ghostscript in PATH (install via preview-enhanced addon or apt).

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return Err("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		local ok = Command("gs")
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

		if ok then
			return Image:new(job, cache):show()
		end

		return Err("EPS preview requires ghostscript: aibox addon add preview-enhanced")
	end,
}
"#;

/// SVG previewer plugin — converts SVG to PNG via resvg or rsvg-convert.
const DEFAULT_YAZI_PLUGIN_SVG: &str = r#"-- svg.yazi — SVG previewer for yazi
-- Converts SVG to PNG using resvg (x86_64) or rsvg-convert (aarch64 fallback).

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return Err("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		-- Try resvg first (high quality, static binary — available on x86_64)
		local ok = Command("resvg")
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

		if ok then
			return Image:new(job, cache):show()
		end

		-- Fallback: rsvg-convert (from librsvg2-bin, available on all architectures)
		ok = Command("rsvg-convert")
			:args({
				"--width", tostring(job.area.w * 4),
				"--height", tostring(job.area.h * 4),
				"--keep-aspect-ratio",
				"--output", tostring(cache),
				tostring(job.file.url),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if ok then
			return Image:new(job, cache):show()
		end

		return Err("SVG preview failed: install resvg or librsvg2-bin")
	end,
}
"#;

/// Yazi init.lua — registers plugins that need setup on every startup.
const DEFAULT_YAZI_INIT: &str = r#"-- =============================================================================
-- Yazi init.lua — aibox defaults
-- Runs on every Yazi startup. Register plugins that need setup here.
-- =============================================================================

-- git.yazi: show git status (modified/untracked/staged) in file list.
-- Fetcher registration is in yazi.toml [plugin.prepend_fetchers].
require("git"):setup {}
"#;

/// git.yazi plugin main — shows git status signs next to file names.
const DEFAULT_YAZI_PLUGIN_GIT_MAIN: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/git.yazi/main.lua");

/// git.yazi plugin types — type annotations for the git plugin.
const DEFAULT_YAZI_PLUGIN_GIT_TYPES: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/git.yazi/types.lua");

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
  ZELLIJ (leader: Ctrl+g)    YAZI (file manager)
  Alt+h/j/k/l     Move pane  h/j/k/l  Navigate
  Ctrl+g h/j/k/l  Move pane  Enter    Open in vim
  Ctrl+g [/]      Prev/next  q        Quit yazi
  Ctrl+g 1-5      Jump tab   /        Search
  Ctrl+g f        Fullscreen .        Hidden files
  Ctrl+g x        Close pane Space    Select
  Ctrl+g n/d/r    New pane
  Ctrl+g t/w      Tab +/-
  Ctrl+g p        Scratch pad (vim)
  Ctrl+g s        Strider
  Ctrl+g u        Scroll
  Ctrl+g /        Search
  Ctrl+q          QUIT (or Ctrl+g q)

  LAYOUTS: aibox start --layout dev|focus|cowork|cowork-swap|browse|ai
  TABS: Ctrl+g 1 dev  2 git  3 shell
"#;

/// Default .asoundrc for PulseAudio over TCP.
const DEFAULT_ASOUNDRC: &str = r#"pcm.!default {
    type pulse
}
ctl.!default {
    type pulse
}
"#;

/// Claude Code keybindings — disables Ctrl+g (reserved for zellij leader key).
const DEFAULT_CLAUDE_KEYBINDINGS: &str = r#"[
  {
    "key": "ctrl+g",
    "command": null
  }
]
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
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("eps.yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("svg.yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("git.yazi"),
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
            crate::config::AiProvider::OpenAI => {
                dirs.push(root.join(".codex"));
            }
            crate::config::AiProvider::Continue => {
                dirs.push(root.join(".continue"));
            }
            crate::config::AiProvider::Copilot => {
                dirs.push(root.join(".copilot"));
            }
            // Cursor is a host-side IDE extension only — no in-container
            // persistence directory. MCP registration files for all providers
            // are written by mcp_registration.rs at the project root.
            crate::config::AiProvider::Cursor => {}
        }
    }

    for dir in &dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    // Seed config files (never overwrite)
    let theme = &config.customization.theme;
    let vimrc = DEFAULT_VIMRC
        .replace(
            "AIBOX_VIM_COLORSCHEME",
            crate::themes::vim_colorscheme(theme),
        )
        .replace("AIBOX_VIM_BG", crate::themes::vim_background(theme));
    seed_file(&root.join(".vim").join("vimrc"), &vimrc)?;
    seed_file(
        &root.join(".config").join("git").join("config"),
        DEFAULT_GITCONFIG,
    )?;

    // Zellij config — apply selected theme and default layout
    let zellij_config = DEFAULT_ZELLIJ_CONFIG
        .replace("AIBOX_THEME", &theme.to_string())
        .replace("AIBOX_LAYOUT", &config.customization.layout.to_string());
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
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("browse.kdl"),
        &generate_browse_layout(providers),
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("ai.kdl"),
        &generate_ai_layout(providers),
    )?;
    seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("cowork-swap.kdl"),
        &generate_cowork_swap_layout(providers),
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
    // Yazi init.lua — register plugins on startup
    seed_file(
        &root.join(".config").join("yazi").join("init.lua"),
        DEFAULT_YAZI_INIT,
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
    // Yazi git plugin — shows git status in file list
    seed_file(
        &root
            .join(".config")
            .join("yazi")
            .join("plugins")
            .join("git.yazi")
            .join("main.lua"),
        DEFAULT_YAZI_PLUGIN_GIT_MAIN,
    )?;
    seed_file(
        &root
            .join(".config")
            .join("yazi")
            .join("plugins")
            .join("git.yazi")
            .join("types.lua"),
        DEFAULT_YAZI_PLUGIN_GIT_TYPES,
    )?;

    // Cheatsheet
    seed_file(
        &root.join(".config").join("cheatsheet.txt"),
        DEFAULT_CHEATSHEET,
    )?;

    // Starship prompt config
    let prompt = &config.customization.prompt;
    let starship_content = crate::themes::starship_config(prompt, theme);
    seed_file(
        &root.join(".config").join("starship.toml"),
        &starship_content,
    )?;

    // lazygit theme config
    seed_file(
        &root.join(".config").join("lazygit").join("config.yml"),
        crate::themes::lazygit_theme(theme),
    )?;

    // Audio config
    if config.audio.enabled {
        seed_file(&root.join(".asoundrc"), DEFAULT_ASOUNDRC)?;
    }

    // Claude Code keybindings — disable Ctrl+g (reserved for zellij leader key).
    // Only seeded when Claude is configured as a provider.
    if config
        .ai
        .providers
        .contains(&crate::config::AiProvider::Claude)
    {
        seed_file(
            &root.join(".claude").join("keybindings.json"),
            DEFAULT_CLAUDE_KEYBINDINGS,
        )?;
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

/// Migrate the deprecated yazi `[manager]` section name to `[mgr]`.
///
/// Yazi 25+ renamed the section, and uses of `[manager]` are silently
/// ignored. This helper edits an existing yazi config file in place,
/// rewriting any line that begins with `[manager]` to `[mgr]`. It is
/// idempotent — files already using `[mgr]` are left untouched.
///
/// Used for `yazi.toml`, `keymap.toml`, and `theme.toml` (which all
/// previously used `[manager]`). User customizations OUTSIDE the section
/// header are preserved.
///
/// Returns Ok(true) if the file was modified, Ok(false) otherwise (file
/// missing or no `[manager]` line found).
pub fn migrate_yazi_section(path: &Path) -> Result<bool> {
    if !path.is_file() {
        return Ok(false);
    }
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    if !content.lines().any(|l| l.trim_end() == "[manager]") {
        return Ok(false);
    }
    let new_content: String = content
        .lines()
        .map(|line| {
            if line.trim_end() == "[manager]" {
                "[mgr]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        // Preserve trailing newline if the original had one
        + if content.ends_with('\n') { "\n" } else { "" };
    fs::write(path, new_content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

/// Force-seed all theme-dependent and AI-provider-dependent config files.
/// Overwrites existing files when content has changed. Used by `aibox sync`.
pub fn sync_theme_files(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let theme = &config.customization.theme;
    let providers = &config.ai.providers;
    let mut updated = Vec::new();

    // vimrc — colorscheme and background
    let vimrc = DEFAULT_VIMRC
        .replace(
            "AIBOX_VIM_COLORSCHEME",
            crate::themes::vim_colorscheme(theme),
        )
        .replace("AIBOX_VIM_BG", crate::themes::vim_background(theme));
    if force_seed_file(&root.join(".vim").join("vimrc"), &vimrc)? {
        updated.push(".vim/vimrc".to_string());
    }

    // Zellij config — theme name and default layout
    let zellij_config = DEFAULT_ZELLIJ_CONFIG
        .replace("AIBOX_THEME", &theme.to_string())
        .replace("AIBOX_LAYOUT", &config.customization.layout.to_string());
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
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("browse.kdl"),
        &generate_browse_layout(providers),
    )? {
        updated.push(".config/zellij/layouts/browse.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("ai.kdl"),
        &generate_ai_layout(providers),
    )? {
        updated.push(".config/zellij/layouts/ai.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("cowork-swap.kdl"),
        &generate_cowork_swap_layout(providers),
    )? {
        updated.push(".config/zellij/layouts/cowork-swap.kdl".to_string());
    }

    // lazygit config
    if force_seed_file(
        &root.join(".config").join("lazygit").join("config.yml"),
        crate::themes::lazygit_theme(theme),
    )? {
        updated.push(".config/lazygit/config.yml".to_string());
    }

    // Yazi theme — force-update from the bundled theme for the selected theme
    if force_seed_file(
        &root.join(".config").join("yazi").join("theme.toml"),
        crate::themes::yazi_theme(theme),
    )? {
        updated.push(".config/yazi/theme.toml".to_string());
    }

    // Yazi config migration: rewrite [manager] → [mgr] in existing files.
    // Yazi 25+ silently ignores [manager], so any user customization that
    // still uses the old section name (from older aibox releases) needs to
    // be migrated. The migration is idempotent and preserves user content
    // outside the section header.
    let yazi_dir = root.join(".config").join("yazi");
    for filename in ["yazi.toml", "keymap.toml", "theme.toml"] {
        let path = yazi_dir.join(filename);
        if migrate_yazi_section(&path)? {
            updated.push(format!(
                ".config/yazi/{} (migrated [manager] → [mgr])",
                filename
            ));
        }
    }

    // Starship prompt
    let prompt = &config.customization.prompt;
    let starship_content = crate::themes::starship_config(prompt, theme);
    if force_seed_file(
        &root.join(".config").join("starship.toml"),
        &starship_content,
    )? {
        updated.push(".config/starship.toml".to_string());
    }

    // Claude Code keybindings — disable Ctrl+g (reserved for zellij leader key).
    if providers.contains(&crate::config::AiProvider::Claude)
        && force_seed_file(
            &root.join(".claude").join("keybindings.json"),
            DEFAULT_CLAUDE_KEYBINDINGS,
        )?
    {
        updated.push(".claude/keybindings.json".to_string());
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
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("browse.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("ai.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("cowork-swap.kdl")
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
        assert!(
            layout.contains("tab name=\"claude\""),
            "should have claude tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude command"
        );
        assert!(!layout.contains("aider"), "should not have aider");
    }

    #[test]
    fn dev_layout_aider_only() {
        let providers = vec![AiProvider::Aider];
        let layout = generate_dev_layout(&providers);
        assert!(
            layout.contains("tab name=\"aider\""),
            "should have aider tab"
        );
        assert!(
            layout.contains("command \"aider\""),
            "should have aider command"
        );
        assert!(!layout.contains("claude"), "should not have claude");
    }

    #[test]
    fn dev_layout_multiple_providers() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_dev_layout(&providers);
        assert!(
            layout.contains("tab name=\"claude\""),
            "should have claude tab"
        );
        assert!(
            layout.contains("tab name=\"aider\""),
            "should have aider tab"
        );
    }

    #[test]
    fn dev_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_dev_layout(&providers);
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(!layout.contains("aider"), "should not have aider");
        assert!(!layout.contains("gemini"), "should not have gemini");
        assert!(
            layout.contains("tab name=\"dev\""),
            "should still have dev tab"
        );
        assert!(
            layout.contains("tab name=\"git\""),
            "should still have git tab"
        );
    }

    #[test]
    fn focus_layout_gemini() {
        let providers = vec![AiProvider::Gemini];
        let layout = generate_focus_layout(&providers);
        assert!(
            layout.contains("tab name=\"gemini\""),
            "should have gemini tab"
        );
        assert!(
            layout.contains("command \"gemini\""),
            "should have gemini command"
        );
    }

    #[test]
    fn cowork_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_layout(&providers);
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn cowork_layout_multiple_providers_stacked() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_cowork_layout(&providers);
        assert!(
            layout.contains("stacked=true"),
            "multiple providers should be stacked"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
    }

    #[test]
    fn cowork_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_cowork_layout(&providers);
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(
            layout.contains("tab name=\"cowork\""),
            "should still have cowork tab"
        );
    }

    #[test]
    fn browse_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_browse_layout(&providers);
        assert!(
            layout.contains("tab name=\"browse\""),
            "should have browse tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            layout.contains("tab name=\"editor\""),
            "should have editor tab"
        );
        assert!(layout.contains("tab name=\"git\""), "should have git tab");
        assert!(
            layout.contains("AIBOX_EDITOR_DIR=tab"),
            "should use tab editor direction"
        );
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn browse_layout_multiple_providers_stacked() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_browse_layout(&providers);
        assert!(
            layout.contains("stacked=true"),
            "multiple providers should be stacked"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
    }

    #[test]
    fn browse_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_browse_layout(&providers);
        assert!(
            layout.contains("tab name=\"browse\""),
            "should still have browse tab"
        );
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(
            layout.contains("tab name=\"editor\""),
            "should still have editor tab"
        );
    }

    #[test]
    fn browse_layout_yazi_above_ai() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_browse_layout(&providers);
        let yazi_pos = layout.find("yazi").unwrap();
        let claude_pos = layout.find("command \"claude\"").unwrap();
        assert!(
            yazi_pos < claude_pos,
            "yazi should appear before AI pane (top position)"
        );
        assert!(layout.contains("size=\"60%\""), "yazi pane should be 60%");
    }

    #[test]
    fn ai_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_ai_layout(&providers);
        assert!(layout.contains("tab name=\"ai\""), "should have ai tab");
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            layout.contains("tab name=\"editor\""),
            "should have editor tab"
        );
        assert!(layout.contains("tab name=\"git\""), "should have git tab");
        assert!(
            layout.contains("tab name=\"shell\""),
            "should have shell tab"
        );
        assert!(
            layout.contains("split_direction=\"vertical\""),
            "should split vertically"
        );
        // v0.16.5: yazi gets 50%, AI pane gets 50% (was 53/47 in v0.14.5+)
        assert!(
            layout.contains("size=\"50%\" name=\"files\""),
            "yazi pane should be 50%"
        );
        assert!(layout.contains("size=\"50%\""), "ai pane should be 50%");
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn ai_layout_multiple_providers_stacked() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_ai_layout(&providers);
        assert!(
            layout.contains("stacked=true"),
            "multiple providers should be stacked"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
    }

    #[test]
    fn ai_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_ai_layout(&providers);
        assert!(
            layout.contains("tab name=\"ai\""),
            "should still have ai tab"
        );
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(
            layout.contains("tab name=\"editor\""),
            "should still have editor tab"
        );
        assert!(layout.contains("yazi"), "should still have yazi pane");
    }

    #[test]
    fn ai_layout_yazi_left_of_ai() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_ai_layout(&providers);
        let yazi_pos = layout.find("yazi").unwrap();
        let claude_pos = layout.find("command \"claude\"").unwrap();
        assert!(
            yazi_pos < claude_pos,
            "yazi should appear left of (before) AI pane"
        );
    }

    #[test]
    fn cowork_swap_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_swap_layout(&providers);
        assert!(
            layout.contains("tab name=\"cowork-swap\""),
            "should have cowork-swap tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            layout.contains("name=\"editor\""),
            "should have editor pane"
        );
        assert!(layout.contains("vim-loop"), "should run vim-loop");
        assert!(layout.contains("yazi"), "should run yazi");
        assert!(layout.contains("tab name=\"git\""), "should have git tab");
        assert!(
            layout.contains("tab name=\"shell\""),
            "should have shell tab"
        );
        // Outer split: left 40% / right 60% (editor on the right gets the bigger half)
        assert!(
            layout.contains("size=\"40%\" split_direction=\"horizontal\""),
            "left side should be 40% with horizontal sub-split"
        );
        assert!(
            layout.contains("size=\"60%\" name=\"editor\""),
            "right side (editor) should be 60%"
        );
        // Inner left split: yazi 40% top, AI 60% bottom
        assert!(
            layout.contains("size=\"40%\" name=\"files\""),
            "yazi pane should be 40% of left stack"
        );
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn cowork_swap_layout_multiple_providers_stacked() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_cowork_swap_layout(&providers);
        assert!(
            layout.contains("stacked=true"),
            "multiple providers should be stacked"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
    }

    #[test]
    fn cowork_swap_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_cowork_swap_layout(&providers);
        assert!(
            layout.contains("tab name=\"cowork-swap\""),
            "should still have cowork-swap tab"
        );
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(layout.contains("yazi"), "should still have yazi pane");
        assert!(layout.contains("vim-loop"), "should still have vim editor");
        assert!(
            layout.contains("size=\"40%\" name=\"files\""),
            "yazi should be 40% (left)"
        );
        assert!(
            layout.contains("size=\"60%\" name=\"editor\""),
            "vim should be 60% (right)"
        );
    }

    #[test]
    fn cowork_swap_layout_editor_right_of_files() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_swap_layout(&providers);
        let files_pos = layout.find("name=\"files\"").unwrap();
        let editor_pos = layout.find("name=\"editor\"").unwrap();
        assert!(
            files_pos < editor_pos,
            "yazi (files) should appear before editor in layout source — editor sits to the right"
        );
    }

    #[test]
    fn cowork_swap_layout_ai_below_files_in_left_stack() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_swap_layout(&providers);
        let files_pos = layout.find("name=\"files\"").unwrap();
        let claude_pos = layout.find("command \"claude\"").unwrap();
        let editor_pos = layout.find("name=\"editor\"").unwrap();
        assert!(
            files_pos < claude_pos && claude_pos < editor_pos,
            "yazi → claude (left stack top→bottom) → editor (right) order in source"
        );
    }

    #[test]
    fn default_yazi_config_uses_mgr_section() {
        // Regression test for the [manager] → [mgr] rename in yazi 25+.
        // The seeded config must use [mgr] or yazi will silently ignore it.
        assert!(
            DEFAULT_YAZI_CONFIG.contains("[mgr]"),
            "default yazi config must use [mgr] section"
        );
        assert!(
            !DEFAULT_YAZI_CONFIG.contains("[manager]"),
            "default yazi config must not use deprecated [manager] section"
        );
    }

    #[test]
    fn migrate_yazi_section_rewrites_manager_to_mgr() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        fs::write(
            &path,
            "[manager]\nratio = [1, 3, 4]\nsort_by = \"natural\"\n",
        )
        .unwrap();

        let changed = migrate_yazi_section(&path).unwrap();
        assert!(changed, "should report change");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("[mgr]\n"), "should rename to [mgr]");
        assert!(
            content.contains("ratio = [1, 3, 4]"),
            "should preserve user values"
        );
        assert!(
            content.contains("sort_by = \"natural\""),
            "should preserve other lines"
        );
        assert!(
            !content.contains("[manager]"),
            "should not contain old section name"
        );
    }

    #[test]
    fn migrate_yazi_section_idempotent_on_mgr() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        let original = "[mgr]\nratio = [1, 3, 4]\n";
        fs::write(&path, original).unwrap();

        let changed = migrate_yazi_section(&path).unwrap();
        assert!(
            !changed,
            "no change should be reported for already-migrated file"
        );

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, original, "file content must be unchanged");
    }

    #[test]
    fn migrate_yazi_section_missing_file_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.toml");
        let changed = migrate_yazi_section(&path).unwrap();
        assert!(!changed);
    }

    #[test]
    fn migrate_yazi_section_preserves_user_customization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        // User has changed ratio and added a comment — both must survive.
        let custom = "# my customizations\n[manager]\nratio = [2, 4, 1]\n# end\n";
        fs::write(&path, custom).unwrap();

        let changed = migrate_yazi_section(&path).unwrap();
        assert!(changed);

        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("ratio = [2, 4, 1]"),
            "user ratio must be preserved"
        );
        assert!(
            content.contains("# my customizations"),
            "user comments must be preserved"
        );
        assert!(
            content.contains("# end"),
            "trailing comment must be preserved"
        );
        assert!(content.contains("[mgr]"), "section must be renamed");
        assert!(!content.contains("[manager]"));
    }

    #[test]
    fn migrate_yazi_section_does_not_touch_substring_matches() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        // A line that contains "manager" as part of a value should NOT be touched.
        let content = "[mgr]\ndescription = \"the manager pane\"\n";
        fs::write(&path, content).unwrap();
        let changed = migrate_yazi_section(&path).unwrap();
        assert!(!changed);
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn zellij_config_substitutes_layout() {
        // Regression test for the layout-sync bug fixed in v0.14.2:
        // DEFAULT_ZELLIJ_CONFIG must contain the AIBOX_LAYOUT placeholder
        // that seed_root_dir / sync_theme_files replace with the configured layout.
        assert!(
            DEFAULT_ZELLIJ_CONFIG.contains("AIBOX_LAYOUT"),
            "config template must contain AIBOX_LAYOUT placeholder"
        );
        assert!(
            !DEFAULT_ZELLIJ_CONFIG.contains("default_layout \"dev\""),
            "config template must not hard-code dev as default_layout"
        );
    }

    #[test]
    fn ai_pane_kdl_empty() {
        let result = ai_pane_kdl(&[]);
        assert!(
            result.is_empty(),
            "empty providers should produce empty string"
        );
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

        assert!(
            root.join(".aider").is_dir(),
            ".aider directory should be created"
        );
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

        assert!(
            root.join(".gemini").is_dir(),
            ".gemini directory should be created"
        );

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }
}
