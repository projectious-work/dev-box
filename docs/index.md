# dev-box

**Manage AI-ready development container environments.**

dev-box is to AI development environments what [uv](https://github.com/astral-sh/uv) is to Python packaging: a single tool that eliminates duplication, enforces consistency, and gets out of your way.

## The Problem

Every AI-assisted project needs the same boilerplate:

- A Dockerfile with the right tools (editor, multiplexer, git, Claude CLI)
- A `docker-compose.yml` with volume mounts for persistence
- A `devcontainer.json` for VS Code integration
- Structured context files so AI agents understand the project
- Audio bridging so voice features work inside the container

Copying and maintaining these files across projects is tedious and error-prone. When you improve one project's setup, the others fall behind.

## The Solution

dev-box provides three integrated pillars:

### 1. Published Container Images

Eight pre-built images based on Debian Trixie Slim, each with a complete development environment:

| Image | What it adds |
|-------|-------------|
| `base` | Zellij, Vim, Git, lazygit, Claude CLI, audio support |
| `python` | Python 3.13, uv, MkDocs Material |
| `latex` | TeX Live with common packages |
| `typst` | Typst (modern typesetting) |
| `rust` | Rust toolchain via rustup |
| `python-latex` | Python + TeX Live combined |
| `python-typst` | Python + Typst combined |
| `rust-latex` | Rust + TeX Live combined |

### 2. A Rust CLI

A single binary that manages the full lifecycle:

```bash
# Initialize a new project
dev-box init --name my-app --image python --process product

# Build and start
dev-box build
dev-box start
```

The CLI reads `dev-box.toml` as the single source of truth, generates all devcontainer files, manages container lifecycle, and validates project structure.

### 3. Context Schemas for AI Work Processes

Structured context files that give AI agents the information they need:

- **DECISIONS.md** -- architectural decisions with rationale
- **BACKLOG.md** -- prioritized work items
- **STANDUPS.md** -- session-by-session progress
- **OWNER.md** -- per-project identity and preferences for AI agents

Four process flavors (`minimal`, `managed`, `research`, `product`) scale from simple scripts to full product development.

## Quick Start

```bash
# Install dev-box
curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/scripts/install.sh | bash

# Create a new project
mkdir my-project && cd my-project
dev-box init --name my-project --image python --process managed

# Build the container image and start working
dev-box build
dev-box start
```

After `dev-box start`, you are inside a Zellij session with three tabs:

- **dev** -- Vim editor, terminal panes, Claude Code
- **git** -- lazygit for version control
- **shell** -- clean bash terminal

## Why dev-box?

**Reproducibility.** Every team member gets the same environment. No "works on my machine."

**AI-native.** Context schemas give AI agents structured project knowledge instead of relying on ad-hoc prompts.

**Zero lock-in.** dev-box generates standard devcontainer files. Stop using the CLI any time -- your `.devcontainer/` directory still works.

**Composable.** Start with `base`, add language support when you need it. Extend with `extra_packages` in `dev-box.toml` without forking images.

**AI-configurable.** Declare which AI providers your project uses via the `[ai]` section, and dev-box ensures their CLIs and credentials are available inside the container.

## Project Status

dev-box is at version 0.3.5. The core workflow (init, generate, build, start, stop, attach, status, doctor) is functional, along with shell completions (`dev-box completions bash/zsh/fish`), interactive init prompts, registry-based version checking via `dev-box update --check`, `post_create_command`/`vscode_extensions` support in devcontainer.json, and host-side audio diagnostics via `dev-box audio check/setup`. Recent additions include AI provider configuration (`[ai]` section), non-root user support (`container.user`), renamed `.dev-box-home/` for persistent config (with `.root/` backward compatibility), and language-specific `.gitignore` blocks generated per image flavor.

## Next Steps

- [Install dev-box](getting-started/installation.md)
- [Create your first project](getting-started/new-project.md)
- [Explore container images](container/base-image.md)
- [Understand the context system](context/overview.md)
- [CLI reference](cli/commands.md)
