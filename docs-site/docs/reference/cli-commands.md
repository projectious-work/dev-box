---
sidebar_position: 1
title: CLI Commands
---

# CLI Commands

Complete reference for all `aibox` commands.

## Global Options

These options apply to all commands:

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `--config <PATH>` | -- | `./aibox.toml` | Path to configuration file |
| `--log-level <LEVEL>` | `AIBOX_LOG_LEVEL` | `info` | Log verbosity: `trace`, `debug`, `info`, `warn`, `error` |

---

## aibox init

Initialize a new project with `aibox.toml` and generated devcontainer files.

### Usage

```bash
aibox init [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--name <NAME>` | Current directory name | Project and container name |
| `--base <BASE>` | `debian` | Base image: `debian` |
| `--process <PACKAGES>` | `core` | Process packages (space-separated): package names or presets |
| `--ai <PROVIDER>` | `claude` | AI provider(s): `claude`, `aider`, `gemini`, `mistral` (can be repeated) |
| `--theme <THEME>` | `gruvbox-dark` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord` |
| `--user <USER>` | `aibox` | Container user |
| `--addons <NAME>` | -- | Addon names (can be repeated): `python`, `rust`, `node`, `go`, `latex`, `typst`, `infrastructure`, `kubernetes`, `cloud-aws`, etc. |

### What It Does

1. Creates `aibox.toml` with the specified settings
2. Creates `.aibox-home/` directory with default configuration files
3. Generates `.devcontainer/Dockerfile`, `docker-compose.yml`, and `devcontainer.json`
4. Scaffolds context files based on the chosen process flavor
5. Updates `.gitignore` with required entries (`.aibox-home/`, `.devcontainer/`, etc.)

### Examples

```bash
# Basic initialization (uses directory name, base image, product process)
aibox init

# Specify all options
aibox init --name my-api --image python --process managed

# Rust project with minimal context
aibox init --image rust --process minimal

# Specify a non-root user
aibox init --name my-api --image python --user devuser

# Configure multiple AI providers
aibox init --ai claude --ai aider

# All three providers
aibox init --ai claude --ai aider --ai gemini

# Choose a color theme
aibox init --name my-app --image python --theme catppuccin-mocha

# Include addon bundles
aibox init --name my-app --image python --addons infrastructure --addons kubernetes
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | `aibox.toml` already exists, or invalid option value |

### Interactive Mode

When `--name`, `--image`, or `--process` flags are omitted and the terminal is interactive, `aibox init` prompts for each missing value. This lets you explore the available options without memorizing flag values.

In non-interactive environments (scripts, CI pipelines), omitted flags silently use defaults: the current directory name for `--name`, `base` for `--image`, and `product` for `--process`.

!!! warning "Will not overwrite"
    If `aibox.toml` already exists, `init` exits with an error. Delete the file first or edit it directly.

---

## aibox sync

Reconcile project state with `aibox.toml`. The single command for applying any config change.

### Usage

```bash
aibox sync [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--no-cache` | Build the container image without using the layer cache |

### What It Does

1. **Checks for version migrations** (schema updates between versions)
2. **Force-updates theme-dependent config files** in `.aibox-home/`
3. **Seeds `.aibox-home/`** directory with default configs (if missing)
4. **Regenerates `.devcontainer/` files** from `aibox.toml`
5. **Reconciles skills** — deploys missing skills, reports orphans
6. **Generates AIBOX.md** — universal agent baseline document
7. **Builds the container image** via `docker compose build` (skipped gracefully if no runtime)

Only files whose content has actually changed are written. Reports what was updated.

### Examples

```bash
# After any config change
aibox sync

# Force full image rebuild
aibox sync --no-cache
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No `aibox.toml` found, or invalid config |

!!! note "`generate` is an alias"
    `aibox generate` still works as an alias for `sync`. New projects should use `sync`.

---

## aibox start

Start the container and attach via Zellij.

### Usage

```bash
aibox start [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--layout <LAYOUT>` | `dev` | Zellij layout: `dev`, `focus`, or `cowork` |

### What It Does

This is the primary command for daily use. It handles the full lifecycle:

1. Seeds `.aibox-home/` directory with default configs (first run only)
2. Generates devcontainer files from `aibox.toml`
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
aibox start

# Start with focus layout (one tool per tab)
aibox start --layout focus

# Start with cowork layout (side-by-side with Claude)
aibox start --layout cowork
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (after Zellij session ends) |
| 1 | Config error, runtime not found, container failed to start |

---

## aibox stop

Stop the running container.

### Usage

```bash
aibox stop
```

### What It Does

Stops the container via `docker compose stop` (or `podman compose stop`). All data in `.aibox-home/` and the workspace is preserved. The container can be restarted with `aibox start`.

### Examples

```bash
aibox stop
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Container stopped, or was already stopped/missing |
| 1 | Config error or runtime not found |

---

## aibox remove

Stop and remove the container entirely.

### Usage

```bash
aibox remove
```

### What It Does

Unlike `stop`, this removes the container entirely (like `docker rm`). Use before switching to VS Code or when you want a clean slate. The container is stopped first if running, then removed.

All data in `.aibox-home/` and the workspace is preserved -- only the container itself is deleted.

### Aliases

`aibox rm` is a shorthand alias for `aibox remove`.

### Examples

```bash
aibox remove
aibox rm
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Container removed, or was already missing |
| 1 | Config error or runtime not found |

---

## aibox status

Show the current container state.

### Usage

```bash
aibox status
```

### What It Does

Inspects the container and reports one of three states:

- **Running** -- container is active
- **Stopped** -- container exists but is not running
- **Missing** -- no container found with the configured name

### Examples

```bash
aibox status
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

## aibox doctor

Validate project structure and run diagnostics.

### Usage

```bash
aibox doctor
```

### What It Does

1. Validates `aibox.toml` (syntax, field values, semver versions)
2. Detects the container runtime
3. Checks for `.aibox-home/` directory (suggests renaming `.root/` if found for backward compatibility)
4. Checks for `.devcontainer/` directory
5. Validates `.gitignore` contains required entries (`.aibox-home/`, `.devcontainer/`)
6. Validates mount source paths exist on the host
7. Reports image flavor, process flavor, and container name
8. Compares schema versions for migration needs

### Examples

```bash
aibox doctor
```

Output:

```
==> Running diagnostics...
 ✓ Config version: 0.8.0
 ✓ Image: python
 ✓ Process: product
 ✓ Container name: my-app
 ✓ Container runtime: podman
 ✓ .aibox-home/ directory exists
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

## aibox completions

Generate shell completion scripts for bash, zsh, fish, powershell, or elvish.

### Usage

```bash
aibox completions <SHELL>
```

Where `<SHELL>` is one of: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

### Examples

```bash
aibox completions bash
aibox completions zsh
aibox completions fish
```

### Setup

Add to your shell profile for persistent completions:

**Bash** (`~/.bashrc`):

```bash
eval "$(aibox completions bash)"
```

**Zsh** (`~/.zshrc`):

```bash
eval "$(aibox completions zsh)"
```

**Fish** (`~/.config/fish/config.fish`):

```bash
aibox completions fish | source
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Unknown shell name |

---

## aibox env

Manage named environments for switching between configurations within a single project.

Environments save `aibox.toml`, `CLAUDE.md`, and `context/` (excluding `context/shared/`) to `.aibox-env/<name>/`. Files in `context/shared/` are shared across all environments.

### Subcommands

#### aibox env create

Save the current project state as a named environment.

```bash
aibox env create <name>
```

Creates a snapshot of `aibox.toml`, `CLAUDE.md`, and `context/` (excluding `context/shared/`) in `.aibox-env/<name>/`.

#### aibox env switch

Switch to a different environment.

```bash
aibox env switch <name> [--yes]
```

1. Stops any running container
2. Saves the current environment
3. Restores the target environment
4. Regenerates `.devcontainer/` files

Requires confirmation (type `switch`). Use `--yes` to skip.

#### aibox env list

List available environments and show which is current.

```bash
aibox env list
```

#### aibox env delete

Delete a saved environment.

```bash
aibox env delete <name> [--yes]
```

Requires confirmation (type the environment name). Use `--yes` to skip.

#### aibox env status

Show current environment name and config summary.

```bash
aibox env status
```

### Examples

```bash
# Create two environments from different configurations
aibox env create research
# ... edit aibox.toml to change image/process ...
aibox env create product

# Switch between them
aibox env switch research
aibox env switch product

# List environments
aibox env list

# Delete an environment
aibox env delete research --yes
```

### Shared Files

Files in `context/shared/` are **not** copied during environment switches — they stay in place and are shared across all environments. By default, `OWNER.md` is placed in `context/shared/`. Move any file there to share it across environments.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Environment not found, name invalid, or config error |

---

## aibox backup

Back up aibox files to a timestamped directory.

### Usage

```bash
aibox backup [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--output-dir <DIR>` | `.aibox-backup/` | Output directory for backup |
| `--dry-run` | -- | Preview what would be backed up without copying |

### What It Does

Copies all aibox managed files to a timestamped subdirectory:

- `aibox.toml`
- `.devcontainer/`
- `.aibox-home/`
- `.aibox-version`
- `context/`
- `CLAUDE.md`
- `.gitignore`

The backup directory is named `aibox-<version>-backup-<date>-<time>` (e.g., `aibox-0.8.0-backup-2026-03-22-1430`).

### Examples

```bash
# Back up current state
aibox backup

# Preview without copying
aibox backup --dry-run

# Back up to a custom directory
aibox backup --output-dir /tmp/my-backup
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Config error or no files found |

---

## aibox reset

Remove all aibox files and reset the project to its pre-init state.

!!! danger "Danger zone"
    This command deletes aibox.toml, .devcontainer/, .aibox-home/, context/, CLAUDE.md, and .aibox-version. By default a backup is created first. `.gitignore` is backed up but **not** deleted.

### Usage

```bash
aibox reset [OPTIONS]
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
5. Deletes all aibox files except `.gitignore`

### Examples

```bash
# Reset with backup (safe default)
aibox reset

# Preview what would happen
aibox reset --dry-run

# Delete without backup (requires typing "DELETE" to confirm)
aibox reset --no-backup

# Non-interactive reset with backup
aibox reset --yes
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success or user aborted |
| 1 | Config error or deletion failed |

---

## aibox update

Check for or apply updates.

### Usage

```bash
aibox update [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--check` | Only check for updates, do not apply |
| `--dry-run` | Preview what would change without writing files |

### What It Does

Checks the current version against the latest available release. Without flags, upgrades the image version in `aibox.toml` and regenerates container files.

- **`--check`** — Queries GHCR for the latest image tag and GitHub Releases for the latest CLI version. Reports whether updates are available without changing anything.
- **No flags** — Fetches the latest image version, updates `version` in `aibox.toml`, regenerates `.devcontainer/` files, and updates `.aibox-version`. You still need to rebuild the container to apply changes.
- **`--dry-run`** — Shows what would change without writing any files.

### Examples

```bash
# Check for updates (read-only)
aibox update --check

# Preview upgrade without applying
aibox update --dry-run

# Upgrade image version and regenerate files
aibox update
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Config error |

---

## aibox addon

Manage add-ons (language runtimes, tools, AI agents).

### Subcommands

#### aibox addon list

List all available add-ons and their install status.

```bash
aibox addon list
```

Shows each add-on's name, tool count, and whether it's installed in the current project.

#### aibox addon add

Add an add-on to `aibox.toml` and run sync.

```bash
aibox addon add <name>
```

Inserts `[addons.<name>.tools]` with default-enabled tools into `aibox.toml`, then runs a full sync (regenerates files and rebuilds the image).

#### aibox addon remove

Remove an add-on from `aibox.toml` and run sync.

```bash
aibox addon remove <name>
```

#### aibox addon info

Show detailed info about an add-on: tools, supported versions, and defaults.

```bash
aibox addon info <name>
```

### Examples

```bash
# See what's available
aibox addon list

# Add Python runtime
aibox addon add python

# Add Kubernetes tools
aibox addon add kubernetes

# Check what's in an add-on
aibox addon info rust

# Remove an add-on
aibox addon remove python
```

---

## aibox skill

Manage skills (AI agent capabilities).

### Subcommands

#### aibox skill list

List all available skills and their deploy status.

```bash
aibox skill list
```

Shows each skill's name, source package, and whether it's active in the current project.

#### aibox skill add

Add a skill to `[skills].include` in `aibox.toml`.

```bash
aibox skill add <name>
```

If the skill was in `[skills].exclude`, it's removed from there. Runs skill reconciliation after.

#### aibox skill remove

Remove a skill by managing `[skills].include` and `[skills].exclude`.

```bash
aibox skill remove <name>
```

If the skill was in `[skills].include`, removes it. Otherwise, adds it to `[skills].exclude`. Core skills (`agent-management`, `owner-profile`) cannot be removed.

#### aibox skill info

Show info about a skill, including a preview of its SKILL.md content.

```bash
aibox skill info <name>
```

### Examples

```bash
# See all skills and their status
aibox skill list

# Add a skill not in your process packages
aibox skill add data-science

# Remove a skill from your active set
aibox skill remove debugging

# Preview a skill's content
aibox skill info code-review
```

---

## aibox audio

Host-side audio diagnostics and setup for PulseAudio over TCP.

### Subcommands

#### aibox audio check

Check if the host is correctly configured for container audio.

```bash
aibox audio check [--port <PORT>]
```

Runs diagnostics:

- PulseAudio installation and version
- Daemon status
- TCP module (`module-native-protocol-tcp`) loaded on the expected port
- Persistent configuration in `default.pa`
- Port listening
- macOS launchd agent status
- TCP connectivity test

#### aibox audio setup

Automatically install and configure PulseAudio on the host.

```bash
aibox audio setup [--port <PORT>]
```

On macOS:

1. Installs PulseAudio via Homebrew (if not present)
2. Configures `~/.config/pulse/default.pa` with the TCP module
3. Creates a launchd agent (`com.aibox.pulseaudio`) with `KeepAlive` for auto-start
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
aibox audio check

# Full automated setup (macOS)
aibox audio setup

# Use a custom port
aibox audio setup --port 4715
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (check passed or setup completed) |
| 1 | Setup failed (e.g., brew install failed) |
