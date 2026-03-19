# Changelog

All notable changes to dev-box are documented here.

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
- Renamed `.root/` to `.dev-box-home/` — backward compatible (falls back to `.root/` if it exists)
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
