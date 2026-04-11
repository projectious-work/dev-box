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
