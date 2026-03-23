# Backlog

Central task registry. Each item has a unique ID for cross-referencing.
Source of truth — GitHub issues are for external visibility.

## Next ID: BACK-040

## Format

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|

Status values: `todo`, `in-progress`, `done`, `blocked`, `archived`
Priority values: `must`, `should`, `could`, `wont`

---

## Active Items

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
| BACK-002 | Security review | todo | must | #23 — input validation, container security, supply chain. Sequenced after architecture overhaul (BACK-022) and CLI overhaul (BACK-028) |
| BACK-004 | Skill eval framework | todo | should | Test/benchmark skills per Anthropic skill-creator pattern |
| BACK-006 | Docs review (existing-project, base-image pages) | todo | could | Deprioritized — do after Docusaurus migration (BACK-021) and architecture changes settle |
| BACK-007 | Plugin / extension system | todo | should | #20 — hooks, template overrides, community features |
| BACK-008 | Zellij plugin integration (zjstatus) | todo | should | #21 — custom dev-box status plugin; needs further discussion with owner |
| BACK-010 | Evaluate multi-service support approach | todo | could | Analysis only: should we support multi-service or leave to user? Factor in SSH/remote dev plans |
| BACK-011 | Remote development | todo | should | Environments on remote hosts, local thin client |
| BACK-012 | Session handover format | todo | should | Standard process template for AI session continuity; may already be done — needs review and gap analysis |
| BACK-014 | Binary checksum verification | todo | must | Verify downloaded binaries in base Dockerfile. Sequenced after architecture overhaul |
| BACK-015 | Image signing (cosign) | todo | must | sigstore/cosign for published images. Sequenced after architecture overhaul |
| BACK-016 | Skill security vetting | todo | must | Hash verification, allowed-tools audit. Sequenced after architecture overhaul |
| BACK-017 | LaTeX addon package groups | todo | could | Add music (lilypond, musixtex), chemistry (chemfig, mhchem), linguistics tool groups to the latex add-on definition in addon_registry.rs |
| BACK-018 | AIUC-1 compliance alignment | todo | could | Awareness for dev-box |
| BACK-019 | Skill marketplace integration | todo | could | ClawHub, Skills.sh |
| BACK-020 | `dev-box doctor` skill consistency | todo | could | Check installed vs declared skills |
| BACK-021 | Investigate migration from Zensical to Docusaurus | todo | must | Analyze docs/ structure, evaluate Docusaurus fit, plan migration path |
| BACK-024 | External skill installation | todo | could | Allow installing skills from sources outside curated list. Deprioritized. |
| BACK-025 | Skills gap analysis — internet research | todo | should | Research most-used agent skills/categories externally; research common SE/infra/docs/research tasks; compare with our 83 skills; identify gaps |
| BACK-026 | Existing skills quality review | todo | should | Audit all skills for missing examples, code snippets, and tools per SKILL.md format. Evaluate where tools would improve reliability (algorithmic vs probabilistic) |
| BACK-027 | Three-level document structure audit | todo | should | Investigate whether all documents (project + derived project templates) follow the SKILL.md three-level rule (intro → overview → details) for context-efficient consumption by agents |
| BACK-028 | Complete CLI/UX overhaul | todo | must | Merge of BACK-005 + BACK-023. kubectl as reference model. See expanded scope below |
| BACK-029 | CLI output formats (table/JSON/YAML) | todo | could | Add structured output formats to list commands. Deprioritized, investigate later |
| BACK-030 | Bring-your-own-model AI provider support | todo | could | Allow custom/self-hosted model endpoints (e.g., vast.ai) in [ai] config. Deferred. |
| BACK-031 | Revise process bundles from competitive research | todo | should | After BACK-022 implementation, revisit context/ideas/competitive-landscape-2026-03.md and process-frameworks-research.md to refine process packages and extract ideas (e.g., event log pattern) |
| BACK-032 | Clean up context/project-notes directory | todo | should | After BACK-022 implementation, consolidate scattered files in project-notes/; valuable content but too unstructured |
| BACK-033 | Context ideas research review | todo | should | After BACK-022 implementation, review all files in context/ideas/ for actionable insights not yet captured in backlog |
| BACK-034 | New skill: software modularization | todo | should | Skill on keeping software in small, independent packages optimized for AI agent context limits. Covers module boundaries, package decomposition, API surface design for agent comprehension |
| BACK-035 | New skill: microservice creation & orchestration | todo | should | Skill on creating new services/microservices and orchestrating them. Service boundaries, inter-service communication, deployment patterns |
| BACK-036 | Bug: Yazi "e" key does not open files in vim | todo | must | Pressing "e" in yazi does nothing. Enter opens file in vim but in yazi's own pane (not the layout vim pane) and with wrong theme settings. Likely open-in-editor.sh or yazi keymap issue in v0.8.0 |
| BACK-037 | Bug: Yazi preview broken for images/PDF/GIF | todo | must | PNG/SVG preview shows nothing; PDF fills screen with binary data; GIF preview also broken. Likely missing preview dependencies (ueberzugpp, file, poppler-utils) in base image |
| BACK-038 | Rename project to "aibox" | todo | must | Rename everywhere: GitHub repo (projectious-work/aibox), CLI binary, GHCR registry, all code references (dev-box → aibox, dev_box → aibox), dev-box.toml → aibox.toml, .dev-box-home → .aibox-home, docs, CLAUDE.md templates, install script, maintain.sh. Also: clean up GHCR — delete all images except v0.8.0 set (mark v0.8.0 for deprecation, remove by 2026-04-06). Working title — final name TBD |
| BACK-039 | Develop visual identity | todo | must | Research and create brand identity: logo (SVG, multiple sizes), tagline/claim, color palette (for web + docs), page design vibe, font selection (headings + body), favicon. Produce 4-5 alternative concepts, then select one. Informs Docusaurus theme, README, social preview image |

### BACK-028 Expanded Scope

Complete CLI/UX overhaul with kubectl as the reference model:

1. **Command simplification**: Merge build/attach/generate into `dev-box sync` as the primary reconciliation command. Reduce command surface area.
2. **`dev-box skill` subcommands**: Imperative interface to browse, add, remove, list curated skills. Updates dev-box.toml declaratively.
3. **`dev-box addon` subcommands**: Imperative interface to add, remove, list add-ons (Python, Node, Go, LaTeX, etc.) with version selection. Updates dev-box.toml.
4. **kubectl-style UX patterns**: Resource-oriented commands (`dev-box get`, `dev-box describe`?), consistent verb patterns, discoverable help. Investigate what makes sense for dev-box — not a blind copy, but learn from kubectl's ergonomics.
5. **Relationship to BACK-022**: The CLI overhaul implements the user-facing interface for the declarative config model designed in BACK-022. BACK-022 (architecture) should be resolved first, then BACK-028 (CLI) follows.

---

## Archive

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
| BACK-001 | Theming screenshots gallery | done | — | #14 — completed in session 2026-03-23 |
| BACK-003 | `dev-box skill install` command | archived | — | Split into BACK-023 (skill command) and BACK-024 (external skills) |
| BACK-005 | CLI simplification | archived | — | Merged into BACK-028 (CLI/UX overhaul) |
| BACK-009 | Automated context migration | archived | — | Merged into BACK-022 (items 9–10) |
| BACK-013 | TeX Live builder deduplication | done | — | Resolved by BACK-022 — LaTeX is now a single add-on |
| BACK-022 | Declarative config + minimal base images | done | — | DEC-016. 5 phases: addon registry, single base image, process packages, sync expansion, migration system |
| BACK-023 | `dev-box skill` command | archived | — | Merged into BACK-028 (CLI/UX overhaul) |
| — | dev-box sync | done | — | #25 — theme switching without manual file deletion |
| — | Shell enhancement tools | done | — | ripgrep, fd, bat, eza, zoxide, fzf, delta + aliases |
| — | Starship prompt | done | — | #28 — installed in base image |
| — | Keyboard shortcuts cheatsheet | done | — | #16 — docs page |
| — | generate deprecated | done | — | Replaced by sync (#25) |
| — | AI provider flexibility | done | — | #19 — Claude/Aider/Gemini, dynamic layouts, optional |
| — | Process templates | done | — | #29 — release, code-review, feature-dev, bug-fix |
| — | SKILL.md support | done | — | #30 — 3 example skills scaffolded |
| — | Addon packages | done | — | #18 — infrastructure, kubernetes, cloud, 6 docs addons |
| — | Security audit command | done | — | #24 — dev-box audit: cargo audit, pip-audit, trivy |
| — | Zensical migration | done | — | #26 — config + maintain.sh support |
| — | Dockerfile optimization | done | — | #27 — pinned versions, fontconfig fix, COPY consolidation |
| — | Version in docs header | done | — | #12 — site_name includes version |
| — | Starship prompt presets | done | — | #17 — 6 presets with theme-aware colors |
| — | Additional image flavors | done | — | Node.js + Go (10 images total) |
| — | Code review for simplification | done | — | #22 — dedup, test helpers, dead code removal |
| — | Curated skill library | done | — | #30 — 83 skills, 14 categories, 57 reference files |
| — | Skills Library docs | done | — | 15 category pages, search with autocomplete |
| — | assist→cowork cleanup | done | — | Removed all stale assist.kdl references |
| — | Vim colorscheme fix | done | — | Granular .vim mounts so image colors survive |
| — | Remove mkdocs.yml | done | — | Fully migrated to zensical.toml |
| — | Dogfood product process | done | — | Adopt own product template, migrate GitHub issues |
