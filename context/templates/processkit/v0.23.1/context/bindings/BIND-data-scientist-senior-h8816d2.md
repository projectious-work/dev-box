---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-data-scientist-senior-h8816d2
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior data scientist — modelling and notebook work
  description: "default-pack: Senior data scientist — modelling and notebook work"
---
