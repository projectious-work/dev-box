# Decisions Log

Inverse chronological. Each decision has a rationale and alternatives considered.

## DEC-024 — Directory sharding per entity type (2026-04-06)

**Decision:** Projects may shard entity directories by time, state, or other axes on a per-primitive basis. Default is flat (one directory per primitive kind). Sharding is configured in `aibox.toml` under `[context.sharding.<kind>]`.

**Rationale:** Flat directories become unwieldy past ~500 files. Large projects benefit from date-based sharding for logs (`context/logs/2026/04/`), state-based sharding for work items (`context/workitems/done/`, `context/workitems/active/`), or flat for small projects. Making this configurable rather than imposed avoids premature organization.

**Alternatives:** Always flat (breaks at scale), always sharded (overhead for small projects), per-repo fixed scheme (less flexible).

**Source:** DISC-002 Q3 carry-forward from DISC-001.

## DEC-023 — Binding as generalized primitive (replaces RoleBinding) (2026-04-06)

**Decision:** Rename the 18th primitive from RoleBinding to **Binding**. A Binding connects any two entities with optional scope, temporality, and conditions — not just Actor-to-Role. Rule: if a relationship has scope, time, or its own attributes, use a Binding; if it is just "A relates to B," use a cross-reference in frontmatter.

**Rationale:** The indirection pattern (put a third thing between two things so either can change independently) is a fundamental software design principle (GoF patterns, dependency injection, junction tables with attributes). Inventory of processkit relationships shows ≥7 types that benefit from this pattern: role-assignment, work-assignment, process-gate, process-scope, schedule-scope, constraint-scope, category-assignment. One generalized Binding primitive handles all of them without multiplying primitives.

**Alternatives:** Keep RoleBinding specific and add more specific bindings as needed (rejected — grows primitive count without benefit), no bindings, just references everywhere (rejected — cannot express scope/time on relationships without editing endpoints).

**Source:** DISC-002 §11.

## DEC-022 — Configurable ID format (word/uuid × with/without slug) (2026-04-06)

**Decision:** ID format is configurable in `aibox.toml`. Two independent axes: base format (`id_format = "word"` via petname crate, or `"uuid"`) and slug (`id_slug = true` or `false`). All four combinations are valid. The kind prefix (`BACK-`, `LOG-`, `DEC-`, ...) is not configurable. Default: word without slug.

**Rationale:** Solo developers prefer short memorable IDs (`BACK-calm-fox`); larger teams may want uniqueness guarantees (`BACK-550e8400-e29b`); projects with lots of IDs in prose benefit from slugs (`BACK-calm-fox-add-lint`). None of these choices affects interop between projects — the prefix and structure are constant. Making the format configurable resolves the DISC-001 contradiction between Decision 4 (word-based) and Decision 40 (UUID).

**Alternatives:** Fixed word-based (excludes teams needing uniqueness), fixed UUID (unfriendly for solo work), per-entity-type configuration (overkill).

**Source:** DISC-002 §3 Q3 resolution.

## DEC-021 — SQLite index lives in processkit MCP servers (2026-04-06)

**Decision:** Entity indexing (parse markdown+frontmatter, build SQLite tables, serve queries) lives in a processkit MCP server (`skills/index-management/mcp/server.py`), not in the aibox CLI. aibox CLI performs basic structural validation only (`apiVersion`, `kind`, `metadata.id` present); it is schema-agnostic.

**Rationale:** Schemas live in processkit. Putting schema-aware code in aibox CLI creates tight coupling — every primitive or schema change would require an aibox release. Putting the indexer in processkit makes schema evolution self-contained. The MCP server becomes the canonical query interface for agents, which is where the queries are actually issued from.

**Alternatives:** Option A — index in aibox CLI (rejected: tight coupling, release friction). Option C — generic parsing in aibox + schema-aware overlay in processkit (rejected: two-step, unclear ownership).

**Source:** DISC-002 Q2 resolution.

## DEC-020 — MCP servers = official Python SDK + uv PEP 723 inline deps (2026-04-06)

**Decision:** Skill MCP servers are Python source code using the official `mcp` SDK, delivered as standalone scripts with PEP 723 inline dependency metadata. No `pyproject.toml`, no manual venv. `uv` (already present in all aibox containers) handles resolution and caching on first run. STDIO transport only. Container requirements: Python ≥ 3.10 and `uv` — both already present.

**Rationale:** The official SDK is the standard; avoiding it means maintaining custom JSON-RPC code. PEP 723 + uv eliminates per-skill environment setup. First-run cost (~5–10s dependency resolution) is amortized by uv's cache. ~300–400 MB added is acceptable for a dev container that already carries Rust/Node/LaTeX toolchains. Option B (pydantic-only) is the documented escape hatch if container size becomes critical.

**Alternatives:** Option A — raw JSON-RPC with zero dependencies (rejected: reimplements the protocol, fragile for complex servers). Option B — pydantic-only minimal server (kept as escape hatch; not default). Pre-install MCP packages in the base image (rejected: couples aibox image to SDK version, breaks skill independence).

**Source:** DISC-002 §8.

## DEC-019 — Skills are multi-artifact packages (2026-04-06)

**Decision:** A skill is no longer a single `SKILL.md` file. It is a directory containing `SKILL.md` (three-level instructions), `examples/`, `templates/` (YAML frontmatter entity scaffolds), and optionally `mcp/` (Python MCP server source + `mcp-config.json` snippet). Skills declare `uses:` dependencies in frontmatter; the dependency graph is strictly downward (Layer 0 → 4).

**Rationale:** Markdown alone cannot deliver what a capable skill needs: examples of good output, parametric templates for new entities, and programmatic tool capabilities. Bundling these as a package keeps them versioned together and makes skills composable via explicit `uses:` references. The three-level principle (Level 1: 1-3 sentences, Level 2: key workflows, Level 3: full reference) keeps `SKILL.md` scannable for agents.

**Alternatives:** Keep single-file skills (rejected: no way to ship deterministic tool capabilities or templates). Separate examples repo (rejected: breaks versioning and discovery). Skills as code-only (rejected: loses the instructional markdown that agents read first).

**Source:** DISC-002 §3, §6, P3, P4, P15.

## DEC-018 — Two-repo split: aibox + processkit (2026-04-06)

**Decision:** Split content from infrastructure into two repos. **aibox** (`projectious-work/aibox`) holds the Rust CLI, container images, and devcontainer scaffolding. **processkit** (`projectious-work/processkit`) holds primitives, schemas, all 85 skills + new process-primitive skills, process templates, packages, and MCP servers. processkit releases as git tags; aibox consumes a specific tag via `aibox init`. Both repos dogfood aibox for their own dev environments.

**Rationale:** The DISC-001 exploration conflated content (what gets scaffolded into projects) with infrastructure (how the container runs). The conflation made both harder to evolve — every skill change required an aibox CLI release, every CLI change risked destabilizing skills. Splitting by concern lets skills evolve at their own pace, enables community skill packages (via `aibox process install <git-url>`), and gives users a clear mental model: aibox = infra, processkit = content. The bootstrap loop (aibox needs processkit for content, processkit needs aibox for its devcontainer) is resolved by version pinning on both sides.

**Alternatives:** Keep everything in aibox (rejected: couples release cycles, bloats repo). Three+ repos, e.g. separating technical skills from process skills (rejected: the distinction is blurry and splitting creates friction). Skills in aibox but primitives in processkit (rejected: arbitrary boundary).

**Source:** DISC-002 §5, P7, Q1 (name resolution), §15 (all 85 skills in processkit).

## DEC-017 — aibox scope refocus: dev container + skills scaffolding (2026-04-06)

**Decision:** Refocus aibox around one job: **provide consistent, containerized development environments for AI-assisted work.** Analogy: uv is for Python environments, aibox is for AI work environments. Drop from scope: RBAC enforcement, enterprise governance, multi-repo trust architectures, certificate-based authorization, verification manifests, deterministic event logging, workflow execution, Docker wrapping. These are either another project's concern (governance → likely a Kubernetes-based platform) or things aibox should not do (inner-system fallacy — re-exposing Docker behind its own config layer).

**Rationale:** DISC-001 explored enterprise scenarios in depth and produced 74 decisions with 14 internal contradictions — a sign the scope had expanded beyond what aibox should be. Tightening scope to the dev-environment job removes the contradictions, clarifies the product pitch, and lets the remaining work (processkit content, MCP servers, CLI polish) proceed without governance coupling. DISC-001 research is preserved for whatever governance platform eventually needs it.

**Alternatives:** Keep enterprise governance in scope (rejected: scope creep produced contradictions and no clear product). Build both the environment and the governance platform together (rejected: violates single-responsibility, delays both). Drop processkit content too, just ship containers (rejected: users would build the same context/skills scaffolding by hand every time).

**Source:** DISC-002 §1-4, §12, P1, P2, P9.

## DEC-016 — Declarative config + minimal base images (2026-03-23)

**Decision:** Redesign aibox around a single published base image (base-debian), unified add-on system with per-tool version selection, 13 composable process packages replacing 4 monolithic levels, and declarative skill management. No backward compatibility — clean break.

**Rationale:** The 10 pre-compiled image architecture creates maintenance burden (TeX Live duplicated 3x across 3 Dockerfiles), limits composability (can't combine Node+Go without a dedicated image), and gives users no control over which skills they deploy. The 4 monolithic process levels (minimal/managed/research/product) don't fit non-software projects (document, research, data). Moving everything to add-ons + composable process packages gives users full control while reducing our maintenance surface from 10 images to 1.

**Key decisions within:**
- Abstract base contract (Debian now, Alpine later) — not tied to specific distro
- LaTeX becomes an add-on with multi-stage builder (no dedicated base-latex image)
- Add-ons have internal recipe versioning; users select per-tool versions from curated lists
- 13 atomic process packages + 4 convenience presets, freely composable
- Core package (always present): agent-management + owner-profile skills, AIBOX.md + OWNER.md
- Content-addressed skill updates on sync
- AI providers: Claude, Aider, Gemini, Mistral (bring-your-own-model deferred)

**Alternatives:** Keep base-latex image for build speed (rejected — Docker layer caching + future GHCR cache image sufficient), keep 4 monolithic processes (rejected — too rigid for non-software projects), maintain backward compat (rejected — too few users, baggage not worth carrying).

## DEC-015 — Dogfood the product process template (2026-03-23)

**Decision:** Align aibox's own `context/` with the product process template it ships to users. Adopt BACK-NNN IDs in BACKLOG.md, add PROJECTS.md and PRD.md, install 8 product-relevant skills in `.claude/skills/`, close 13 completed GitHub issues, and update the public roadmap.

**Rationale:** aibox promotes structured work processes but wasn't fully following its own product template. Eating our own dogfood validates the template and reveals friction. The existing context/ was close but used a different backlog format (checkboxes vs BACK-NNN table) and lacked structured project tracking. GitHub had 16 open issues, 13 of which were already done — creating a false impression of outstanding work.

**Deviations from template:** STANDUPS.md omitted — session handovers in `project-notes/session-*.md` are more detailed and serve the same purpose. OWNER.md kept (not in product template but useful). Extra work-instructions kept (DOCKERFILE-PRACTICES.md, SCREENCASTS.md) as project-specific extensions. `backlog-context` skill customized for table format with BACK-NNN IDs.

**Alternatives:** Full template adoption including STANDUPS.md (redundant with session handovers), keep current format (misses dogfooding opportunity), automated migration tool (over-engineering for a one-time task).

## DEC-014 — Skills Library: curated quality over marketplace quantity (2026-03-22)

**Decision:** Ship 83 curated skills with reference files rather than providing a marketplace integration or a smaller "starter" set. Skills are embedded in the binary via `include_str!` and scaffolded on `aibox init`. No external download step.

**Rationale:** Marketplace research (SkillsMP: 97K skills, Skills.sh: 40K, ClawHub: 13.7K) revealed that 46.3% of publicly available skills are duplicates or near-duplicates (HuggingFace analysis). The ecosystem's #1 problem is quality, not quantity. A curated library with progressive disclosure (SKILL.md < 150 lines, reference files for depth) differentiates aibox from "skill slop." Embedding in the binary ensures skills work offline and are version-locked to the CLI.

**Categories chosen (14):** Process, Development, Language, Infrastructure, Architecture, Design & Visual, Data & Analytics, AI & ML, API & Integration, Security, Observability, Database, Performance, Framework & SEO. Based on marketplace demand analysis: infrastructure and data skills are vastly underserved relative to frontend/framework skills.

**Alternatives:** Marketplace-first (ClawHub/Skills.sh integration — deferred to backlog), smaller starter set with `aibox skill install` (adds complexity, network dependency), external file download during init (fragile, no offline support).

## DEC-013 — Granular vim mounts preserve image colorschemes (2026-03-22)

**Decision:** The docker-compose template mounts `.vim/vimrc` and `.vim/undo` individually instead of the entire `.vim/` directory. This preserves the image-baked `~/.vim/colors/` and `~/.vim/pack/` directories.

**Rationale:** The base image downloads 6 vim colorscheme files (gruvbox, catppuccin-mocha, catppuccin-latte, tokyonight, nord, dracula) into `/root/.vim/colors/` during Docker build. When the entire `.vim/` was mounted from the host (`.aibox-home/.vim/`), the image's `colors/` directory was shadowed. Result: `E185: Cannot find color scheme 'gruvbox'` in derived projects. Mounting only the two files we actually persist (vimrc and undo/) leaves the image's baked-in directories visible.

**Alternatives:** Copy colorschemes into `.aibox-home/.vim/colors/` during seed (duplicates files, maintenance burden), embed colorschemes via `include_str!` in the binary (bloat, version drift), post-create command to copy (fragile).

## DEC-012 — Reference file scaffolding via SkillDef type (2026-03-22)

**Decision:** Extend `scaffold_skills()` to deploy reference files alongside SKILL.md. Changed the skills data structure from `(&str, &str)` to a `SkillDef` type alias: `(&str, &str, &[(&str, &str)])` = `(name, content, [(ref_filename, ref_content)])`. Reference files go in `.claude/skills/<name>/references/`.

**Rationale:** 8 of the original 26 skills had reference files on disk (11 files total) but they were never deployed to derived projects — `scaffold_skills()` only wrote SKILL.md. With the expansion to 83 skills and 57 reference files, fixing this was a prerequisite. The `SkillDef` type alias satisfies clippy's type_complexity lint while keeping the flat `include_str!` embedding pattern.

**Alternatives:** Struct-based SkillDef (heavier, overkill for static data), dynamic file discovery at runtime (fragile, no compile-time guarantees), separate scaffolding function for references (unnecessary split).

## DEC-011 — Skills + Processes architecture: separate WHAT from HOW (2026-03-22)

**Decision:** Process declarations in context/ define WHAT processes exist ("there shall be backlog management"). Skills (SKILL.md standard) define HOW they're executed. Context stores the resulting artifacts (BACKLOG.md, DECISIONS.md, etc.). Skills come in flavors (e.g., backlog-context vs backlog-github) that users choose.

**Rationale:** Today's process presets bake both "what" and "how" into context template files. This makes them rigid — you can't swap from a context-file backlog to GitHub Issues without restructuring. By separating concerns: process declarations become thin ("there shall be X"), skills become the executable implementation, and artifacts remain in context/. This enables: swappable implementations, testable skills (via SKILL.md eval framework), thinner aibox scaffolding, and a clear boundary between aibox (infrastructure + curated skills) and derived projects (tailoring + execution).

**Relationship to SKILL.md standard:** The open standard at agentskills.io/specification provides the skill format. aibox provides curated, vetted skills. External marketplaces (ClawHub) are user responsibility.

**Implications:** Process presets (minimal/managed/research/product) become skill compositions. aibox.toml gains a [skills] section mapping processes to skill flavors. aibox doctor checks consistency between declared processes and installed skills.

**Alternatives:** Keep current monolithic process templates (simpler but rigid). Full framework integration like SAFe/PMBOK (too heavy for aibox scope — that's kaits territory).

---

Older decisions (DEC-001 through DEC-010): [archive/DECISIONS.md](archive/DECISIONS.md)
