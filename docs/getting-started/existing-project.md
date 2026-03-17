# Existing Project

This guide covers adding dev-box to a project that already exists.

## Create dev-box.toml

If your project does not yet have a `dev-box.toml`, create one manually or use `init`:

```bash
cd my-existing-project
dev-box init --name my-existing-project --image python --process managed
```

!!! warning "init will not overwrite"
    If `dev-box.toml` already exists, `init` will refuse to run. Either delete the existing file or edit it directly.

If you prefer to write it by hand:

```toml
[dev-box]
version = "0.3.2"
image = "python"
process = "managed"

[container]
name = "my-existing-project"
hostname = "my-existing-project"
ports = []
extra_packages = []
environment = {}

[context]
owner = "~/.config/dev-box/OWNER.md"
schema_version = "1.0.0"

[audio]
enabled = false
```

## Generate Devcontainer Files

Run `generate` to create the `.devcontainer/` directory from your config:

```bash
dev-box generate
```

This creates:

- `.devcontainer/Dockerfile`
- `.devcontainer/docker-compose.yml`
- `.devcontainer/devcontainer.json`

## Replacing Hand-Written Devcontainer Files

If your project already has a `.devcontainer/` directory with hand-written files, you have two options:

### Option A: Let dev-box take over

1. Back up your existing files:
   ```bash
   cp -r .devcontainer .devcontainer.bak
   ```
2. Run `dev-box generate` -- it will overwrite the existing files
3. Move any custom configuration into `dev-box.toml`:
   - Extra apt packages go in `container.extra_packages`
   - Port mappings go in `container.ports`
   - Environment variables go in `container.environment`
4. Rebuild: `dev-box build --no-cache`

### Option B: Keep hand-written files

If your devcontainer setup is heavily customized, you can still use dev-box for context management and skip the container lifecycle commands. Just use `dev-box.toml` for the `[dev-box]` and `[context]` sections, and manage `.devcontainer/` yourself.

## Running Diagnostics

Use `doctor` to validate your project structure:

```bash
dev-box doctor
```

This checks:

- Config file validity and version
- Container runtime availability (podman or docker)
- `.root/` directory existence
- `.devcontainer/` directory existence
- Image and process settings

Example output:

```
==> Running diagnostics...
 ✓ Config version: 0.3.2
 ✓ Image: python
 ✓ Process: managed
 ✓ Container name: my-existing-project
 ✓ Container runtime: podman
 ✓ .root/ directory exists at .root
 ✓ .devcontainer/ directory exists
 ✓ Diagnostics complete
```

## Migrating Context Structure

If your project already has context files (like `DECISIONS.md` or `BACKLOG.md`) that predate dev-box, `doctor` can help identify what needs to change. See [Migration](../context/migration.md) for the full guide.

## Real-World Migration Examples

The `migrations/` directory in the dev-box repository contains ready-made
`dev-box.toml` files and step-by-step migration guides for several projects:

| Project | Image | Process | Key considerations |
|---------|-------|---------|--------------------|
| kaits | python | product | Node.js via extra_packages, Playwright post-install, audio |
| internal | python-latex | product | TeX Live + Python combo, git identity in postCreateCommand |
| kubernetes-learning | python-latex | research | Gemini/Jules CLI as extra volumes, audio, release script |
| ai-learning | python-latex | research | Existing context/ maps cleanly to research flavor |
| vim-cheatsheet | python-latex | research | Default branch is master, shared image name |

### Common gaps to watch for

- **Node.js version pinning** -- `extra_packages` installs the Debian version, not NodeSource LTS. Pin via a post-create script if needed.
- **postCreateCommand** -- use `post_create_command` in `[container]` config. For git identity, use `.root/.config/git/config` instead.
- **VS Code extensions/settings** -- use `vscode_extensions` in `[container]` config to add project-specific extensions. For project-specific settings, keep a `.vscode/settings.json`.
- **Third-party CLI tools** (Gemini, Jules) -- install via extra_packages or mount from host via extra_volumes.

## Build and Start

Once `dev-box.toml` and `.devcontainer/` are in place:

```bash
dev-box build
dev-box start
```

The workflow is identical to a [new project](new-project.md#build-and-start) from this point forward.

## Next Steps

- [Configuration reference](../cli/configuration.md)
- [CLI commands](../cli/commands.md)
- [Context migration guide](../context/migration.md)
