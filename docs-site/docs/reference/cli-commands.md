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
| `--process <PACKAGES>` | `managed` | processkit package(s): `minimal`, `managed`, `software`, `research`, `product` (can be repeated) |
| `--ai <PROVIDER>` | `claude` | AI provider(s): `claude`, `aider`, `gemini`, `mistral` (can be repeated) |
| `--theme <THEME>` | `gruvbox-dark` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord`, `projectious` |
| `--user <USER>` | `aibox` | Container user |
| `--addons <NAME>` | -- | Addon names (can be repeated): `python`, `rust`, `node`, `go`, `latex`, `typst`, `infrastructure`, `kubernetes`, `cloud-aws`, etc. |

### What It Does

1. Creates `aibox.toml` with the specified settings
2. Creates `.aibox-home/` directory with default configuration files
3. Generates `.devcontainer/Dockerfile`, `docker-compose.yml`, and `devcontainer.json`
4. Scaffolds the slim project skeleton: `.aibox-version`, `.gitignore`, an empty
   `context/` directory, and a thin `CLAUDE.md` pointer to `AGENTS.md` (when the
   `claude` provider is enabled)
5. Updates `.gitignore` with required entries (`.aibox-home/`, `.devcontainer/`, etc.)
6. **Installs processkit content** — fetches the version pinned in `[processkit]`,
   extracts it to `context/templates/processkit/<version>/`, and materialises an
   editable copy under `context/skills/`. The canonical `AGENTS.md` is rendered
   from the processkit scaffolding template at this point.

> If `[processkit].version` is the sentinel `unset`, the processkit install step
> is skipped — pin a real tag (e.g. `v0.5.1`) and re-run `aibox sync` to land
> the content.

### Examples

```bash
# Basic initialization (uses directory name, defaults)
aibox init

# Specify project name and processkit package
aibox init --name my-api --process managed

# With addons
aibox init --name my-app --addons python --addons infrastructure

# Configure multiple AI providers
aibox init --ai claude --ai aider

# Choose a color theme
aibox init --name my-app --theme catppuccin-mocha
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | `aibox.toml` already exists, or invalid option value |

### Interactive Mode

When `--name` or `--process` flags are omitted and the terminal is interactive, `aibox init` prompts for each missing value.

The `--process` prompt shows the five processkit packages (`minimal`, `managed`, `software`, `research`, `product`). `managed` is the recommended default.

In non-interactive environments (scripts, CI pipelines), omitted flags silently use defaults: the current directory name for `--name`, `debian` for `--base`, and `managed` for `--process`.

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
5. **Installs / refreshes processkit content** — fetches the pinned
   `[processkit].version`, verifies the release-asset SHA256 (when present),
   extracts it to `context/templates/processkit/<version>/`, and three-way
   diffs the editable copy under `context/skills/` against it. Skipped when
   `[processkit].version` is `unset`.
6. **Builds the container image** via `docker compose build` (skipped gracefully if no runtime)

`aibox sync` no longer generates a `context/AIBOX.md` file — that file
existed in pre-v0.16 releases and has been removed. The canonical agent
entry document is now `AGENTS.md`, owned by processkit and rendered at
`aibox init` time (write-if-missing, never overwritten).

Only files whose content has actually changed are written. Reports what was updated.

### Sync perimeter

`aibox sync` is allowed to create, modify, or delete **only** the files
listed below. Anything else — including `README.md`, `AGENTS.md`,
`CLAUDE.md`, `LICENSE`, `src/`, `tests/`, `docs/`, `context/BACKLOG.md`,
`context/skills/**`, and any user-authored content — is **out of perimeter
and will never be touched**, in any release, under any configuration.

| Path | Why |
|------|-----|
| `aibox.toml` | One-time schema migrations (e.g. inserting `[processkit]`) |
| `.aibox-version` | Tracks installed CLI version for migration detection |
| `.aibox-home/**` | Runtime config seed (shells, vim, zellij, yazi, …); gitignored |
| `.devcontainer/Dockerfile` | Regenerated from `aibox.toml` |
| `.devcontainer/docker-compose.yml` | Regenerated from `aibox.toml` |
| `.devcontainer/devcontainer.json` | Regenerated from `aibox.toml` |
| `context/templates/processkit/<version>/**` | Immutable upstream snapshot used as the base of three-way diffs |
| `context/migrations/**` | Migration documents (additive; never overwrites) |

> **Editable processkit content** under `context/skills/`, `context/processes/`,
> and similar directories is **never** rewritten by `aibox sync`. Sync only
> reports drift (via the three-way diff against the immutable snapshot under
> `context/templates/processkit/<version>/`) and lets the user reconcile
> changes by hand.

The perimeter is enforced two ways:

1. **Statically** — a unit test (`all_known_sync_write_targets_are_in_perimeter`)
   verifies every known sync write site against the list at every build.
   Adding a new sync write outside the perimeter fails CI immediately.
2. **At runtime** — `aibox sync` snapshots a set of representative
   out-of-perimeter sentinel files (`README.md`, `AGENTS.md`, `CLAUDE.md`,
   `context/BACKLOG.md`, etc.) before running, and verifies after that
   none were modified. A violation aborts with an error naming the
   offending path before the (slow) image build runs.

This perimeter is a **stable contract**. If a future release needs to
expand it, that change requires an entry in `DECISIONS.md` and a
visible note in the changelog. See [issue #34](https://github.com/projectious-work/aibox/issues/34).

> **Note:** This perimeter applies to `aibox sync`, not `aibox init`.
> `init` is allowed to create files outside this list as part of project
> bootstrap (`README.md`, `.gitignore`, `CLAUDE.md`, etc.) — its contract
> is "I am setting up a new project root", not "I am refreshing
> aibox-managed files in an existing one".

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

Environments save `aibox.toml`, `AGENTS.md`, `CLAUDE.md`, and `context/` (excluding `context/shared/`) to `.aibox-env/<name>/`. Files in `context/shared/` are shared across all environments.

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
- `AGENTS.md`
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
    This command deletes aibox.toml, .devcontainer/, .aibox-home/, context/, AGENTS.md, CLAUDE.md, and .aibox-version. By default a backup is created first. `.gitignore` is backed up but **not** deleted.

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

If the registry is unreachable or returns an authentication error, `aibox update` prints a warning and exits cleanly (exit code 0) rather than failing. Run `docker login ghcr.io` if you receive repeated authentication errors.

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
