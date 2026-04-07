---
sidebar_position: 2
title: Configuration
---

# Configuration

`aibox.toml` is the single source of truth for an aibox project. All generated files derive from it.

## Full Specification

```toml
[aibox]
version = "0.16.0"                    # aibox version used to generate this project
base    = "debian"                    # Base image

[container]
name     = "my-app"                   # Container name
hostname = "my-app"                   # Container hostname
user     = "aibox"                    # Container user (default: aibox)
post_create_command = "npm install"   # Command to run after container creation

[context]
schema_version = "1.0.0"              # Context schema version (semver)
# processkit packages: minimal, managed (default), software, research, product
packages = ["managed"]

[processkit]
source   = "https://github.com/projectious-work/processkit.git"
version  = "v0.5.1"                   # Pin a real tag; "unset" skips fetching
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

[customization]
theme  = "gruvbox-dark"               # Color theme (7 options)
prompt = "default"                    # Starship preset (7 options)
layout = "dev"                        # Zellij layout (4 options)

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

:::tip Customizing ports, packages, volumes, and environment variables
Use `Dockerfile.local` for installing additional packages, and `docker-compose.override.yml` for ports, volumes, and environment variables. Both files are scaffolded by `aibox init` and are never overwritten by `aibox sync`.
:::

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

> In v0.16.0, `packages` is **declarative metadata only**. aibox installs every
> skill processkit ships under `src/skills/`, regardless of the selected
> package(s). Agents read `packages` to decide which skills to *prefer*. The
> install set may become package-aware in a future release; do not depend on
> "this skill is not installed" as a guarantee.

### [addons]

Addons install language runtimes, tool bundles, and AI agents into the container. Each addon is a named table with a `tools` sub-table.

```toml
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
```

Run `aibox addon list` to see all 21 available addons, or `aibox addon info <name>` for tool details and supported versions. See the [Addons page](../addons/overview.md) for full documentation.

### [skills]

**Reserved / no-op in v0.16.0.** The TOML parser still accepts a `[skills]`
table with `include` / `exclude` arrays for forward compatibility, but aibox
**does not act on it**. Every project gets every processkit skill installed
under `context/skills/`, and the agent decides which to use based on
`[context].packages` and the skill's own `description`.

A future release may re-introduce per-skill include/exclude semantics. Until
then, treat the section as informational. See the [Skills page](../skills/index.md)
for the rationale and the full processkit boundary.

### [ai]

AI provider configuration. Providers listed here are automatically installed as addons.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `providers` | Array of strings | No | `["claude"]` | AI providers: `claude`, `aider`, `gemini`, `mistral` |

### [processkit]

The **load-bearing** content section in v0.16.0. Configures the content source
the project consumes — skills, primitives, processes, package YAMLs, and the
canonical `AGENTS.md` template. The default upstream is the canonical
[projectious-work/processkit](https://github.com/projectious-work/processkit)
repo, but any processkit-compatible source works (forks, self-hosted, private
mirrors).

If `version` is the sentinel `unset`, both `aibox init` and `aibox sync` skip
the processkit fetch entirely. Pin a real tag (e.g. `v0.5.1`) to land the
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
Mistral) use config files rather than markdown entries and are not
affected by this section.

Existing files are never overwritten. If you already have a hand-written
`AGENTS.md` or `CLAUDE.md`, `aibox init` leaves it alone.

### [customization]

Visual and layout configuration. See [Themes](../customization/themes.md) and [Layouts](../customization/layouts.md) for details.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `theme` | String | No | `"gruvbox-dark"` | Color theme: `gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`, `tokyo-night`, `nord`, `projectious` |
| `prompt` | String | No | `"default"` | Starship preset: `default`, `plain`, `arrow`, `minimal`, `nerd-font`, `pastel`, `bracketed` |
| `layout` | String | No | `"dev"` | Zellij layout: `dev`, `focus`, `cowork`, `browse` |

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
