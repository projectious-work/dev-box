# New Project

This guide walks through creating a new project from scratch with dev-box.

## Initialize the Project

```bash
mkdir my-app && cd my-app
git init

dev-box init --name my-app --image python --process product
```

The `init` command accepts three options:

| Option | Default | Description |
|--------|---------|-------------|
| `--name` | Current directory name | Container and hostname |
| `--image` | `base` | Image flavor (`base`, `python`, `latex`, `typst`, `rust`, `node`, `go`, `python-latex`, `python-typst`, `rust-latex`) |
| `--process` | `product` | Work process flavor (`minimal`, `managed`, `research`, `product`) |

## What Gets Created

After running `init`, your project looks like this:

```
my-app/
├── dev-box.toml              # Single source of truth
├── .gitignore                 # Generated — language-specific blocks
├── .dev-box-home/             # Persistent config (git-ignored)
├── .devcontainer/
│   ├── Dockerfile             # Generated — references the chosen image
│   ├── docker-compose.yml     # Generated — volume mounts, env vars
│   └── devcontainer.json      # Generated — VS Code integration
└── context/                   # Scaffolded based on --process
    ├── OWNER.md
    ├── DECISIONS.md
    ├── BACKLOG.md
    ├── STANDUPS.md
    ├── PROJECTS.md
    ├── PRD.md
    ├── work-instructions/
    │   ├── GENERAL.md
    │   ├── DEVELOPMENT.md
    │   └── TEAM.md
    ├── project-notes/
    └── ideas/
```

!!! tip "Context files vary by process"
    The example above shows the `product` process. A `minimal` process creates only `CLAUDE.md`. See [Work Processes](../context/work-processes.md) for details on each flavor.

## Customizing dev-box.toml

Open `dev-box.toml` and adjust as needed:

```toml
[dev-box]
version = "0.7.0"
image = "python"
process = "product"

[container]
name = "my-app"
hostname = "my-app"
ports = ["8000:8000"]
extra_packages = ["postgresql-client"]
environment = { PYTHONDONTWRITEBYTECODE = "1" }

[context]
schema_version = "1.0.0"

[ai]

[audio]
enabled = true
pulse_server = "tcp:host.docker.internal:4714"
```

After editing, sync the devcontainer files:

```bash
dev-box sync
```

## Build and Start

```bash
# Build the container image
dev-box build

# Start the container and attach via zellij
dev-box start
```

On first `start`, dev-box:

1. Creates the `.dev-box-home/` directory for persistent configuration
2. Seeds default configs (vim, git, zellij) from built-in templates
3. Generates `.devcontainer/` files from `dev-box.toml`
4. Starts the container via docker/podman compose
5. Waits for the container to be ready
6. Attaches via zellij with the default layout

## Inside the Container

You land in a Zellij session with the **dev** layout: Yazi file browser (40%) and Vim editor (60%) side by side, plus tabs for Claude Code, git, and shell.

Two additional layouts are available: **focus** (one tool per tab, fullscreen) and **cowork** (Yazi+Vim left, Claude right). See [Base Image — Layouts](../container/base-image.md#layouts) for details.

The project root is mounted at `/workspace`. Your persistent configuration lives in `.dev-box-home/` on the host, mounted into the container at the appropriate paths.

## VS Code Integration

The generated `devcontainer.json` works with VS Code's Dev Containers extension:

1. Open the project folder in VS Code
2. When prompted, click "Reopen in Container"
3. VS Code builds and starts the container automatically
4. The integrated terminal opens zellij by default

You can also open a plain bash terminal from the VS Code terminal profile dropdown.

!!! note "Parallel usage"
    Both `dev-box start` (terminal) and VS Code can use the same container simultaneously. The container stays alive via `sleep infinity` and both tools exec into it.

## .gitignore

`dev-box init` automatically creates a comprehensive `.gitignore` with:

- **dev-box entries** -- `.dev-box-home/`, `.dev-box/`, and other dev-box internals
- **Language-specific blocks** -- based on your chosen image flavor (e.g., Python bytecode and virtualenv patterns for `python`, `target/` for `rust`)
- **Project-specific section** -- a clearly marked area for your own additions

The `.dev-box-home/` directory contains SSH keys and personal configuration -- it must never be committed. The `.devcontainer/` directory should be committed so team members get the same environment.

## Next Steps

- [Explore the base image](../container/base-image.md)
- [Choose the right image flavor](../container/flavors.md)
- [Understand work processes](../context/work-processes.md)
- [Full CLI reference](../cli/commands.md)
