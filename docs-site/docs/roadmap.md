---
title: Roadmap
---

# Roadmap

This page outlines planned features and improvements for dev-box.
The internal source of truth is `context/BACKLOG.md` (BACK-NNN IDs);
this page is the public-facing summary.

## Current — v0.8.0

The current release includes:

- Rust CLI with 16 commands (init, sync, build, start, stop, remove, attach, status, doctor, completions, update, env, backup, reset, audit, audio)
- 10 container image flavors (base, python, latex, typst, rust, node, go, python-latex, python-typst, rust-latex)
- `dev-box.toml` configuration system with 7 sections (dev-box, container, context, ai, addons, appearance, audio)
- 4 work process flavors (minimal, managed, research, product)
- Context scaffolding with `context/shared/` for cross-environment files
- Named environment management (`dev-box env create/switch/list/delete/status`)
- `dev-box sync` — reconcile config changes (themes, AI, etc.) without manual file deletion
- Color theming across Zellij, Vim, Yazi, and lazygit (6 themes) with 6 starship prompt presets
- Three IDE layouts: dev, focus, cowork (Ctrl+b leader keybindings)
- AI provider flexibility: Claude, Aider, Gemini — optional, stackable, dynamic layouts
- Process templates (release, code-review, feature-dev, bug-fix) with SKILL.md support
- 83 curated skills across 14 categories with 57 reference files
- Addon bundles (11 total): infrastructure, kubernetes, cloud-aws/gcp/azure, docs-mkdocs/zensical/docusaurus/starlight/mdbook/hugo
- `dev-box audit` — security scanning (cargo audit, pip-audit, trivy)
- Shell tools: ripgrep, fd, bat, eza, zoxide, fzf, delta, starship + aliases
- Yazi file manager with vim-loop (Enter/e to open files)
- `dev-box backup`, `dev-box reset`, `dev-box remove`/`rm`
- Audio support (PulseAudio bridging), shell completions, interactive init
- Zensical documentation migration (with MkDocs fallback)

## Planned — Near Term

### Theming Screenshots (BACK-001)

Interactive asciinema recordings and screenshot gallery showing all 6 themes across all tools in docs.

### Security Review (BACK-002)

Comprehensive input validation, container security audit, and supply chain review.

### Skill Install Command (BACK-003)

`dev-box skill install` — install skills from external sources into project `.claude/skills/`.

### CLI Simplification (BACK-005)

Merge `build`/`attach` into `start` to reduce command surface area.

## Planned — Medium Term

### Skill Eval Framework (BACK-004)

Test and benchmark skills per Anthropic's skill-creator pattern.

### Plugin / Extension System (BACK-007)

Hook system, custom template overrides, community-distributed features.

### Automated Context Migration (BACK-009)

AI-assisted prompts for schema version upgrades with safe auto-migration for additive changes.

### Multi-Service Support (BACK-010)

Additional docker-compose services, `dev-box ps`/`dev-box logs` commands.

## Planned — Long Term

### Remote Development (BACK-011)

Run dev-box environments on remote hosts with local CLI as thin client.

### Image Signing (BACK-015)

sigstore/cosign for published container images.

### Zellij Plugin Integration (BACK-008)

zjstatus (configurable status bar), custom dev-box status plugin.
