# Session Handover — 2026-03-22 (final)

## What Was Done

### Bug Fixes
- Completed assist→cowork cleanup (Dockerfile, maintain.sh, cheatsheets, CLI help)
- Fixed vim colorscheme E185: granular .vim/vimrc + .vim/undo mounts (DEC-013)
- Fixed ralph-loop stop hook permission
- Fixed maintain.sh docs commands: pass `-f zensical.toml` to zensical (it doesn't auto-detect)
- Added libc6-dev-amd64-cross for x86_64 cross-compilation
- Deleted temporary migrations/ directory

### Skill Library (26 → 83 skills)
- Marketplace research (97K skills, 46% duplicates, quality is #1 gap)
- Scaffolding refactor: scaffold_skills() deploys reference files (DEC-012)
- 57 new skills across 14 categories with 46 new reference files
- All registered in context.rs with include_str! constants
- 151 tests pass, clippy clean

### Documentation Overhaul
- Homepage reworked: new competitive positioning ("containerized dev environments for AI-assisted work"), dropped "uv for AI" analogy
- README.md now leads; docs/index.md mirrors and extends
- Nav restructured: Reference tab (CLI + Themes + Skills), Cheatsheet→Getting Started, Maintenance→Contributing
- Skills Library: 15 category pages documenting all 83 skills
- Installation page simplified: 3 clear parallel methods
- New-project page: actual scaffolded directory tree and full dev-box.toml with comments
- Search features: suggest, highlight, share
- Removed mkdocs.yml (fully Zensical now)

### Recording Infrastructure
- Docker socket mounted in dev-container for sibling container access
- docker.io CLI + VHS + ffmpeg + chromium + ttyd added to Dockerfile
- record-screenshots.sh: launches real dev-box container, runs VHS tape scripts, copies output
- 5 tape scripts: dev/focus/cowork layouts, init demo, theme template

### Release v0.8.0
- GitHub release created with Linux binaries (aarch64 + x86_64)
- Docs deployed to gh-pages
- User handling macOS builds and image push

### Research Persisted
- context/research/skill-marketplace-landscape-2026-03.md
- context/research/competitive-tools-2026-03.md
- context/research/competitive-dev-environments-2026-03.md

### Decisions Added
- DEC-012: Reference file scaffolding via SkillDef type
- DEC-013: Granular vim mounts preserve image colorschemes
- DEC-014: Curated quality over marketplace quantity

## What Needs to Happen Next

### Immediate (after container rebuild)
1. Run `./scripts/record-screenshots.sh 0.8.0` to generate screenshots
2. Embed screenshots in docs (new-project page, base-image page, themes page)
3. Continue docs review (existing-project page, other pages per user feedback)

### Backlog
- **CLI simplification** — merge build into start, merge attach into start (user request)
- **Theming screenshots** (#14) — blocked on recording infra, now unblocked
- **Security review** (#23) — input validation, container security
- **`dev-box skill install`** — install skills from external sources
- **Skill eval framework** — test/benchmark per Anthropic's skill-creator pattern

## Key Context
- Zensical is the docs tool, NOT MkDocs. Config: zensical.toml. Requires `-f` flag.
- GHCR public packages are free ($0.00 confirmed)
- Background agents with WebFetch can get stuck — do research in main thread
- README.md is the leading content; docs homepage derives from it
