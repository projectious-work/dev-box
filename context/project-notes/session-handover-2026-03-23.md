# Session Handover — 2026-03-23

## What Was Done

### BACK-038: Rename dev-box → aibox (Phases 0–4)

Full project rename completed across the entire codebase in 5 commits:

1. **Phase 0** (`b5b64f8`) — Removed old Zensical docs (`docs/`, `zensical.toml`) — 57 files deleted, reducing rename surface by ~413 occurrences. Cleaned `.gitignore`.
2. **Phase 1** (`c128273`) — Rust codebase: renamed all structs (`DevBoxConfig` → `AiboxConfig`), constants, env vars (`DEV_BOX_*` → `AIBOX_*`, `DEVBOX_*` → `AIBOX_*`), string literals, templates, and integration tests across 35 files. Binary is now `aibox`. All 239 tests pass, clippy clean.
3. **Phase 2** (`699a7ef`) — Renamed `dev-box.toml` → `aibox.toml`, updated TOML section `[dev-box]` → `[aibox]`. Added Docusaurus build artifacts to `.gitignore`.
4. **Phase 3** (`a0274a6`) — Renamed in 46 files: image configs, `.devcontainer`, scripts (`maintain.sh`, `install.sh`, `build-macos.sh`), process templates, schemas, and all Docusaurus docs.
5. **Phase 4** (`03123d6`) — Renamed in 27 meta-documents: `CLAUDE.md`, `README.md`, backlog, decisions, PRD, work instructions, project notes, research docs.

**Final grep confirms zero remaining `dev-box`/`DevBox`/`DEVBOX`/`dev_box` occurrences** in tracked source files (only internal variable names `devbox` in test scripts remain, which is fine).

### Backlog Maintenance
- Verified BACK-036 (yazi editor key) and BACK-037 (yazi preview) already fixed in `06d9505` — marked done
- Marked BACK-021 (Zensical → Docusaurus migration) as done
- Filed BACK-040: Analyse base image Dockerfile for multistage build optimization (Node/Bun focus)
- Filed BACK-041: Backlog structure — separate active from archive for efficient reads
- Updated BACK-038 with phased plan

## What Needs to Happen Next

### Immediate: Phase 5 (GitHub — manual, by owner)
1. Rename GitHub repo `projectious-work/dev-box` → `projectious-work/aibox`
2. Update local git remote: `git remote set-url origin git@github.com:projectious-work/aibox.git`
3. Push all commits: `git push origin main`
4. Clean up old GHCR images under `dev-box` path
5. Rebuild and push images under `ghcr.io/projectious-work/aibox`
6. Tag a new release (e.g., v0.9.0 or v1.0.0)
7. Rebuild and deploy docs to GitHub Pages (new URL: `projectious-work.github.io/aibox/`)
8. Verify: install script, `aibox update --check`, docs site

### Follow-up items
- **Re-record screencasts** — `.cast` files in `docs-site/static/screencasts/` still contain old `dev-box` references. Need re-recording after rename is live.
- **BACK-039** (visual identity) — logo, tagline, color palette. Now makes sense to tackle since the name is settled.
- **BACK-040** (Dockerfile multistage optimization) — analyse Node/Bun builder stage for docs builds
- **BACK-041** (backlog structure) — separate archive from active items

## Key Context
- Binary name is now `aibox`, config file is `aibox.toml`, TOML section is `[aibox]`
- Home dir: `.aibox-home/`, version file: `.aibox-version`, env dir: `.aibox-env/`
- Env vars: `AIBOX_LOG_LEVEL`, `AIBOX_HOST_ROOT`, `AIBOX_EDITOR_DIR`, `AIBOX_THEME`, etc.
- Docker stage name in generated Dockerfiles: `AS aibox`
- Starship palette name: `aibox`
- Old Zensical docs are gone — only Docusaurus (`docs-site/`) remains
- `.gitignore` now excludes `docs-site/.docusaurus/`, `docs-site/build/`, `docs-site/node_modules/`
