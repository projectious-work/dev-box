# AGENTS.md

<!-- pk-compliance-contract v2 BEGIN -->
<!-- pk-compliance v2 -->

## processkit Compliance Contract (ToC)

Ten non-negotiables — full text in
[`context/skills/processkit/skill-gate/SKILL.md`](context/skills/processkit/skill-gate/SKILL.md):
(1) `route_task` before writes; (2) 1% skill-finder rule; (3) commit
to entity creation same turn; (4) write via MCP, never hand-edit
`context/` YAML; (5) read via `index-management`; (6) log events for
non-MCP state changes; (7) record accepted cross-cutting decisions;
(8) acknowledge user decision language (`record_decision` or
`skip_decision_record`); (9) `context/templates/` is read-only;
(10) never hand-edit merged harness MCP config.
<!-- pk-compliance-contract v2 END -->

## About & session start

<!-- PLACEHOLDER:PROJECT_DESCRIPTION class=B
     One paragraph: what aibox does, who it is for, and what
     success looks like. Anything longer belongs in README.md. -->
{{PROJECT_DESCRIPTION}}

Run `pk-resume` before acting. Provider-specific files (`CLAUDE.md`,
`CODEX.md`, `.cursor/rules`, …) are thin pointers — edit **this** file.

## First-time setup (delete once done)

This file ships from
[processkit](https://github.com/projectious-work/processkit) as a
template. Scan for `{{TOKEN}}` strings and `<!-- PLACEHOLDER:TOKEN
class=X -->` comments, then follow the matching protocol below. Delete
this entire section once every placeholder is filled in.

| Class | Source | What you do |
|---|---|---|
| **A — installer** | Installer substitutes at render time (`aibox`, `v0.23.1`). | If still literal, read `aibox.toml` or ask the owner. |
| **B — owner** | Owner-only info: description, conventions, gotchas. | Interview the owner one question at a time. |
| **C — discoverable** | Build/test/lint commands from `package.json` / `Makefile` / `pyproject.toml` / etc. | Propose a value, owner confirms, write it in. |

Unannotated placeholders default to **Class B**.

## Skill guards (if/then)

- Editing/creating a file under `context/` → call `find_skill`; never
  hand-edit entity YAML.
- Creating or transitioning any entity (WorkItem, Decision, Note,
  Artifact, Discussion, Scope, Gate, Binding) → use the relevant
  `*-management` MCP tool.
- Pending migration under `context/migrations/pending/` → use
  `migration-management` MCP; don't move files by hand.
- Cross-cutting recommendation accepted → `record_decision` or
  `skip_decision_record(reason=...)` same turn.
- Authoring/reviewing a skill → `skill-builder` / `skill-reviewer`.
- Status briefing / standup / wrap-up → `status-briefing` /
  `standup-context` / `session-handover`.
- Retro / retrospective / post-release review / post-mortem →
  `retrospective` skill (`/pk-retro`).
- Any domain-specific task (PRD, audit, research ingest, discussion,
  backlog add) → `find_skill` first; see the six mandatory skill-check
  classes in `skill-gate/SKILL.md`.
- Otherwise → browse `context/skills/INDEX.md` before falling back to
  general knowledge.
- Schema-invalid LogEntry repair → prefer
  `pk-doctor --fix=schema_filename --yes` (narrow, known-safe patches
  like inserting `actor: system` for pre-TeamMember logs). Direct
  hand-edit of a schema-invalid entry is permitted as an escape
  hatch; commit with a clear reference. LogEntries remain
  append-only for the normal write path.

## Sub-agent delegation

Harness `Agent`-tool sub-agents inherit the main session's permission
policy but run ephemerally — any unallowed tool call errors
immediately rather than prompting. Delegate **read-only** work to
sub-agents (Read, search Bash, MCP `query_*` / `get_*` / `search_*` /
`list_*`); keep **mutating** work on the main session (Write, Edit,
new `mkdir`, MCP `create_*` / `transition_*` / `record_*` / `link_*`
/ `open_*` / `log_event`, `git` mutations). A sub-agent permission
block is not a cue to broaden the allowlist — move the write back to
main.

## Setup

<!-- PLACEHOLDER:BUILD_COMMAND class=C -->
<!-- PLACEHOLDER:TEST_COMMAND class=C -->
<!-- PLACEHOLDER:LINT_COMMAND class=C -->

```sh
{{BUILD_COMMAND}}
{{TEST_COMMAND}}
{{LINT_COMMAND}}
```

<!-- pk-commands BEGIN -->
<!--
build: "{{BUILD_COMMAND}}"
test: "{{TEST_COMMAND}}"
lint: "{{LINT_COMMAND}}"
fmt: ""
typecheck: ""
-->
<!-- pk-commands END -->

## Code style & PRs

<!-- PLACEHOLDER:CODE_STYLE_NOTES class=B -->
<!-- PLACEHOLDER:PR_CONVENTIONS class=B -->
{{CODE_STYLE_NOTES}}
{{PR_CONVENTIONS}}

## processkit preferences

Runtime config lives in per-skill `context/skills/<name>/config/settings.toml`.
MCP servers read them on every call — no restart needed.
<!-- PLACEHOLDER:PROCESSKIT_PREFS class=B -->
{{PROCESSKIT_PREFS}}

## AI agents on this project

Configured providers: **claude, codex**. Coordinate via the entity
layer (`workitem-management`, `event-log`, `discussion-management`)
rather than assuming you are alone.

### Team model (v0.19.0)

- **Roles** live under `context/roles/` — organisational labels for
  routing. Seniority is a pure-ordinal attribute, not baked into slugs:
  ladder `junior → specialist → expert → senior → principal`.
- **Persistent identities** (humans + named AI personas + services)
  live as directory trees under `context/team-members/<slug>/` —
  entity file, `persona.md`, A2A `card.json`, and six memory tiers
  (`knowledge/`, `journal/`, `skills/`, `relations/`, `lessons/`,
  `private/`; the last is gitignored by `.gitignore.example`).
- **Ephemeral invocations** are `(role, seniority)` dispatches resolved
  at call time — no team-member entity needed.
- **Models** live under `context/models/` as first-class entities, one
  per `(provider, family)` with nested versions. Each declares an
  `equivalent_tier` in the T-shirt capacity ladder
  (`xs / s / m / l / xl / xxl`, extensible both directions).
- **Routing** via `model-recommender.resolve_model(role, seniority,
  team_member?, scope?, task_hints?)` — an 8-layer precedence chain
  reading `model-assignment` bindings from `context/bindings/`. A
  default binding pack ships at
  `context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml`.
  Use `/pk-explain-routing` for a trace.

Create a new persistent AI team-member with
`team-manager.create_team_member(name, type=ai-agent, slug, default_role, default_seniority, ...)`;
the `name` must come from the international name pool at
`context/skills/processkit/team-manager/data/name-pool.yaml`.

## Project-specific notes

<!-- PLACEHOLDER:NONOBVIOUS_GOTCHAS class=B -->
{{NONOBVIOUS_GOTCHAS}}

---

<sub>Scaffolded by processkit `v0.23.1` on `2026-04-29`. Re-rendered on each installer sync.</sub>
