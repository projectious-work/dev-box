---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-technical-writer-senior-hb93488
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Senior tech writer — style-sensitive long-form authoring
  description: "default-pack: Senior tech writer — style-sensitive long-form authoring"
---
