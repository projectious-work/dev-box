# Example: logging a WorkItem state transition

## Situation

The agent just moved `BACK-calm-fox` ("Add aibox lint command") from
`in-progress` to `review` after finishing the implementation and opening a PR.

## LogEntry produced

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-bright-owl
  created: 2026-04-06T14:22:00Z
spec:
  event_type: workitem.transitioned
  timestamp: 2026-04-06T14:22:00Z
  actor: ACTOR-claude
  subject: BACK-calm-fox
  subject_kind: WorkItem
  summary: "Moved BACK-calm-fox from in-progress to review (PR #42 opened)"
  details:
    from_state: in-progress
    to_state: review
    triggered_by: pr-opened
    pr_url: https://github.com/example/repo/pull/42
    reviewers: [ACTOR-alice, ACTOR-bob]
---

Implementation is complete and passes all existing tests. PR opened with
three new tests covering structural validation of apiVersion, kind, and
metadata.id. Waiting on review from the two assigned reviewers.
```

## Why this shape

- `event_type: workitem.transitioned` follows the dotted convention.
- `timestamp` equals `metadata.created` because the log was written in real
  time. For retroactive logging, they would differ.
- `details.from_state` and `details.to_state` are the de-facto fields every
  transition event uses.
- `details.triggered_by` captures *why* the transition happened (not just *what*).
- `pr_url` and `reviewers` are transition-specific metadata; they live in
  `details` because they are not part of every transition event.
