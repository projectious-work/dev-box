---
sidebar_position: 2
title: "New Project"
---

# New Project

This guide walks through creating a new project from scratch with aibox.

## Initialize the Project

```bash
mkdir my-app && cd my-app
git init

aibox init --name my-app --image python --process product
```

The `init` command accepts these options:

| Option | Default | Description |
|--------|---------|-------------|
| `--name` | Current directory name | Container and hostname |
| `--image` | `base` | Image flavor (`base`, `python`, `latex`, `typst`, `rust`, `node`, `go`, `python-latex`, `python-typst`, `rust-latex`) |
| `--process` | `product` | Work process flavor (`minimal`, `managed`, `research`, `product`) |

If you omit options, `aibox init` runs interactively and prompts for each value.

<div class="asciinema" data-cast="/screencasts/init-demo.cast" data-poster="npt:0" data-fit="width"></div>

## What Gets Created

After running `init` with `--process product`, your project looks like this:

```
my-app/
├── aibox.toml                  # Single source of truth
├── CLAUDE.md                     # AI agent entry point
├── .gitignore                    # Generated with language-specific blocks
├── .aibox-version              # Tracks schema version
├── .aibox-home/                # Persistent config (git-ignored)
├── .devcontainer/
│   ├── Dockerfile                # Generated from aibox.toml
│   ├── docker-compose.yml        # Generated — volume mounts, env vars
│   └── devcontainer.json         # Generated — VS Code integration
├── .claude/
│   └── skills/                   # 83 curated agent skills
│       ├── code-review/SKILL.md
│       ├── kubernetes-basics/
│       │   ├── SKILL.md
│       │   └── references/
│       └── ...
├── context/
│   ├── shared/
│   │   └── OWNER.md              # Project identity (shared across envs)
│   ├── BACKLOG.md                # Prioritized work items
│   ├── DECISIONS.md              # Architectural decision records
│   ├── STANDUPS.md               # Session progress notes
│   ├── PROJECTS.md               # Project portfolio tracking
│   ├── PRD.md                    # Product requirements document
│   ├── work-instructions/
│   │   ├── GENERAL.md            # General rules and conventions
│   │   ├── DEVELOPMENT.md        # Build, test, project structure
│   │   └── TEAM.md               # Agent strategy and team setup
│   ├── processes/
│   │   ├── README.md
│   │   ├── release.md
│   │   ├── code-review.md
│   │   ├── feature-development.md
│   │   └── bug-fix.md
│   ├── research/
│   │   └── _template.md          # Template for research documents
│   ├── project-notes/
│   └── ideas/
└── experiments/
    └── README.md
```

:::tip Context files vary by process

The example above shows the `product` process (fullest). Other flavors scaffold less:

- **minimal** — only `CLAUDE.md` and `.aibox-version`
- **managed** — adds `BACKLOG.md`, `DECISIONS.md`, work-instructions
- **research** — adds progress tracking, notes, research directory

See [Process Packages](../context/process-packages.md) for details.

:::

## The Generated aibox.toml

The scaffolded config file comes with commented documentation for every option:

```toml
# aibox.toml — project configuration for aibox.
# All generated files (.devcontainer/) derive from this file.
# Run `aibox sync` after editing to regenerate.
#
# Full documentation: https://projectious-work.github.io/aibox/cli/configuration/

[aibox]
version = "0.8.0"
# Container image flavor. Options: base, python, latex, typst, rust, node, go,
# python-latex, python-typst, rust-latex
image = "python"
# Work process flavor. Controls which context files are scaffolded.
# Options: minimal (CLAUDE.md only), managed (backlog + decisions),
#          research (progress + notes), product (full: PRD + backlog + standups)
process = "product"

[container]
name = "my-app"
hostname = "my-app"
# user = "root"  # Container user (default: root). Change to run as non-root.
# ports = ["8080:80"]  # Host:container port forwarding
# extra_packages = ["ripgrep", "fd-find"]  # Additional apt packages
# vscode_extensions = ["eamodio.gitlens"]  # Additional VS Code extensions
# post_create_command = "npm install"  # Run after container creation
#
# Extra volumes: [[container.extra_volumes]]
# source = "/host/path"
# target = "/container/path"
# read_only = false
#
# Extra environment: [container.environment]
# MY_VAR = "value"

# Addon bundles install additional tool sets into the container.
# Options: infrastructure, kubernetes, cloud-aws, cloud-gcp, cloud-azure,
#          docs-mkdocs, docs-zensical, docs-docusaurus, docs-starlight,
#          docs-mdbook, docs-hugo
[addons]
# bundles = ["infrastructure", "kubernetes"]

[context]
schema_version = "1.0.0"

# AI tool providers. Controls which AI CLI tools are installed and configured.
# Options: claude, aider, gemini
[ai]
providers = ["claude"]

# Color theme applied across Zellij, Vim, Yazi, and lazygit.
# Options: gruvbox-dark, catppuccin-mocha, catppuccin-latte, dracula,
#          tokyo-night, nord
[appearance]
theme = "gruvbox-dark"
# Starship prompt preset.
# Options: default, plain, minimal, nerd-font, pastel, bracketed
prompt = "default"

# Audio support for PulseAudio bridging (e.g., Claude Code voice).
# Requires host-side PulseAudio setup: run `aibox audio setup`
[audio]
enabled = false
# pulse_server = "tcp:host.docker.internal:4714"
```

After editing, regenerate devcontainer files:

```bash
aibox sync
```

## Build and Start

```bash
aibox sync     # Reconcile config, regenerate files, build image
aibox start    # Start the container and attach via Zellij
```

You land in a Zellij session with the **dev** layout: Yazi file browser (40%) and Vim editor (60%) side by side, plus tabs for lazygit and shell.

Two additional layouts are available: **focus** (one tool per tab, fullscreen) and **cowork** (Yazi+Vim left, Claude right). See [Layouts](../container/base-image.md#layouts).

The project root is mounted at `/workspace`. Persistent configuration lives in `.aibox-home/` on the host, mounted into the container automatically.

## VS Code Integration

The generated `devcontainer.json` works with VS Code's Dev Containers extension:

1. Open the project folder in VS Code
2. When prompted, click "Reopen in Container"
3. VS Code builds and starts the container automatically

Both `aibox start` (terminal) and VS Code can use the same container simultaneously.

## Next Steps

- [Explore the base image](../container/base-image.md)
- [Choose the right image addon](../container/addons.md)
- [Understand process packages](../context/process-packages.md)
- [Browse the Skills Library](../skills/index.md)
- [Full CLI reference](../reference/cli-commands.md)
