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

GitHub Copilot CLI reads `.mcp.json` (the Claude Code MCP format). aibox writes processkit MCP server registrations to `.mcp.json` automatically on `aibox sync`.

## Installation

GitHub Copilot CLI is installed via npm (`npm install -g @github/copilot`).
