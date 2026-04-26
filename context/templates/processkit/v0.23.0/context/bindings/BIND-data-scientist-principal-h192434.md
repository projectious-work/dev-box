---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-data-scientist-principal-h192434
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal data scientist — methodology design and audits
  description: "default-pack: Principal data scientist — methodology design and audits"
---
