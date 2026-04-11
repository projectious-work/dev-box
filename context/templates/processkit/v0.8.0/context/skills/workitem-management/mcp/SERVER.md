# workitem-management MCP server

WorkItem creation, transitions, queries, and links. Layer 2 — depends on
`event-log` and `actor-profile`.

## Tools

| Tool                                                                                                | Purpose                                                       |
|-----------------------------------------------------------------------------------------------------|---------------------------------------------------------------|
| `create_workitem(title, type?, priority?, assignee?, description?, parent?, scope?, labels?)`      | Create a new WorkItem under `context/workitems/`              |
| `transition_workitem(id, to_state, note?)`                                                          | Move a WorkItem to a new state (validates the state machine) |
| `query_workitems(state?, type?, assignee?, limit?)`                                                 | List WorkItems matching filters                               |
| `get_workitem(id)`                                                                                  | Fetch a WorkItem with full spec                               |
| `link_workitems(from_id, to_id, relation)`                                                          | Add a typed cross-reference (`blocks`/`blocked_by`/`parent`/`children`/`related_decisions`) |

## State machine

The default state machine is `workitem` (source:
`src/primitives/state-machines/workitem.yaml`):

```
backlog → in-progress → review → done
            ↓      ↑
          blocked ↑
            ↓      ↑
          backlog ←
(any state) → cancelled (terminal)
```

`transition_workitem` rejects invalid transitions and reports the
allowed targets in the error message.

Projects can override the machine by placing a same-named YAML at
`context/state-machines/workitem.yaml`.

## Auto-stamping

- `started_at` is set on first entry to `in-progress`
- `completed_at` is set on entering a terminal state (`done`/`cancelled`)
- `metadata.updated` is refreshed on every write

## Schema validation

`spec` is validated against `src/primitives/schemas/workitem.yaml` (or
the consumer override) using `jsonschema`. Validation errors are
returned as a list of human-readable strings.

## Running

```bash
uv run context/skills/workitem-management/mcp/server.py
```
