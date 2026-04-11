---
sidebar_position: 99
title: "Compatibility"
---

# Compatibility

## aibox ↔ processkit Version Matrix

Each aibox release is tested against a specific processkit version. The table
below shows the minimum compatible processkit version for each aibox release.

| aibox version | Min. processkit | Notes |
|--------------|-----------------|-------|
| 0.17.16 | v0.13.0 | **BREAKING**: rename `providers = ["codex"]` → `["openai"]`; fix zellij `--layout` flag; fix Rust x86_64 cross-compile target |
| 0.17.15 | v0.13.0 | MCP config model, zjstatus hints, Zellij Ctrl+q, processkit v0.13.0 |
| 0.17.5 | v0.8.0 | processkit v0.8.0 GrandLily src/ restructure |
| 0.17.4 | v0.6.0 | content migration documents (pending/in-progress/applied) |
| 0.17.3 | v0.6.0 | Claude Code slash-command adapters |
| 0.17.2 | v0.6.0 | core skill enforcement, processkit v0.6.0 compat |
| 0.17.0 | v0.5.0 | aibox.lock sectioned format |
| 0.16.1 | v0.4.0 | sync auto-install added |
| 0.16.0 | v0.4.0 | initial processkit integration |

## How compatibility is enforced

`aibox sync` compares the `[processkit].version` in your `aibox.toml` against
the minimum required version for the running aibox binary. If the pinned
processkit version is older than the minimum, a warning is emitted:

```
Warning: processkit v0.6.0 is below the minimum recommended version v0.8.0 for aibox v0.17.5 ...
```

This is a warning, not an error — older processkit versions can still install
successfully (the installer handles both v0.7.0 and v0.8.0 layouts). The warning
is a nudge to upgrade, not a blocker.

## Upgrading processkit

To upgrade processkit in an existing project:

1. Edit `aibox.toml`:
   ```toml
   [processkit]
   version = "v0.8.0"
   ```

2. Run `aibox sync` on the host — the 3-way diff will show changed content
   and generate processkit content migration documents in `context/migrations/pending/`.

3. Review and apply the pending migrations.
