---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-technical-writer-principal-h56b361
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Principal tech writer — reference docs and API-design prose
  description: "default-pack: Principal tech writer — reference docs and API-design prose"
---
