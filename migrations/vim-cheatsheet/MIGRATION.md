# Migration Guide: vim-cheatsheet

## Current Setup
- Image: Custom multi-stage Dockerfile based on `debian:trixie-slim`
- TeX Live (LuaLaTeX) + Python 3 + MkDocs Material + Claude CLI
- No audio support in current compose
- Volumes: SSH (read-only), Claude config, Git config
- Container image name: `localhost/latex-base:latest`
- Default branch: `master` (not `main`)
- Has `CLAUDE.md` and `context/analysis/` directory with 3 files

## dev-box Configuration
- Image flavor: `python-latex` (covers Python + pip + TeX Live)
- Process flavor: `research` (learning/research project with existing context directory)
- Audio: yes (standardized baseline; can be disabled if not needed)

## Migration Steps

### 1. Install dev-box CLI
```
cargo install --path /path/to/dev-box/cli
```

### 2. Add dev-box.toml
Copy the `dev-box.toml` from this migration directory to the project root.

### 3. Generate devcontainer files
```
dev-box generate
```

### 4. Remove old files
- [ ] Delete `.devcontainer/Dockerfile` (now generated from the `python-latex` image)
- [ ] Delete `.devcontainer/docker-compose.yml` (now generated)
- [ ] Delete `.devcontainer/devcontainer.json` (now generated)
- [ ] Delete `.devcontainer/config/` if present (configs now in published image)
- [ ] Delete `scripts/dev.sh` if present (replaced by `dev-box` CLI)

### 5. Run diagnostics
```
dev-box doctor
```

### 6. Context adjustments
- The project already has a `CLAUDE.md` at the root. No changes needed.
- The project has a `context/analysis/` directory with 3 files. This is a subdirectory layout; verify that the `research` process flavor's scaffolding does not conflict with the existing `context/` structure. Existing files should be preserved.
- Note: the default branch is `master`, not `main`. Ensure any dev-box git integrations or CI templates account for this.

## Gaps and Manual Steps

### TeX Live package set
The current Dockerfile builds TeX Live from TUG with a curated package list for LuaLaTeX. The dev-box `python-latex` image must include an equivalent installation. Test by running `latexmk` on the project after migration. Missing packages can be added via `tlmgr install <pkg>`.

### MkDocs Material
The current Dockerfile installs `mkdocs-material` via pip. This must be available in the dev-box `python-latex` image or installed separately at container start (e.g., via `pip install --break-system-packages mkdocs-material` or a post-create script).

### Claude CLI
The current Dockerfile installs Claude CLI via `curl -fsSL https://claude.ai/install.sh | bash`. Verify that the dev-box base image includes this, or add it as a post-create step.

### Shared image name
The current compose uses `localhost/latex-base:latest` as the image name, suggesting this image may be shared across multiple projects. After migration, each project gets its own generated Dockerfile. If you were previously sharing a single built image, each project will now build independently.

### No extra volumes needed
The standard SSH, Claude, and Git mounts are handled by dev-box automatically. No additional volume configuration is required.
