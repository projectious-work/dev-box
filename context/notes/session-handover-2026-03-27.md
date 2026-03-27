---
date: 2026-03-27
session_type: major feature + research + architecture discussion
duration: long (full day)
---

# Session Handover — 2026-03-27

## What was accomplished

### Releases
- **v0.14.0**: Major config restructuring
  - `[appearance]` → `[customization]` with new `layout` field (dev/focus/cowork/browse)
  - `[process]` merged into `[context]` (packages + schema_version)
  - `[container]` simplified: removed ports, extra_packages, extra_volumes, environment, vscode_extensions (use Dockerfile.local and docker-compose.override.yml instead)
  - Backward compatibility via serde aliases for old configs
  - SSH key security fix (.devcontainer/ssh-e2e/ → .aibox-e2e-runner-home/.ssh/)
  - Browse layout added to CLI
  - AI Provider docs moved to own sidebar chapter
  - Git history scrubbed to remove leaked SSH private key (force-push)
- **v0.14.1**: Image size reduction
  - Removed ffmpeg (~450MB), imagemagick (~50MB), ghostscript (~70MB) from base image
  - Created `preview-enhanced` addon for users who need them
  - SVG preview: rsvg-convert fallback for aarch64 (resvg only on x86_64)
  - Added librsvg2-bin to base image

### Research (17 new reports this session)
All in `context/research/`:
- preview-companion-design, remote-development, skill-customization-design
- kubernetes-deployment, event-log-design, skills-gap-analysis
- mozilla-cq-evaluation, vscode-themes, version-upgrade-flows
- scheduled-tasks, skills-quality-audit, document-structure-audit
- issue-handling-design, addon-skill-mapping-audit, skill-versioning-design
- brand-design-skills, owner-profiling-skill, docusaurus-addon-improvements
- self-hosted-ai-models, infrastructure-provisioning, rag-context-layer
- **process-ontology-primitives** (foundational — 15 universal process primitives)
- **context-database-architecture** (markdown+frontmatter vs database analysis)
- **file-per-entity-scaling** (git scaling limits, three-tier architecture)

### Backlog cleanup
- 25 completed research items archived
- 22 review/discussion items filed (BACK-078 through BACK-105)
- New items: BACK-088 (RAG), BACK-093 (brand skills), BACK-094 (self-hosted AI),
  BACK-095 (owner profiling), BACK-096 (infra provisioning), BACK-097 (process retrospective)

### Architecture discussion (DISC-001)
Major discussion on context system redesign, documented in:
`context/discussions/DISC-001-context-system-redesign.md`

**Tentative decisions made:**
1. **Storage**: Markdown+frontmatter as single source of truth. SQLite as derived runtime index (gitignored).
2. **Scaling**: Three-tier hot/warm/cold. Directory sharding. Sparse checkout for large repos.
3. **kaits boundary**: Repo-per-project. aibox handles per-project context (up to 100K items).
4. **IDs**: Short UUID (8 hex chars). No coordination needed.
5. **Discussions**: Are a 16th process primitive. Stored in `context/discussions/`.

## What to do next

**Immediate (next session — continue DISC-001):**
1. **Map 16 primitives to storage structure** — for each primitive, define:
   - File location in `context/` directory (with sharding scheme)
   - YAML frontmatter schema (required fields, optional fields, custom: map)
   - State machine (allowed states and transitions)
   - Relationships to other primitives
   - Hot/cold classification
2. **Design new `context/` directory layout** with date-based sharding
3. **Design YAML frontmatter schemas** per primitive type
4. **Continue the remaining BACK-097 sub-discussions**: event log format (BACK-082),
   document quality (BACK-090), issue handling (BACK-091), RAG (BACK-099)

**Before implementing:**
- Record tentative decisions as formal DEC-NNN entries
- Prototype migration of BACKLOG.md to file-per-entity
- Validate the new format works with `aibox sync`

## Key files modified this session
- `cli/src/config.rs` — major restructuring (CustomizationSection, merged ContextSection)
- `cli/src/container.rs` — serialize_config_with_comments rewritten, container fields removed
- `cli/src/cli.rs` — Browse layout, Optional layout
- `cli/src/main.rs` — config-aware layout resolution
- `cli/src/seed.rs` — svg.yazi and eps.yazi plugin updates
- `cli/src/generate.rs` — removed container field templates
- `images/base-debian/Dockerfile` — removed ffmpeg/imagemagick/ghostscript, added librsvg2-bin
- `addons/tools/preview-enhanced.yaml` — NEW addon
- `docs-site/` — AI Providers chapter, layouts page, config reference updates
- `aibox.toml` — migrated to new format with full comments
- `.devcontainer/` — SSH key migration, compose override updates
- `context/discussions/DISC-001-context-system-redesign.md` — NEW, active discussion
- `context/research/` — 17 new research reports

## Context for next agent

- The `discussions/` directory is new — DISC-001 is the first entry. It captures the full
  train of thought for the context system redesign.
- Owner values: single source of truth (no dual-master), git-native storage, flexible
  schemas, human-readable files. Strongly opinionated about data integrity.
- Owner intends aibox to be the BASIS for kaits (multi-agent company simulator). Scaling
  decisions must account for this — don't throw problems over the fence.
- The process ontology (15+1 primitives) is foundational work. The next step is mapping
  these to concrete file structures and YAML schemas.
- All research items from the backlog are now complete. 22 review items await discussion.
- v0.14.1 is the current release. Both phases (container + host) are done.
- Git history was rewritten (force-push) to remove SSH private key. Host needs re-clone.
