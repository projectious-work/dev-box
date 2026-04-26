---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-software-engineer-senior-h7bd319
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior IC coding — best price-capability ratio for day-to-day work
  description: "default-pack: Senior IC coding — best price-capability ratio for day-to-day work"
---
