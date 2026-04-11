# discussion-management MCP server

Manages Discussion entities — structured, multi-turn conversations
that explore questions and produce DecisionRecords.

## Tools

| Tool | Purpose |
|---|---|
| `open_discussion(question, participants?, related?, body?)` | Open a new Discussion in the `active` state |
| `get_discussion(id)` | Fetch a single Discussion by ID |
| `transition_discussion(id, to_state)` | Move through `active ↔ resolved → archived` |
| `add_outcome(id, decision_id)` | Append a DecisionRecord ID to the discussion's outcomes |
| `list_discussions(state?, limit?)` | List Discussion entities |

## State machine

```
active ──▶ resolved ──▶ archived
  ▲           │
  └───────────┘     (reopening allowed)
```

`archived` is terminal. Reopening a resolved discussion is intentional
— sometimes a question comes back.

## Storage

Discussion entities live at `<project-root>/context/discussions/<id>.md`.
The Markdown body holds the actual conversation; the frontmatter
captures metadata (question, participants, outcomes, timestamps).

## Running

```bash
uv run context/skills/discussion-management/mcp/server.py
```

## Notes

Use a Discussion when you don't yet know the answer; use a
DecisionRecord when you do. The two are complementary — discussions
often produce 0..many DecisionRecords as their `outcomes`. The
`add_outcome` tool records this linkage so later queries can find the
audit trail behind a decision.
