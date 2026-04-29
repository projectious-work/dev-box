---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-GentleGlade-model-assignment
  created: '2026-04-25T09:55:57+00:00'
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist qa-engineer — inherits junior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist qa-engineer'
---
