# artifact-management MCP server

Artifact registration, retrieval, and update. Layer 2 — depends on
`event-log` and `index-management`.

## Tools

| Tool | Purpose |
|---|---|
| `create_artifact(name, kind, location?, format?, version?, owner?, produced_by?, tags?, description?)` | Register a new Artifact under `context/artifacts/` |
| `get_artifact(id)` | Fetch one Artifact with full spec |
| `query_artifacts(kind?, owner?, tags?, limit?)` | List Artifacts matching filters |
| `update_artifact(id, name?, kind?, location?, format?, version?, owner?, produced_by?, tags?)` | Update metadata fields on an existing Artifact |

## Valid kinds

`document`, `design`, `dataset`, `build`, `slides`, `video`, `spec`,
`diagram`, `url`, `other`

## Conventions

- Artifacts have no state machine — create and update are the only
  write operations.
- `produced_at` is auto-stamped on creation.
- `location` is required for pointer artifacts (content lives outside
  the repository). Omit for self-hosted artifacts whose content is
  the Markdown body.
- `update_artifact` only modifies fields you supply; omitted fields
  are preserved.

## Running

```bash
uv run context/skills/processkit/artifact-management/mcp/server.py
```
