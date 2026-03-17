# Changelog

All notable changes to dev-box are documented here.

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
