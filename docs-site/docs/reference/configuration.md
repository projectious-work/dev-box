---
sidebar_position: 2
title: Configuration
---

# Configuration

`aibox.toml` is the single source of truth for an aibox project. All generated files derive from it.

## Full Specification

```toml
[aibox]
version = "0.17.5"                    # aibox version used to generate this project
base    = "debian"                    # Base image

[container]
name     = "my-app"                   # Container name
hostname = "my-app"                   # Container hostname
user     = "aibox"                    # Container user (default: aibox)
post_create_command = "npm install"   # Command to run after container creation
keepalive = false                     # Network keepalive (default: false)

[container.environment]
NODE_ENV = "development"              # Project-wide env vars (non-secret; use .aibox-local.toml for secrets)

[[container.extra_volumes]]
source    = "~/.config/gh"            # Host path (~ expanded)
target    = "/home/aibox/.config/gh"  # Container path
read_only = true

[context]
schema_version = "1.0.0"              # Context schema version (semver)
# processkit packages: minimal, managed (default), software, research, product
packages = ["managed"]

[processkit]
source   = "https://github.com/projectious-work/processkit.git"
version  = "v0.8.0"                   # Pin a real tag; "unset" skips fetching
src_path = "src"
# branch = "main"                     # Optional; tarball-first, branch as fallback
# release_asset_url_template = "..."  # Optional, for non-GitHub hosts

[addons.python.tools]                 # Addon: Python runtime
python = { version = "3.13" }
uv     = { version = "0.7" }

[addons.rust.tools]                   # Addon: Rust toolchain
rustc   = { version = "1.87" }
clippy  = {}
rustfmt = {}

[ai]
providers = ["claude", "aider"]       # AI providers to install

[mcp]
# Team-shared MCP servers merged into all generated MCP client configs
# (see also [[mcp.servers]] in .aibox-local.toml for personal servers)

[[mcp.servers]]
name    = "my-team-tool"              # Unique server name
command = "npx"                       # Executable to run
args    = ["-y", "@acme/team-server"] # Arguments
# [mcp.servers.env]                   # Optional environment variables
# API_KEY = "..."

[customization]
theme  = "gruvbox-dark"               # Color theme (7 options)
prompt = "default"                    # Starship preset (7 options)
layout = "dev"                        # Zellij layout (6 options)

[audio]
enabled      = false                  # Enable audio bridging
pulse_server = "tcp:host.docker.internal:4714"
```

## Section Reference

### [aibox]

Top-level project metadata.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `version` | String (semver) | Yes | -- | Project version |
| `base` | String | No | `"debian"` | Base image: `debian` |

### [container]

Container configuration. Controls the generated `docker-compose.yml` and `Dockerfile`.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | -- | Container name (used by compose and runtime inspect) |
| `hostname` | String | No | `"aibox"` | Container hostname |
| `user` | String | No | `"aibox"` | Container user |
| `post_create_command` | String | No | -- | Command to run after container creation |
| `keepalive` | Boolean | No | `false` | Network keepalive (prevents OrbStack/VM NAT idle dropout) |
| `environment` | Map (String → String) | No | `{}` | Environment variables injected into the container. Suitable for non-secret project-wide values; use `.aibox-local.toml` for secrets. |
| `extra_volumes` | Array of ExtraVolume | No | `[]` | Additional bind mounts. Each entry has `source`, `target`, and optional `read_only`. |

#### [[container.extra_volumes]]

Each entry in the `extra_volumes` array is an `ExtraVolume` with these fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | String | Yes | -- | Host path (supports `~` expansion) |
| `target` | String | Yes | -- | Container path where the volume is mounted |
| `read_only` | Boolean | No | `false` | Mount the volume read-only |

Example — mount your GitHub CLI config:

```toml
[[container.extra_volumes]]
source    = "~/.config/gh"
target    = "/home/aibox/.config/gh"
read_only = true
```

:::tip Customizing ports, packages, volumes, and environment variables
Use `Dockerfile.local` for installing additional packages, and `docker-compose.override.yml` for ports and additional services. Both files are scaffolded by `aibox init` and are never overwritten by `aibox sync`.

Environment variables and bind mounts can also be configured directly in `[container.environment]` / `[[container.extra_volumes]]` in `aibox.toml`, or — for secrets and per-developer settings that should not be committed — in [`.aibox-local.toml`](./local-config.md).
:::

### .aibox-local.toml

`.aibox-local.toml` is a personal, gitignored overlay for per-developer settings that should never be committed — API keys, personal bind mounts, and similar secrets. It lives next to `aibox.toml` in the project root and is automatically added to `.gitignore` by `aibox init` and `aibox sync`.

Three sections are supported:

- **`[container.environment]`** — merged on top of `aibox.toml`'s `[container.environment]`. Local values win on conflicts.
- **`[[container.extra_volumes]]`** — appended after any volumes declared in `aibox.toml`.
- **`[[mcp.servers]]`** — personal MCP servers appended to the team MCP servers from `aibox.toml [mcp]`. All sources are merged into each generated MCP client config file.

All other configuration (container name, addons, processkit version, etc.) must remain in `aibox.toml`.

See the dedicated [Local Config reference](./local-config.md) for a full example and merge-behavior details.

### [context]

Context-system metadata. Controls the schema version and the **declarative**
processkit package selection — the list agents read to decide which subset of
the installed processkit content is in scope for this project.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `schema_version` | String (semver) | No | `"1.0.0"` | Context schema version |
| `packages` | Array of strings | No | `["managed"]` | processkit packages in scope for this project. |

**Five processkit packages**, defined by upstream YAMLs in
`packages/{minimal,managed,software,research,product}.yaml` and composed via
`extends:`:

| Package | Purpose |
|---------|---------|
| `minimal` | Bare-minimum skill set for scripts and experiments |
| `managed` | Recommended default — backlog, decisions, standups, handover |
| `software` | `managed` + code review, testing, debugging, refactoring, architecture |
| `research` | `managed` + data science, documentation, research artefacts |
| `product` | Everything — `software` + design, security, operations, product planning |

> `packages` is **declarative metadata for agents** — it tells agents which
> skills to *prefer*, but does not filter what gets installed. Use
> `[skills].include` / `[skills].exclude` to control the installed skill set.

### [addons]

Addons install language runtimes, tool bundles, and AI agents into the container. Each addon is a named table with a `tools` sub-table.

```toml
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
```

Run `aibox addon list` to see all 25 available addons, or `aibox addon info <name>` for tool details and supported versions. See the [Addons page](../addons/overview.md) for full documentation.

### [skills]

Controls which skills from processkit are installed into `context/skills/`.
By default every skill in the processkit version you've pinned is installed.

```toml
[skills]
include = ["python-best-practices", "fastapi-patterns"]  # install only these
exclude = ["pandas-polars"]                               # install all except these
```

`include` and `exclude` are mutually exclusive: use one or the other, not both.
Both accept skill names (the filename without `.md`). An empty `[skills]` table
(or omitting the section entirely) installs all skills.

See the [Skills page](../skills/index.md) for the full processkit boundary.

### [ai]

AI provider configuration. Providers listed here are automatically installed as addons.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `providers` | Array of strings | No | `["claude"]` | AI providers: `claude`, `aider`, `gemini`, `mistral`, `openai`, `copilot`, `continue`. `cursor` is MCP-registration only (no container CLI). |

### [processkit]

The **load-bearing** content section. Configures the content source
the project consumes — skills, primitives, processes, package YAMLs, and the
canonical `AGENTS.md` template. The default upstream is the canonical
[projectious-work/processkit](https://github.com/projectious-work/processkit)
repo, but any processkit-compatible source works (forks, self-hosted, private
mirrors).

If `version` is the sentinel `unset`, both `aibox init` and `aibox sync` skip
the processkit fetch entirely. Pin a real tag (e.g. `v0.8.0`) to land the
content. The downloaded tarball is git-tracked under
`context/templates/processkit/<version>/` so derived projects always have the
original to diff against.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source` | String | No | `https://github.com/projectious-work/processkit.git` | Git URL of the content source. |
| `version` | String | No | `unset` | Semver tag to consume. The sentinel `unset` skips fetching until a real tag is set. |
| `src_path` | String | No | `src` | Subdirectory inside the source repo containing the shippable payload. Auto-detected for flat release-asset tarballs. |
| `branch` | String | No | _(none)_ | Optional branch override for testing pre-release work. Discouraged but supported. |
| `release_asset_url_template` | String | No | _(GitHub-style default)_ | URL template for the release-asset tarball. Placeholders: `{source}` (`.git` stripped), `{version}`, `{org}`, `{name}`. Set this for non-GitHub hosts. |

#### Fetch strategy

The fetcher tries strategies in priority order:

1. **Branch override** (if `branch` is set) — `git clone --branch <name>`.
2. **Release-asset tarball** — downloads a purpose-built `.tar.gz` from
   the URL built from `release_asset_url_template` (or the GitHub-style
   default `{source}/releases/download/{version}/{name}-{version}.tar.gz`).
   When a sibling `<asset>.sha256` file is present, the tarball bytes
   are verified against it before extraction. The verified SHA256 is
   recorded in `aibox.lock` as `release_asset_sha256` for bit-exact
   reproducibility.
3. **Host auto-tarball** — falls back to GitHub / GitLab's auto-generated
   `archive/refs/tags/<version>.tar.gz` when no release asset is
   available.
4. **Git clone** of the tag — last resort for hosts that serve neither
   tarball form (typical for self-hosted git over SSH).

The release-asset path lets producers (processkit and any compatible
content source) ship a smaller, explicit shippable artifact. Consumers
get bit-exact reproducibility for free.

A SHA256 mismatch is a hard error (does NOT fall through), since it
indicates either tampering or a producer bug; both are situations the
user should be told about.

#### Example: consume a Gitea-hosted fork

```toml
[processkit]
source                     = "https://gitea.acme.com/platform/processkit-acme.git"
version                    = "v1.2.0"
release_asset_url_template = "https://gitea.acme.com/{org}/{name}/releases/download/{version}/{name}-{version}.tar.gz"
```

### [mcp]

MCP server definitions and permission configuration. `aibox sync` merges servers from three sources and regenerates all MCP client config files:

1. **Built-in processkit servers** — always included (the processkit MCP server and any extras it ships)
2. **`aibox.toml [mcp]`** — team-shared servers committed to version control
3. **`.aibox-local.toml [mcp]`** — personal servers, gitignored

Generated files (`.mcp.json`, `.cursor/mcp.json`, `.gemini/settings.json`, `.codex/config.toml`, `.continue/mcpServers/`) are **gitignored**. They are always reproducible from the config sources above and must not be committed — doing so would embed personal server definitions or credentials from `.aibox-local.toml`.

#### Server Definitions: [[mcp.servers]]

Each `[[mcp.servers]]` entry has these fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | -- | Unique server name (used as the key in generated configs) |
| `command` | String | Yes | -- | Executable to run (e.g. `npx`, `/usr/local/bin/my-server`) |
| `args` | Array of strings | No | `[]` | Arguments passed to `command` |
| `env` | Map (String → String) | No | `{}` | Environment variables set when the server process starts |

Example:

```toml
[[mcp.servers]]
name    = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]
[mcp.servers.env]
GITHUB_TOKEN = "ghp_..."
```

#### Permission Configuration: [mcp.permissions]

Controls which MCP servers harnesses are permitted to use, eliminating repetitive permission prompts. `aibox sync` expands glob patterns into concrete server names and regenerates harness-specific permission files for Claude Code, OpenCode, Continue, Cursor, Gemini CLI, GitHub Copilot, Aider, and Codex.

**Global defaults:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `default_mode` | String | No | `"allow"` | Default permission: `"allow"`, `"ask"`, or `"deny"`. Recommend `"allow"` for aibox-shipped processkit tools (trusted content). |
| `allow_patterns` | Array of strings | No | `["mcp__processkit-*"]` | Glob patterns to auto-allow. Supports wildcards: `prefix-*`, `*-suffix`, `*-middle*`, exact matches. First-match-wins semantics. |
| `deny_patterns` | Array of strings | No | `[]` | Glob patterns to auto-deny (takes precedence over allow). Use for restricting specific tool families. |

**Per-harness overrides** (optional):

```toml
[mcp.permissions.harness.claude-code]
mode = "allow"              # Override global default if needed
extra_patterns = []         # Add harness-specific patterns

[mcp.permissions.harness.opencode]
mode = "allow"
deny_patterns = []          # Restrict specific tools per harness

[mcp.permissions.harness.codex]
trust_level = "trusted"     # Codex uses project-level trust instead of per-tool lists
```

**Example:**

```toml
[mcp.permissions]
default_mode    = "allow"
allow_patterns  = ["mcp__processkit-*", "bash"]
deny_patterns   = ["mcp__processkit-dangerous-admin"]  # Deny a specific pattern if needed

[mcp.permissions.harness.claude-code]
# Use default settings; Claude Code will auto-allow all processkit tools

[mcp.permissions.harness.continue]
# Continue defaults to "ask" for safety; override to "allow" to auto-approve
mode = "allow"
```

:::tip Personal MCP servers
Servers that require personal credentials or are not relevant to all team members belong in `[[mcp.servers]]` in `.aibox-local.toml`, not here. See [Local Config](./local-config.md).
:::

### [agents]

Controls how `aibox init` scaffolds the canonical agent entry document
(`AGENTS.md`) and the provider-specific entry files (`CLAUDE.md`, future
`CODEX.md`, …). The principle is **provider neutrality**: every agent
harness reads the same `AGENTS.md` so a project doesn't have to keep
N copies of the same instructions in sync. Provider files exist only
to satisfy specific harnesses' auto-load conventions (Claude Code
auto-loads `CLAUDE.md` at startup, etc.).

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `canonical` | String | No | `"AGENTS.md"` | Filename of the canonical agent entry document. Almost no one should override this — the default matches the [agents.md](https://agents.md/) ecosystem convention. |
| `provider_mode` | String | No | `"pointer"` | How provider files are scaffolded. `pointer` (recommended): provider files are thin pointers that say "see AGENTS.md". `full`: provider files contain the rich provider-flavoured content — use only when a project genuinely needs different instructions per harness. |

`aibox init` always creates `AGENTS.md` (write-if-missing — never
overwrites). When the Claude provider is enabled, it also creates
`CLAUDE.md`, either as a thin pointer (default) or with the full rich
content (`provider_mode = "full"`). Other providers (Aider, Gemini,
Mistral, OpenAI, Copilot, Continue) use config files rather than markdown
entries and are not affected by this section.

Existing files are never overwritten. If you already have a hand-written
`AGENTS.md` or `CLAUDE.md`, `aibox init` leaves it alone.

### [customization]

Visual and layout configuration. See [Themes](../customization/themes.md) and [Layouts](../customization/layouts.md) for details.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `theme` | String | No | `"gruvbox-dark"` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord`, `projectious` |
| `prompt` | String | No | `"default"` | Starship preset: `default`, `plain`, `arrow`, `minimal`, `nerd-font`, `pastel`, `bracketed` |
| `layout` | String | No | `"dev"` | Zellij layout: `dev`, `focus`, `cowork`, `cowork-swap`, `browse`, `ai` |

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
| `AIBOX_HOST_ROOT` | `.aibox-home/` path | Host directory for persistent config (default: `.aibox-home/`) |
| `AIBOX_WORKSPACE_DIR` | Workspace mount source | Host directory mounted as `/workspace` |
| `AIBOX_LOG_LEVEL` | `--log-level` | Log verbosity (`trace`, `debug`, `info`, `warn`, `error`) |

Example:

```bash
AIBOX_WORKSPACE_DIR=/home/user/projects/my-app aibox start
```
