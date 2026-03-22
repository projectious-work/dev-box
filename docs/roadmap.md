# Roadmap

This page outlines planned features and improvements for dev-box.

## Current — v0.4.1

The current release includes:

- Rust CLI with full container lifecycle (init, generate, build, start, stop, remove, attach, status, doctor, update)
- 8 container image flavors (base, python, latex, typst, rust, python-latex, python-typst, rust-latex)
- `dev-box.toml` configuration system
- 4 work process flavors (minimal, managed, research, product)
- Context scaffolding with `context/shared/` for cross-environment files
- Named environment management (`dev-box env create/switch/list/delete/status`)
- Color theming across Zellij, Vim, Yazi, and lazygit (6 themes)
- Three IDE layouts: dev, focus, cowork
- Yazi file manager with vim-loop integration (Enter/e to open files)
- Audio support (PulseAudio bridging) with `dev-box audio check/setup`
- `dev-box backup` and `dev-box reset` commands
- Shell completions, interactive init, registry-based update/upgrade
- Dockerfile.local for project-specific build layers
- AI provider configuration (`[ai]` section)
- Non-root user support (`container.user`)

## In Progress

### Consistent Color Theming (#14)

Infrastructure complete: 6 themes (gruvbox-dark, catppuccin-mocha, catppuccin-latte, dracula, tokyo-night, nord) applied across Zellij, Vim, Yazi, and lazygit. Remaining: screenshot gallery in docs, theme switching without manual file deletion.

## Planned — Near Term

### Shell Enhancement Tools

Install modern CLI tools in the base image: ripgrep, fd, bat, eza, zoxide, fzf, delta, Starship prompt. Shell aliases (`ls→eza`, `cat→bat`). Starship prompt themed to match `[appearance]` setting.

### Keyboard Shortcuts Cheatsheet Page (#16)

Comprehensive, compact reference page in docs covering all keybindings for Zellij (`Ctrl+b` leader), Yazi, Vim, and lazygit.

### AI Provider Flexibility (#19)

Make AI tools fully optional and selectable:
- Move Claude CLI from base image to generated Dockerfile layer
- Support: Claude Code, Aider (open-source, multi-model), Gemini CLI, Codex CLI, Goose
- Multiple providers stacked in one layout pane
- No AI pane if `providers = []`
- Open-source model support via Aider + Ollama

### Addon Packages System (#18)

Selectable tool bundles added to generated Dockerfile:

- **infrastructure** — OpenTofu, Ansible, kubectl, Helm, k9s
- **cloud-aws** — AWS CLI v2, aws-vault, SSM plugin
- **cloud-gcp** — Google Cloud CLI
- **cloud-azure** — Azure CLI
- **shell-tools** — ripgrep, fd, bat, eza, zoxide, fzf, delta, Starship (default ON)
- **data-science** — Jupyter, pandas, numpy (python images only)
- **docs** — Pandoc, Quarto

```toml
[addons]
bundles = ["shell-tools", "infrastructure"]
```

### Bash Prompt Themes (#17)

Starship prompt with theme presets matching the color theme. From minimal to full-featured with git status, language versions, kubernetes context, and more.

## Planned — Medium Term

### Plugin / Extension System (#20)

Extensibility architecture for dev-box:
- Hook system (pre/post lifecycle commands)
- Custom template overrides (Dockerfile.j2, layouts)
- Community-distributed features (via git repos)
- Investigation: shell scripts vs WASM plugins

### Zellij Plugin Integration (#21)

Evaluate existing Zellij WASM plugins:
- **zjstatus** — configurable status bar with git info, hostname, time
- **yazelix** — deeper Yazi+editor integration
- Custom dev-box status plugin showing container state, environment name, theme

### Additional Image Flavors

New flavors based on demand:
- **node** — Node.js LTS
- **go** — Go toolchain
- **python-rust** — Python + Rust combined
- **python-node** — Python + Node.js (full-stack)

### Automated Context Migration

When upgrading between schema versions, automate safe migrations (additive changes) while generating AI-assisted migration prompts for breaking changes.

## Planned — Long Term

### Documentation System Migration

Evaluate alternatives to MkDocs (Zensical, mdBook, etc.) or pin to stable MkDocs version before 2.0 breaking changes.

### Multi-Service Support

Support for additional services in docker-compose (databases, caches, message queues) configured via `dev-box.toml`. `dev-box ps` / `dev-box logs` commands.

### Remote Development

Support for running dev-box environments on remote hosts (cloud VMs, SSH targets) with local CLI as a thin client.
