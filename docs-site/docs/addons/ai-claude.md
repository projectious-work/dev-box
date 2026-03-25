---
sidebar_position: 5
title: "Claude"
---

# Claude Code

[Claude Code](https://claude.ai) by Anthropic is a terminal-based AI coding assistant. It's the default provider in aibox.

## Setup

```toml
[ai]
providers = ["claude"]
```

Run `aibox sync`, then inside the container:

```bash
claude   # Launches Claude Code CLI
```

On first launch, Claude prompts for authentication via browser login.

## Configuration

Claude's configuration and memory are persisted in `.aibox-home/.claude/`, which is mounted into the container at `/home/aibox/.claude/`.

Key files:
- `.claude/settings.json` — Claude Code settings
- `.claude/projects/` — Per-project memory and context

## Audio (Voice)

Claude Code supports voice input. To enable it, configure [audio bridging](../container/audio.md):

```toml
[audio]
enabled = true
```

## Zellij Integration

When Claude is configured as a provider, Zellij layouts include a dedicated Claude pane:

- **dev layout:** Claude gets its own tab
- **focus layout:** Claude gets its own tab
- **cowork layout:** Claude appears in a side-by-side pane next to the editor
