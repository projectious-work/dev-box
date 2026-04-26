---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-KindPeak-model-assignment
  created: '2026-04-25T09:55:50+00:00'
spec:
  type: model-assignment
  subject: ROLE-data-scientist
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Expert data-scientist — inherits senior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert data-scientist'
---
