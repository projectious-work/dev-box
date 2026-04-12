# Changelog

All notable changes to processkit are documented here.
Versions follow [Semantic Versioning](https://semver.org/).

---

## [v0.13.0] — 2026-04-11

### Added

- **`task-router` skill and MCP server** — primary routing entry point
  for processkit agents. `route_task(task_description)` returns the
  matching skill, project-specific process override (from
  `context/processes/`), and recommended MCP tool in a single call
  without an LLM call. Two-phase heuristic routing: keyword match
  against 13 domain groups (Phase 1), token-overlap scoring within
  the group's tools (Phase 2), skill-finder trigger-table fallback for
  cross-domain tasks. Return shape includes `skill_description_excerpt`
  (first 150 chars), `tool_qualified` (`{server}__{tool}` collision-safe
  naming), `confidence`, `routing_basis`, and `candidate_tools[]`.
- **`skill-finder` MCP server** — new MCP server for the existing
  skill-finder skill. Tools: `find_skill(task_description)` (trigger-
  phrase table lookup + token-overlap scoring) and `list_skills(category?)`
  (catalog browser). Both tools are read-only with `readOnlyHint=true`.
- **`skill-gate` meta-skill** — provider-neutral prose skill that
  enforces the 1% rule: if there is even a 1% chance a processkit skill
  covers the task, check the router first. Includes decision graph,
  rationalization pre-emption table (5 entries), and escape hatch for
  agents already operating inside a named skill workflow.

### Changed

- **MCP tool prerequisite prompts (Track C)** — all 20 entity-mutating
  MCP tools (`create_*`, `transition_*`, `link_*`, `record_*`, `open_*`)
  now carry a prerequisite sentence in their docstring: call
  `route_task()` or confirm you are inside a named skill workflow before
  using the tool. These prompts appear in the tool schema every turn.
- **`skill-builder` and `skill-reviewer` updated (Track E)** — skill
  `description:` field convention changed from "Use when…" summaries to
  one-sentence imperatives (verb-noun, ≤150 chars). Both skills enforce
  this: skill-builder's step 4 template uses the new format; skill-
  reviewer's Category 6 and Skill Killer #1 check for violations.
- **`AGENTS.md` routing reference** — `find_skill()` replaced by
  `route_task()` as the primary 1% rule entry point. `task-router` added
  to the mandatory MCP server table.
- **`context/processes/release.md`** — fleshed out from generic stub to
  processkit-specific: added `breaking-change-audit`, `update-docs-site`,
  `stamp-provenance`, `push`, `build-and-upload-release`, `deploy-docs`
  steps; docs-deploy WildButter/aibox#42 blocker documented.

---

## [v0.12.0] — 2026-04-11

### Added

- **`artifact-management` skill and MCP server** — new Layer 2
  processkit primitive for registering and retrieving completed
  deliverables (documents, datasets, builds, diagrams, URLs, etc.).
  Supports two usage patterns: self-hosted (Markdown body in the
  entity file) and pointer (external URL or path via `location`).
  MCP tools: `create_artifact`, `get_artifact`, `query_artifacts`,
  `update_artifact`. Artifact has no state machine (`state_machine:
  null`) — it is a catalogue record, not a work-tracking entity.
- **`skill-finder` updated** — new trigger phrases for
  `artifact-management` (`"register an artifact"`, `"catalog this
  document"`, `"store this deliverable"`, `"link this design file"`)
  and a one-liner in the Process category.
- **`context/skills/INDEX.md`** — `processkit/` skill count updated
  to 31; Layer 2 entry updated to include `artifact-management **MCP**`.

---

## [v0.11.1] — 2026-04-11

### Fixed

- **`ids.py`: `pascal` and `camel` are now distinct word styles** —
  `pascal` produces PascalCase (`BoldVale`, every word capitalised);
  `camel` now correctly produces true camelCase (`boldVale`, first word
  lowercase). Previously `camel` incorrectly generated PascalCase.
  `id-management/config/settings.toml` updated to `word_style = "pascal"`.

### Changed

- **`context/` dogfood mirror synced** — `resolve_entity()` and
  partial-ID lookup now live in the installed `context/` copies of
  `index.py` and all MCP servers, consistent with the `src/`
  implementation shipped in v0.10.0.
- **`session-handover` SKILL.md** — log-entry writing steps updated
  with `generate_id` call, date-sharded path derivation, and word-pair
  ID template.
- **`context/skills/INDEX.md`** — expanded with skill package layout
  section and `_lib/` note.
- **`aibox.lock`** — updated to aibox v0.17.12, processkit v0.10.0.

---

## [v0.11.0] — 2026-04-11

### Added

- **Note schema: `links` field** — qualified Zettelkasten note-to-note
  links with `target`, `relation` (enum: elaborates, contradicts,
  supports, is-example-of, see-also, refines, sourced-from), and a
  required `context` sentence explaining *why* the connection matters.
  Tags group notes by topic; links build arguments.
- **note-management SKILL.md: linking section** — new "Linking notes"
  section with a full relation table and usage guidance; note template
  updated with links example.

### Changed

- **Note schema: type descriptions aligned with Luhmann/Ahrens taxonomy**
  — `insight` = permanent note (never discarded, part of the knowledge
  base); `reference` = literature note; `fleeting` = fleeting note.
  Schema description clarifies that permanent notes are not ephemeral.
  `type` descriptions and note-management SKILL.md type table updated
  accordingly.
- **Artifact schema: self-hosted and pointer patterns both documented** —
  description updated to acknowledge two valid usage patterns:
  self-hosted (Markdown body in the file, `location` optional) and
  pointer (external URL/path, `location` required). `location` removed
  from the `required` array. Note vs Artifact distinction added.
- **`config.py` `_skill_config_dir`** — now tries the `processkit/`
  category subdirectory before falling back to the flat layout, fixing
  ID generation and config loading after the v0.10.0 skills
  reorganization.
- **`.mcp.json`** — all 12 MCP server paths corrected from
  `context/skills/<name>/` to `context/skills/processkit/<name>/` after
  the v0.10.0 SteadyLeaf reorganization.
- **`AGENTS.md`** — added AGENTS.md lean-scope principle; expanded
  `context/` layout table to all 15 directories including `artifacts/`,
  `decisions/`, `discussions/`, `notes/`, `workitems/`, `logs/`,
  `migrations/`, `owner/`, `actors/`, `roles/`.

---

## [v0.10.0] — 2026-04-10

### Added

- **`index.resolve_entity()`** — new shared library function with 3-stage ID
  resolution: exact match → prefix match (missing slug) → word-pair match (bare
  word-pair like `StoutCrow`). Enables agents and humans to look up entities by
  the colloquial word-pair shorthand rather than needing the full canonical ID.

### Changed

- **Skills directory reorganized into 7 category subdirectories** (SteadyLeaf):
  `processkit/` (30), `engineering/` (46), `devops/` (15), `data-ai/` (11),
  `product/` (11), `documents/` (8), `design/` (5). All 126 skills moved in
  both `src/context/skills/` and `context/skills/`. All SKILL.md files updated
  with a consistent `category:` frontmatter field.
- **`skill-finder`** updated with a "By directory" table mapping the 7 categories
  to their on-disk subdirectory names.
- **`get_workitem`**, **`get_decision`**, **`get_entity`** (index-management) now
  use `resolve_entity()` and return `{"error": "..."}` instead of `null`/`None`
  when the entity is not found, and `{"error": "ambiguous..."}` with a candidate
  list when multiple entities match.
- **All `_load_*` helpers** (`_load_scope`, `_load_gate`, `_load_actor`,
  `_load_role`, `_load_discussion`, `_load_decision`, `_load_workitem`) updated
  to use `resolve_entity()` so word-pair lookup also works in write tools
  (transition, link, etc.).
- **`smoke-test-servers.py`** updated to find MCP server files in the new
  category subdirectory layout.

### Fixed

- `get_workitem` and `get_decision` previously returned `null` for not-found
  IDs, making it impossible for callers to distinguish "not found" from
  "found but empty". Both now return a structured error dict (SnappyCrane).
- Agents looking up entities by word-pair shorthand (e.g. `StoutCrow` instead
  of the full `BACK-20260410_1050-StoutCrow-create-brand-design-skill`) now
  resolve correctly via the new `resolve_entity()` fallback chain (SteadyPeak).

---

## [v0.9.0] — 2026-04-09

`src/` → `context/` mirror restructure (GrandLily). All processkit content
now lives under `src/context/` in the source tree and is mirrored to
`context/` in the target project root on `aibox sync`.

---

## [v0.8.0] — 2026-04-09

Auto-log side effects in all entity-mutating MCP servers. Every
`create_*`, `transition_*`, and `link_*` call now appends a `LogEntry`
without the caller doing anything extra.

---

## [v0.7.0] — 2026-04-09

Initial processkit v0.7.0 release. Core entity MCP servers, state machines,
schemas, and 85+ skills migrated from the original aibox templates.
