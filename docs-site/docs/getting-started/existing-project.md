---
sidebar_position: 3
title: "Existing Project"
---

# Existing Project

This guide covers adding aibox to a project that already exists.

## Create aibox.toml

If your project does not yet have a `aibox.toml`, create one manually or use `init`:

```bash
cd my-existing-project
aibox init --name my-existing-project --image python --process managed
```

:::warning init will not overwrite

If `aibox.toml` already exists, `init` will refuse to run. Either delete the existing file or edit it directly.

:::

If you prefer to write it by hand:

```toml
[aibox]
version = "0.8.0"
image = "python"
process = "managed"

[container]
name = "my-existing-project"
hostname = "my-existing-project"
ports = []
extra_packages = []
environment = {}

[context]
schema_version = "1.0.0"

[ai]
providers = ["claude"]

[audio]
enabled = false
```

## Sync Devcontainer Files

Run `sync` to create the `.devcontainer/` directory from your config:

```bash
aibox sync
```

This creates:

- `.devcontainer/Dockerfile`
- `.devcontainer/docker-compose.yml`
- `.devcontainer/devcontainer.json`

## Replacing Hand-Written Devcontainer Files

If your project already has a `.devcontainer/` directory with hand-written files, you have two options:

### Option A: Let aibox take over

1. Back up your existing files:
   ```bash
   cp -r .devcontainer .devcontainer.bak
   ```
2. Run `aibox sync` -- it will overwrite the existing files
3. Move any custom configuration into `aibox.toml`:
   - Extra apt packages go in `container.extra_packages`
   - Port mappings go in `container.ports`
   - Environment variables go in `container.environment`
4. Rebuild: `aibox sync --no-cache`

### Option B: Keep hand-written files

If your devcontainer setup is heavily customized, you can still use aibox for context management and skip the container lifecycle commands. Just use `aibox.toml` for the `[aibox]` and `[context]` sections, and manage `.devcontainer/` yourself.

## Running Diagnostics

Use `doctor` to validate your project structure:

```bash
aibox doctor
```

This checks:

- Config file validity and version
- Container runtime availability (podman or docker)
- `.aibox-home/` directory existence
- `.devcontainer/` directory existence
- Image and process settings

Example output:

```
==> Running diagnostics...
 ✓ Config version: 0.8.0
 ✓ Image: python
 ✓ Process: managed
 ✓ Container name: my-existing-project
 ✓ Container runtime: podman
 ✓ .aibox-home/ directory exists at .aibox-home
 ✓ .devcontainer/ directory exists
 ✓ Diagnostics complete
```

## Migrating from `.root/` to `.aibox-home/`

If you are upgrading from aibox ≤ v0.3.4, the persisted config directory was renamed from `.root/` to `.aibox-home/`. This directory is **gitignored and not tracked**, so use a plain filesystem rename:

```bash
mv .root .aibox-home
```

:::warning Do not use `git mv`

`git mv .root .aibox-home` will fail because `.root/` is listed in `.gitignore` and was never committed. Use a regular `mv` command.

:::

aibox will fall back to `.root/` automatically if `.aibox-home/` does not exist, so this migration is optional but recommended.

## Migrating Context Structure

If your project already has context files (like `DECISIONS.md` or `BACKLOG.md`) that predate aibox, `doctor` can help identify what needs to change. See [Migration](../context/migration.md) for the full guide.

## Real-World Migration Examples

The `migrations/` directory in the aibox repository contains ready-made
`aibox.toml` files and step-by-step migration guides for several projects:

| Project | Image | Process | Key considerations |
|---------|-------|---------|--------------------|
| kaits | python | product | Node.js via extra_packages, Playwright post-install, audio |
| internal | python-latex | product | TeX Live + Python combo, git identity in postCreateCommand |
| kubernetes-learning | python-latex | research | Gemini/Jules CLI as extra volumes, audio, release script |
| ai-learning | python-latex | research | Existing context/ maps cleanly to research flavor |
| vim-cheatsheet | python-latex | research | Default branch is master, shared image name |

### Common gaps to watch for

- **Node.js version pinning** -- `extra_packages` installs the Debian version, not NodeSource LTS. Pin via a post-create script if needed.
- **postCreateCommand** -- use `post_create_command` in `[container]` config. For git identity, use `.aibox-home/.config/git/config` instead.
- **VS Code extensions/settings** -- use `vscode_extensions` in `[container]` config to add project-specific extensions. For project-specific settings, keep a `.vscode/settings.json`.
- **Third-party CLI tools** (Gemini, Jules) -- install via extra_packages or mount from host via extra_volumes.

## Build and Start

Once `aibox.toml` and `.devcontainer/` are in place:

```bash
aibox sync     # Regenerate files and build image
aibox start    # Start and attach
```

The workflow is identical to a [new project](new-project.md#build-and-start) from this point forward.

## Next Steps

- [Configuration reference](../reference/configuration.md)
- [CLI commands](../reference/cli-commands.md)
- [Context migration guide](../context/migration.md)
