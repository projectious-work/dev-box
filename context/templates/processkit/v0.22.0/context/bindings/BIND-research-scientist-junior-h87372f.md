---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-research-scientist-junior-h87372f
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Junior researcher — paper reading and hypothesis drafting
  description: "default-pack: Junior researcher — paper reading and hypothesis drafting"
---
