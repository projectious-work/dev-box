---
sidebar_position: 9
title: "Codex (OpenAI)"
---

# OpenAI Codex CLI

[Codex CLI](https://github.com/openai/codex) is OpenAI's open-source terminal coding agent. Built in Rust, GA since April 2025.

## Setup

```toml
[ai]
providers = ["codex"]
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

Codex's configuration is persisted in `.aibox-home/.codex/`, mounted at `/home/aibox/.codex/`.

Key files:
- `.codex/config.toml` — Codex settings and model preferences
- `.codex/rules/` — Custom coding rules
- `.codex/sessions/` — Session history

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

Codex CLI is installed via npm (`npm install -g @openai/codex`).
