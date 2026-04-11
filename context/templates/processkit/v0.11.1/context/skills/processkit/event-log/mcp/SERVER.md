# event-log MCP server

Append-only LogEntry management. Layer 0 — depended on by every other
process-primitive skill.

## Tools

| Tool                                                                            | Purpose                                              |
|---------------------------------------------------------------------------------|------------------------------------------------------|
| `log_event(event_type, summary, actor?, subject?, subject_kind?, details?, timestamp?, correlation_id?)` | Append a new LogEntry to `context/logs/` |
| `query_events(event_type?, subject?, actor?, limit?)`                           | Query LogEntries via the index                       |
| `recent_events(limit?)`                                                         | Most recent LogEntries                               |

## Notes

- LogEntries are immutable. There is no edit or delete tool — corrections
  are recorded as new LogEntries with `event_type: logentry.corrected`.
- Each call writes a Markdown file under `<root>/context/logs/` and
  upserts the row in the SQLite index in one shot.
- IDs follow the project's `processkit.toml` `id_format` setting (word/uuid +
  optional slug). Default: word.

## Running

```bash
uv run context/skills/event-log/mcp/server.py
```
