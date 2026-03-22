# Configuration

`dev-box.toml` is the single source of truth for a dev-box project. All generated files derive from it.

## Full Specification

```toml
[dev-box]
version = "0.6.0"                    # Project version (semver)
image = "python"                      # Image flavor
process = "product"                   # Work process flavor

[container]
name = "my-app"                       # Container name
hostname = "my-app"                   # Container hostname
user = "root"                         # Container user
ports = ["8000:8000", "5432:5432"]    # Port mappings (host:container)
extra_packages = ["ripgrep", "fzf"]   # Additional apt packages
environment = { MY_VAR = "value" }    # Environment variables
post_create_command = "npm install"   # Command to run after container creation
vscode_extensions = [                 # VS Code extensions to install
    "ms-python.python",
    "ms-python.vscode-pylance",
]

[container.extra_volumes]
# Additional volume mounts (beyond the standard ones)
# source = "host-path"
# target = "container-path"
# read_only = false

[context]
schema_version = "1.0.0"             # Context schema version (semver)

[ai]
providers = ["claude"]                # AI providers to configure

[appearance]
theme = "gruvbox-dark"               # Color theme for all tools

[audio]
enabled = true                        # Enable audio bridging
pulse_server = "tcp:host.docker.internal:4714"  # PulseAudio server address
```

## Section Reference

### [dev-box]

Top-level project metadata.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `version` | String (semver) | Yes | -- | Project version |
| `image` | String | Yes | -- | Image flavor: `base`, `python`, `latex`, `typst`, `rust`, `python-latex`, `python-typst`, `rust-latex` |
| `process` | String | Yes | -- | Work process: `minimal`, `managed`, `research`, `product` |

### [container]

Container configuration. Controls the generated `docker-compose.yml` and `Dockerfile`.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | -- | Container name (used by compose and runtime inspect) |
| `hostname` | String | No | `"dev-box"` | Container hostname |
| `user` | String | No | `"root"` | Container user |
| `ports` | Array of strings | No | `[]` | Port mappings in `host:container` format |
| `extra_packages` | Array of strings | No | `[]` | Additional apt packages to install |
| `extra_volumes` | Array of objects | No | `[]` | Additional volume mounts (see below) |
| `environment` | Map of strings | No | `{}` | Environment variables set in the container |
| `post_create_command` | String | No | -- | Command to run after container creation (devcontainer.json `postCreateCommand`) |
| `vscode_extensions` | Array of strings | No | `[]` | VS Code extensions to install (added to devcontainer.json `customizations.vscode.extensions`) |

#### Extra Volumes

Each entry in `extra_volumes` has:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | String | Yes | -- | Host path |
| `target` | String | Yes | -- | Container path |
| `read_only` | Boolean | No | `false` | Mount as read-only |

### [context]

Context system configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `schema_version` | String (semver) | No | `"1.0.0"` | Context schema version |

### [ai]

AI provider configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `providers` | Array of strings | No | `["claude"]` | AI providers to configure (e.g., `"claude"`, `"gemini"`) |

### [appearance]

Color theme configuration. See [Themes](../themes.md) for details and previews.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `theme` | String | No | `"gruvbox-dark"` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord` |

### [audio]

Audio bridging configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | Boolean | No | `false` | Enable PulseAudio environment setup |
| `pulse_server` | String | No | `"tcp:host.docker.internal:4714"` | PulseAudio server address |

## Environment Variable Overrides

Some settings can be overridden via environment variables:

| Variable | Overrides | Description |
|----------|-----------|-------------|
| `DEV_BOX_HOST_ROOT` | `.dev-box-home/` path | Host directory for persistent config (default: `.dev-box-home/`) |
| `DEV_BOX_WORKSPACE_DIR` | Workspace mount source | Host directory mounted as `/workspace` |
| `DEV_BOX_LOG_LEVEL` | `--log-level` | Log verbosity (`trace`, `debug`, `info`, `warn`, `error`) |

Example:

```bash
DEV_BOX_HOST_ROOT=/tmp/dev-root dev-box start
```

!!! note "Backward compatibility with `.root/`"
    If your project uses the legacy `.root/` directory name, dev-box will continue to work with it. `dev-box doctor` will suggest renaming it to `.dev-box-home/` for consistency with the current convention.

## Default Values Summary

When a field is omitted from `dev-box.toml`, these defaults apply:

| Field | Default |
|-------|---------|
| `container.hostname` | `"dev-box"` |
| `container.user` | `"root"` |
| `container.ports` | `[]` |
| `container.extra_packages` | `[]` |
| `container.extra_volumes` | `[]` |
| `container.environment` | `{}` |
| `container.post_create_command` | -- (not set) |
| `container.vscode_extensions` | `[]` |
| `context.schema_version` | `"1.0.0"` |
| `ai.providers` | `["claude"]` |
| `audio.enabled` | `false` |
| `audio.pulse_server` | `"tcp:host.docker.internal:4714"` |

## Custom Dockerfile Layers (Dockerfile.local)

For project-specific build steps that go beyond `extra_packages`, create `.devcontainer/Dockerfile.local`. This file is appended to the generated Dockerfile by `dev-box sync` and is never overwritten.

`dev-box init` creates a placeholder with usage examples. The generated base image is aliased as the `dev-box` stage, which you can reference in multi-stage builds.

### Simple usage — append layers

```dockerfile
# .devcontainer/Dockerfile.local
RUN npx playwright install --with-deps chromium
RUN pip install some-special-package
```

### Advanced usage — multi-stage build

```dockerfile
# .devcontainer/Dockerfile.local
FROM node:20 AS node-builder
WORKDIR /app
COPY package*.json ./
RUN npm ci && npm run build

FROM dev-box
COPY --from=node-builder /app/dist /workspace/dist
```

The `FROM dev-box` line references the generated base stage. This lets you bring in artifacts from other build stages while keeping the final image based on your dev-box configuration.

!!! note
    `Dockerfile.local` is your file — `dev-box sync` never modifies it. If the file doesn't exist or is empty, no extra layers are added.

## The .dev-box-version File

A plain text file at the project root containing the context schema version that was last applied:

```
1.0.0
```

This is created by `dev-box init` and compared against `context.schema_version` by `dev-box doctor` to detect when migration is needed. Keep this file in version control.

## Example Configurations

### Python web application

```toml
[dev-box]
version = "0.3.7"
image = "python"
process = "product"

[container]
name = "web-api"
hostname = "web-api"
ports = ["8000:8000", "5432:5432"]
extra_packages = ["postgresql-client"]
environment = { DATABASE_URL = "postgresql://localhost:5432/mydb" }

[context]
schema_version = "1.0.0"

[ai]
providers = ["claude"]

[audio]
enabled = false
```

### Rust CLI tool

```toml
[dev-box]
version = "0.3.7"
image = "rust"
process = "managed"

[container]
name = "my-cli"
hostname = "my-cli"
extra_packages = ["musl-tools"]

[context]
schema_version = "1.0.0"

[ai]
providers = ["claude"]

[audio]
enabled = false
```

### LaTeX thesis

```toml
[dev-box]
version = "0.3.7"
image = "latex"
process = "research"

[container]
name = "thesis"
hostname = "thesis"

[context]
schema_version = "1.0.0"

[ai]
providers = ["claude"]

[audio]
enabled = false
```

### Data science with voice

```toml
[dev-box]
version = "0.3.7"
image = "python-latex"
process = "research"

[container]
name = "data-analysis"
hostname = "data-analysis"
ports = ["8888:8888"]
extra_packages = ["graphviz"]
environment = { JUPYTER_TOKEN = "dev" }

[context]
schema_version = "1.0.0"

[ai]
providers = ["claude"]

[audio]
enabled = true
pulse_server = "tcp:host.docker.internal:4714"
```
