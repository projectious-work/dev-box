---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-ai-research-scientist-senior-h18c312
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Senior AI researcher — frontier reasoning for novel ML work
  description: "default-pack: Senior AI researcher — frontier reasoning for novel ML work"
---
