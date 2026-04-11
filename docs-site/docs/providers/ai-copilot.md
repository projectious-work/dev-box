---
sidebar_position: 10
title: "Copilot (GitHub)"
---

# GitHub Copilot CLI

[GitHub Copilot CLI](https://github.com/github/copilot-cli) is GitHub's terminal coding agent. GA since February 2026, validated as a Dev Container feature.

## Setup

```toml
[ai]
providers = ["copilot"]
```

Run `aibox sync`, then inside the container:

```bash
copilot /login   # Authenticate on first launch
copilot          # Launches GitHub Copilot CLI
```

## Requirements

A GitHub Copilot subscription (Individual, Business, or Enterprise) is required.

## Configuration

Copilot's configuration is persisted in `.aibox-home/.copilot/`, mounted at `/home/aibox/.copilot/`.

Key files:
- `.copilot/config.json` — Copilot settings (overridable via `COPILOT_HOME`)

## MCP Integration

GitHub Copilot CLI reads `.mcp.json` (the Claude Code MCP format). aibox generates `.mcp.json` automatically on `aibox sync`, merging processkit built-in servers, team servers from `aibox.toml [mcp]`, and personal servers from `.aibox-local.toml [mcp]`.

`.mcp.json` is **gitignored** — it is regenerated on every `aibox sync` and must not be committed.

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

GitHub Copilot CLI is installed via npm (`npm install -g @github/copilot`).
