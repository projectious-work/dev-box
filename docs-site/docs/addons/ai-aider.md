---
sidebar_position: 6
title: "Aider"
---

# Aider

[Aider](https://aider.chat) is an open-source AI pair programming tool that works with multiple LLM providers from the terminal.

## Setup

```toml
[ai]
providers = ["aider"]
```

Run `aibox sync`, then inside the container:

```bash
aider    # Launches Aider CLI
```

## API Key

Aider requires an API key for the LLM provider you want to use. Set it in your environment:

```toml
[container.environment]
ANTHROPIC_API_KEY = "sk-ant-..."
# Or for OpenAI:
# OPENAI_API_KEY = "sk-..."
```

Alternatively, create a `.aider.conf.yml` in `.aibox-home/.aider/`.

## Configuration

Aider's configuration is persisted in `.aibox-home/.aider/`, mounted at `/home/aibox/.aider/`.

## Installation

Aider is installed via `uv tool install aider-chat` — a fast, isolated Python tool installation.
