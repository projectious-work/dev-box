---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-SleekStream-model-assignment
  created: '2026-04-25T09:55:52+00:00'
spec:
  type: model-assignment
  subject: ROLE-product-manager
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist product-manager — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist product-manager'
---
