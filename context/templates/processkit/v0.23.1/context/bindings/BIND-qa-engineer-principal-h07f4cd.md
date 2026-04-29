---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-qa-engineer-principal-h07f4cd
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal QA — test-architecture for complex systems
  description: "default-pack: Principal QA — test-architecture for complex systems"
---
