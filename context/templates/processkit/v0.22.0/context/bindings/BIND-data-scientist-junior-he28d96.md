---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-data-scientist-junior-he28d96
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior data scientist — cheap SQL and quick analyses
  description: "default-pack: Junior data scientist — cheap SQL and quick analyses"
---
