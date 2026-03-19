# dev-box

**Manage AI-ready development container environments.**

dev-box is a CLI tool and container image suite that provides reproducible,
AI-integrated development environments. Think of it as **uv for AI work
environments** — a single tool that handles container images, project
scaffolding, AI context structure, and work process management.

## What it provides

- **8 container images** — base, python, latex, typst, rust, python-latex,
  python-typst, rust-latex — published to GHCR, built on Debian Trixie Slim
- **A Rust CLI (`dev-box`)** — init, generate, build, start, stop, doctor, update
- **`dev-box.toml`** — single source of truth for project configuration
- **4 work process flavors** — minimal, managed, research, product — with
  structured AI context scaffolding
- **Context schemas** — versioned structure definitions with AI-driven migration

## Quick start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/scripts/install.sh | bash

# Or from source (requires Rust)
# cargo install --path cli

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
| zellij | Terminal multiplexer (Alt-key bindings) |
| vim | Editor with programming config |
| git + lazygit | Version control |
| gh | GitHub CLI |
| claude | Claude Code CLI |
| sox + pulseaudio | Audio support (Claude voice) |
| unzip | Archive extraction |
| curl, jq, less | Utilities |

## Image flavors

| Image | Adds |
|-------|------|
| `base` | Core tools only |
| `python` | Python 3.13 + uv + MkDocs Material |
| `latex` | TeX Live (LuaLaTeX, 100+ packages) |
| `typst` | Typst (modern typesetting) |
| `rust` | Rust toolchain (stable + clippy + rustfmt) |
| `python-latex` | Python + TeX Live |
| `python-typst` | Python + Typst |
| `rust-latex` | Rust + TeX Live |

## Work process flavors

| Flavor | Files | Use case |
|--------|-------|----------|
| `minimal` | CLAUDE.md only | Scripts, experiments |
| `managed` | + DECISIONS, BACKLOG, STANDUPS | Ongoing projects |
| `research` | + PROGRESS, research/, analysis/ | Learning, documentation |
| `product` | + PROJECTS, PRD, work-instructions/ | Full product development |

## dev-box.toml

```toml
[dev-box]
version = "0.3.5"
image = "python"
process = "product"

[container]
name = "my-project"
hostname = "my-project"
# user = "devuser"  # optional: run as non-root user

[ai]
providers = ["claude"]

[audio]
enabled = true
```

All devcontainer files (Dockerfile, docker-compose.yml, devcontainer.json)
are generated from this config via `dev-box generate`.

## CLI commands

```
dev-box init       # Create new project
dev-box generate   # Re-generate devcontainer files
dev-box build      # Build container image
dev-box start      # Start + attach via zellij
dev-box stop       # Stop container
dev-box attach     # Reattach to running container
dev-box status     # Show container state
dev-box doctor     # Validate context structure
dev-box update     # Check for updates
dev-box audio      # Audio diagnostics (check/setup)
```

## Documentation

Full documentation: [projectious-work.github.io/dev-box](https://projectious-work.github.io/dev-box/)

- [Installation](docs/getting-started/installation.md)
- [New project guide](docs/getting-started/new-project.md)
- [Existing project migration](docs/getting-started/existing-project.md)
- [CLI reference](docs/cli/commands.md)
- [Configuration reference](docs/cli/configuration.md)

## Development

This project is developed inside its own dev-container. The `.devcontainer/`
directory is hand-maintained (not generated) since this is the project that
builds the published images.

```bash
# Build CLI
cd cli && cargo build

# Run tests
cargo test

# Build docs (requires mkdocs-material)
mkdocs serve
```

## License

MIT
