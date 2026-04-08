---
sidebar_position: 11
title: "Continue"
---

# Continue CLI

[Continue](https://github.com/continuedev/continue) is an open-source, provider-agnostic coding agent CLI. Designed for headless environments and containers (Apache 2.0).

## Setup

```toml
[ai]
providers = ["continue"]
```

Run `aibox sync`, then inside the container:

```bash
cn          # Interactive mode
cn -p "..."  # Headless/non-interactive mode (great for scripts and CI)
```

Note: the binary is `cn`, not `continue`.

## API Key

Continue is provider-agnostic — configure the LLM you want to use:

```toml
[container.environment]
CONTINUE_API_KEY = "..."   # Generic key for headless use
# Or provider-specific:
# ANTHROPIC_API_KEY = "sk-ant-..."
# OPENAI_API_KEY = "sk-..."
```

## Configuration

Continue's configuration is persisted in `.aibox-home/.continue/`, mounted at `/home/aibox/.continue/`.

## MCP Integration

Continue has a native MCP client with a per-server file model. aibox writes processkit MCP server registrations to `.continue/mcpServers/` automatically on `aibox sync` (one file per server).

## Installation

Continue CLI is installed via npm (`npm install -g @continuedev/cli`).
