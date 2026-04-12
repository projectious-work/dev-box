# scope-management MCP server

Manages Scope entities — bounded containers grouping related work
(sprint, milestone, project, release, quarter).

## Tools

| Tool | Purpose |
|---|---|
| `create_scope(name, kind, starts_at?, ends_at?, goals?, description?, parent?)` | Create a new Scope in the `planned` state |
| `get_scope(id)` | Fetch a single Scope by ID |
| `transition_scope(id, to_state)` | Move through `planned → active → completed`, plus `cancelled` |
| `list_scopes(kind?, state?, limit?)` | List Scopes with optional filters |

Allowed `kind` values: `sprint`, `milestone`, `quarter`, `project`,
`release`, `other`.

## State machine

```
planned ──▶ active ──▶ completed
  │           │
  └─▶ cancelled ◀─┘
```

`completed` and `cancelled` are terminal. Reactivation is intentionally
not allowed; create a new Scope so historical entities that reference
the old one stay stable.

## Storage

Scope entities live at `<project-root>/context/scopes/<id>.md`.

## Running

```bash
uv run context/skills/scope-management/mcp/server.py
```
