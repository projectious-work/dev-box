# dev-box

**Containerized development environments for AI-assisted work.**

AI-assisted development works best on the console and inside containers — for security, reproducibility, and control. But setting up a proper containerized environment with the right tools, AI integrations, structured context, and work processes is tedious boilerplate that drifts across projects.

dev-box eliminates that boilerplate. One config file, one CLI, one command to go from empty directory to a fully equipped development environment with terminal multiplexer, file manager, editor, AI assistants, and curated agent skills — all inside a container.

## The gap dev-box fills

The market is split: **environment tools** (DevPod, Codespaces, Coder) know nothing about AI context. **AI tools** (Cursor, Windsurf, Claude Code) know nothing about environment management. **Context standards** (AGENTS.md, SKILL.md) are files without a management layer.

dev-box bridges this gap — it unifies environment definition, AI context structure, and terminal-first tooling into a single coherent system:

- **Environment tools** give you a container. dev-box gives you a container that knows your AI providers, skills, work processes, and theming.
- **AI IDEs** lock you into a GUI. dev-box works with any terminal-based AI tool (Claude Code, Aider, Codex CLI, Gemini CLI) without IDE lock-in.
- **Curated quality over marketplace chaos.** Community skill hubs have 97K+ entries, but nearly half are duplicates. dev-box ships 83 vetted skills with reference files — tested, composable, and safe.

## How it works

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/scripts/install.sh | bash

# Create a project
mkdir my-project && cd my-project
dev-box init --name my-project --image python --process managed

# Build and start
dev-box build
dev-box start
```

After `dev-box start`, you're inside a Zellij terminal session with Yazi file browser, Vim editor, Claude Code, lazygit, and a shell — all themed consistently and ready to work.

<div class="asciinema" data-cast="assets/screencasts/layout-dev.cast" data-poster="npt:4" data-autoplay="false" data-fit="width"></div>

Two additional layouts are available: **focus** (one tool per tab, fullscreen) and **cowork** (Yazi+Vim left, Claude right for pair programming). See [Layouts](container/base-image.md#layouts).

## What dev-box manages

### Container images

Ten pre-built images on Debian Trixie Slim, published to GHCR:

| Image | What it adds |
|-------|-------------|
| `base` | Zellij, Vim, Git, lazygit, Claude CLI, modern shell tools |
| `python` | Python 3.13, uv |
| `latex` | TeX Live with common packages |
| `typst` | Typst (modern typesetting) |
| `rust` | Rust toolchain via rustup |
| `node` | Node.js LTS |
| `go` | Go toolchain |
| `python-latex` | Python + TeX Live combined |
| `python-typst` | Python + Typst combined |
| `rust-latex` | Rust + TeX Live combined |

Every image includes: Zellij, Yazi, Vim, Git, lazygit, GitHub CLI, ripgrep, fd, bat, eza, zoxide, fzf, delta, Starship prompt, and configurable AI assistants. See [Base Image](container/base-image.md) and [Image Flavors](container/flavors.md).

### Project configuration

A single `dev-box.toml` drives everything. The CLI generates `.devcontainer/` files from this config. Change the config, run `dev-box sync`, done.

```toml
[dev-box]
version = "0.8.0"
image = "python"
process = "managed"

[container]
name = "my-project"

[ai]
providers = ["claude", "aider"]

[appearance]
theme = "catppuccin-mocha"

[addons]
bundles = ["infrastructure", "kubernetes"]
```

See [Configuration](cli/configuration.md) for the full specification.

### AI context structure

Structured context files give AI agents project knowledge instead of relying on ad-hoc prompts:

- **DECISIONS.md** — architectural decisions with rationale
- **BACKLOG.md** — prioritized work items
- **STANDUPS.md** — session-by-session progress
- **OWNER.md** — project identity and preferences for AI agents

Four process flavors (`minimal`, `managed`, `research`, `product`) scale from quick scripts to full product development. See [Work Processes](context/work-processes.md).

### 83 curated agent skills

Instructions following the open [SKILL.md standard](https://agentskills.io/specification) across 14 categories: from Kubernetes and SQL patterns to RAG engineering and prompt design. Skills use progressive disclosure — concise instructions load first, detailed reference files on demand.

Browse the full catalog in the [Skills Library](skills/index.md).

### Addon packages

Selectable tool bundles added via config without forking images:

- `infrastructure` — OpenTofu, Ansible, Packer
- `kubernetes` — kubectl, Helm, k9s, Kustomize
- `cloud-aws`, `cloud-gcp`, `cloud-azure` — cloud CLIs
- Documentation tools — MkDocs, Hugo, mdBook, Zensical, and more

### 6 color themes

Gruvbox Dark, Catppuccin Mocha/Latte, Dracula, Tokyo Night, Nord — applied consistently across Zellij, Vim, Yazi, lazygit, and Starship. See [Themes](themes.md).

## Why containers?

**Security.** AI agents run in an isolated environment, not on your host. Container boundaries control what the AI can access.

**Reproducibility.** Every team member gets the same tools and versions. No "works on my machine."

**Control.** You define the environment declaratively. Changes go through `dev-box.toml`, not manual installs.

**Zero lock-in.** dev-box generates standard devcontainer files. Stop using the CLI any time — your `.devcontainer/` directory still works with VS Code, GitHub Codespaces, or any devcontainer-compatible tool.

## Get started

- [Install dev-box](getting-started/installation.md)
- [Create your first project](getting-started/new-project.md)
- [Add dev-box to an existing project](getting-started/existing-project.md)
- [Browse the Skills Library](skills/index.md)
- [CLI reference](cli/commands.md)
