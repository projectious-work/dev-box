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

Codex has a native MCP client. aibox writes processkit MCP server registrations to `.codex/config.toml` automatically on `aibox sync`.

## Installation

Codex CLI is installed via npm (`npm install -g @openai/codex`).
