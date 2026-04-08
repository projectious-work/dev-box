# aibox v0.16.4 — INDEX.md install fix + AGENTS.md placeholder vocabulary

Patch release. Two fixes that came out of the parallel processkit
cleanup session:

## Bug fix — INDEX.md files are now installed (BACK-116)

processkit ships INDEX.md files at every content level
(top-level, `skills/`, `processes/`, `primitives/schemas/`,
`primitives/state-machines/`, `scaffolding/`, `packages/`, plus the
`primitives/` parent). They're tracked in `PROVENANCE.toml` and are
explicitly part of the shipping contract — agents browsing a project's
`context/` directory expect to find them.

v0.16.0's `install_action_for` incorrectly classified them as
"processkit-internal docs" and skipped all 8 files via a blanket
`parts.last() == "INDEX.md" → Skip` rule. v0.16.4 drops the blanket
skip and routes each INDEX.md to its parent directory's destination:

| Cache path | Live install destination |
|---|---|
| `INDEX.md` (top of src) | `context/INDEX.md` |
| `skills/INDEX.md` | `context/skills/INDEX.md` |
| `processes/INDEX.md` | `context/processes/INDEX.md` |
| `primitives/schemas/INDEX.md` | `context/schemas/INDEX.md` |
| `primitives/state-machines/INDEX.md` | `context/state-machines/INDEX.md` |

The remaining three (`primitives/INDEX.md`, `scaffolding/INDEX.md`,
`packages/INDEX.md`) have no sensible live destination and remain
skipped — but they're still present in the immutable cache mirror at
`context/templates/processkit/<version>/`, so no information is lost.

8 new tests in `content_install::tests` covering each routing case,
plus the synth-cache count tests in `content_init::tests` updated for
the new install/skip totals.

## Feature — Class A placeholder vocabulary for templated AGENTS.md (DEC-032 / BACK-117)

processkit's next release ships an `AGENTS.md` template using the
**three-class placeholder model**:

- **Class A — aibox-rendered** (11 placeholders, locked contract).
  Substituted by aibox at install time from `aibox.toml` + install
  context.
- **Class B — owner-supplied.** Project-specific facts processkit's
  onboarding skill asks the owner to fill in conversation. aibox does
  not touch these.
- **Class C — discoverable.** Facts the agent extracts from the
  codebase and the owner confirms. aibox does not touch these.

v0.16.4 extends `cli/src/context.rs::render` from the single-key
`{{project_name}}` substitution shipped through v0.16.3 to a
HashMap-based renderer taking the locked Class A vocabulary:

```text
{{PROJECT_NAME}}        {{CONTAINER_HOSTNAME}}    {{CONTAINER_USER}}
{{AIBOX_VERSION}}       {{AIBOX_BASE}}            {{PROCESSKIT_SOURCE}}
{{PROCESSKIT_VERSION}}  {{INSTALL_DATE}}          {{ADDONS}}
{{AI_PROVIDERS}}        {{CONTEXT_PACKAGES}}
```

Unknown placeholders (Class B/C, future Class A additions, typos)
**pass through `render()` untouched** so processkit's onboarding skill
can find and fill them with the project owner.

### New `InstallAction::InstallTemplated` variant

`install_action_for` returns this for files that need rendering
before the copy. Today: only `scaffolding/AGENTS.md`. The walker in
`install_files_from_cache_with_vars` dispatches:

- `Install` → `fs::copy` (verbatim, as before)
- `InstallTemplated` → `read → render → write_if_missing`
- `Skip` → no-op

### Templated files use write_if_missing semantics

The first install writes the rendered content; subsequent syncs leave
the file alone. This prevents the install path from clobbering owner
edits to AGENTS.md (which Strawman D explicitly permits). Trade-off:
upstream improvements to a templated file do NOT auto-propagate. The
user re-inits or manually copies from the cache mirror to pick them
up. v0.16.5+ will fix this properly with rendered-mirror three-way
diff.

### Three-way diff treats templated files as skipped

`content_diff.rs` has a third arm for `InstallTemplated` that returns
Skip. The reason: the templates mirror holds the unrendered cache
content while the live file holds the rendered output, so SHA
comparison would always false-positive as "ChangedLocally". Fixed
properly in v0.16.5+.

### Lowercase `{{project_name}}` alias removed

The v0.16.3 thin-pointer template used `{{project_name}}` (lowercase).
v0.16.4 normalizes to uppercase per the locked vocabulary; the
lowercase alias is **deliberately removed**. Existing v0.16.3 CLAUDE.md
thin pointers will need re-init or a manual edit. Per session
decision: clean up, no version compatibility.

## Documentation

`context/notes/session-handover-2026-04-08.md` — comprehensive
session handover document written this session. Captures the
post-v0.16.x project state, the aibox/processkit boundary, common
principles, the release ritual, the four-release session log
(v0.16.0 → v0.16.3), open work, and references for the next agent
picking up the project.

## New backlog item

**BACK-118** — Implement `[skills].include` / `[skills].exclude`
filtering. Discovered while designing MCP registration (C4 review):
these fields exist in the schema and parse correctly but are
reserved/no-op in v0.16.x. When implemented, the install pipeline
filters per-skill-name and the MCP registration step (queued for
v0.16.5) will follow automatically.

## Quality gates

- 467/467 tests pass (was 460 in v0.16.3) — 7 new render/build_map
  tests + 6 new INDEX.md routing tests
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo audit` clean

## Migration impact

**Mostly backwards compatible.** No config schema changes. The only
break is the dropped lowercase `{{project_name}}` alias — any v0.16.3
CLAUDE.md thin pointer using the lowercase form will see the literal
`{{project_name}}` text after a re-render. Re-init or hand-edit to
fix.

Existing v0.16.3 projects pick up:
- INDEX.md files on the next `aibox sync` (which auto-installs
  processkit content per v0.16.1's BACK-110 path)
- The render extension transparently — no user action needed

## What's next (v0.16.5)

- MCP server registration: aibox walks `context/skills/*/mcp/mcp-config.json`
  and writes the harness-specific config file (`.mcp.json` for Claude
  Code, `.codex/mcp.json` for Codex CLI, etc.) with non-destructive
  merge for user-added entries.
- Survey of harness-specific MCP config shapes (Claude Code, Codex CLI,
  Cursor, Continue, OpenCode) to confirm whether processkit can ship a
  single shape or aibox needs per-harness translators.
- Bake `python3` + `uv` into the base-debian image (currently in the
  python addon) so PEP 723 MCP server scripts work out of the box for
  any project that consumes processkit.
- Rendered-mirror three-way diff for templated files (proper fix for
  the v0.16.4 limitation above).

## Linked decisions

- **DEC-032** — Class A placeholder vocabulary for templated installs
- **DEC-031** — xdg-open shim in base image
- **DEC-030** — three-tier privacy model
- **DEC-029** — list_versions GitHub fallback + sync perimeter catch-up
- **DEC-028** — sync auto-installs processkit, init picks the version
- **DEC-027** — aibox v0.16.0: rip the bundled process layer
- **DEC-026** — Cache-tracked processkit reference
- **DEC-025** — Generic content-source release-asset fetcher
