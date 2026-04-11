# id-management MCP server

Allocate unique entity IDs following the project's configured format.
The write-side foundation, peer to `index-management`. Layer 0.

## Tools

| Tool                                  | Purpose                                                              |
|---------------------------------------|----------------------------------------------------------------------|
| `generate_id(kind, slug_text?)`       | Produce a fresh, collision-free ID for the given primitive kind     |
| `validate_id(id)`                     | Check format and decompose into kind/prefix/body                    |
| `list_used_ids(kind?, limit?)`        | List IDs already in use (read from the index)                       |
| `format_info()`                       | Return the project's ID configuration and the prefix registry       |

## When agents call this directly

Most of the time, agents create entities via skills like
`workitem-management.create_workitem` and never see id-management. Direct
calls are useful for:

- Reserving an ID before fully writing the entity
- Validating an ID a human typed in
- Listing used IDs of a kind for bulk operations or audits
- Confirming the project's ID format

## How other servers use it

Other entity-creating servers (`event-log`, `workitem-management`,
`decision-record`, `binding-management`) call into the same lib function
(`processkit.ids.generate_id`) directly. They do NOT cross the MCP
protocol boundary to call this server. This avoids the awkwardness of
"MCP server inside MCP server" while keeping the policy in one place.

The skill graph still declares the dependency via `uses: [id-management]`
so consumers know what to install.

## Configuration

Set in the consumer project's `processkit.toml`:

```toml
[context]
id_format = "word"   # word | uuid
id_slug   = false    # true | false
```

`format_info()` returns the resolved values plus the prefix registry.

## Running

```bash
uv run context/skills/id-management/mcp/server.py
```
