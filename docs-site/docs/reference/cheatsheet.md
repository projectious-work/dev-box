---
sidebar_position: 3
title: Cheatsheet
---

# Keyboard Shortcuts

Quick reference for all tools in the dev-box environment. Press the tab for the tool you need.

!!! tip "In-app help"
    - **Zellij:** The status bar always shows available keys for the current mode
    - **Yazi:** Press `~` or `F1` to see all keybindings
    - **Vim:** Type `:help` for built-in help
    - **lazygit:** Press `?` to see context-sensitive keybindings

=== "Zellij"

    ## Zellij (Terminal Multiplexer)

    Leader key: ++ctrl+b++ — press and release, then press the action key.

    ### Pane Navigation

    | Key | Action |
    |-----|--------|
    | `Ctrl+b` `h` / `Left` | Focus pane left |
    | `Ctrl+b` `j` / `Down` | Focus pane down |
    | `Ctrl+b` `k` / `Up` | Focus pane up |
    | `Ctrl+b` `l` / `Right` | Focus pane right |

    ### Pane Management

    | Key | Action |
    |-----|--------|
    | `Ctrl+b` `n` | New pane (best direction) |
    | `Ctrl+b` `d` | Split down |
    | `Ctrl+b` `r` | Split right |
    | `Ctrl+b` `x` | Close current pane |
    | `Ctrl+b` `f` | Toggle fullscreen |
    | `Ctrl+b` `e` | Toggle embed / floating |
    | `Ctrl+b` `z` | Toggle pane frames |
    | `Ctrl+b` `=` | Increase pane size |
    | `Ctrl+b` `-` | Decrease pane size |

    ### Tab Management

    | Key | Action |
    |-----|--------|
    | `Ctrl+b` `t` | New tab |
    | `Ctrl+b` `w` | Close tab |
    | `Ctrl+b` `[` | Previous tab |
    | `Ctrl+b` `]` | Next tab |
    | `Ctrl+b` `1`..`5` | Jump to tab by number |
    | `Ctrl+b` `i` | Move tab left |
    | `Ctrl+b` `o` | Move tab right |

    ### Scroll & Search

    | Key | Action |
    |-----|--------|
    | `Ctrl+b` `u` | Enter scroll mode |
    | `Ctrl+b` `/` | Search scrollback |

    **In scroll mode:**

    | Key | Action |
    |-----|--------|
    | `j` / `k` | Scroll down / up |
    | `d` / `u` | Half-page down / up |
    | `f` / `b` | Full page down / up |
    | `g` / `G` | Top / bottom |
    | `/` | Search |
    | `q` or `Esc` | Exit scroll mode |

    **In search mode:**

    | Key | Action |
    |-----|--------|
    | `n` / `N` | Next / previous match |
    | `c` | Toggle case sensitivity |
    | `w` | Toggle wrap |
    | `o` | Toggle whole word |

    ### Plugins & Session

    | Key | Action |
    |-----|--------|
    | `Ctrl+b` `s` | Strider file picker (floating) |
    | `Ctrl+b` `m` | Session manager |

    ### Quit

    | Key | Action |
    |-----|--------|
    | `Ctrl+b` `q` | Quit Zellij |
    | `Ctrl+q` | Quit Zellij (global) |

    !!! info "Default layout tabs"
        The `dev` layout opens with pre-configured tabs:
        **Tab 1** — dev (files + editor + terminal),
        **Tab 2** — git (lazygit),
        **Tab 3** — shell (extra terminal),
        **Tab 4** — help (cheatsheet)

=== "Yazi"

    ## Yazi (File Manager)

    Yazi uses Vim-style navigation. The dev-box config adds a few custom bindings on top of the defaults.

    ### Navigation

    | Key | Action |
    |-----|--------|
    | `h` / `Left` | Go to parent directory |
    | `j` / `Down` | Move cursor down |
    | `k` / `Up` | Move cursor up |
    | `l` / `Right` / `Enter` | Open file or enter directory |
    | `g` `g` | Go to first item |
    | `G` | Go to last item |
    | `~` | Go to home directory |

    ### Opening Files (dev-box custom)

    | Key | Action |
    |-----|--------|
    | `Enter` | Open file in-place (suspends Yazi, `:q` returns) |
    | `e` | Open in adjacent Vim pane (stays in Yazi) |
    | `O` | Interactive opener selection |

    ### File Operations

    | Key | Action |
    |-----|--------|
    | `a` | Create new file or directory (append `/` for directory) |
    | `r` | Rename file |
    | `d` | Trash selected files |
    | `D` | Permanently delete selected files |
    | `y` | Yank (copy) selected files |
    | `x` | Yank (cut) selected files |
    | `p` | Paste yanked files |
    | `Space` | Toggle selection on current file |
    | `v` | Visual mode (select range) |
    | `V` | Invert selection |

    ### Search & Filter

    | Key | Action |
    |-----|--------|
    | `/` | Search files in current directory |
    | `f` | Filter files (fuzzy match) |
    | `.` | Toggle hidden files |

    ### Preview & Tabs

    | Key | Action |
    |-----|--------|
    | `Tab` | Switch preview pane |
    | `t` | Create new tab |
    | `1`..`9` | Switch to tab by number |
    | `[` / `]` | Previous / next tab |

    ### Misc

    | Key | Action |
    |-----|--------|
    | `z` | Jump to directory (zoxide) |
    | `:` | Open command shell |
    | `~` / `F1` | View all keybindings |
    | `q` | Quit Yazi |

=== "Vim"

    ## Vim (Editor)

    Leader key: `Space`

    ### Leader Commands

    | Key | Action |
    |-----|--------|
    | `Space` `w` | Save file |
    | `Space` `q` | Quit |
    | `Space` `x` | Save and quit |
    | `Space` `n` | Next buffer |
    | `Space` `p` | Previous buffer |
    | `Space` `l` | List buffers |
    | `Space` `e` | Open netrw file explorer |

    ### Split Navigation

    | Key | Action |
    |-----|--------|
    | `Ctrl+h` | Move to left split |
    | `Ctrl+j` | Move to split below |
    | `Ctrl+k` | Move to split above |
    | `Ctrl+l` | Move to right split |

    ### Essential Motions

    | Key | Action |
    |-----|--------|
    | `h` `j` `k` `l` | Left, down, up, right |
    | `w` / `b` | Next / previous word |
    | `0` / `$` | Start / end of line |
    | `gg` / `G` | Top / bottom of file |
    | `Ctrl+d` / `Ctrl+u` | Half-page down / up |
    | `%` | Jump to matching bracket |
    | `f&#123;char&#125;` | Jump to next &#123;char&#125; on line |

    ### Editing

    | Key | Action |
    |-----|--------|
    | `i` / `a` | Insert before / after cursor |
    | `I` / `A` | Insert at start / end of line |
    | `o` / `O` | New line below / above |
    | `dd` | Delete line |
    | `yy` | Yank (copy) line |
    | `p` | Paste after cursor |
    | `u` / `Ctrl+r` | Undo / redo |
    | `.` | Repeat last change |
    | `ciw` | Change inner word |
    | `>>` / `<<` | Indent / dedent line |

    ### Search

    | Key | Action |
    |-----|--------|
    | `/pattern` | Search forward |
    | `?pattern` | Search backward |
    | `n` / `N` | Next / previous match |
    | `Esc` `Esc` | Clear search highlight |
    | `*` | Search word under cursor |

    ### Commands

    | Key | Action |
    |-----|--------|
    | `:w` | Save |
    | `:q` / `:q!` | Quit / force quit |
    | `:wq` or `:x` | Save and quit |
    | `:e <file>` | Open file |
    | `:%s/old/new/g` | Find and replace in file |

    !!! note "Dev-box Vim settings"
        - Relative line numbers are enabled for fast `&#123;N&#125;j`/`&#123;N&#125;k` jumps
        - Tabs expand to 4 spaces (2 for YAML, JSON, HTML, CSS, JS, TS)
        - Trailing whitespace is stripped on save
        - Persistent undo is enabled across sessions

=== "lazygit"

    ## lazygit (Git TUI)

    lazygit is panel-based. Press `?` at any time to see context-sensitive keybindings.

    ### Panel Navigation

    | Key | Action |
    |-----|--------|
    | `1` | Status panel |
    | `2` | Files panel |
    | `3` | Branches panel |
    | `4` | Commits panel |
    | `5` | Stash panel |
    | `h` / `l` | Switch panels left / right |
    | `j` / `k` | Move up / down within panel |
    | `[` / `]` | Previous / next tab within panel |

    ### Files Panel

    | Key | Action |
    |-----|--------|
    | `Space` | Stage / unstage file |
    | `a` | Stage / unstage all files |
    | `c` | Commit staged changes |
    | `A` | Amend last commit |
    | `d` | Discard changes to file |
    | `e` | Edit file in editor |
    | `o` | Open file in default application |
    | `i` | Add to .gitignore |
    | `S` | Stash all changes |
    | `Enter` | Focus on file to see diff hunks |

    ### Branches Panel

    | Key | Action |
    |-----|--------|
    | `Space` | Checkout branch |
    | `n` | New branch |
    | `d` | Delete branch |
    | `M` | Merge into current branch |
    | `r` | Rebase current branch onto selected |
    | `R` | Rename branch |
    | `f` | Fetch branch |
    | `P` | Push |
    | `p` | Pull |

    ### Commits Panel

    | Key | Action |
    |-----|--------|
    | `s` | Squash commit into one below |
    | `r` | Reword commit message |
    | `R` | Reword with editor |
    | `d` | Delete commit |
    | `e` | Edit commit (interactive rebase) |
    | `c` | Copy commit (cherry-pick) |
    | `v` | Paste (cherry-pick) commit |
    | `F` | Create fixup commit |
    | `S` | Squash all fixup commits |
    | `g` | Reset to this commit |
    | `t` | Tag commit |

    ### Stash Panel

    | Key | Action |
    |-----|--------|
    | `Space` | Apply stash (keep in list) |
    | `g` | Pop stash (apply + remove) |
    | `d` | Drop stash entry |

    ### Global

    | Key | Action |
    |-----|--------|
    | `?` | Show keybindings for current panel |
    | `+` | Show command log |
    | `@` | Show command log menu |
    | `P` | Push |
    | `p` | Pull |
    | `z` / `Ctrl+z` | Undo last action |
    | `q` | Quit lazygit |

    !!! tip "Accessing lazygit"
        In the default dev-box layout, lazygit runs in **Tab 2** (`Ctrl+b` `2`).
