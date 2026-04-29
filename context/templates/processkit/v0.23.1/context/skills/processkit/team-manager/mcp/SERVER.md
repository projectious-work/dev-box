# team-manager MCP server

Manages TeamMember entities — persistent participants (humans, named
AI personas, services) with identity, personality, and tiered memory.
Replaces the deprecated actor-profile server.

## Tools

### Lifecycle

| Tool | Purpose |
|---|---|
| `create_team_member(name, type, slug, default_role?, default_seniority?, personality?, email?, handle?, joined_at?)` | Create a new TeamMember entity. Auto-reserves name if `type=ai-agent`. |
| `get_team_member(id)` | Fetch a single TeamMember by ID. |
| `list_team_members(type?, active_only?, role?, limit?)` | List TeamMembers with optional filters. |
| `update_team_member(id, **fields)` | Update mutable fields on an existing TeamMember. |
| `deactivate_team_member(id, left_at?)` | Mark inactive (sets active=false + left_at). |
| `reactivate_team_member(id)` | Undo deactivation (active=true, clears left_at). |

### Name pool

| Tool | Purpose |
|---|---|
| `reserve_name(name, team_member_slug)` | Mark a pool name reserved for a slug. |
| `release_name(name)` | Drop a reservation. |
| `list_available_names(kind?)` | List unreserved names, optionally filtered by gender bucket (`feminine`/`masculine`/`neutral`). |
| `suggest_name(kind?)` | Pick one unreserved name at random. |

### Memory tree

| Tool | Purpose |
|---|---|
| `init_memory_tree(slug)` | Create tier subdirectories, `.gitkeep` files, and scaffolded `persona.md` + `card.json` + `team-member.md` under `context/team-members/<slug>/`. |

### Export / import

| Tool | Purpose |
|---|---|
| `export_team_member(slug, output_path?)` | Build a tar.gz bundle with persona + card + entity + knowledge/ + skills/ + lessons/. Excludes journal/, relations/, private/. Redacts confidential/pii memory files. |
| `import_team_member(tarball_path)` | Extract, validate entity + Agent Card, require signature field present (crypto verification deferred), copy into `context/team-members/<slug>/`. |

### Consistency

| Tool | Purpose |
|---|---|
| `check_consistency(slug)` | 10 checks on a single team-member; returns structured findings. |
| `check_all_consistency()` | Aggregated findings across every team-member. |

## Storage

TeamMember entities live at
`<project-root>/context/team-members/<slug>/team-member.md`. Their ID
is `TEAMMEMBER-<slug>`.

## Running

```bash
uv run context/skills/processkit/team-manager/mcp/server.py
```

## Notes

Deprecates `actor-profile` per DEC-20260422_0233-SpryTulip. AI-agent
names are drawn from `data/name-pool.yaml`; reservations are managed
by `reserve_name` / `release_name` with atomic file rewrites.

For ephemeral role+seniority dispatches (no persistence), use
`task-router` — TeamMembers are only for persistent personas.
