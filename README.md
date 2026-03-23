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

![dev-box dev layout](docs/assets/readme-dev-layout.gif)

## What dev-box manages

**Container images** — 10 pre-built images (base, python, latex, typst, rust, node, go, and combinations) on Debian Trixie Slim. Each includes Zellij, Yazi, Vim, Git, lazygit, GitHub CLI, ripgrep, fd, bat, fzf, delta, Starship, and configurable AI assistants.

**Project configuration** — A single `dev-box.toml` drives everything. The CLI generates `.devcontainer/` files from this config. Change the config, run `dev-box sync`, done.

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

**AI context structure** — Structured context files (DECISIONS.md, BACKLOG.md, OWNER.md) give AI agents project knowledge. Four process flavors (minimal, managed, research, product) scale from quick scripts to full product development.

**83 curated agent skills** — Instructions following the open [SKILL.md standard](https://agentskills.io/specification) across 14 categories: from Kubernetes and SQL patterns to RAG engineering and prompt design. Progressive disclosure keeps agent context lean.

**Addon packages** — Selectable tool bundles (infrastructure, kubernetes, cloud-aws/gcp/azure) added via config.

**6 color themes** — Gruvbox Dark, Catppuccin Mocha/Latte, Dracula, Tokyo Night, Nord — applied consistently across Zellij, Vim, Yazi, lazygit, and Starship.

## Why containers?

- **Security** — AI agents run in an isolated environment, not on your host
- **Reproducibility** — Every team member gets the same tools and versions
- **Control** — You define the environment declaratively; changes go through config, not manual installs
- **Zero lock-in** — dev-box generates standard devcontainer files; stop using the CLI any time

## CLI commands

```
dev-box init       Create new project (interactive or with flags)
dev-box sync       Apply config changes (themes, AI providers, addons)
dev-box build      Build container image
dev-box start      Start and attach via Zellij
dev-box stop       Stop container
dev-box remove     Stop and remove container
dev-box attach     Reattach to running container
dev-box status     Show container state
dev-box doctor     Validate project structure
dev-box update     Check for and apply CLI updates
dev-box env        Manage named environments
dev-box backup     Snapshot dev-box files
dev-box reset      Remove all dev-box files
dev-box audit      Run security checks (cargo audit, pip-audit, trivy)
dev-box audio      Audio diagnostics for voice features
```

## Documentation

Full documentation: [projectious-work.github.io/dev-box](https://projectious-work.github.io/dev-box/)

## Development

This project is developed inside its own dev-container.

```bash
cd cli && cargo build                    # Build CLI
cd cli && cargo test                     # Run tests (151 tests)
cd cli && cargo clippy -- -D warnings    # Lint
```

## License

MIT
