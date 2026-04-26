---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-product-manager-junior-h9275a0
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-product-manager
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior PM — fast iteration on briefs and summaries
  description: "default-pack: Junior PM — fast iteration on briefs and summaries"
---
