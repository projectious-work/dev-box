---
sidebar_position: 9
title: "OpenAI (Codex CLI)"
---

# OpenAI Codex CLI

[Codex CLI](https://github.com/openai/codex) is OpenAI's open-source terminal coding agent. Built in Rust, GA since April 2025.

## Setup

```toml
[ai]
providers = ["openai"]
```

Run `aibox sync`, then inside the container:

```bash
codex    # Launches OpenAI Codex CLI
```

## API Key

```toml
[container.environment]
OPENAI_API_KEY = "sk-..."
```

Alternatively, use a ChatGPT Plus/Pro/Team/Enterprise account — Codex prompts for authentication on first launch.

## Configuration

Codex's home-directory state is persisted in `.aibox-home/.codex/`, mounted at `/home/aibox/.codex/`. This survives devcontainer rebuilds, so device sign-in only needs to be completed once per host cache unless you clear it.

Key files:
- `.aibox-home/.codex/auth.json` — cached ChatGPT/device authentication reused across rebuilds
- `.aibox-home/.codex/rules/` — home-directory Codex rules and local state
- `.aibox-home/.codex/sessions/` — Codex session history

Separately, aibox also generates a project-local `.codex/config.toml` for MCP server registration.

## MCP Integration

Codex has a native MCP client. aibox generates `.codex/config.toml` automatically on `aibox sync`, merging processkit built-in servers, team servers from `aibox.toml [mcp]`, and personal servers from `.aibox-local.toml [mcp]`.

`.codex/config.toml` is **gitignored** — it is regenerated on every `aibox sync` and must not be committed.

To add MCP servers:

```toml
# aibox.toml — team-shared servers
[[mcp.servers]]
name    = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]

# .aibox-local.toml — personal servers
[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]
```

## Installation

Codex CLI is installed via npm (`npm install -g @openai/codex`). To pin a specific version, set it in `aibox.toml`:

```toml
[addons.ai-codex.tools]
codex = { version = "0.1.0" }
```
