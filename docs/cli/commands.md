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
| `--image <FLAVOR>` | `base` | Image flavor: `base`, `python`, `latex`, `typst`, `rust`, `python-latex`, `python-typst`, `rust-latex` |
| `--process <FLAVOR>` | `product` | Work process flavor: `minimal`, `managed`, `research`, `product` |
| `--ai <PROVIDER>` | `claude` | AI provider(s) to configure (can be specified multiple times) |
| `--user <USER>` | `root` | Container user |

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

# Configure AI providers
dev-box init --ai claude --ai gemini
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

## dev-box generate

Re-generate devcontainer files from `dev-box.toml`.

### Usage

```bash
dev-box generate
```

### What It Does

Reads `dev-box.toml` and regenerates:

- `.devcontainer/Dockerfile`
- `.devcontainer/docker-compose.yml`
- `.devcontainer/devcontainer.json`

It also re-seeds `.dev-box-home/` with any missing default configuration files (vim, git, zellij, etc.), ensuring new config templates introduced in later versions are picked up.

This is useful after editing `dev-box.toml` to apply changes without rebuilding the container.

### Examples

```bash
# Edit config, then regenerate
vim dev-box.toml
dev-box generate
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No `dev-box.toml` found, or invalid config |

!!! note "Overwrites generated files"
    `generate` always overwrites the devcontainer files. Do not hand-edit files in `.devcontainer/` if you are using `dev-box generate` -- your changes will be lost.

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
2. Runs `generate` to ensure devcontainer files are current
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
dev-box start
```

### What It Does

This is the primary command for daily use. It handles the full lifecycle:

1. Seeds `.dev-box-home/` directory with default configs (first run only)
2. Generates devcontainer files from `dev-box.toml`
3. Checks container state:
   - **Running:** Skips to attach
   - **Stopped:** Starts the existing container
   - **Missing:** Builds the image (if needed) and creates the container
4. Waits for the container to be ready (up to 7.5 seconds)
5. Attaches via `zellij --layout dev`

### Examples

```bash
# Start working (handles everything)
dev-box start
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

## dev-box attach

Attach to a running container via Zellij.

### Usage

```bash
dev-box attach
```

### What It Does

Execs into the container and launches `zellij --layout dev`. Unlike `start`, this command does not create or start the container -- it must already be running.

### Examples

```bash
# Attach from a second terminal
dev-box attach
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
 ✓ Config version: 0.3.5
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

### What It Does

Checks the current version against the latest available release. Without `--check`, applies the update.

### Examples

```bash
# Check for updates
dev-box update --check

# Apply updates
dev-box update
```

`dev-box update --check` queries GHCR for the latest image tag and GitHub Releases for the latest CLI version, comparing against current versions. Without `--check`, it displays manual update instructions.

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
