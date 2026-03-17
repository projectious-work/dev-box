# Roadmap

This page outlines planned features and improvements for dev-box.

## Current — v0.3.1

The current release includes:

- Rust CLI with full container lifecycle (init, generate, build, start, stop, attach, status, doctor, update)
- 8 container image flavors (base, python, latex, typst, rust, python-latex, python-typst, rust-latex)
- `dev-box.toml` configuration system
- 4 work process flavors (minimal, managed, research, product)
- Context scaffolding with OWNER.md sharing
- Audio support (PulseAudio bridging)
- Install script for pre-built binaries
- Shell completions for bash, zsh, fish, powershell, elvish
- Interactive init prompts when flags are omitted
- Registry-based update checking (`dev-box update --check`)
- Minijinja template engine for Dockerfile and docker-compose.yml generation
- Dockerfile.local support for project-specific build layers

## Recently Completed

### Shell Completions (v0.3.1)

`dev-box completions <shell>` generates completion scripts for all major shells.

### Interactive Init (v0.3.1)

`dev-box init` prompts for name, image, and process when flags are omitted in interactive terminals.

### Registry-Based Update Checking (v0.3.1)

`dev-box update --check` queries GHCR for the latest image tag and GitHub Releases for the latest CLI version.

### Template Engine Migration (v0.3.1)

Replaced string formatting in `generate.rs` with minijinja templates for better maintainability and extensibility.

### postCreateCommand and vscode_extensions (v0.3.1)

`post_create_command` and `vscode_extensions` fields in `[container]` config, generated into devcontainer.json.

### Dockerfile.local (v0.2.3)

Project-specific Dockerfile layers appended to the generated Dockerfile, with `AS dev-box` stage alias for multi-stage builds.

## Planned

### Evaluate Zensical as MkDocs Successor

MkDocs 2.0 introduces breaking changes. Evaluate Zensical and other alternatives for documentation generation, or pin to a stable MkDocs version.

### Automated Context Migration

When upgrading between schema versions, `dev-box doctor` will generate migration artifacts. A future version may automate safe migrations (additive changes) while prompting for manual review on breaking changes.

### Additional Image Flavors

Potential new flavors based on demand:

- **node** — Node.js LTS via NodeSource
- **go** — Go toolchain
- **python-rust** — Python + Rust combined

### Plugin System

Extensibility mechanism for custom commands and image overlays without forking.
