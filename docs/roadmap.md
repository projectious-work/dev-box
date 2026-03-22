# Roadmap

This page outlines planned features and improvements for dev-box.

## Current — v0.7.0

The current release includes:

- Rust CLI with 16 commands (init, sync, build, start, stop, remove, attach, status, doctor, completions, update, env, backup, reset, audit, audio)
- 10 container image flavors (base, python, latex, typst, rust, node, go, python-latex, python-typst, rust-latex)
- `dev-box.toml` configuration system with 7 sections (dev-box, container, context, ai, addons, appearance, audio)
- 4 work process flavors (minimal, managed, research, product)
- Context scaffolding with `context/shared/` for cross-environment files
- Named environment management (`dev-box env create/switch/list/delete/status`)
- `dev-box sync` — reconcile config changes (themes, AI, etc.) without manual file deletion
- Color theming across Zellij, Vim, Yazi, and lazygit (6 themes)
- Three IDE layouts: dev, focus, cowork (Ctrl+b leader keybindings)
- AI provider flexibility: Claude, Aider, Gemini — optional, stackable, dynamic layouts
- Process templates (release, code-review, feature-dev, bug-fix) with SKILL.md support
- Addon bundles (11 total): infrastructure, kubernetes, cloud-aws/gcp/azure, docs-mkdocs/zensical/docusaurus/starlight/mdbook/hugo
- `dev-box audit` — security scanning (cargo audit, pip-audit, trivy)
- Shell tools: ripgrep, fd, bat, eza, zoxide, fzf, delta, starship + aliases
- Yazi file manager with vim-loop (Enter/e to open files)
- `dev-box backup`, `dev-box reset`, `dev-box remove`/`rm`
- Audio support (PulseAudio bridging), shell completions, interactive init
- Zensical documentation migration (with MkDocs fallback)

## Planned — Near Term

### Theming Screenshots (#14)

Screenshot gallery in docs showing all 6 themes across all 4 tools.

### Starship Prompt Presets (#17)

Configurable prompt presets in `dev-box.toml`, themed to match the selected color theme.

### Additional Image Flavors

- **python-rust** — Python + Rust combined

### Curated Skill Library (#30)

Expand from 3 example skills to 50-100 vetted skills covering development, process, language-specific, infrastructure, and security categories. `dev-box skill install` command.

## Planned — Medium Term

### Plugin / Extension System (#20)

Hook system, custom template overrides, community-distributed features.

### Zellij Plugin Integration (#21)

zjstatus (configurable status bar), custom dev-box status plugin.

### Automated Context Migration

AI-assisted prompts for schema version upgrades with safe auto-migration for additive changes.

### Multi-Service Support

Additional docker-compose services, `dev-box ps`/`dev-box logs` commands.

## Planned — Long Term

### Remote Development

Run dev-box environments on remote hosts with local CLI as thin client.

### Dockerfile Best Practices (#27)

Cache mounts, digest pinning, binary checksum verification, SBOM generation.

### Security Hardening (#23)

Container image signing (cosign), comprehensive input validation audit, supply chain verification.
