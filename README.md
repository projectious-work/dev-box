# aibox

**Containerized development environments for AI-assisted work.**

AI-assisted development works best on the console and inside containers — for security, reproducibility, and control. But setting up a proper containerized environment with the right tools, AI integrations, structured context, and work processes is tedious boilerplate that drifts across projects.

aibox eliminates that boilerplate. One config file, one CLI, one command to go from empty directory to a fully equipped development environment with terminal multiplexer, file manager, editor, AI assistants, and curated agent skills — all inside a container.

## The gap aibox fills

The market is split: **environment tools** (DevPod, Codespaces, Coder) know nothing about AI context. **AI tools** (Cursor, Windsurf, Claude Code) know nothing about environment management. **Context standards** (AGENTS.md, SKILL.md) are files without a management layer.

aibox bridges this gap — it unifies environment definition, AI context structure, and terminal-first tooling into a single coherent system:

- **Environment tools** give you a container. aibox gives you a container that knows your AI providers, skills, work processes, and theming.
- **AI IDEs** lock you into a GUI. aibox works with any terminal-based AI tool (Claude Code, Aider, Codex CLI, Gemini CLI) without IDE lock-in.
- **Curated quality over marketplace chaos.** Community skill hubs have 97K+ entries, but nearly half are duplicates. aibox ships 83 vetted skills with reference files — tested, composable, and safe.

## How it works

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash

# Create a project
mkdir my-project && cd my-project
aibox init --name my-project --image python --process managed

# Sync config and build
aibox sync

# Start and attach
aibox start
```

After `aibox start`, you're inside a Zellij terminal session with Yazi file browser, Vim editor, lazygit, and a shell — all themed consistently and ready to work.

![aibox dev layout](docs/assets/readme-dev-layout.gif)

## What aibox manages

**Container images** — One minimal base image (Debian Trixie Slim) with composable addons. Includes Zellij, Yazi, Vim, Git, lazygit, GitHub CLI, ripgrep, fd, bat, fzf, delta, and Starship. AI assistants and language runtimes are installed per-project via addons.

**Project configuration** — A single `aibox.toml` drives everything. The CLI generates `.devcontainer/` files from this config. Change the config, run `aibox sync`, done.

```toml
[aibox]
version = "0.9.0"

[container]
name = "my-project"

[ai]
providers = ["claude", "aider"]

[process]
packages = ["managed"]

[appearance]
theme = "catppuccin-mocha"

[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
```

**AI context structure** — Structured context files (DECISIONS.md, BACKLOG.md, OWNER.md) give AI agents project knowledge. 13 composable process packages and 4 convenience presets scale from quick scripts to full product development.

**84 curated agent skills** — Instructions following the open [SKILL.md standard](https://agentskills.io/specification) across 14 categories: from Kubernetes and SQL patterns to RAG engineering and prompt design. Managed via `aibox skill {list,add,remove,info}`.

**21 composable addons** — Language runtimes (Python, Rust, Node, Go), tool bundles (infrastructure, kubernetes, cloud providers), documentation frameworks, and AI coding agents. Managed via `aibox addon {list,add,remove,info}`.

**6 color themes** — Gruvbox Dark, Catppuccin Mocha/Latte, Dracula, Tokyo Night, Nord — applied consistently across Zellij, Vim, Yazi, lazygit, and Starship.

## Why containers?

- **Security** — AI agents run in an isolated environment, not on your host
- **Reproducibility** — Every team member gets the same tools and versions
- **Control** — You define the environment declaratively; changes go through config, not manual installs
- **Zero lock-in** — aibox generates standard devcontainer files; stop using the CLI any time

## CLI commands

```
aibox init       Create new project (interactive or with flags)
aibox sync       Reconcile config, regenerate files, build image
aibox start      Start container and attach via Zellij
aibox stop       Stop container
aibox remove     Stop and remove container
aibox status     Show container state
aibox addon      Manage addons (list, add, remove, info)
aibox skill      Manage skills (list, add, remove, info)
aibox doctor     Validate project structure
aibox update     Check for and apply CLI updates
aibox env        Manage named environments
aibox backup     Snapshot aibox files
aibox reset      Remove all aibox files
aibox audit      Run security checks (cargo audit, pip-audit, trivy)
aibox audio      Audio diagnostics for voice features
```

## Documentation

Full documentation: [projectious-work.github.io/aibox](https://projectious-work.github.io/aibox/)

## Development

This project is developed inside its own dev-container.

```bash
cd cli && cargo build                    # Build CLI
cd cli && cargo test                     # Run tests (241+ tests)
cd cli && cargo clippy -- -D warnings    # Lint
```

## License

MIT
