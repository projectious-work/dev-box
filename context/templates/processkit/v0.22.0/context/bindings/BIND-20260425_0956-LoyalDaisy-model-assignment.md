---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0956-LoyalDaisy-model-assignment
  created: '2026-04-25T09:56:02+00:00'
spec:
  type: model-assignment
  subject: ROLE-research-scientist
  target: MODEL-anthropic-claude-opus
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: 'Seed: Expert research-scientist — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert research-scientist'
---
