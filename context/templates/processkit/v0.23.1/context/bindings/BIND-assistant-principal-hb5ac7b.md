---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-assistant-principal-hb5ac7b
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Principal assistant — full frontier capabilities when needed
  description: "default-pack: Principal assistant — full frontier capabilities when needed"
---
