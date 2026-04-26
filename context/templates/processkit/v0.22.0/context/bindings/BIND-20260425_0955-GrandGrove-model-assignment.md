---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-GrandGrove-model-assignment
  created: '2026-04-25T09:55:43+00:00'
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  target: MODEL-anthropic-claude-opus
  conditions:
    seniority: expert
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: 'Seed: Expert ai-research-scientist — inherits senior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Expert ai-research-scientist'
---
