---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-ai-research-scientist-principal-h5e96d8
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: extra-high
    effort_ceiling: max
    rationale: Principal AI researcher — deep reasoning on state-of-the-art ML
  description: "default-pack: Principal AI researcher — deep reasoning on state-of-the-art ML"
---
