# gate-management MCP server

Manages Gate entities — validation checkpoints that processes must
pass (code review, tests green, security scan, stakeholder approval).

## Tools

| Tool | Purpose |
|---|---|
| `create_gate(name, description, kind, validator, validator_command?, required_roles?, blocking?, evidence_required?)` | Create a new Gate |
| `get_gate(id)` | Fetch a single Gate by ID |
| `list_gates(kind?, blocking?, limit?)` | List Gate entities |
| `evaluate_gate(id, outcome, actor?, evidence?, reason?)` | Record an evaluation as a LogEntry |

`kind` must be one of `manual`, `automated`, `hybrid`.
`outcome` (in evaluate_gate) must be one of `passed`, `failed`,
`waived`. A waived outcome requires a `reason`. If a Gate has
`evidence_required: true`, a `passed` outcome requires `evidence`.

## Storage

Gate entities live at `<project-root>/context/gates/<id>.md`.
Gate evaluations are LogEntries (not Gate edits) stored at
`<project-root>/context/logs/<id>.md`. The Gate file itself is
immutable from the evaluation perspective — the gate is the rule;
evaluations are events.

## Running

```bash
uv run context/skills/gate-management/mcp/server.py
```

## Notes

Binding gates to processes is done via binding-management with
`type: process-gate` so the same Gate can apply to multiple processes
with different scopes. See the binding-management SKILL.md for the
pattern.
