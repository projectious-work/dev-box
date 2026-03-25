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

## Installation

Gemini CLI is installed via npm (`npm install -g @google/generative-ai-cli`), with a pip fallback.
