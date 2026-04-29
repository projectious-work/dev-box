---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-technical-writer-junior-h431a38
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior tech writer — fast drafts of routine docs
  description: "default-pack: Junior tech writer — fast drafts of routine docs"
---
