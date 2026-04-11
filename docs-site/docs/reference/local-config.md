---
sidebar_position: 3
title: "Local Config (.aibox-local.toml)"
---

# Local Config (.aibox-local.toml)

`.aibox-local.toml` is a personal, gitignored overlay that sits next to `aibox.toml` in the project root. It exists for secrets and per-developer settings that must never be committed to version control — API tokens, personal credential paths, and similar values that differ between contributors.

## Why it exists

`aibox.toml` is committed and shared across the team. That's the right place for project-wide settings: container name, addons, processkit version, shared environment variables, and so on. But tokens and personal bind mounts don't belong there. `.aibox-local.toml` gives every developer a private escape valve without requiring `.gitignore` discipline on every secret.

## Location and gitignore

`.aibox-local.toml` lives in the **project root**, next to `aibox.toml`:

```
my-project/
├── aibox.toml               ← committed, shared
├── .aibox-local.toml        ← gitignored, personal
├── .devcontainer/
└── context/
```

`aibox init` and `aibox sync` automatically add `.aibox-local.toml` to `.gitignore`. You do not need to do this manually.

## Supported sections

Three sections are supported. Everything else must remain in `aibox.toml`.

### [container.environment]

Inject environment variables into the container. These are merged **on top of** any `[container.environment]` values in `aibox.toml`. If the same key appears in both files, the local value wins.

```toml
[container.environment]
GH_TOKEN            = "ghp_xxxxxxxxxxxxxxxxxxxx"
ANTHROPIC_API_KEY   = "sk-ant-api03-..."
OPENAI_API_KEY      = "sk-proj-..."
AWS_PROFILE         = "my-dev-profile"
```

### [[container.extra_volumes]]

Personal bind mounts appended **after** any volumes declared in `aibox.toml`. Each entry requires `source` (host path) and `target` (container path). `read_only` defaults to `false`.

```toml
[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"

[[container.extra_volumes]]
source = "~/.aws"
target = "/home/aibox/.aws"
read_only = true

[[container.extra_volumes]]
source = "~/.ssh/id_ed25519"
target = "/home/aibox/.ssh/id_ed25519"
read_only = true
```

### [mcp]

Personal MCP servers appended to the generated MCP client configs on `aibox sync`. Use this section for servers you want only on your machine — internal tools, local scripts, or servers that require credentials you don't want to share.

Each server entry is an `[[mcp.servers]]` table with the same fields as `[mcp]` in `aibox.toml`:

```toml
[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]

[[mcp.servers]]
name    = "local-notes"
command = "/home/user/bin/notes-mcp"
args    = ["--db", "~/notes.db"]

[[mcp.servers]]
name    = "stripe"
command = "npx"
args    = ["-y", "@stripe/mcp"]
[mcp.servers.env]
STRIPE_SECRET_KEY = "sk_test_..."
```

`aibox sync` merges personal servers with team servers (from `aibox.toml [mcp]`) and built-in processkit servers, then regenerates all MCP client config files. The generated files are **gitignored** — they are never committed to version control, so personal keys and server definitions stay private.

## Merge behavior

| Section | Merge rule |
|---------|-----------|
| `[container.environment]` | Merged with `aibox.toml`; local values win on key conflicts |
| `[[container.extra_volumes]]` | Appended after `aibox.toml` volumes; no deduplication |
| `[[mcp.servers]]` | Appended after `aibox.toml` MCP servers; all sources merged into each generated config file |

## Full example

A typical `.aibox-local.toml` for a developer working with Claude, GitHub, and AWS, plus a personal MCP server:

```toml
[container.environment]
ANTHROPIC_API_KEY = "sk-ant-api03-..."
GH_TOKEN          = "ghp_xxxxxxxxxxxxxxxxxxxx"
AWS_PROFILE       = "my-dev-profile"
AWS_REGION        = "eu-west-1"

[[container.extra_volumes]]
source = "~/.config/gh"
target = "/home/aibox/.config/gh"

[[container.extra_volumes]]
source = "~/.aws"
target = "/home/aibox/.aws"
read_only = true

[[container.extra_volumes]]
source = "~/.ssh/id_ed25519"
target = "/home/aibox/.ssh/id_ed25519"
read_only = true

[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]
```

## What is NOT supported

Everything outside of `[container.environment]`, `[[container.extra_volumes]]`, and `[[mcp.servers]]` is ignored. The following must remain in `aibox.toml`:

- Container name, hostname, user, `post_create_command`, `keepalive`
- `[addons]` — addon configuration
- `[processkit]` — content source and version pin
- `[skills]` — include/exclude lists
- `[ai]` — provider list
- `[customization]` — theme, prompt, layout
- `[audio]` — audio bridging

:::tip Applying changes
After editing `.aibox-local.toml`, run `aibox sync` (or `aibox sync --no-build` for a config-only refresh) to regenerate `.devcontainer/` files with the updated environment and volumes, and MCP client config files with the updated server list.
:::
