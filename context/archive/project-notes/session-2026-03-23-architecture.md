# Session Handover — 2026-03-23 (Architecture Revision)

## What was done

### BACK-022: Declarative Config + Minimal Base Images (DEC-016)
Major architecture revision implemented in 5 phases:
1. **Phase 1**: Addon registry (17 addons with per-tool version selection), process registry (13 composable packages + 4 presets), config.rs rewrite (new TOML structure)
2. **Phase 2**: Retired 9 image Dockerfiles, kept only `images/base-debian/`. Dockerfile template now supports multi-stage builder stages for add-ons.
3. **Phase 3**: Process-driven scaffolding — context files and skills are deployed based on which packages the user selects. Selective skill deployment (not all 83).
4. **Phase 4**: Sync expansion — skill reconciliation (deploy missing, warn orphans), AIBOX.md generation (universal baseline doc), agent entry point checks.
5. **Phase 5**: Migration system — version detection on sync, standardized migration document generation with safety headers and verification checklists.

### BACK-021: Zensical to Docusaurus Migration
- Scaffolded Docusaurus site in `docs-site/`
- Custom homepage (hero + feature grid + quick start), features page, changelog page
- 31 docs pages migrated with frontmatter
- Flat top nav (no dropdowns), sidebar docs, dark mode default
- Build: `cd docs-site && bun run build && bun run serve -- --host 0.0.0.0`
- Note: `bun start` (dev server) renders black page — use build+serve for preview

### Bug Fixes
- **BACK-036**: Yazi "e" key — changed `$0` to `$1` in keymap (Yazi 25.x syntax)
- **BACK-037**: Yazi preview — added chafa, poppler-utils, ffmpeg, imagemagick to base Dockerfile

### Backlog Updates
- Full backlog revision: 30 active items, BACK-022/013 archived as done
- New items filed: BACK-030 through BACK-037
- DEC-016 recorded in DECISIONS.md

### Infrastructure
- Node.js 22 (NodeSource) added to `.devcontainer/Dockerfile` for Docusaurus builds
- Zensical removed from dev container tool installs

## Release
- **v0.9.0** tagged and pushed

## What needs attention next

### Immediate (must)
- **BACK-028**: Complete CLI/UX overhaul (kubectl reference, `aibox skill/addon` subcommands)
- **BACK-002, 014, 015, 016**: Security items (sequenced after CLI overhaul)
- **BACK-036/037**: Yazi fixes are in the Dockerfile but need a container rebuild + image push to verify

### Soon (should)
- **BACK-031**: Revise process bundles from competitive research (now unblocked by BACK-022)
- **BACK-032**: Clean up context/project-notes (scattered files)
- **BACK-033**: Review context/ideas/ for actionable insights
- Old `docs/` directory and `zensical.toml` should be deleted once Docusaurus is confirmed working in production

### Content updates needed
- Container docs: still reference 10 image flavors (need update for 1 base + add-ons)
- Configuration docs: need update for new aibox.toml structure
- Context docs: need update for 13 composable packages replacing 4 levels

## Key files changed
- `cli/src/addon_registry.rs` — NEW (17 addons)
- `cli/src/process_registry.rs` — NEW (13 packages)
- `cli/src/migration.rs` — NEW (migration system)
- `cli/src/config.rs` — REWRITTEN (new TOML structure)
- `cli/src/context.rs` — REWRITTEN (selective scaffolding)
- `cli/src/addons.rs` — REWRITTEN (bridge to registry)
- `images/base-debian/` — renamed from `images/base/`, 9 others deleted
- `docs-site/` — NEW (entire Docusaurus project)
- `.devcontainer/Dockerfile` — Node.js added, Zensical removed

## Test status
- 239 tests passing, 0 failures, clippy clean (Rust CLI)
- Docusaurus builds successfully (`bun run build`)
