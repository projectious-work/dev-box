---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-SnowyReef-model-assignment
  created: '2026-04-25T09:55:46+00:00'
spec:
  type: model-assignment
  subject: ROLE-assistant
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: expert
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Expert assistant — inherits senior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert assistant'
---
