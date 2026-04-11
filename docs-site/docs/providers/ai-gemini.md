---
sidebar_position: 7
title: "Gemini"
---

# Gemini

[Gemini CLI](https://github.com/google-gemini/gemini-cli) is Google's command-line interface for Gemini AI models.

## Setup

```toml
[ai]
providers = ["gemini"]
```

Run `aibox sync`, then inside the container:

```bash
gemini   # Launches Gemini CLI
```

## API Key

```toml
[container.environment]
GOOGLE_API_KEY = "..."
```

## Configuration

Gemini's configuration is persisted in `.aibox-home/.gemini/`, mounted at `/home/aibox/.gemini/`.

## MCP Integration

Gemini CLI reads `.gemini/settings.json`. aibox generates this file automatically on `aibox sync`, merging processkit built-in servers, team servers from `aibox.toml [mcp]`, and personal servers from `.aibox-local.toml [mcp]`.

`.gemini/settings.json` is **gitignored** — it is regenerated on every `aibox sync` and must not be committed.

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

Gemini CLI is installed via npm (`npm install -g @google/generative-ai-cli`), with a pip fallback.
