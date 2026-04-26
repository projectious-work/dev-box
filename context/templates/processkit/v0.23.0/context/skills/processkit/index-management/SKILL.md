---
name: index-management
description: |
  SQLite-backed index over all entity files in the project. The read-side foundation for every other MCP server. Use whenever an agent needs to look up entities by ID, kind, state, or text — instead of grepping the filesystem.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-index-management
    version: "1.0.0"
    created: 2026-04-06T00:00:00Z
    category: processkit
    layer: 0
    provides:
      primitives: []
      mcp_tools: [reindex, query_entities, get_entity, search_entities, query_events, list_errors, 
            stats]
---

# Index Management

## Intro

`index-management` is the **read-side** foundation. It walks the project's
`context/` directory, parses every entity file, and writes a SQLite
database that other tools query. Its peer at Layer 0 is `id-management`
(the write side, which allocates new IDs). Both are foundational to
every entity-creating skill.

> **MCP server.** This skill ships a self-contained MCP server at
> `mcp/server.py` (PEP 723 script — requires `uv` and Python ≥ 3.10 on
> PATH). Agent harnesses reach its tools by reading a single MCP config
> file at startup, so the contents of `mcp/mcp-config.json` must be merged
> into the harness's MCP config and placed at the harness-specific path
> before this skill is usable. If processkit was installed by an installer,
> that wiring is the installer's responsibility; if processkit was
> installed manually, the project owner must do it by hand.

## Overview

### What it provides

- **`reindex()`** — rebuild the SQLite index from scratch
- **`query_entities(kind?, state?, limit?)`** — list entities matching filters
- **`get_entity(id)`** — fetch a single entity by ID
- **`search_entities(text, limit?)`** — full-text search across IDs, titles, bodies, specs
- **`query_events(event_type?, subject?, actor?, limit?)`** — query the event log
- **`list_errors()`** — files that failed to parse during the last reindex
- **`stats()`** — counts of entities/events/errors in the index

### When to use

The MCP server runs continuously inside the dev container. Agents call it
whenever they need fast lookup. Other MCP servers (`workitem-management`,
`decision-record`, etc.) call `reindex` after writing a new entity to
keep the index fresh.

### Where the database lives

`<project-root>/context/.cache/processkit/index.sqlite`. Gitignored.
Rebuildable from source files at any time via `reindex()`.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Grepping the filesystem instead of calling `query_entities`.** The
  index is faster, context-cheaper, and reflects parsed semantics
  (state, kind, links) that grep can't see. Reach for `query_entities`
  first; fall back to grep only if the query you need isn't supported.
- **Trusting the index immediately after a hand-edit.** When you edit
  an entity file directly (bypassing the MCP write tools), the index
  is stale until the next `reindex()` call. Either go through the
  write tools, or call `reindex()` yourself before querying.
- **Forgetting to call `reindex` after writing a new entity.** Every
  entity-creating MCP server should call `index_management.reindex`
  (or the lighter `upsert_entity`) after a write. Not doing so means
  the next query won't see the just-created entity.
- **Treating index queries as transactional.** The index is eventually
  consistent. A query immediately after a write may or may not return
  the new row depending on whether reindex completed. If you need
  strict consistency, reindex explicitly first.
- **Trusting `list_errors()` as proof of correctness.** A clean errors
  table means parsing succeeded, not that the entities are
  semantically valid. Schema validation is a separate concern from
  parse success.
- **Using `search_entities` for exact-ID lookup.** `search_entities`
  is full-text — it ranks by relevance and may miss exact ID matches
  if the ID appears in many bodies. Use `get_entity(id)` for exact
  lookup.
- **Re-indexing inside a hot loop.** A full reindex walks the whole
  `context/` tree and is expensive. Batch your writes and reindex
  once at the end, not after every entity.

## Full reference

### Database schema

Three tables — see `src/lib/processkit/index.py` for the DDL.

```sql
entities(id, kind, api_version, path, created, updated, title, state, labels_json, spec_json, body)
events(id, timestamp, event_type, actor, subject, subject_kind, summary, details_json, correlation_id, path)
errors(path, message)
```

`spec_json` holds the full `spec` block as JSON for queries we did not
anticipate. `entities` and `events` overlap for LogEntry rows: a LogEntry
appears in both, with the `events` table denormalizing the event-specific
fields for fast filtering.

### Reindex strategy

`reindex()` is **destructive and atomic** — it deletes all rows and
re-inserts. For typical projects this is fast (sub-second). For very
large projects, an incremental update mode lands later (Phase 4+).

### Errors table

Files that fail to parse get a row in the `errors` table instead of
crashing the reindex. `list_errors()` returns all such rows so the agent
can fix them. The errors table is cleared at the start of each reindex.

### Tools that other servers call

`workitem-management`, `decision-record`, `binding-management`, and
`event-log` all call `index_management.upsert_entity` after writing a
new file. This keeps the index in sync without a full reindex on every
mutation. From an MCP-protocol perspective each server has its own
process — they communicate by sharing the same SQLite database file
(WAL mode would be enabled in a future release for concurrent writes).

### Configuration

The index database path is configurable via:

- `processkit.toml` `[index] path = "..."` (relative to project root)
- The `PROCESSKIT_INDEX_DB` environment variable
- Default: `context/.cache/processkit/index.sqlite` (gitignored cache)

### Limitations at v0.3.0

- **WAL mode enabled (v0.4.0+).** Multiple readers and a single writer
  are safe; concurrent writes still serialize via WAL. Typical
  AI-assisted sessions are single-writer anyway.
- **No FTS.** Search uses `LIKE %text%`. SQLite FTS5 integration lands
  in a later release.
- **No incremental indexing.** Every reindex is a full sweep.
