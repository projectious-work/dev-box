# Migration Guide: internal

Migration from hand-written devcontainer to dev-box managed configuration.

---

## Current Setup Summary

- **Base:** Debian Trixie Slim with multi-stage build (TeX Live builder + runtime)
- **TeX Live:** Vanilla TUG install with scheme-basic + extensive package list (LuaLaTeX, biblatex/biber, TikZ/PGF, tcolorbox, fontspec, etc.)
- **Languages:** Python 3 (system), uv package manager, Node.js 20 (NodeSource)
- **Tools:** gh, inkscape, mkdocs-material, claude CLI, poppler-utils, Playwright system deps
- **Audio:** Not configured
- **Config persistence:** `.root/` directory with bind mounts for SSH, claude, git
- **postCreateCommand:** Sets git identity (`info@projectious.work` / `Projectious`)
- **Context directory:** Already has BACKLOG, DECISIONS, OWNER, PROJECTS, STANDUPS, project-notes/

---

## Step 1: Files to Delete

These files will be replaced by `dev-box generate`:

```
.devcontainer/Dockerfile
.devcontainer/docker-compose.yml
.devcontainer/devcontainer.json
```

> **Back up first** if you have any local modifications not committed to git. The Dockerfile contains a carefully curated TeX Live package list -- save this for reference.

---

## Step 2: Files to Keep

These are project-specific and should remain untouched:

```
context/                           (OWNER, BACKLOG, DECISIONS, PROJECTS, STANDUPS, project-notes/)
projects/                          (project content)
docs/                              (MkDocs content)
CLAUDE.md                          (project instructions)
.claude/                           (Claude settings checked into repo)
.root/                             (persisted host config -- SSH keys, git, claude)
```

---

## Step 3: Add dev-box.toml

Copy the generated `dev-box.toml` from this directory into the project root:

```bash
cp dev-box.toml /path/to/internal/dev-box.toml
```

---

## Step 4: Generate and Verify

```bash
cd /path/to/internal
dev-box generate       # Regenerates .devcontainer/ from dev-box.toml
dev-box doctor         # Validates the configuration and checks for issues
```

---

## Step 5: Manual Adjustments Required

### TeX Live Package List

This is the most critical migration concern. The current Dockerfile installs a curated set of TeX Live packages from the vanilla TUG installer. The `python-latex` image must include an equivalent TeX Live installation. Verify that the following package groups are present:

- **Engine:** LuaLaTeX (luatex, luahbtex, luaotfload, latexmk)
- **Fonts:** fontspec, lm, lm-math, amsfonts, unicode-math, gnu-freefont, sourcecodepro, tex-gyre, emoji/twemoji-colr
- **Languages:** babel (english, french, german), csquotes, hyphenation patterns
- **Graphics:** xcolor, TikZ/PGF, pgfplots, svg, tcolorbox
- **Tables:** tabularray, ninecolors, booktabs
- **Code listings:** minted, fancyvrb, listings
- **Bibliography:** biblatex + biber
- **Math:** amsmath, nicematrix, siunitx, unicode-math
- **Misc:** hyperref, geometry, fancyhdr, enumitem, microtype, bytefield

If the `python-latex` image uses a different TeX Live scheme or is missing packages, you will need to add a `tlmgr install` step via a post-create hook or custom layer.

### Node.js 20 LTS

Same concern as kaits: the Debian Trixie `nodejs` package may not be version 20. The current Dockerfile uses the NodeSource repo for Node 20 specifically. Check if the version from `extra_packages` is sufficient.

### Git Identity (postCreateCommand)

The current `devcontainer.json` runs:

```bash
git config --global user.email 'info@projectious.work'
git config --global user.name 'Projectious'
```

dev-box does not currently support `postCreateCommand`. Workarounds:

1. **Bake it into `.root/.config/git/config`** on the host (recommended). Add:
   ```ini
   [user]
       email = info@projectious.work
       name = Projectious
   ```
2. **Add a shell rc hook** that sets git identity on container start.
3. **Wait for dev-box post-create hook support** (future feature).

### inkscape

Currently installed for SVG-to-PDF conversion in LaTeX documents. Listed in `extra_packages` in the toml. Verify it installs correctly alongside the TeX Live tree.

### Playwright System Dependencies

The current Dockerfile installs ~20 system libraries for headless Chromium (Excalidraw diagram rendering). These are listed in `extra_packages` in the toml. The actual Chromium browser binary is installed per-skill via `uv run playwright install chromium` and is not part of the image.

### MkDocs Material

Currently installed via `pip install --break-system-packages mkdocs-material==9.7.4` (pinned). Move this dependency to a `pyproject.toml` or `requirements.txt` and install via `uv sync` / `uv pip install`.

### VS Code Extensions

The current `devcontainer.json` installs:
- `james-yu.latex-workshop`
- `mblode.zotero`

Plus LaTeX Workshop settings for LuaLaTeX recipes and synctex. Verify that `dev-box generate` provides a way to specify VS Code extensions and settings, or add them to a workspace `.vscode/settings.json`.

---

## Context Directory Comparison

internal already has a `context/` directory:

| File | Status |
|------|--------|
| `OWNER.md` | Exists |
| `BACKLOG.md` | Exists |
| `DECISIONS.md` | Exists |
| `PROJECTS.md` | Exists |
| `STANDUPS.md` | Exists |
| `project-notes/` | Exists |
| `PRD.md` | Not present (may not be needed for an internal ops repo) |
| `RESEARCH.md` | Not present |

dev-box scaffolding should detect existing files and skip them. Missing files (PRD, RESEARCH) can be created later if the process type calls for them.

---

## Features Not Yet Supported by dev-box

- **postCreateCommand**: Needed for git identity setup. See workaround above.
- **Custom TeX Live package management**: If the `python-latex` image does not match the exact package list, there is no declarative way to specify additional `tlmgr install` packages in `dev-box.toml`.
- **Pinned image digests**: The current Dockerfile pins the Debian base image by SHA256 digest for reproducibility. Verify whether dev-box images support digest pinning.
- **VS Code extension settings**: LaTeX Workshop requires specific tool/recipe configuration. These may need to live in `.vscode/settings.json` rather than in `devcontainer.json`.
- **Workspace path**: The current setup uses `/workspaces` (plural) as WORKDIR, which differs from dev-box's `/workspace`. Update any scripts or configs that reference the workspace path.
