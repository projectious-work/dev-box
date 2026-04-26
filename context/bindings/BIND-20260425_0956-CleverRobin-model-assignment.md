---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0956-CleverRobin-model-assignment
  created: '2026-04-25T09:56:08+00:00'
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist solutions-architect — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist solutions-architect'
---
