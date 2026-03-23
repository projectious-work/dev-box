# Backlog

Central task registry. Each item has a unique ID for cross-referencing.
Source of truth — GitHub issues are for external visibility.

## Next ID: BACK-038

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
| BACK-013 | TeX Live builder deduplication | todo | should | 3 Dockerfiles share identical 90-line build stage; keep until base image decision in BACK-022 is resolved |
| BACK-014 | Binary checksum verification | todo | must | Verify downloaded binaries in base Dockerfile. Sequenced after architecture overhaul |
| BACK-015 | Image signing (cosign) | todo | must | sigstore/cosign for published images. Sequenced after architecture overhaul |
| BACK-016 | Skill security vetting | todo | must | Hash verification, allowed-tools audit. Sequenced after architecture overhaul |
| BACK-017 | LaTeX addon package groups | todo | could | Music, chemistry, linguistics; keep until base image decision in BACK-022 is resolved |
| BACK-018 | AIUC-1 compliance alignment | todo | could | Awareness for dev-box |
| BACK-019 | Skill marketplace integration | todo | could | ClawHub, Skills.sh |
| BACK-020 | `dev-box doctor` skill consistency | todo | could | Check installed vs declared skills |
| BACK-021 | Investigate migration from Zensical to Docusaurus | todo | must | Analyze docs/ structure, evaluate Docusaurus fit, plan migration path |
| BACK-022 | Investigate shift to declarative config + minimal base images | todo | must | Most urgent. Major architecture revision — see expanded scope below |
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

### BACK-022 Expanded Scope

This is the central architecture investigation for the project revision:

1. **Base images**: Reduce from 10 flavors to 2–3 stock base images. Consider: a general base + a LaTeX base (LaTeX builds are slow, pre-baking saves users significant time). Evaluate whether build-time cost justifies dedicated base images vs pure add-on approach.
2. **Add-on system**: Everything currently baked into images or offered as add-ons becomes a declarative add-on in dev-box.toml. Each add-on (e.g., Python) specifies what to inject into the derived Dockerfile via `dev-box sync`. Users can select/deselect individual packages within an add-on. Curated version selection per add-on (e.g., Python 3.13 vs 3.14) — opinionated, small supported set.
3. **Process packages**: A process (minimal, managed, product, research) = collection of skills + context document templates, selectable in dev-box.toml. Users can override defaults (deselect skills from a process).
4. **Skills in dev-box.toml**: Declarative skill selection/deselection, independent of process package.
5. **Investigate context vs skills boundary**: Is context purely artifacts? Or does some "how" knowledge need to live there? Working hypothesis: skills hold procedural knowledge, context holds artifacts + light structural conventions — but edge cases (backlog column definitions, standup cadence) may blur the line.
6. **dev-box sync as single reconciliation command**: Generates/updates Dockerfiles, context files, skill files from declarative dev-box.toml state. May produce migration scripts for derived project agents.
7. **Universal baseline document**: Agent-independent root document (e.g., `context/DEVBOX.md`) that is always present and not deselectable. Three-level layout (intro → overview → details). Defines: where context lives, how processes work, migration pickup protocol, safety rules. This is the main entry point for any agent at any session.
8. **Agent entry point scaffolding**: When user selects providers in `[ai]`, scaffold/update each agent's native entry point (CLAUDE.md, `.aider.conf.yml`, Gemini equivalent) with a pointer to the universal baseline document in context/.
9. **Migration document system** (absorbed from BACK-009): On `dev-box sync`, detect version differences and generate a migration markdown for the derived project. Standardized format includes: safety header ("never execute automatically, always discuss with user"), auto-generated diff from git history of template changes, description of what changed and why. Derived project agents must check for and pick up migration documents at session start — enforced via the universal baseline document.
10. **Non-deselectable base process**: A minimal process layer always present in every dev-box project that ensures agents check for migrations, follow the baseline contract, and respect safety rules. This sits beneath all selectable process packages.

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
