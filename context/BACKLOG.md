# Backlog

Central task registry. Each item has a unique ID for cross-referencing.
Source of truth — GitHub issues are for external visibility.
Archive of completed/merged items: [archive/BACKLOG.md](archive/BACKLOG.md)

## Next ID: BACK-044

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
| BACK-008 | Zellij plugin integration (zjstatus) | todo | should | #21 — custom aibox status plugin; needs further discussion with owner |
| BACK-010 | Evaluate multi-service support approach | todo | could | Analysis only: should we support multi-service or leave to user? Factor in SSH/remote dev plans |
| BACK-011 | Remote development | todo | should | Environments on remote hosts, local thin client |
| BACK-012 | Session handover format | todo | should | Standard process template for AI session continuity; may already be done — needs review and gap analysis |
| BACK-014 | Binary checksum verification | todo | must | Verify downloaded binaries in base Dockerfile. Sequenced after architecture overhaul |
| BACK-015 | Image signing (cosign) | todo | must | sigstore/cosign for published images. Sequenced after architecture overhaul |
| BACK-016 | Skill security vetting | todo | must | Hash verification, allowed-tools audit. Sequenced after architecture overhaul |
| BACK-017 | LaTeX addon package groups | todo | could | Add music (lilypond, musixtex), chemistry (chemfig, mhchem), linguistics tool groups to the latex add-on definition in addon_registry.rs |
| BACK-018 | AIUC-1 compliance alignment | todo | could | Awareness for aibox |
| BACK-019 | Skill marketplace integration | todo | could | ClawHub, Skills.sh |
| BACK-020 | `aibox doctor` skill consistency | todo | could | Check installed vs declared skills |
| BACK-024 | External skill installation | todo | could | Allow installing skills from sources outside curated list. Deprioritized. |
| BACK-025 | Skills gap analysis — internet research | todo | should | Research most-used agent skills/categories externally; research common SE/infra/docs/research tasks; compare with our 83 skills; identify gaps |
| BACK-026 | Existing skills quality review | todo | should | Audit all skills for missing examples, code snippets, and tools per SKILL.md format. Evaluate where tools would improve reliability (algorithmic vs probabilistic). Also audit skill descriptions for trigger optimization — Anthropic's research shows Claude tends to undertrigger; descriptions should be "a little pushy" (see context/ideas/competitive-landscape-2026-03.md R12) |
| BACK-027 | Three-level document structure audit | todo | should | Investigate whether all documents (project + derived project templates) follow the SKILL.md three-level rule (intro → overview → details) for context-efficient consumption by agents |
| BACK-028 | Complete CLI/UX overhaul | done | must | 4 phases complete: (1) AI providers as addons in registry, (2) command simplification — `build`/`attach` removed, `sync` now seeds+generates+builds, (3) `aibox addon {list,add,remove,info}` with toml_edit, (4) `aibox skill {list,add,remove,info}` with toml_edit. kubectl-style resource commands (get/describe) deferred — current verb pattern sufficient |
| BACK-029 | CLI output formats (table/JSON/YAML) | todo | could | Add structured output formats to list commands. Deprioritized, investigate later |
| BACK-030 | Bring-your-own-model AI provider support | todo | could | Allow custom/self-hosted model endpoints (e.g., vast.ai) in [ai] config. Deferred. |
| BACK-031 | Revise process bundles from competitive research | done | should | Reviewed against SAFe/PMBOK/IPMA/CMMI frameworks. Current 13 packages + 4 presets are well-scoped for a dev environment tool. Event log pattern already captured (tracking package). Maturity model is kaits-scope. Process document structure (YAML frontmatter + section IDs) noted for future context file refinement. Process frameworks research promoted to work-instructions/PROCESS-ARCHITECTURE.md |
| BACK-032 | Clean up context/project-notes directory | done | should | Archived 5 completed session/plan files to archive/project-notes/. Kept architecture-skills-processes.md (core reference) and release-process.md (operational). Promoted process-frameworks-research.md to work-instructions/. Deleted ideas/.gitkeep |
| BACK-033 | Context ideas research review | done | should | Reviewed competitive-landscape-2026-03.md — key recommendations already captured: SKILL.md adoption (done), AIUC-1 (BACK-018), marketplace (BACK-019/024), skill trigger optimization added to BACK-026. No new backlog items needed |
| BACK-034 | New skill: software modularization | todo | should | Skill on keeping software in small, independent packages optimized for AI agent context limits. Covers module boundaries, package decomposition, API surface design for agent comprehension |
| BACK-035 | New skill: microservice creation & orchestration | todo | should | Skill on creating new services/microservices and orchestrating them. Service boundaries, inter-service communication, deployment patterns |
| BACK-039 | Develop visual identity | todo | must | Research and create brand identity: logo (SVG, multiple sizes), tagline/claim, color palette (for web + docs), page design vibe, font selection (headings + body), favicon. Produce 4-5 alternative concepts, then select one. Informs Docusaurus theme, README, social preview image |
| BACK-040 | Analyse base image Dockerfile for multistage build optimization | done | should | All phases complete. Non-root user "aibox" + gosu entrypoint (session 2026-03-23b). Node multi-stage COPY in .devcontainer (session 2026-03-23b). Published image: 12 parallel BuildKit stages (10 tools + vim colors + fetch-base), per-tool cache invalidation, vim colorschemes in builder stage, apt groups documented with size estimates (~490 MB total). Apt kept as single RUN (splitting hurts more than helps). |
| BACK-042 | Internal project site for context documents | todo | could | Investigate and design a project-internal site that renders all markdown files in `./context/` as a browsable, nicely formatted site. Could be extended to a GitHub-independent wiki. **Intent:** collaboration for future multiple human workers, better readability of context files. Options: lightweight static site generator (e.g., Docusaurus second instance, mdBook, wiki.js), or extend existing docs-site with a context section. Should be simple to start — just rendered markdown with navigation |
| BACK-043 | Research additional AI provider integrations | todo | should | Research which additional AI coding agents/providers should be supported as addons. Minimum: **OpenAI Codex CLI** (open source, npm-based), **GitHub Copilot CLI**. Also evaluate: Cline, Continue.dev, Cursor CLI, Amazon Q Developer CLI, Cody. For each candidate: installation method, config directory, binary name, maturity level, license. Output: decision on which to add to addon_registry.rs |
