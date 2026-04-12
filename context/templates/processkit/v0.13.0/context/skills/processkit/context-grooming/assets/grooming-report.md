---
apiVersion: processkit.projectious.work/v1
kind: Context
metadata:
  id: GROOM-TODO
  privacy: project-private
  created: TODO
spec:
  description: "Grooming report — proposals for archive, summarization, and skill disablement. State: pending → approved → applied."
  state: pending           # pending | approved | applied
  generated: TODO
  generated_by: TODO       # agent identifier
---

# Grooming report — TODO date

## Summary

TODO — one line. Example: "12 workitems eligible for archive, 3 unused skills, 8 standup entries from Q1 ready to summarize."

## Workitems eligible for archive

(WorkItems in terminal state for >30 days. Default rule, override in
[context.grooming.rules] in processkit.toml.)

| ID | Title | State | Last touched |
|---|---|---|---|
| TODO | TODO | done | TODO |

[ ] Approve all
[ ] Approve individually (list specific IDs)

## LogEntries eligible for archive

(LogEntries older than 90 days, not referenced by current entities.)

| ID | Event type | Timestamp |
|---|---|---|
| TODO | TODO | TODO |

[ ] Approve all

## Standups eligible for summarization

(Standup entries from previous quarters get summarized into one
quarterly summary; originals are archived.)

Period: TODO

[ ] Approve summarization

## Decisions eligible for archive

(DecisionRecords superseded for >180 days.)

| ID | Title | Superseded by | Superseded date |
|---|---|---|---|
| TODO | TODO | TODO | TODO |

[ ] Approve all

## Oversize files

(Files over 500 lines that haven't been edited in 30 days. Proposed for
splitting or summarization — not auto-archived.)

| Path | Lines | Last edited |
|---|---|---|
| TODO | TODO | TODO |

(Manual review needed for each.)

## Unused skills

(Skills in `context/skills/` not invoked in any session for 60+ days.)

| Skill | Last used |
|---|---|
| TODO | TODO |

[ ] Approve disabling (move from `context/skills/` to `context/skills/.disabled/`)

## Notes

(Anything else the agent noticed during the walk.)

TODO

---

## Action log

(Filled in after the user approves and the agent applies the moves. One
line per moved/archived item.)

- (none yet)
