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

aibox init --name my-app --process managed
```

The `init` command accepts these options:

| Option | Default | Description |
|--------|---------|-------------|
| `--name` | Current directory name | Container and hostname |
| `--base` | `debian` | Base image |
| `--process` | `managed` | processkit package(s): `minimal`, `managed`, `software`, `research`, `product` (can be repeated) |
| `--ai` | `claude` | AI providers (can be repeated): `claude`, `aider`, `gemini`, `mistral` |
| `--addons` | — | Addon names (can be repeated): `python`, `rust`, `node`, `go`, `latex`, etc. |
| `--theme` | `gruvbox-dark` | Color theme |

If you omit options, `aibox init` runs interactively and prompts for each value.

<div class="asciinema" data-cast="/screencasts/init-demo.cast" data-poster="npt:0" data-fit="width"></div>

## What Gets Created

`aibox init` lays down a **slim project skeleton** — devcontainer files,
config, and an empty `context/` directory. The actual content (skills,
processes, the canonical `AGENTS.md`) is then installed by **processkit** as
the last step of `init`.

```
my-app/
├── aibox.toml                  # Single source of truth (includes [processkit])
├── AGENTS.md                   # Canonical agent entry — rendered from processkit scaffolding
├── CLAUDE.md                   # Thin pointer to AGENTS.md (when [ai].providers includes "claude")
├── .gitignore                  # Generated with language-specific blocks
├── .aibox-version              # Tracks installed CLI version
├── .aibox-home/                # Persistent config (git-ignored)
├── .devcontainer/
│   ├── Dockerfile              # Generated from aibox.toml
│   ├── docker-compose.yml      # Generated — volume mounts, env vars
│   └── devcontainer.json       # Generated — VS Code integration
└── context/
    ├── skills/                 # Editable skill copies — installed by processkit
    │   ├── code-review/SKILL.md
    │   ├── backlog-context/SKILL.md
    │   └── ... (108 skills in processkit v0.5.1)
    ├── processes/              # release, code-review, feature-development, bug-fix
    ├── primitives/             # schemas, state-machines
    └── templates/
        └── processkit/
            └── v0.5.1/         # Immutable upstream snapshot, used by `aibox sync` for three-way diffs
```

:::warning Pin a processkit version

By default the `[processkit].version` is the sentinel `unset`, which **skips**
the processkit fetch. You will get the slim skeleton above but no skills,
processes, or `AGENTS.md` until you set a real tag:

```toml
[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.5.1"
```

Then re-run `aibox sync` to land the content.

:::

:::tip Single-file vs entity-sharded context tracks

Skills like `backlog-context`, `decisions-adr`, and `standup-context` operate
on **single files** (`context/BACKLOG.md`, `context/DECISIONS.md`,
`context/STANDUPS.md`). They do not ship a starter template — the agent
creates the file in place the first time it needs to write to it.

The entity-sharded counterparts (`workitem-management`, `decision-record`, …)
use per-item YAML files plus an MCP server. Both tracks ship in every
processkit release; pick the one that fits your project.

:::

## The Generated aibox.toml

The scaffolded config file comes with commented documentation for every option:

```toml
# aibox.toml — project configuration for aibox.
# All generated files (.devcontainer/) derive from this file.
# Run `aibox sync` after editing to regenerate.
#
# Full documentation: https://projectious-work.github.io/aibox/docs/reference/configuration

[aibox]
version = "0.16.0"
base    = "debian"

[container]
name     = "my-app"
hostname = "my-app"
# user = "aibox"  # Container user (default: aibox)

[context]
schema_version = "1.0.0"
# processkit packages: minimal, managed (default), software, research, product
packages = ["managed"]

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.5.1"   # Pin a real tag — "unset" skips fetching

# Addons install tool sets into the container.
# Run `aibox addon list` to see all available addons.
# [addons.python.tools]
# python = { version = "3.13" }
# uv     = { version = "0.7" }

# AI providers — controls which AI CLI tools are installed.
# Options: claude, aider, gemini, mistral
[ai]
providers = ["claude"]

[customization]
theme  = "gruvbox-dark"
prompt = "default"
layout = "dev"

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
- [Choose the right image addon](../addons/overview.md)
- [Understand process packages](../context/process-packages.md)
- [Skills (via processkit)](../skills/index.md)
- [Full CLI reference](../reference/cli-commands.md)
