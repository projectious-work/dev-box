# binding-management MCP server

Manage Binding entities — scoped, temporal, many-to-many relationships
between any two entities. Layer 2 — depends on `event-log` and
`actor-profile`.

## Tools

| Tool                                                                                                       | Purpose                                                  |
|------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|
| `create_binding(type, subject, target, scope?, valid_from?, valid_until?, conditions?, description?)`     | Create a new Binding under `context/bindings/`           |
| `end_binding(id, end_date?)`                                                                              | Set `valid_until` (default: today)                       |
| `query_bindings(type?, subject?, target?, scope?, active_only?, limit?)`                                  | Query Bindings with filters; default returns active only |
| `resolve_bindings_for(entity_id, type?, at_time?)`                                                        | Find all Bindings whose subject or target is `entity_id` |

## Active vs ended

A binding is **active** at a time `t` if `valid_from ≤ t ≤ valid_until`
(treating absent endpoints as open). The default for `query_bindings` is
`active_only=True`. Pass `False` to see historical bindings.

## When to use a Binding vs a cross-reference

| Situation                                              | Use         |
|--------------------------------------------------------|-------------|
| "A blocks B"                                           | cross-ref   |
| "Alice is a developer" (globally)                     | cross-ref (in Actor) |
| "Alice is tech lead on project X for 2026"             | **Binding** |
| "Security gate applies to release process on main"    | **Binding** |

See `binding-management` SKILL.md for the rule and examples.

## Running

```bash
uv run context/skills/binding-management/mcp/server.py
```
