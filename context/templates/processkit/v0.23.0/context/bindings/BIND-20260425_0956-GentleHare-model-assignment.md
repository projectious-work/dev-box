---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0956-GentleHare-model-assignment
  created: '2026-04-25T09:56:06+00:00'
spec:
  type: model-assignment
  subject: ROLE-security-architect
  target: MODEL-anthropic-claude-opus
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: 'Seed: Expert security-architect — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert security-architect'
---
