# Migration Guide: kaits

Migration from hand-written devcontainer to dev-box managed configuration.

---

## Current Setup Summary

- **Base:** Debian Trixie Slim with multi-stage build (Zellij builder + runtime)
- **Languages:** Python 3 (system), uv package manager, Node.js (via apt)
- **Tools:** gh, vim, lazygit, zellij, mkdocs-material, playwright + chromium, claude CLI, poethepoet
- **Audio:** PulseAudio bridging for Claude voice support
- **Config persistence:** `.root/` directory with bind mounts for SSH, vim, zellij, git, claude
- **Context directory:** Already has BACKLOG, DECISIONS, OWNER, PRD, PROJECTS, RESEARCH, STANDUPS, ideas/, work-instructions/

---

## Step 1: Files to Delete

These files will be replaced by `dev-box generate`:

```
.devcontainer/Dockerfile
.devcontainer/docker-compose.yml
.devcontainer/devcontainer.json
.devcontainer/config/              (entire directory — vimrc, gitconfig, zellij configs)
scripts/dev.sh                     (replaced by dev-box CLI)
```

> **Back up first** if you have any local modifications to these files that are not committed.

---

## Step 2: Files to Keep

These are project-specific and should remain untouched:

```
context/                           (OWNER, BACKLOG, DECISIONS, PRD, PROJECTS, RESEARCH, STANDUPS, etc.)
agents/                            (agent YAML configs)
pyproject.toml                     (project dependencies, Poe tasks)
rxconfig.py                        (Reflex configuration)
mkdocs.yml                         (documentation config)
kaits/                             (application source)
tests/                             (test suite)
docs/                              (MkDocs content)
CLAUDE.md                          (project instructions)
.root/                             (persisted host config — SSH keys, vim, git, etc.)
```

---

## Step 3: Add dev-box.toml

Copy the generated `dev-box.toml` from this directory into the project root:

```bash
cp dev-box.toml /path/to/kaits/dev-box.toml
```

---

## Step 4: Generate and Verify

```bash
cd /path/to/kaits
dev-box generate       # Regenerates .devcontainer/ from dev-box.toml
dev-box doctor         # Validates the configuration and checks for issues
```

---

## Step 5: Manual Adjustments Required

### Node.js 20 LTS

The standard `python` image does not include Node.js. The current Dockerfile installs Node.js 20 from the NodeSource apt repository. With dev-box, you have two options:

1. **List `nodejs` in `extra_packages`** (already done in the toml) — this installs whatever version is in the Debian Trixie repos, which may not be Node 20 LTS.
2. **If Node 20 specifically is required**, you may need a post-generate hook or a custom Dockerfile layer to add the NodeSource repo. Check if the Debian Trixie `nodejs` package version is acceptable for Reflex builds.

### Playwright + Chromium

The current Dockerfile runs `pip install playwright && playwright install --with-deps chromium`. With dev-box:

- The system-level Playwright dependencies are covered by `extra_packages` entries (or will need to be if the python image does not include them).
- You will still need to run `playwright install chromium` inside the container after creation. Consider adding this to a post-create script or documenting it in your project README.

### poethepoet (Poe task runner)

Currently installed via `uv tool install poethepoet`. This is not handled by dev-box and should be installed as part of your project setup (e.g., in a post-create script or via `uv sync` from pyproject.toml dev dependencies).

### MkDocs Material

Currently installed via `pip install --break-system-packages mkdocs-material`. With dev-box, either:
- Add it to your `pyproject.toml` dependencies and install via `uv sync`, or
- Document it as a manual post-create step.

### PulseAudio / Audio

The `dev-box.toml` has `[audio] enabled = true` and the PulseAudio environment variables are set. Ensure the `.root/.asoundrc` file is still seeded on first run. The host-side PulseAudio daemon setup remains unchanged:

```bash
brew install pulseaudio
pulseaudio --load=module-native-protocol-tcp auth-ip-acl=0.0.0.0/0 auth-anonymous=1 port=4714 --exit-idle-time=-1 --daemon
```

### VS Code Extensions

The current `devcontainer.json` installs these extensions:
- `charliermarsh.ruff`
- `bradlc.vscode-tailwindcss`
- `redhat.vscode-yaml`

Verify that `dev-box generate` carries these forward, or add them to a VS Code workspace settings file.

---

## Context Directory Comparison

kaits already has a well-populated `context/` directory:

| File | Status |
|------|--------|
| `OWNER.md` | Exists |
| `BACKLOG.md` | Exists |
| `DECISIONS.md` | Exists |
| `PRD.md` | Exists |
| `PROJECTS.md` | Exists |
| `RESEARCH.md` | Exists |
| `STANDUPS.md` | Exists |
| `ideas/` | Exists |
| `work-instructions/` | Exists |

dev-box scaffolding should detect these and skip creation. No action needed.

---

## Features Not Yet Supported by dev-box

- **postCreateCommand**: The current devcontainer.json has an empty `postCreateCommand`. No migration concern, but if it were populated, dev-box would need a hook mechanism.
- **Custom Zellij layouts**: The current setup uses a project-specific `dev.kdl` layout with tabs for dev/git/shell. Verify that dev-box provides equivalent layout customization or allows overrides.
- **VS Code terminal profiles**: The current setup configures zellij, bash, and claude as terminal profiles. Check if dev-box generates equivalent profiles.
