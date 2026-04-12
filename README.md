# aibox

**One CLI for reproducible, AI-ready dev containers.**

`aibox` is a Rust CLI that scaffolds and manages containerized development environments for terminal-first AI workflows. It generates standard devcontainer files from `aibox.toml`, installs curated tool bundles through addons, and pins a `processkit` release into `context/` so agents have structured project context from day one.

It is designed for solo developers, small teams, and consultants who want a consistent setup: `aibox init` to a themed Zellij workspace with process content in place in minutes.

![aibox dev layout](docs-site/build/img/readme-dev-layout.gif)

## Table of Contents

- [Why aibox](#why-aibox)
- [What it does](#what-it-does)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Using aibox on an existing project](#using-aibox-on-an-existing-project)
- [How it works](#how-it-works)
- [Documentation](#documentation)
- [Development](#development)
- [Repository Structure](#repository-structure)
- [Contributing](#contributing)
- [License](#license)

## Why aibox

AI agents work better when the environment is reproducible, the tooling is already installed, and the project has structured context on disk instead of scattered across chat history.

Most tools cover only one part of that problem:

- devcontainer and remote-dev tools manage containers but not AI workflow structure
- AI CLIs and IDEs manage prompts and chats but not reproducible project environments
- `AGENTS.md` and `SKILL.md` provide conventions, but not installation and lifecycle management

`aibox` closes that gap with one config file, one CLI, and a clean split of responsibilities:

- **Containers:** generate `.devcontainer/Dockerfile`, `docker-compose.yml`, and `devcontainer.json` from `aibox.toml`
- **Tooling:** install language runtimes, AI CLIs, documentation frameworks, and infrastructure tools via addons
- **Context:** pin and install a `processkit` release into `context/`, including the canonical `AGENTS.md` template and process content
- **Terminal UX:** start directly into a themed Zellij workspace with layouts tuned for development and AI collaboration

## What it does

- Generates standard devcontainer files instead of inventing a proprietary runtime format
- Uses a single Debian-based base image with **25 composable addons** from `addons/`
- Supports AI tooling such as Claude Code, Aider, Gemini CLI, Mistral, OpenAI Codex CLI, Continue, and GitHub Copilot CLI
- Ships **6 Zellij layouts**: `dev`, `focus`, `cowork`, `cowork-swap`, `browse`, and `ai`
- Applies a consistent theme across Zellij, Vim, Yazi, lazygit, and Starship
- Installs editable `processkit` content under `context/` while keeping an immutable upstream snapshot for diffing under `context/templates/processkit/<version>/`
- Keeps project state local to the repository so a fresh clone plus `aibox sync` can reproduce the environment

## Installation

### Prerequisites

Install one supported container runtime on the host:

- **Podman** (recommended)
- **Docker**

`aibox` auto-detects the runtime. If both are installed, Podman takes priority.

### Install the CLI

```bash
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
```

The install script downloads the correct prebuilt binary for macOS or Linux and installs it to `~/.local/bin/` by default.

Verify the installation:

```bash
aibox --version
```

For more installation options, see the [installation guide](https://projectious-work.github.io/aibox/docs/getting-started/installation).

## Quick Start

### New project

```bash
mkdir my-project && cd my-project
git init

aibox init --name my-project --process managed --ai claude --addons python
aibox sync
aibox start
```

This gives you:

- `aibox.toml` as the single source of truth
- generated `.devcontainer/` files
- a persistent `.aibox-home/` runtime config directory
- a project `context/` directory populated from pinned `processkit` content
- a Zellij session mounted at `/workspace`

A minimal config looks like this:

```toml
[aibox]
version = "0.17.19"
base = "debian"

[container]
name = "my-project"
hostname = "my-project"

[context]
schema_version = "1.0.0"
packages = ["managed"]

[processkit]
source = "https://github.com/projectious-work/processkit.git"
version = "v0.8.0"

[ai]
providers = ["claude"]

[customization]
theme = "gruvbox-dark"
layout = "dev"

[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
```

### Daily workflow

```bash
aibox sync                # apply config changes and rebuild if needed
aibox start               # start or attach to the container
aibox start --layout ai   # override the default layout for one session
aibox doctor              # validate project structure and environment
```

## Using aibox on an existing project

You can adopt `aibox` incrementally.

1. Add or generate `aibox.toml` in the project root.
2. Pin a real `processkit` version in `[processkit].version`.
3. Run `aibox sync` to generate `.devcontainer/` files and install process content.
4. Move any custom environment variables, mounts, or post-create steps into `aibox.toml`.

If your project already has hand-written `.devcontainer/` files, back them up first. `aibox sync` treats generated devcontainer files as managed output.

See the [existing-project guide](https://projectious-work.github.io/aibox/docs/getting-started/existing-project) for migration details.

## How it works

### 1. `aibox.toml` drives everything

`aibox.toml` is the project contract. It declares the base image, container settings, addons, AI providers, theme, layout, and the pinned `processkit` source/version.

### 2. `aibox sync` reconciles managed state

`aibox sync` is the command that applies configuration changes. It:

- seeds `.aibox-home/`
- regenerates `.devcontainer/` files
- refreshes pinned `processkit` content under `context/`
- updates `aibox.lock`
- builds the container image unless `--no-build` is used

### 3. `aibox start` launches the workspace

`aibox start` creates or starts the container and attaches through Zellij. The default experience is terminal-first, with built-in layouts for editing, browsing, git work, shells, and AI collaboration.

### 4. `processkit` owns the process layer

As of `aibox` v0.16.0, process content is intentionally separated:

- **aibox owns:** container generation, addon resolution, install/sync/migrate machinery, and the slim project skeleton
- **processkit owns:** skills, processes, schemas, state machines, and the canonical `AGENTS.md` template

That boundary keeps container logic and process content independently versioned.

## Documentation

Full documentation lives at [projectious-work.github.io/aibox](https://projectious-work.github.io/aibox/).

Good starting points:

- [Installation](https://projectious-work.github.io/aibox/docs/getting-started/installation)
- [New Project](https://projectious-work.github.io/aibox/docs/getting-started/new-project)
- [Existing Project](https://projectious-work.github.io/aibox/docs/getting-started/existing-project)
- [Configuration Reference](https://projectious-work.github.io/aibox/docs/reference/configuration)
- [CLI Commands](https://projectious-work.github.io/aibox/docs/reference/cli-commands)
- [Addons Overview](https://projectious-work.github.io/aibox/docs/addons/overview)
- [Layouts](https://projectious-work.github.io/aibox/docs/customization/layouts)
- [Context Overview](https://projectious-work.github.io/aibox/docs/context/overview)

## Development

This repository is developed inside its own devcontainer.

```bash
cd cli && cargo build
cd cli && cargo test
cd cli && cargo clippy --all-targets -- -D warnings
cd cli && cargo fmt -- --check
```

For E2E tier 2 tests:

```bash
cd cli && cargo test --features e2e
```

Release quality expectations in this repo are strict:

- zero Clippy warnings
- all tests passing
- `cargo audit` clean before release
- use `./scripts/maintain.sh release <version>` for releases

## Repository Structure

| Path | Purpose |
| --- | --- |
| `cli/` | Rust CLI source for the `aibox` binary |
| `addons/` | YAML addon definitions for runtimes, tools, docs frameworks, and AI CLIs |
| `images/` | Base and published image recipes for downstream projects |
| `docs-site/` | Docusaurus documentation source |
| `context/` | This repository's own processkit-managed project context |
| `scripts/` | Release, install, and maintenance tooling |
| `.devcontainer/` | This repository's own development container |

## Contributing

Direct commits to `main` are the norm in this repository. Before contributing, read [AGENTS.md](AGENTS.md) and the docs under [`docs-site/docs/contributing/`](docs-site/docs/contributing/).

At minimum:

- keep changes focused
- update docs when behavior changes
- run build, test, clippy, and format checks before committing
- do not hardcode processkit vocabulary in production Rust code; use `cli/src/processkit_vocab.rs`

Issue tracker: <https://github.com/projectious-work/aibox/issues>

## License

MIT
