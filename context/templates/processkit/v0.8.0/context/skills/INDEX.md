# src/skills/

Multi-artifact skill packages. Each skill is a directory containing instructions
(SKILL.md), examples, templates, and optionally a Python MCP server.

See [FORMAT.md](FORMAT.md) for the complete skill package format spec.

## Status at v0.4.0

- **106 skill directories total.**
- **85 skills** migrated mechanically from the original aibox `templates/skills/`.
  Frontmatter upgraded to processkit v1 format. Bodies preserved as-is. Most
  already follow a "When to Use" + "Instructions" structure that roughly maps to
  Level 1 + Level 2 + Level 3 content, but they do not yet carry explicit
  `## Level 1/2/3` headings. A full explicit three-level rewrite is tracked as
  a follow-up (see context/BACKLOG.md in this repo).
- **21 new process-primitive skills** authored specifically for processkit.
  Three new skills added in v0.4.0:
  - `migration-management` (Layer 3) — manages Migration entities
  - `owner-profiling` (Layer 4) — interview-driven owner profile bootstrap
  - `context-grooming` (Layer 4) — periodic context cleanup and pruning

  All 21 use explicit `## Level 1/2/3` headings and serve as the reference
  style for future skills and for the three-level rewrite. Six of them ship
  working Python MCP servers (added in v0.3.0). The three new v0.4.0 skills
  are markdown-only (no MCP servers yet).

## The 21 new process-primitive skills

Layer 0 (foundation — sibling skills, both required by every entity-creating skill):
- `index-management` — read side: SQLite index over all entity files. **MCP server.** (added v0.3.0)
- `id-management` — write side: allocate unique IDs. **MCP server.** (added v0.3.0)
- `event-log` — append-only LogEntry, probabilistic. **MCP server.** (uses both above)

> **Intra-Layer-0 ordering:** `index-management` and `id-management` are the
> two truly foundational skills. `event-log` is also Layer 0 but conceptually
> "sits atop" the other two — it uses them. This is the only intra-layer
> dependency in the entire hierarchy. The strict-downward rule applies to
> Layers 1+ unchanged.

Layer 1 (primitive management):
- `actor-profile` — humans, agents, services
- `role-management` — named sets of responsibilities

Layer 2 (core entities):
- `workitem-management` — WorkItem lifecycle. **MCP server.**
- `decision-record` — DecisionRecord (ADR pattern). **MCP server.**
- `scope-management` — Scope (sprints, milestones)
- `category-management` — classification axes
- `cross-reference-management` — lightweight frontmatter references
- `binding-management` — scoped/temporal many-to-many relationships. **MCP server.**

Layer 3 (process orchestration):
- `process-management` — Process definitions
- `state-machine-management` — StateMachine customization
- `gate-management` — validation checkpoints
- `schedule-management` — cadences and triggers
- `constraint-management` — rules and limits
- `migration-management` — manages Migration entities (added v0.4.0)

Layer 4 (cross-cutting):
- `discussion-management` — multi-turn exploratory conversations
- `metrics-management` — quantified observations
- `owner-profiling` — interview-driven owner profile bootstrap (added v0.4.0)
- `context-grooming` — periodic context cleanup (added v0.4.0)

## Skill package layout

```
<skill-name>/
  SKILL.md              ← Intro / Overview / Gotchas / Full reference
  scripts/              ← optional executable code (Python, Bash) — may be empty
  references/           ← optional deep-dive reference docs loaded on demand
  assets/               ← optional templates, fonts, icons used in output
  examples/             ← optional example outputs (kept; not in Anthropic canonical layout)
  mcp/                  ← optional Python MCP server (processkit-specific)
```

For the canonical specification, see [`FORMAT.md`](FORMAT.md).

## Skill hierarchy

Skills reference lower-layer skills via `uses:` in frontmatter. Strictly
downward, with one documented exception in Layer 0.

```
Layer 0: index-management, id-management, event-log (foundation)
           - index-management and id-management are the absolute foundation (no deps)
           - event-log is also Layer 0 but uses both above (intra-layer edge)
Layer 1: role-management, actor-profile
Layer 2: workitem-management, decision-record, scope-management,
         category-management, cross-reference-management, binding-management
Layer 3: process-management, state-machine-management, gate-management,
         schedule-management, constraint-management, migration-management
Layer 4: discussion-management, metrics-management,
         owner-profiling, context-grooming
```

Technical and language skills (python-best-practices, rust-conventions,
fastapi-patterns, ...) are `layer: null` — the hierarchy only applies to
process-primitive skills.
