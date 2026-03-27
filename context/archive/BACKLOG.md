# Backlog Archive

Completed, merged, and archived items. Active backlog: [../BACKLOG.md](../BACKLOG.md)

---

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
| BACK-001 | Theming screenshots gallery | done | — | #14 — completed in session 2026-03-23 |
| BACK-003 | `aibox skill install` command | archived | — | Split into BACK-023 (skill command) and BACK-024 (external skills) |
| BACK-005 | CLI simplification | archived | — | Merged into BACK-028 (CLI/UX overhaul) |
| BACK-009 | Automated context migration | archived | — | Merged into BACK-022 (items 9–10) |
| BACK-013 | TeX Live builder deduplication | done | — | Resolved by BACK-022 — LaTeX is now a single add-on |
| BACK-014 | Binary checksum verification | archived | — | Merged into BACK-002 (security review). Scope: verify downloaded binaries in base Dockerfile. Sequencing was after architecture overhaul (now done — BACK-022/BACK-028). |
| BACK-015 | Image signing (cosign) | archived | — | Merged into BACK-002 (security review). Scope: sigstore/cosign for published images. |
| BACK-016 | Skill security vetting | archived | — | Merged into BACK-002 (security review). Scope: hash verification, allowed-tools audit. |
| BACK-021 | Investigate migration from Zensical to Docusaurus | done | — | Migration complete — docs-site/ has Docusaurus content. Old docs/ and zensical.toml cleanup done in BACK-038 Phase 0 |
| BACK-022 | Declarative config + minimal base images | done | — | DEC-016. 5 phases: addon registry, single base image, process packages, sync expansion, migration system |
| BACK-023 | `aibox skill` command | archived | — | Merged into BACK-028 (CLI/UX overhaul) |
| BACK-028 | Complete CLI/UX overhaul | done | — | 4 phases complete: (1) AI providers as addons in registry, (2) command simplification — `build`/`attach` removed, `sync` now seeds+generates+builds, (3) `aibox addon {list,add,remove,info}` with toml_edit, (4) `aibox skill {list,add,remove,info}` with toml_edit. kubectl-style resource commands deferred — current verb pattern sufficient. |
| BACK-031 | Revise process bundles from competitive research | done | — | Reviewed against SAFe/PMBOK/IPMA/CMMI. Current 13 packages + 4 presets well-scoped. Process frameworks research promoted to work-instructions/PROCESS-ARCHITECTURE.md. |
| BACK-032 | Clean up context/project-notes directory | done | — | Archived 5 completed session/plan files. Promoted process-frameworks-research.md to work-instructions/. |
| BACK-033 | Context ideas research review | done | — | Reviewed competitive-landscape-2026-03.md — key recommendations captured in BACK-018/019/024/026. |
| BACK-036 | Bug: Yazi "e" key does not open files in vim | done | — | Fixed in 06d9505 — corrected $0→$1 variable syntax in keymap.toml, added EDITOR/VISUAL exports |
| BACK-037 | Bug: Yazi preview broken for images/PDF/GIF | done | — | Fixed in 06d9505 — added chafa, poppler-utils, ffmpeg, imagemagick to base image |
| BACK-038 | Rename project to "aibox" | done | — | 5 phases: Rust core, config rename, scripts/docs, meta-docs, GitHub rename. Completed 2026-03-23 |
| BACK-039 | Develop visual identity | done | — | projectious.work brand system applied. aibox logo (terminal brackets + sparkle stars) with full variant set. Docusaurus themed. "projectious" terminal theme. Social preview/OG image still todo (see active backlog). |
| BACK-040 | Analyse base image Dockerfile for multistage build optimization | done | — | Non-root user "aibox" + gosu entrypoint. 12 parallel BuildKit stages. Per-tool cache invalidation. Apt kept as single RUN. |
| BACK-041 | Backlog structure: separate active from archive | done | — | Split BACKLOG.md into active + BACKLOG-ARCHIVE.md |
| BACK-045 | E2E testing environment design | done | — | docker-compose companion container (aibox-e2e-testrunner) accessible via SSH (pre-seeded keys). SCP deploy model — no shared volumes. Podman rootless for nested container ops. |
| BACK-048 | Externalize addon definitions from hardcoded Rust | done | — | Addon definitions moved to YAML files in addons/. CLI loads from AIBOX_ADDONS_DIR (default /opt/aibox/addons). Version pinning and checksum fields in schema. |
| BACK-049 | CLI `uninstall` command (self-removal) | done | — | `aibox uninstall` implemented in reset.rs (cmd_uninstall). Confirmation prompt, --dry-run, --yes flags. Removes CLI binary and global state; preserves project-level files. |
| BACK-053 | Fix `aibox init` process selection — show presets only | done | — | `process_selection_items()` returns only presets; individual packages still accepted via --process flag. E2E tests added in lifecycle.rs. |
| BACK-055 | Strengthen yazi `e`-key e2e assertion | done | — | Root causes: `[manager]` → `[mgr]` rename in yazi 26.1.22; stale ZELLIJ_SESSION_NAME; `--confirm` blocking headless. All fixed in seed.rs + driver. |
| BACK-056 | Add ANSI color-code assertions to visual theme tests | done | — | True-color escape sequence assertions added per theme (e.g. gruvbox-dark: `\e[38;2;235;219;178m`). |
| BACK-057 | Research systemd unit-file format for addon dependency design | archived | — | Merged into BACK-052 (addon dependency tree). Key reference fields: After=, Requires=, Wants=, Conflicts=, PartOf=. Apply when designing addon YAML dependency resolver. |
| BACK-058 | Fix `aibox update` 401 error in derived projects | done | — | `do_upgrade` now matches on fetch_latest_image_version error and warns + exits Ok. E2E tests in tests/e2e/update.rs. |
| BACK-059 | Upgrade e2e keybinding tests to zellij 0.44 native CLI automation | done | — | `send-keys --pane-id <terminal_id>` + `dump-screen --pane-id` + session discovery via `zellij list-sessions`. All 8 visual keybinding tests pass. |
| BACK-061 | Improve seeded config file comments — show all options inline | done | — | `serialize_config_with_comments` rewritten with `===` section separators, inline trailing comments, every option present. Dockerfile.j2 and docker-compose.yml.j2 headers added. |
| BACK-010 | Evaluate multi-service support approach | done | — | Resolved via BACK-066: `docker-compose.override.yml` support. Users add extra services via standard Docker Compose override mechanism. |
| BACK-012 | Session handover format | done | — | Templates + skill implemented. 6-section fill-in-the-blanks format with git log review and backlog check. |
| BACK-052 | Addon dependency tree — investigate and prototype | done | — | Depth-1 tree confirmed. Kahn's algorithm circular detection verified. `aibox addon info` shows `Requires:` line. `conflicts` not needed. |
| BACK-066 | Compose override support (`docker-compose.override.yml`) | done | — | Scaffold during `aibox init`, auto-detect in `aibox sync`, wire into `devcontainer.json` as array. Shipped in v0.13.1. |
| — | aibox sync | done | — | #25 — theme switching without manual file deletion |
| — | Shell enhancement tools | done | — | ripgrep, fd, bat, eza, zoxide, fzf, delta + aliases |
| — | Starship prompt | done | — | #28 — installed in base image |
| — | Keyboard shortcuts cheatsheet | done | — | #16 — docs page |
| — | generate deprecated | done | — | Replaced by sync (#25) |
| — | AI provider flexibility | done | — | #19 — Claude/Aider/Gemini, dynamic layouts, optional |
| — | Process templates | done | — | #29 — release, code-review, feature-dev, bug-fix |
| — | SKILL.md support | done | — | #30 — 3 example skills scaffolded |
| — | Addon packages | done | — | #18 — infrastructure, kubernetes, cloud, 6 docs addons |
| — | Security audit command | done | — | #24 — aibox audit: cargo audit, pip-audit, trivy |
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
| BACK-011 | Remote development | done | — | Research complete: `context/research/remote-development-2026-03.md`. Review → BACK-079. |
| BACK-025 | Skills gap analysis — internet research | done | — | Research complete: `context/research/skills-gap-analysis-2026-03.md`. Review → BACK-083. |
| BACK-026 | Existing skills quality review | done | — | Audit complete: `context/research/skills-quality-audit-2026-03.md`. Review → BACK-089. |
| BACK-027 | Three-level document structure audit | done | — | Audit complete: `context/research/document-structure-audit-2026-03.md`. Review → BACK-090. |
| BACK-043 | Research additional AI provider integrations | done | — | Research complete: `context/research/ai-provider-integrations-2026-03.md`. Review → BACK-105. Implementation → BACK-064. |
| BACK-044 | Evaluate Mozilla cq integration | done | — | Research complete: `context/research/mozilla-cq-evaluation-2026-03.md`. Review → BACK-084. Recommendation: skip for now, reassess Q3 2026. |
| BACK-046 | Issue handling skill + agent architecture | done | — | Design complete: `context/research/issue-handling-design-2026-03.md`. Review → BACK-091. |
| BACK-047 | Investigate Docusaurus addon improvements | done | — | Research complete: `context/research/docusaurus-addon-improvements-2026-03.md`. Review → BACK-103. |
| BACK-050 | Addon-skill mapping completeness + orphan skill audit | done | — | Audit complete: `context/research/addon-skill-mapping-audit-2026-03.md`. 31 orphans found. Review → BACK-092. |
| BACK-051 | Organizational learning via skill customization | done | — | Design complete: `context/research/skill-customization-design-2026-03.md`. Review → BACK-080. |
| BACK-054 | Research VS Code themes for aibox | done | — | Research complete: `context/research/vscode-themes-2026-03.md`. Recommends Rose Pine, Everforest, Kanagawa. Review → BACK-085. |
| BACK-060 | Investigate aibox version upgrade flows | done | — | Design complete: `context/research/version-upgrade-flows-2026-03.md`. Proposes `aibox migrate` command. Review → BACK-086. |
| BACK-067 | New zellij layout: yazi-focused with large preview | done | — | "browse" layout implemented in v0.14.0. |
| BACK-068 | Kubernetes deployment: scaffold Helm charts | done | — | Research complete: `context/research/kubernetes-deployment-2026-03.md`. Review → BACK-081. |
| BACK-069 | Skill definition versioning and status metadata | done | — | Design complete: `context/research/skill-versioning-design-2026-03.md`. Review → BACK-098. |
| BACK-070 | AI-provider-independent scheduled tasks | done | — | Research complete: `context/research/scheduled-tasks-2026-03.md`. Review → BACK-087. |
| BACK-073 | Event log in ./context + event logging skill | done | — | Design complete: `context/research/event-log-design-2026-03.md`. Review → BACK-082. |
| BACK-074 | Bug: Rust addon builder HOME path mismatch | done | — | Fixed in v0.13.2. |
| BACK-075 | Bug: Dockerfile template writes /etc/aibox-version without root | done | — | Fixed in v0.13.2. |
| BACK-088 | Research: RAG layer over ./context for semantic search | done | — | Research complete: `context/research/rag-context-layer-2026-03.md`. Recommends fastembed+sqlite-vec. Review → BACK-099. |
| BACK-093 | New skill category: Brand & Design | done | — | Research complete: `context/research/brand-design-skills-2026-03.md`. Review → BACK-100. |
| BACK-094 | Research: self-hosted AI models and GPU providers | done | — | Research complete: `context/research/self-hosted-ai-models-2026-03.md`. Review → BACK-101. |
| BACK-095 | Research: owner profiling skill | done | — | Design complete: `context/research/owner-profiling-skill-2026-03.md`. Review → BACK-102. |
| BACK-096 | Research: infrastructure provisioning | done | — | Research complete: `context/research/infrastructure-provisioning-2026-03.md`. Review → BACK-104. |
