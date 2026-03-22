# dev-box

**Manage AI-ready development container environments.**

dev-box is a CLI tool and container image suite that provides reproducible,
AI-integrated development environments. Think of it as **uv for AI work
environments** — a single tool that handles container images, project
scaffolding, AI context structure, and work process management.

## What it provides

- **8 container images** — base, python, latex, typst, rust, python-latex,
  python-typst, rust-latex — published to GHCR, built on Debian Trixie Slim
- **A Rust CLI (`dev-box`)** — init, sync, build, start, stop, remove, doctor, audit, and more
- **`dev-box.toml`** — single source of truth for project configuration
- **4 work process flavors** — minimal, managed, research, product — with
  structured AI context scaffolding
- **6 color themes** — gruvbox-dark, catppuccin-mocha/latte, dracula, tokyo-night, nord
- **3 IDE layouts** — dev, focus, cowork — with Ctrl+b leader keybindings
- **AI provider flexibility** — Claude, Aider, Gemini — optional, stackable
- **Process templates + SKILL.md** — standard workflows + agent-executable skills
- **Addon packages** — infrastructure, kubernetes, cloud CLIs as selectable bundles
- **Modern shell tools** — ripgrep, fd, bat, eza, zoxide, fzf, delta, starship

## Quick start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/scripts/install.sh | bash

# Create a new project
mkdir my-project && cd my-project
dev-box init --name my-project --image python --process product

# Build and start
dev-box build
dev-box start
```

## Base image tools

All images include:

| Tool | Purpose |
|------|---------|
| Zellij | Terminal multiplexer (Ctrl+b leader) |
| Yazi | Terminal file manager |
| Vim | Editor with programming config |
| Git + lazygit | Version control |
| GitHub CLI (`gh`) | GitHub integration |
| ripgrep, fd, bat, eza | Modern CLI replacements |
| zoxide, fzf, delta | Smart cd, fuzzy finder, diff viewer |
| Starship | Fast, themed shell prompt |
| Claude CLI | AI assistant (configurable) |

## dev-box.toml

```toml
[dev-box]
version = "0.7.0"
image = "python"
process = "product"

[container]
name = "my-project"

[ai]
providers = ["claude", "aider"]

[appearance]
theme = "catppuccin-mocha"

[addons]
bundles = ["infrastructure", "kubernetes"]
```

All devcontainer files are generated from this config via `dev-box sync`.

## CLI commands

```
dev-box init       # Create new project (interactive or with flags)
dev-box sync       # Apply config changes (themes, AI providers, etc.)
dev-box build      # Build container image
dev-box start      # Start + attach via zellij
dev-box stop       # Stop container
dev-box remove     # Stop + remove container (alias: rm)
dev-box attach     # Reattach to running container
dev-box status     # Show container state
dev-box doctor     # Validate project structure
dev-box update     # Check for / apply updates
dev-box env        # Manage named environments (create/switch/list/delete)
dev-box backup     # Backup dev-box files
dev-box reset      # Remove all dev-box files (danger zone)
dev-box audit      # Run security checks (cargo audit, pip-audit, trivy)
dev-box audio      # Audio diagnostics (check/setup)
```

## Documentation

Full documentation: [projectious-work.github.io/dev-box](https://projectious-work.github.io/dev-box/)

## Development

This project is developed inside its own dev-container.

```bash
cd cli && cargo build        # Build CLI
cd cli && cargo test         # Run tests (147 tests)
cd cli && cargo clippy -- -D warnings  # Lint
```

## License

MIT
