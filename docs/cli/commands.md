# CLI Commands

Complete reference for all `dev-box` commands.

## Global Options

These options apply to all commands:

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `--config <PATH>` | -- | `./dev-box.toml` | Path to configuration file |
| `--log-level <LEVEL>` | `DEV_BOX_LOG_LEVEL` | `info` | Log verbosity: `trace`, `debug`, `info`, `warn`, `error` |

---

## dev-box init

Initialize a new project with `dev-box.toml` and generated devcontainer files.

### Usage

```bash
dev-box init [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--name <NAME>` | Current directory name | Project and container name |
| `--image <FLAVOR>` | `base` | Image flavor: `base`, `python`, `latex`, `typst`, `rust`, `node`, `go`, `python-latex`, `python-typst`, `rust-latex` |
| `--process <FLAVOR>` | `product` | Work process flavor: `minimal`, `managed`, `research`, `product` |
| `--ai <PROVIDER>` | `claude` | AI provider(s) to configure: `claude`, `aider`, `gemini` (can be specified multiple times) |
| `--theme <THEME>` | `gruvbox-dark` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord` |
| `--user <USER>` | `root` | Container user |
| `--addons <BUNDLE>` | -- | Addon bundles to install (can be specified multiple times): `infrastructure`, `kubernetes`, `cloud-aws`, `cloud-gcp`, `cloud-azure`, `docs-mkdocs`, `docs-zensical`, `docs-docusaurus`, `docs-starlight`, `docs-mdbook`, `docs-hugo` |

### What It Does

1. Creates `dev-box.toml` with the specified settings
2. Creates `.dev-box-home/` directory with default configuration files
3. Generates `.devcontainer/Dockerfile`, `docker-compose.yml`, and `devcontainer.json`
4. Scaffolds context files based on the chosen process flavor
5. Updates `.gitignore` with required entries (`.dev-box-home/`, `.devcontainer/`, etc.)

### Examples

```bash
# Basic initialization (uses directory name, base image, product process)
dev-box init

# Specify all options
dev-box init --name my-api --image python --process managed

# Rust project with minimal context
dev-box init --image rust --process minimal

# Specify a non-root user
dev-box init --name my-api --image python --user devuser

# Configure multiple AI providers
dev-box init --ai claude --ai aider

# All three providers
dev-box init --ai claude --ai aider --ai gemini

# Choose a color theme
dev-box init --name my-app --image python --theme catppuccin-mocha

# Include addon bundles
dev-box init --name my-app --image python --addons infrastructure --addons kubernetes
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | `dev-box.toml` already exists, or invalid option value |

### Interactive Mode

When `--name`, `--image`, or `--process` flags are omitted and the terminal is interactive, `dev-box init` prompts for each missing value. This lets you explore the available options without memorizing flag values.

In non-interactive environments (scripts, CI pipelines), omitted flags silently use defaults: the current directory name for `--name`, `base` for `--image`, and `product` for `--process`.

!!! warning "Will not overwrite"
    If `dev-box.toml` already exists, `init` exits with an error. Delete the file first or edit it directly.

---

## dev-box sync

Reconcile project state with `dev-box.toml`. The primary command for applying config changes.

### Usage

```bash
dev-box sync
```

### What It Does

1. **Force-updates theme-dependent config files** in `.dev-box-home/`:
   - `.vim/vimrc` (colorscheme and background)
   - `.config/zellij/config.kdl` (theme name)
   - `.config/zellij/themes/<theme>.kdl` (theme colors)
   - `.config/lazygit/config.yml` (theme colors)
   - `.config/yazi/theme.toml` (theme colors)

2. **Regenerates `.devcontainer/` files** from `dev-box.toml`:
   - `.devcontainer/Dockerfile`
   - `.devcontainer/docker-compose.yml`
   - `.devcontainer/devcontainer.json`

Only files whose content has actually changed are written. Reports what was updated.

### Examples

```bash
# Change theme, then apply
vim dev-box.toml
dev-box sync

# After any config change
dev-box sync
dev-box build
dev-box start
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No `dev-box.toml` found, or invalid config |

!!! note "`generate` is an alias"
    `dev-box generate` still works as an alias for `sync`. New projects should use `sync`.

---

## dev-box build

Build the container image.

### Usage

```bash
dev-box build [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--no-cache` | Build without using the layer cache |

### What It Does

1. Loads and validates `dev-box.toml`
2. Runs `sync` to ensure devcontainer files are current
3. Runs `docker compose build` (or `podman compose build`)

### Examples

```bash
# Standard build (uses cache)
dev-box build

# Full rebuild from scratch
dev-box build --no-cache
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Build succeeded |
| 1 | Config error, runtime not found, or build failure |

---

## dev-box start

Start the container and attach via Zellij.

### Usage

```bash
dev-box start [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--layout <LAYOUT>` | `dev` | Zellij layout: `dev`, `focus`, or `cowork` |

### What It Does

This is the primary command for daily use. It handles the full lifecycle:

1. Seeds `.dev-box-home/` directory with default configs (first run only)
2. Generates devcontainer files from `dev-box.toml`
3. Checks container state:
   - **Running:** Skips to attach
   - **Stopped:** Starts the existing container
   - **Missing:** Builds the image (if needed) and creates the container
4. Waits for the container to be ready (up to 7.5 seconds)
5. Attaches via `zellij --layout <LAYOUT>`

### Available Layouts

| Layout | Description |
|--------|-------------|
| `dev` | Yazi (40%) + Vim (60%) side by side (default) |
| `focus` | One tool per tab, fullscreen |
| `cowork` | Yazi+Vim left, Claude right — side-by-side AI collaboration |

All layouts include shared tabs for **git** (lazygit) and **shell** (extra bash).

### Examples

```bash
# Start working with default layout
dev-box start

# Start with focus layout (one tool per tab)
dev-box start --layout focus

# Start with cowork layout (side-by-side with Claude)
dev-box start --layout cowork
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (after Zellij session ends) |
| 1 | Config error, runtime not found, container failed to start |

---

## dev-box stop

Stop the running container.

### Usage

```bash
dev-box stop
```

### What It Does

Stops the container via `docker compose stop` (or `podman compose stop`). All data in `.dev-box-home/` and the workspace is preserved. The container can be restarted with `dev-box start`.

### Examples

```bash
dev-box stop
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Container stopped, or was already stopped/missing |
| 1 | Config error or runtime not found |

---

## dev-box remove

Stop and remove the container entirely.

### Usage

```bash
dev-box remove
```

### What It Does

Unlike `stop`, this removes the container entirely (like `docker rm`). Use before switching to VS Code or when you want a clean slate. The container is stopped first if running, then removed.

All data in `.dev-box-home/` and the workspace is preserved -- only the container itself is deleted.

### Aliases

`dev-box rm` is a shorthand alias for `dev-box remove`.

### Examples

```bash
dev-box remove
dev-box rm
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Container removed, or was already missing |
| 1 | Config error or runtime not found |

---

## dev-box attach

Attach to a running container via Zellij.

### Usage

```bash
dev-box attach [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--layout <LAYOUT>` | `dev` | Zellij layout: `dev`, `focus`, or `cowork` |

### What It Does

Execs into the container and launches `zellij --layout <LAYOUT>`. Unlike `start`, this command does not create or start the container -- it must already be running.

See [dev-box start](#dev-box-start) for available layouts.

### Examples

```bash
# Attach from a second terminal
dev-box attach

# Attach with focus layout
dev-box attach --layout focus
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (after Zellij session ends) |
| 1 | Container is not running |

---

## dev-box status

Show the current container state.

### Usage

```bash
dev-box status
```

### What It Does

Inspects the container and reports one of three states:

- **Running** -- container is active
- **Stopped** -- container exists but is not running
- **Missing** -- no container found with the configured name

### Examples

```bash
dev-box status
```

Output:

```
 ✓ Container 'my-app' is running
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Status retrieved successfully (regardless of container state) |
| 1 | Config error or runtime not found |

---

## dev-box doctor

Validate project structure and run diagnostics.

### Usage

```bash
dev-box doctor
```

### What It Does

1. Validates `dev-box.toml` (syntax, field values, semver versions)
2. Detects the container runtime
3. Checks for `.dev-box-home/` directory (suggests renaming `.root/` if found for backward compatibility)
4. Checks for `.devcontainer/` directory
5. Validates `.gitignore` contains required entries (`.dev-box-home/`, `.devcontainer/`)
6. Validates mount source paths exist on the host
7. Reports image flavor, process flavor, and container name
8. Compares schema versions for migration needs

### Examples

```bash
dev-box doctor
```

Output:

```
==> Running diagnostics...
 ✓ Config version: 0.7.0
 ✓ Image: python
 ✓ Process: product
 ✓ Container name: my-app
 ✓ Container runtime: podman
 ✓ .dev-box-home/ directory exists
 ✓ .devcontainer/ directory exists
 ✓ .gitignore contains required entries
 ✓ Mount source paths exist
 ✓ Diagnostics complete
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | Config error or critical issue detected |

---

## dev-box completions

Generate shell completion scripts for bash, zsh, fish, powershell, or elvish.

### Usage

```bash
dev-box completions <SHELL>
```

Where `<SHELL>` is one of: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

### Examples

```bash
dev-box completions bash
dev-box completions zsh
dev-box completions fish
```

### Setup

Add to your shell profile for persistent completions:

**Bash** (`~/.bashrc`):

```bash
eval "$(dev-box completions bash)"
```

**Zsh** (`~/.zshrc`):

```bash
eval "$(dev-box completions zsh)"
```

**Fish** (`~/.config/fish/config.fish`):

```bash
dev-box completions fish | source
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Unknown shell name |

---

## dev-box env

Manage named environments for switching between configurations within a single project.

Environments save `dev-box.toml`, `CLAUDE.md`, and `context/` (excluding `context/shared/`) to `.dev-box-env/<name>/`. Files in `context/shared/` are shared across all environments.

### Subcommands

#### dev-box env create

Save the current project state as a named environment.

```bash
dev-box env create <name>
```

Creates a snapshot of `dev-box.toml`, `CLAUDE.md`, and `context/` (excluding `context/shared/`) in `.dev-box-env/<name>/`.

#### dev-box env switch

Switch to a different environment.

```bash
dev-box env switch <name> [--yes]
```

1. Stops any running container
2. Saves the current environment
3. Restores the target environment
4. Regenerates `.devcontainer/` files

Requires confirmation (type `switch`). Use `--yes` to skip.

#### dev-box env list

List available environments and show which is current.

```bash
dev-box env list
```

#### dev-box env delete

Delete a saved environment.

```bash
dev-box env delete <name> [--yes]
```

Requires confirmation (type the environment name). Use `--yes` to skip.

#### dev-box env status

Show current environment name and config summary.

```bash
dev-box env status
```

### Examples

```bash
# Create two environments from different configurations
dev-box env create research
# ... edit dev-box.toml to change image/process ...
dev-box env create product

# Switch between them
dev-box env switch research
dev-box env switch product

# List environments
dev-box env list

# Delete an environment
dev-box env delete research --yes
```

### Shared Files

Files in `context/shared/` are **not** copied during environment switches — they stay in place and are shared across all environments. By default, `OWNER.md` is placed in `context/shared/`. Move any file there to share it across environments.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Environment not found, name invalid, or config error |

---

## dev-box backup

Back up dev-box files to a timestamped directory.

### Usage

```bash
dev-box backup [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--output-dir <DIR>` | `.dev-box-backup/` | Output directory for backup |
| `--dry-run` | -- | Preview what would be backed up without copying |

### What It Does

Copies all dev-box managed files to a timestamped subdirectory:

- `dev-box.toml`
- `.devcontainer/`
- `.dev-box-home/`
- `.dev-box-version`
- `context/`
- `CLAUDE.md`
- `.gitignore`

The backup directory is named `dev-box-<version>-backup-<date>-<time>` (e.g., `dev-box-0.7.0-backup-2026-03-22-1430`).

### Examples

```bash
# Back up current state
dev-box backup

# Preview without copying
dev-box backup --dry-run

# Back up to a custom directory
dev-box backup --output-dir /tmp/my-backup
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Config error or no files found |

---

## dev-box reset

Remove all dev-box files and reset the project to its pre-init state.

!!! danger "Danger zone"
    This command deletes dev-box.toml, .devcontainer/, .dev-box-home/, context/, CLAUDE.md, and .dev-box-version. By default a backup is created first. `.gitignore` is backed up but **not** deleted.

### Usage

```bash
dev-box reset [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--no-backup` | Skip backup — permanently delete without saving |
| `--dry-run` | Preview what would happen without modifying anything |
| `--yes` | Skip confirmation prompt |

### What It Does

1. Displays a table of files showing backup and deletion status
2. Prompts for confirmation (type `reset` or `DELETE` with `--no-backup`)
3. Stops any running container
4. Creates a backup (unless `--no-backup`)
5. Deletes all dev-box files except `.gitignore`

### Examples

```bash
# Reset with backup (safe default)
dev-box reset

# Preview what would happen
dev-box reset --dry-run

# Delete without backup (requires typing "DELETE" to confirm)
dev-box reset --no-backup

# Non-interactive reset with backup
dev-box reset --yes
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success or user aborted |
| 1 | Config error or deletion failed |

---

## dev-box update

Check for or apply updates.

### Usage

```bash
dev-box update [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--check` | Only check for updates, do not apply |
| `--dry-run` | Preview what would change without writing files |

### What It Does

Checks the current version against the latest available release. Without flags, upgrades the image version in `dev-box.toml` and regenerates container files.

- **`--check`** — Queries GHCR for the latest image tag and GitHub Releases for the latest CLI version. Reports whether updates are available without changing anything.
- **No flags** — Fetches the latest image version, updates `version` in `dev-box.toml`, regenerates `.devcontainer/` files, and updates `.dev-box-version`. You still need to rebuild the container to apply changes.
- **`--dry-run`** — Shows what would change without writing any files.

### Examples

```bash
# Check for updates (read-only)
dev-box update --check

# Preview upgrade without applying
dev-box update --dry-run

# Upgrade image version and regenerate files
dev-box update
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Config error |

---

## dev-box audio

Host-side audio diagnostics and setup for PulseAudio over TCP.

### Subcommands

#### dev-box audio check

Check if the host is correctly configured for container audio.

```bash
dev-box audio check [--port <PORT>]
```

Runs diagnostics:

- PulseAudio installation and version
- Daemon status
- TCP module (`module-native-protocol-tcp`) loaded on the expected port
- Persistent configuration in `default.pa`
- Port listening
- macOS launchd agent status
- TCP connectivity test

#### dev-box audio setup

Automatically install and configure PulseAudio on the host.

```bash
dev-box audio setup [--port <PORT>]
```

On macOS:

1. Installs PulseAudio via Homebrew (if not present)
2. Configures `~/.config/pulse/default.pa` with the TCP module
3. Creates a launchd agent (`com.devbox.pulseaudio`) with `KeepAlive` for auto-start
4. Loads the service immediately
5. Runs `audio check` to verify

On Linux, prints manual setup instructions with the correct `auth-ip-acl` settings.

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--port <PORT>` | `4714` | PulseAudio TCP port |

### Examples

```bash
# Diagnose audio issues
dev-box audio check

# Full automated setup (macOS)
dev-box audio setup

# Use a custom port
dev-box audio setup --port 4715
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (check passed or setup completed) |
| 1 | Setup failed (e.g., brew install failed) |
