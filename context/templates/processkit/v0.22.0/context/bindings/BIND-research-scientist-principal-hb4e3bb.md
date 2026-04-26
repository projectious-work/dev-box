---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-research-scientist-principal-hb4e3bb
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: extra-high
    effort_ceiling: max
    rationale: Principal researcher — maximum thinking for novel problem spaces
  description: "default-pack: Principal researcher — maximum thinking for novel problem spaces"
---
