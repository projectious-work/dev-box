---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-research-scientist-senior-h14c6d3
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Senior researcher — rigorous scientific reasoning
  description: "default-pack: Senior researcher — rigorous scientific reasoning"
---
