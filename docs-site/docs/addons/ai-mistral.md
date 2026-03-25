---
sidebar_position: 8
title: "Mistral"
---

# Mistral

[Mistral AI](https://mistral.ai) provides large language models. The Mistral SDK enables CLI-based interaction.

## Setup

```toml
[ai]
providers = ["mistral"]
```

Run `aibox sync`, then inside the container the Mistral Python SDK is available for scripting and CLI use.

## API Key

```toml
[container.environment]
MISTRAL_API_KEY = "..."
```

## Installation

The Mistral AI SDK is installed via pip (`pip install --no-cache-dir mistralai`).
