# src/skills/

Multi-artifact skill packages. Each skill is a directory containing instructions
(SKILL.md), examples, templates, and optionally a Python MCP server.

See [FORMAT.md](FORMAT.md) for the complete skill package format spec.

## Directory layout

Skills are organized into 7 category subdirectories. `_lib/` is internal shared
utilities (not a public skill category).

```
skills/
  processkit/    — 33 skills for operating the processkit system
  engineering/   — 46 skills: software design, architecture, backend, languages
  devops/        — 15 skills: infrastructure, CI/CD, ops, monitoring
  data-ai/       — 11 skills: data science, ML, AI/LLM, embeddings
  product/       — 11 skills: product management, discovery, communication
  documents/     —  8 skills: document and content authoring
  design/        —  5 skills: visual, UI, and frontend design
  _lib/          — internal processkit Python library (not a skill)
```

## Skill package layout

```
<category>/<skill-name>/
  SKILL.md              ← Intro / Overview / Gotchas / Full reference
  scripts/              ← optional executable code (Python, Bash) — may be empty
  references/           ← optional deep-dive reference docs loaded on demand
  assets/               ← optional templates, fonts, icons used in output
  examples/             ← optional example outputs
  mcp/                  ← optional Python MCP server (processkit-specific)
```

## processkit skill hierarchy

The 33 processkit/ skills form a strict dependency hierarchy where
`layer: 0-4` skills depend only downward; the `layer: null` routing and
meta skills sit outside that dependency chain:

Layer 0 (foundation):
- `index-management` — SQLite index over all entity files. **MCP server.**
- `id-management` — allocate unique IDs. **MCP server.**
- `event-log` — append-only LogEntry. **MCP server.** (uses both above)

Layer 1 (primitive management):
- `actor-profile`, `role-management`

Layer 2 (core entities and operator workflows):
- `workitem-management` **MCP**, `decision-record` **MCP**,
  `artifact-management` **MCP**, `scope-management`,
  `category-management`, `cross-reference-management`,
  `binding-management` **MCP**, `morning-briefing`,
  `note-management`, `session-handover`, `standup-context`,
  `status-update-writer`

Layer 3 (process orchestration):
- `process-management`, `state-machine-management`, `gate-management`,
  `schedule-management`, `constraint-management`, `migration-management`

Layer 4 (cross-cutting coordination):
- `discussion-management`, `owner-profiling`, `context-grooming`,
  `agent-management`

Layer null (routing and meta skills):
- `model-recommender`, `skill-builder`, `skill-finder`, `skill-gate`,
  `skill-reviewer`, `task-router`

Technical and language skills (engineering/, devops/, data-ai/, etc.) are
also `layer: null` — the hierarchy above applies only to processkit/
skills that participate in the process primitive dependency graph.
