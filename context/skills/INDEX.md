# src/skills/

Multi-artifact skill packages. Each skill is a directory containing instructions
(SKILL.md), examples, templates, and optionally a Python MCP server.

See [FORMAT.md](FORMAT.md) for the complete skill package format spec.

## Directory layout

Skills are organized into 7 category subdirectories. `_lib/` is internal shared
utilities (not a public skill category).

```
skills/
  processkit/    — 30 skills for operating the processkit system
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

The 30 processkit/ skills form a strict dependency hierarchy (strictly
downward, except for one documented intra-Layer-0 edge):

Layer 0 (foundation):
- `index-management` — SQLite index over all entity files. **MCP server.**
- `id-management` — allocate unique IDs. **MCP server.**
- `event-log` — append-only LogEntry. **MCP server.** (uses both above)

Layer 1 (primitive management):
- `actor-profile`, `role-management`

Layer 2 (core entities):
- `workitem-management` **MCP**, `decision-record` **MCP**, `scope-management`,
  `category-management`, `cross-reference-management`, `binding-management` **MCP**

Layer 3 (process orchestration):
- `process-management`, `state-machine-management`, `gate-management`,
  `schedule-management`, `constraint-management`, `migration-management`

Layer 4 (cross-cutting):
- `discussion-management`, `metrics-management`, `owner-profiling`,
  `context-grooming`, `note-management`, `morning-briefing`, `standup-context`,
  `status-update-writer`, `model-recommender`, `skill-finder`, `skill-builder`,
  `skill-reviewer`, `session-handover`, `agent-management`

Technical and language skills (engineering/, devops/, data-ai/, etc.) are
`layer: null` — the hierarchy only applies to processkit/ skills.
