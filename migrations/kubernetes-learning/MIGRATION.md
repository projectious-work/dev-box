# Migration Guide: kubernetes-learning

## Current Setup
- Image: Custom multi-stage Dockerfile based on `debian:trixie-slim`
- Stage 1: Vanilla TeX Live from TUG (scheme-basic + extensive package list via `tlmgr`)
- Stage 2: Runtime with Python 3, pip, npm, inkscape, PulseAudio/sox, MkDocs Material
- AI tools installed in image: Claude CLI, Gemini CLI (`@google/gemini-cli`), Jules (`@google/jules`)
- Audio support: PulseAudio TCP bridging to macOS host
- Extra volume mounts: `.asoundrc`, `.gemini`, `.config/google-jules`
- Container image name: `localhost/kubernetes-learning:latest`

## dev-box Configuration
- Image flavor: `python-latex` (covers Python + pip + TeX Live)
- Process flavor: `research` (learning/research project with CLAUDE.md)
- Audio: yes (PulseAudio TCP bridging for Claude voice support)

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
- The project has a `CLAUDE.md` at the root. dev-box expects this file; no changes needed.
- There is no `context/` directory. The `research` process flavor may scaffold one; review scaffolded files and populate as needed.
- `scripts/release.sh` (Codeberg release script) is project-specific and should be kept as-is.

## Gaps and Manual Steps

### TeX Live package set
The current Dockerfile installs vanilla TeX Live from TUG with a curated package list (LuaLaTeX, biblatex/biber, tcolorbox, tikz/pgf, minted, fontspec, etc.). The dev-box `python-latex` image must include an equivalent TeX Live installation. Verify that all required packages are present by running `latexmk` on the project after migration. Missing packages can be added at runtime via `tlmgr install <pkg>`.

### Gemini CLI and Jules
The current Dockerfile installs `@google/gemini-cli` and `@google/jules` globally via npm. These are not part of the dev-box base images. After migration, you must either:
1. Add them to `extra_packages` if dev-box supports npm global installs (currently it does not), or
2. Install them manually inside the container: `npm install -g @google/gemini-cli @google/jules`
3. Alternatively, create a post-create script that runs these installs

### MkDocs Material
The current Dockerfile installs `mkdocs-material` via pip. The dev-box `python-latex` image should ideally include this, or it must be installed via a Python virtualenv / pip at container start.

### inkscape
Listed in `extra_packages` but is a large dependency. Verify it is available in the dev-box base image's apt sources.

### .asoundrc file
The `.root/.asoundrc` file must exist on the host before first start. Seed it manually:
```bash
mkdir -p .root
cat > .root/.asoundrc <<'EOF'
pcm.!default {
    type pulse
}
ctl.!default {
    type pulse
}
EOF
```

### Gemini and Jules config directories
Seed these on the host before first start:
```bash
mkdir -p .root/.gemini
mkdir -p .root/.config/google-jules
```
