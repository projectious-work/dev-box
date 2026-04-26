# role-management MCP server

Manages Role entities — named sets of responsibilities Actors can fill.
processkit roles are descriptive, not restrictive (DEC-017): there is
no RBAC enforcement attached.

## Tools

| Tool | Purpose |
|---|---|
| `create_role(name, description, responsibilities?, skills_required?, default_scope?)` | Create a new Role |
| `get_role(id)` | Fetch a single Role by ID |
| `update_role(id, ...)` | Update description, responsibilities, skills_required, default_scope (name is intentionally not updatable — supersede instead) |
| `list_roles(default_scope?, limit?)` | List Role entities |
| `link_role_to_actor(role_id, actor_id, scope?, valid_from?, valid_until?, description?)` | Create a Binding (type: role-assignment) connecting an Actor to a Role, with optional scope and time bounds |

`default_scope` must be one of `project`, `sprint`, `permanent`.

## Storage

Role entities live at `<project-root>/context/roles/<id>.md`.
Bindings created by `link_role_to_actor` live at
`<project-root>/context/bindings/<id>.md`.

## Running

```bash
uv run context/skills/role-management/mcp/server.py
```

## Why link_role_to_actor creates a Binding

A Role file describes "what a reviewer does" — not "Alice is a
reviewer". The latter is a Binding because it can have scope ("on
project X"), time bounds ("for 2026"), and conditions. Editing the
Role file every time someone joins or leaves a role would mix
descriptive and assignment data and lose history. The Binding pattern
keeps the two concerns separate.
