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

Ten pre-built images based on Debian Trixie Slim, each with a complete development environment:

| Image | What it adds |
|-------|-------------|
| `base` | Zellij, Vim, Git, lazygit, Claude CLI, audio support |
| `python` | Python 3.13, uv, MkDocs Material |
| `latex` | TeX Live with common packages |
| `typst` | Typst (modern typesetting) |
| `rust` | Rust toolchain via rustup |
| `node` | Node.js LTS |
| `go` | Go toolchain |
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

After `dev-box start`, you are inside a Zellij session with the **dev** layout: Yazi file browser (40%) and Vim editor (60%) side by side, plus tabs for Claude Code, lazygit, and shell.

Two additional layouts are available: **focus** (one tool per tab, fullscreen) and **cowork** (Yazi+Vim left, Claude right for AI-assisted coding). See [Base Image — Layouts](container/base-image.md#layouts).

## Why dev-box?

**Reproducibility.** Every team member gets the same environment. No "works on my machine."

**AI-native.** Context schemas give AI agents structured project knowledge instead of relying on ad-hoc prompts.

**Zero lock-in.** dev-box generates standard devcontainer files. Stop using the CLI any time -- your `.devcontainer/` directory still works.

**Composable.** Start with `base`, add language support when you need it. Extend with `extra_packages` in `dev-box.toml` without forking images.

**AI-flexible.** Declare which AI providers your project uses (Claude, Aider, Gemini) via the `[ai]` section, and dev-box ensures their CLIs and credentials are available inside the container. Mix and match providers as needed.

## Project Status

dev-box is at version 0.7.0 with 16 CLI commands, 10 container image flavors, and 11 addon bundles. The core workflow (init, sync, build, start, stop, remove, attach, status, doctor) is functional, along with shell completions, interactive init prompts, registry-based update with upgrade (`dev-box update`), named environments (`dev-box env`), backup/reset lifecycle management, and host-side audio diagnostics via `dev-box audio check/setup`. The project includes Yazi file manager with three IDE layouts (dev, focus, cowork), AI provider flexibility with support for multiple providers (Claude, Aider, Gemini) via the `[ai]` section, addon bundles for infrastructure, cloud, and documentation tools via the `[addons]` section, process templates for standard workflows (release, code review, feature development, bug fix), SKILL.md support for executable AI agent instructions following the [open standard](https://agentskills.io/specification), modern shell tools (ripgrep, fd, bat, eza, zoxide, fzf, delta, starship) in the base image, six color themes across all tools, security scanning (`dev-box audit`), non-root user support (`container.user`), and language-specific `.gitignore` blocks generated per image flavor.

## Next Steps

- [Install dev-box](getting-started/installation.md)
- [Create your first project](getting-started/new-project.md)
- [Explore container images](container/base-image.md)
- [Understand the context system](context/overview.md)
- [CLI reference](cli/commands.md)
