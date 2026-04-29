---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-solutions-architect-junior-heb66a8
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior architect — mid-frontier is enough for scoped design work
  description: "default-pack: Junior architect — mid-frontier is enough for scoped design work"
---
