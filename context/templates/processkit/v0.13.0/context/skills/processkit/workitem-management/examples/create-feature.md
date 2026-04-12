# Example: creating a feature WorkItem

## Situation

The user says: "Add a command to aibox that validates context files against
their primitive schemas." This is a new feature, not trivial, and will involve
research + implementation + tests. It warrants a WorkItem.

## WorkItem file created

Path: `context/workitems/BACK-calm-fox.md`

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-calm-fox
  created: 2026-04-06T10:15:00Z
  labels:
    area: cli
spec:
  title: "Add aibox lint command for structural context validation"
  state: backlog
  type: story
  priority: high
  assignee: null
  description: >
    Add `aibox lint` that walks context/ and checks every entity file has
    apiVersion, kind, metadata.id, and that kind is in the known registry.
    Does NOT need full schema-aware validation — that's the index MCP
    server's job (Phase 3).
  related_decisions: [DEC-021]
---

## User story

As a processkit consumer, I want `aibox lint` to quickly catch malformed
entity files so that I notice mistakes before they propagate into the index.

## Acceptance criteria

- [ ] Walks context/ recursively
- [ ] Parses YAML frontmatter from every *.md file
- [ ] Reports missing apiVersion / kind / metadata.id with file + line
- [ ] Warns on unknown kinds (not in the registry)
- [ ] Exit code 1 on any error, 0 on clean
- [ ] Unit tests covering each error class

## Notes

Schema-aware validation is explicitly out of scope — that lives in the
processkit index-management MCP server per DEC-021. This lint is the
"fast structural sanity check" layer.
```

## Then: log the creation

After writing the file, create a LogEntry:

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-bright-owl
  created: 2026-04-06T10:15:00Z
spec:
  event_type: workitem.created
  timestamp: 2026-04-06T10:15:00Z
  actor: ACTOR-claude
  subject: BACK-calm-fox
  subject_kind: WorkItem
  summary: "Created feature story BACK-calm-fox: aibox lint command"
  details:
    type: story
    priority: high
    related_decisions: [DEC-021]
---
```

## Why this shape

- `type: story` because it is framed as a user-facing capability with a user
  story — not a faceless `task`.
- `priority: high` because it unblocks consumer adoption of processkit.
- `related_decisions: [DEC-021]` captures the decision that scoped this work
  ("SQLite index lives in processkit MCP servers"), which keeps the linting
  intentionally shallow.
- `labels.area: cli` lets future queries filter by code area.
- `assignee: null` — no one has picked it up yet. Will be set on first
  transition to `in-progress`.
