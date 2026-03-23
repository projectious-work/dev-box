# Base Image

The base image is the foundation for all dev-box container flavors. It provides a complete, opinionated development environment built on **Debian Trixie Slim**.

## Installed Tools

| Tool | Version / Source | Purpose |
|------|-----------------|---------|
| Zellij | 0.43.1 (prebuilt binary from GitHub releases) | Terminal multiplexer |
| Yazi | 25.4.8 (prebuilt binary from GitHub releases) | Terminal file manager |
| Vim | Debian package (`vim` + `vim-runtime`) | Editor |
| Git | Debian package | Version control |
| lazygit | Debian package | Git TUI |
| GitHub CLI (`gh`) | Debian package | GitHub integration |
| Claude CLI | Official install script | AI assistant |
| ripgrep (`rg`) | Debian package | Fast recursive search (grep replacement) |
| fd | Debian package | Fast file finder (find replacement) |
| bat | Debian package | Syntax-highlighting cat replacement |
| eza | Debian package | Modern ls replacement with git integration |
| zoxide | Debian package | Smarter cd that learns your habits |
| fzf | Debian package | Fuzzy finder for files, history, and more |
| delta | Debian package | Syntax-highlighting diff viewer (used by git) |
| starship | Prebuilt binary | Minimal, fast shell prompt with context |
| curl | Debian package | HTTP client |
| jq | Debian package | JSON processor |
| less | Debian package | Pager |
| unzip | Debian package | Archive extraction |
| bash-completion | Debian package | Shell completions |
| sox | Debian package | Audio processing |
| pulseaudio-utils | Debian package | Audio bridging |
| ca-certificates | Debian package | TLS root certificates |
| locales | Debian package | Locale support (en_US.UTF-8) |
| tzdata | Debian package | Timezone data |

## Build Architecture

The Dockerfile uses a multi-stage build:

- **Stage 1 (builder):** Downloads the official Zellij prebuilt binary from GitHub releases. Architecture detection uses `uname -m`, which returns `aarch64` or `x86_64` directly -- matching the Zellij release filename convention. This works reliably across Docker, Podman, and Buildah.

- **Stage 2 (runtime):** Pure Debian Trixie Slim with apt packages. Only the Zellij binary is copied from the builder stage.

!!! note "Why prebuilt instead of compiled"
    Compiling Zellij from source requires 8+ GB RAM during the final linker step. On Apple Silicon under Podman/Docker Desktop, the default VM memory cap causes OOM kills. The official musl-static binary is equally portable and downloads in seconds.

## Zellij Configuration

### Key Bindings

All bindings use `Ctrl+b` as a leader key — press `Ctrl+b`, release, then press the action key. This avoids conflicts with macOS Option key (which produces special characters like `@`, `€`, `|`) and with Vim/bash Ctrl bindings.

| Key | Action |
|-----|--------|
| `Ctrl+b` then `h/j/k/l` | Navigate panes (vim-style) |
| `Ctrl+b` then `n` | New pane |
| `Ctrl+b` then `d` | Split down |
| `Ctrl+b` then `r` | Split right |
| `Ctrl+b` then `x` | Close focused pane |
| `Ctrl+b` then `f` | Toggle fullscreen |
| `Ctrl+b` then `z` | Toggle pane frames |
| `Ctrl+b` then `e` | Toggle embed/floating |
| `Ctrl+b` then `=` / `-` | Resize pane (increase / decrease) |
| `Ctrl+b` then `t` | New tab |
| `Ctrl+b` then `w` | Close tab |
| `Ctrl+b` then `[` / `]` | Previous / next tab |
| `Ctrl+b` then `1-5` | Jump to tab N |
| `Ctrl+b` then `i` / `o` | Move tab left / right |
| `Ctrl+b` then `s` | Open Strider file picker (floating) |
| `Ctrl+b` then `m` | Session manager |
| `Ctrl+b` then `u` | Enter scroll mode |
| `Ctrl+b` then `/` | Search scrollback |
| `Ctrl+q` | Quit Zellij |

Press `Escape` or `Ctrl+b` again to cancel the leader and return to normal mode.

### Layouts

dev-box ships three IDE layouts. Select one with `dev-box start --layout <name>` (the default is `dev`). All layouts include shared tabs for **git** (lazygit) and **shell** (extra bash).

#### dev (default) -- file browser + editor

<div class="asciinema" data-cast="assets/screencasts/layout-dev.cast" data-poster="npt:4" data-autoplay="false" data-controls="false" data-fit="width"></div>

Yazi file manager on the left, Vim on the right. Claude Code, git, and shell in separate tabs.

#### focus -- one tool per tab, fullscreen

<div class="asciinema" data-cast="assets/screencasts/layout-focus.cast" data-poster="npt:4" data-autoplay="false" data-controls="false" data-fit="width"></div>

Each tool gets the entire screen in its own tab. Switch with `Ctrl+b [/]` or `Ctrl+b 1-5`.

Tabs: **files** (yazi) | **editor** (vim) | **claude** | **git** (lazygit) | **shell**

#### cowork -- side-by-side coding with AI

<div class="asciinema" data-cast="assets/screencasts/layout-cowork.cast" data-poster="npt:4" data-autoplay="false" data-controls="false" data-fit="width"></div>

Yazi and Vim stacked on the left, Claude Code on the right. Git and shell in separate tabs.

### Opening Files from Yazi

- **`Enter`** -- opens file in vim in-place (suspends Yazi, `:q` returns to Yazi). Works in all layouts.
- **`e`** -- opens file in the adjacent vim pane and focuses it. Works in dev (vim is right), cowork (vim is below), and focus (switches to editor tab).

!!! note "Strider vs Yazi"
    `Ctrl+b` then `s` opens the built-in **Strider** file picker as a floating overlay (Zellij plugin). The sidebar file manager in all layouts is **Yazi**, an external terminal file manager with richer features (preview, bulk operations, async I/O).

### Theme

Gruvbox dark, defined in `themes/gruvbox.kdl`.

## Vim Configuration

Notable settings baked into the image:

- **Leader key:** Space
- **Line numbers:** Relative + absolute (hybrid)
- **Indentation:** 4 spaces default, 2 spaces for YAML, JSON, KDL, HTML, CSS, JavaScript
- **Undo:** Persistent undo files stored in `/root/.vim/undo`
- **No swap files** -- clean container environment
- **Color column** at 88 (Black/PEP8 default)
- **Grep program:** ripgrep if available (`rg --vimgrep --smart-case`)
- **Netrw:** Tree mode, no banner, 25% width
- **Colorscheme:** `desert` (ships with vim-runtime, no plugins needed)

## Git Configuration

Git config lives at `/root/.config/git/config` (XDG path, not `~/.gitconfig`). The environment variable `GIT_CONFIG_GLOBAL` is set in the generated `docker-compose.yml` to point to this location.

Using a directory mount (rather than a single-file mount) allows a `credentials` file to coexist alongside `config`.

## Claude CLI

The Claude CLI is installed via the official install script during image build. It is available at `/root/.local/bin/claude` and added to `PATH`.

## Audio Support

The base image includes `sox` and `pulseaudio-utils` for audio bridging, enabling Claude Code's voice features inside the container. See [Audio Support](audio.md) for setup details.

## Configuration Persistence

All user configuration is persisted on the host under `.dev-box-home/` and bind-mounted into the container:

| Host Path | Container Path | Contents |
|-----------|---------------|----------|
| `.dev-box-home/.ssh/` | `/root/.ssh` (read-only) | SSH keys |
| `.dev-box-home/.vim/` | `/root/.vim` | Vim config, undo history, plugins |
| `.dev-box-home/.config/git/` | `/root/.config/git` | Git config and credentials |
| `.dev-box-home/.config/zellij/` | `/root/.config/zellij` | Zellij config, themes, layouts, plugin cache |
| `.dev-box-home/.config/yazi/` | `/root/.config/yazi` | Yazi file manager config and keymap |

The Dockerfile bakes identical defaults into the image as a fallback. If no mounts are present, the container still works out of the box.

On first `dev-box init` or `dev-box start`, the `.dev-box-home/` directory is auto-seeded from built-in templates. Existing files are never overwritten.

## Container Entrypoint

```dockerfile
CMD ["sleep", "infinity"]
```

The container stays alive and idle. Both VS Code and `dev-box start` exec into it. Zellij is never the container entrypoint -- it is launched on attach.
