---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-BoldSage-model-assignment
  created: '2026-04-25T09:55:45+00:00'
spec:
  type: model-assignment
  subject: ROLE-assistant
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: none
    effort_ceiling: low
    rationale: 'Seed: Specialist assistant — inherits junior defaults pending role-specific
      tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist assistant'
---
