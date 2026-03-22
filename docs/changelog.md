# Changelog

All notable changes to dev-box are documented here.

## v0.7.0 — 2026-03-22

### Added
- **Addon packages system** (#18) — selectable tool bundles in dev-box.toml:
  - `infrastructure` (OpenTofu, Ansible, Packer)
  - `kubernetes` (kubectl, Helm, k9s, Kustomize)
  - `cloud-aws`, `cloud-gcp`, `cloud-azure`
  - `--addons` flag on `dev-box init`
  - Addons generate install layers in the Dockerfile
- **`dev-box audit` command** (#24) — run security checks:
  - cargo audit (Rust dependencies)
  - pip-audit (Python dependencies)
  - trivy (container image scanning)
  - Graceful skip when tools aren't installed
- **Zensical migration** (#26) — `zensical.toml` config created, `maintain.sh` auto-detects Zensical or falls back to MkDocs
- **Skills documentation page** — new `docs/skills.md` covering SKILL.md format, bundled skills, security
- **Comprehensive docs update** — all v0.6.0 features documented: AI providers, process templates, skills, shell tools

### Changed
- `maintain.sh` docs commands prefer Zensical over MkDocs when available

## v0.6.0 — 2026-03-22

### Added
- **AI provider flexibility** (#19) — AI tools are now optional and selectable:
  - `providers = ["claude", "aider", "gemini"]` in dev-box.toml
  - `providers = []` for no AI tools (no AI pane in layouts)
  - Multiple providers stacked in one pane (cowork) or separate tabs (dev/focus)
  - Dockerfile conditionally installs only selected providers
  - docker-compose mounts provider-specific config directories
- **Process templates** (#29, DEC-011) — 4 standard process docs scaffolded for managed/research/product:
  - `context/processes/release.md`, `code-review.md`, `feature-development.md`, `bug-fix.md`
  - Thin declarations (WHAT, not HOW) — skills handle execution
- **SKILL.md support** (#30, DEC-011) — 3 example skills scaffolded into `.claude/skills/`:
  - `backlog-context`, `decisions-adr`, `standup-context`
  - Uses the SKILL.md open standard format

### Fixed
- **Dockerfile optimization** (#27):
  - Pinned uv to v0.7.12 (was `:latest`)
  - Pinned Rust toolchain to v1.87.0 (was `stable`)
  - Fixed fontconfig registration in latex runtime stage (was in builder, got discarded)
  - Consolidated 10 COPY layers into 1 in base Dockerfile
- ripgrep and delta use gnu builds for aarch64 (no musl available)

### Changed
- All `generate` references replaced with `sync` across docs, CLI, and source

## v0.5.0 — 2026-03-22

### Added
- **`dev-box sync` command** — reconcile project state with dev-box.toml. Force-updates theme configs (vimrc, zellij, lazygit, yazi) and regenerates .devcontainer/ files. Primary command for applying config changes. `generate` is now an alias for `sync`.
- **Shell enhancement tools in base image** — ripgrep (rg), fd, bat, eza, zoxide, fzf, delta, starship. All version-pinned, downloaded in builder stage.
- **`.bashrc` with aliases** — `ls`→eza, `cat`→bat, `find`→fd, `grep`→rg + starship prompt, zoxide, fzf keybindings
- **Keyboard shortcuts cheatsheet** — new docs page with tabbed reference for Zellij, Yazi, Vim, lazygit

### Changed
- **`generate` deprecated** — replaced by `sync` (still works as alias). `sync` is a superset: it applies config changes (themes) AND regenerates container files.

## v0.4.2 — 2026-03-22

### Added
- **Complete theming** across all 4 tools: Zellij, Vim, Yazi, lazygit
  - 6 themes: gruvbox-dark (default), catppuccin-mocha, catppuccin-latte, dracula, tokyo-night, nord
  - `[appearance]` section in dev-box.toml with `theme` field
  - `--theme` flag on `dev-box init`
  - Vim colorschemes downloaded and bundled in base image (gruvbox, catppuccin, dracula, tokyo-night, nord)
  - Yazi theme.toml files for all 6 themes
  - lazygit config.yml themed for all 6 themes
  - Zellij KDL theme files for all 6 themes
- **Themes documentation page** with descriptions and color palettes
- `dev-box remove` command (alias `rm`) — stop and remove container

### Fixed
- Yazi parent column restored (`ratio = [1, 3, 4]`)
- vim-loop focus return to yazi on `:q`
- Dev layout starts focused on yazi
- Dockerfile updated: assist→cowork layout, Vim colorscheme downloads, Yazi themes bundled

## v0.4.1 — 2026-03-22

### Added
- **`dev-box remove` command** (alias `rm`) — stop and remove container (like `docker rm`), for Docker/kubectl naming consistency
- **`cowork` layout** — Yazi+Vim stacked left, Claude right, for side-by-side AI collaboration
- **`vim-loop`** — editor pane restarts vim on `:q` (use `:cq` to truly exit), keeping the pane alive for repeated file opens
- **Yazi `e` key** — opens file in the adjacent vim pane and focuses it (works per-layout: right in dev, down in cowork, tab switch in focus)
- Vim `:q` now returns focus to Yazi automatically
- Nerd Font fallback chain in generated `devcontainer.json` for Yazi/Zellij icon support

### Changed
- **Keybindings redesigned** — `Ctrl+b` leader key (Zellij Tmux mode) replaces `Alt` bindings that conflicted with macOS Option key special characters (`@`, `€`, `|`)
- **`Ctrl+b q`** added as quit alternative (for VS Code where `Ctrl+q` is caught)
- **Layouts redesigned**: removed `assist`, added `cowork`
  - `dev` (default): Yazi 40% + Vim 60% side by side, tabs for claude/git/shell
  - `focus`: one tool per tab fullscreen (yazi, vim, claude, git, shell)
  - `cowork`: Yazi+Vim left column, Claude right (50/50), tabs for git/shell
- VS Code default terminal changed from zellij to bash
- Pane borders enabled by default (`pane_frames true`, `rounded_corners true`)
- Yazi ratio restored to `[1, 3, 4]` (parent, file list, preview)
- Yazi Enter opens vim in-place (suspends yazi, `:q` returns) — works in all layouts

### Fixed
- Missing TeX Live packages: ninecolors, transparent, spath3, nicematrix, lipsum (fixes #13)
- TeX Live OpenType fonts registered with fontconfig via symlink + `fc-cache`
- Zellij `Escape` → `Esc` key name fix

## v0.4.0 — 2026-03-21

### Added
- **Environment management** — `dev-box env` command with 5 subcommands:
  - `dev-box env create <name>` — snapshot current state as a named environment
  - `dev-box env switch <name>` — save current, restore target, regenerate container files
  - `dev-box env list` — show available environments with current marker
  - `dev-box env delete <name>` — remove a saved environment
  - `dev-box env status` — show current environment and config summary
- **`context/shared/` directory** — files here are shared across all environments (not copied on switch). `OWNER.md` is seeded here by default. Move any file into `shared/` to share it.
- **`.dev-box-env/` storage** — per-environment snapshots of dev-box.toml, CLAUDE.md, and context/ (excluding shared/)

### Changed
- `OWNER.md` now scaffolded at `context/shared/OWNER.md` (backward compatible — existing `context/OWNER.md` is preserved)
- `.dev-box-env/` added to generated `.gitignore`

## v0.3.9 — 2026-03-21

### Added
- `dev-box backup` command — save dev-box files to timestamped backup directory (`.dev-box-backup/`)
- `dev-box reset` command — danger zone: backup + delete all dev-box files, with `--no-backup`, `--dry-run`, `--yes` flags, interactive confirmation, table output showing backup/delete status
- `.dev-box-backup/` added to generated `.gitignore`

### Fixed
- VS Code auto-forwarding PulseAudio TCP port: `portsAttributes` now added to `devcontainer.json` when audio is enabled (fixes #11)
- Embedded Zellij layouts: use multi-line KDL syntax (fixes parse errors on Zellij 0.43.1)
- Builder stage: added `unzip` for Yazi download

## v0.3.8 — 2026-03-21

### Added
- `--layout` flag on `dev-box start` and `dev-box attach` — choose between `dev` (default), `focus`, or `assist` layouts
- Layout descriptions in CLI help text (`--help` shows what each layout looks like)

### Fixed
- Comprehensive docs update: Strider→Yazi references, documented 3 IDE layouts, updated `update` command with `--dry-run`, updated roadmap to v0.3.8, corrected test counts

## v0.3.7 — 2026-03-20

### Added
- **Yazi file manager** replaces Strider as the default sidebar file manager in all layouts (Strider remains available as `Alt s` floating overlay)
- **Three IDE layouts**: `dev` (VS Code-like, default), `focus` (single-task, stacked panes), `assist` (Claude-focused with center stage) — all with shared git, shell, and help tabs
- LaTeX Workshop settings in generated `devcontainer.json`: latexmk recipes (lualatex, pdflatex, lualatex+biber), `--shell-escape`, output to `./out`, PDF viewer in tab, biber tool, auto-build on save, clean file types
- `out/` added to LaTeX `.gitignore` block

### Fixed
- Runtime detection: prefer docker over podman, verify daemon is responsive via `docker info` / `podman info` — fixes OrbStack compatibility where podman is on PATH but not running
- Better error messages when runtime is on PATH but daemon not responding

### Changed
- Bumped Typst 0.13.1 → 0.14.2

## v0.3.6 — 2026-03-19

### Fixed
- Runtime detection: prefer docker over podman, and verify daemon is responsive (`docker info` / `podman info`) before selecting — fixes OrbStack compatibility where podman is on PATH but not running (fixes #5 regression)
- Better error messages when runtime is on PATH but daemon is not responding
- Bumped Typst 0.13.1 → 0.14.2

## v0.3.5 — 2026-03-19

### Added
- `[ai]` config section with `providers` field — controls which AI tool directories are mounted (currently: `claude`, more planned)
- `container.user` field — run containers as non-root user, adjusts all mount paths automatically
- `--ai` and `--user` flags on `dev-box init`
- CLI help: usage examples in main help text, `--image` and `--process` show valid values via value_enum
- `.gitignore` scaffolding: language-specific blocks (Python, LaTeX, Typst, Rust) based on image flavor, project-specific section at top
- `dev-box doctor` checks: .gitignore entry validation, mount source path verification, .root/ → .dev-box-home/ migration suggestion
- `dev-box generate` now re-seeds `.dev-box-home/` to stay consistent with config changes
- OWNER.md extended fields: domain expertise, primary languages, communication language, timezone, working hours, current focus
- Base image: added `unzip` to standard packages (fixes #5)
- TeX Live images: added `algorithms`, `algorithmicx`, `algorithm2e`, `tikzfill` packages (fixes #7, fixes #8)

### Changed
- Renamed `.root/` to `.dev-box-home/` — backward compatible (falls back to `.root/` if it exists). If upgrading, use a plain filesystem rename: `mv .root .dev-box-home` (not `git mv` — the directory is gitignored and not tracked)
- `dev-box init` now creates `.dev-box-home/` directory (previously only done on `start`)
- OWNER.md created locally in `context/` — removed `~/.config/dev-box/` symlink pattern
- Removed `owner` field from `[context]` config section
- Generated `dev-box.toml` now includes comprehensive comments explaining all options
- Generated `docker-compose.yml` includes comments on each mount and AI provider-conditional volumes
- This project's CLAUDE.md migrated from 965-line monolith to context/ structure

### Fixed
- `dev-box generate` without prior `start` could produce compose files referencing non-existent mount directories

## v0.3.4 — 2026-03-18

### Fixed
- Base image: added `libasound2-plugins` (ALSA PulseAudio backend) and `libsox-fmt-pulse` (sox PulseAudio output) — without these, audio config existed but silently failed
- Container runtime autodiscovery: `maintain.sh` now checks `docker info` / `podman info` instead of just PATH presence (fixes OrbStack compatibility)

## v0.3.3 — 2026-03-18

### Added
- `dev-box audio check` — host-side PulseAudio diagnostics (installation, daemon, TCP module, persistence, port, launchd, connectivity)
- `dev-box audio setup` — automated PulseAudio setup on macOS (brew install, TCP config, launchd agent with KeepAlive); Linux manual instructions
- OrbStack compatibility: documented virtiofs permission workaround and Claude Code OAuth issue (upstream: anthropics/claude-code#14528)

### Fixed
- ALSA config syntax: standardized `.asoundrc` to `pcm.!default { type pulse }` (fixes parse errors, refs #3)
- Removed `:ro` from `.asoundrc` volume mount in generated compose files (fixes #3)
- Added audio support to this project's devcontainer (`.asoundrc`, `PULSE_SERVER`, `AUDIODRIVER`)

### Changed
- Rewrote release workflow in CLAUDE.md with complete 10-step checklist

## v0.3.2 — 2026-03-17

### Fixed
- Python image: added `unzip` to system packages (fixes #2) — required by Reflex, Bun, and other installers

### Changed
- Comprehensive docs review: updated all version references, documented `post_create_command` and `vscode_extensions` in configuration reference, fixed stale claims, corrected roadmap version attributions

## v0.3.1 — 2026-03-17

### Added
- `post_create_command` field in `[container]` config, generated into devcontainer.json
- `vscode_extensions` field in `[container]` config, generated into devcontainer.json

### Changed
- Pinned MkDocs dependency to `<2` to avoid breaking changes from MkDocs 2.0

## v0.3.0 — 2026-03-17

### Added
- Shell completions: `dev-box completions bash/zsh/fish/powershell/elvish`
- Interactive init: prompts for name, image, and process when flags are omitted
- Registry version checking: `dev-box update --check` queries GHCR and GitHub Releases
- Minijinja templates for Dockerfile and docker-compose.yml generation

## v0.2.3 — 2026-03-17

### Fixed
- Generated compose volume paths: `.root/` resolved incorrectly from `.devcontainer/` — now uses `../.root/`
- install.sh: BSD sed `\s` incompatibility on macOS
- install.sh: `info()` stdout leak into version capture
- install.sh: unbound `tmpdir` variable in EXIT trap

### Added
- Dockerfile.local support: project-specific layers appended to generated Dockerfile
- `AS dev-box` stage alias in generated Dockerfile for multi-stage builds
- Cargo cross-compilation config for x86_64

## v0.2.2 — 2026-03-17

### Fixed
- GHCR image path in generated Dockerfile: `FROM registry:image-vX.Y.Z` format

## v0.2.1 — 2026-03-17

### Fixed
- TeX Live image build: ca-certificates, filename typo, symlinks
- Typst image: missing xz-utils dependency
- Doctor: runtime detection warning downgraded (works inside containers)

### Added
- CLI `--version` flag
- `push-images` command in maintain.sh with auto gh-auth login

## v0.2.0 — 2026-03-16

### Added
- Security hardening: input validation for container names, hostnames, package names
- Code quality improvements from comprehensive 3-agent review
- Upstream contribution guidelines

### Changed
- Removed unused dependencies (minijinja, serde_yaml, ureq) — re-added in v0.3.0
- Consolidated duplicated helper functions
- Data-driven context scaffolding shared between init and doctor

## v0.1.0 — 2026-03-16

### Added
- Initial release
- 8 container image flavors (base, python, latex, typst, rust, python-latex, python-typst, rust-latex)
- 4 work process flavors (minimal, managed, research, product)
- CLI commands: init, generate, build, start, stop, attach, status, doctor, update
- Context schema v1.0.0 with migration artifact generation
- MkDocs documentation site
- Install script for macOS and Linux
