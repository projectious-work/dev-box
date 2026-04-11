---
sidebar_position: 8
title: "Mistral"
---

# Mistral (SDK)

:::note SDK addon — not an interactive CLI
The `ai-mistral` addon installs the **mistralai Python SDK**, not an interactive coding CLI. It is intended for projects that call the Mistral API programmatically. For an interactive coding experience, use [Claude](./ai-claude.md), [Gemini](./ai-gemini.md), [Codex](./ai-codex.md), or [Copilot](./ai-copilot.md) instead.
:::

[Mistral AI](https://mistral.ai) provides large language models via Python SDK.

## Setup

```toml
[ai]
providers = ["mistral"]
```

Run `aibox sync`. Inside the container the `mistralai` Python SDK is available for scripting:

```python
from mistralai import Mistral
client = Mistral(api_key="...")
```

## API Key

```toml
[container.environment]
MISTRAL_API_KEY = "..."
```

## MCP Integration

aibox generates `.mcp.json` (the Claude Code MCP format) on `aibox sync`, merging processkit built-in servers, team servers from `aibox.toml [mcp]`, and personal servers from `.aibox-local.toml [mcp]`. A custom Mistral SDK-based tool you build can read MCP server registrations from this file.

`.mcp.json` is **gitignored** — it is regenerated on every `aibox sync` and must not be committed.

## Installation

The Mistral AI SDK is installed via pip (`pip install --no-cache-dir mistralai`).
