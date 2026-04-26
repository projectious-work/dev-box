---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0956-WiseBrook-model-assignment
  created: '2026-04-25T09:56:09+00:00'
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  target: MODEL-anthropic-claude-opus
  conditions:
    seniority: expert
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Expert solutions-architect — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert solutions-architect'
---
