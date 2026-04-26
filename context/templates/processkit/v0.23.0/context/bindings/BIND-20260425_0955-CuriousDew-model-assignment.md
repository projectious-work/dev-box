---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-CuriousDew-model-assignment
  created: '2026-04-25T09:55:59+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Expert qa-engineer — inherits senior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert qa-engineer'
---
