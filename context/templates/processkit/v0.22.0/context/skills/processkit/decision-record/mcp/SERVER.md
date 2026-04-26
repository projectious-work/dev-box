# decision-record MCP server

ADR-style DecisionRecord management. Layer 2 — depends on `event-log`
and `actor-profile`.

## Tools

| Tool                                                                                                                                                | Purpose                                                  |
|-----------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|
| `record_decision(title, decision, rationale?, context?, alternatives?, consequences?, deciders?, related_workitems?, state?)`                       | Create a new DecisionRecord                              |
| `transition_decision(id, to_state)`                                                                                                                 | Move a DecisionRecord to a new state                     |
| `query_decisions(state?, limit?)`                                                                                                                   | List DecisionRecords                                     |
| `get_decision(id)`                                                                                                                                  | Fetch one DecisionRecord                                 |
| `supersede_decision(old_id, new_id)`                                                                                                                | Mark `old_id` as superseded by `new_id` (updates both)   |
| `link_decision_to_workitem(decision_id, workitem_id)`                                                                                               | Add a workitem ID to `spec.related_workitems`            |
| `reload_schemas()`                                                                                                                                  | Clear in-process schema + state-machine caches so a disk edit is picked up without a server restart (returns `{ok, cleared: {schemas, state_machines}}`). Scope: this server only. PEP 723 dep edits still require a harness restart. See DEC-QuickPine. |

## State machine

```
proposed → accepted → superseded (terminal)
    ↓
  rejected (terminal)
```

Source: `src/primitives/state-machines/decisionrecord.yaml`.

## Conventions

- `record_decision(state="accepted")` shortcuts the `proposed → accepted`
  flow for decisions made retroactively.
- `decided_at` is auto-stamped on entering `accepted`.
- `supersede_decision` enforces both sides — both files are updated and
  re-indexed atomically.

## Running

```bash
uv run context/skills/decision-record/mcp/server.py
```
