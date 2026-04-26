---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0956-SoundRaven-model-assignment
  created: '2026-04-25T09:56:11+00:00'
spec:
  type: model-assignment
  subject: ROLE-technical-writer
  target: MODEL-anthropic-claude-haiku
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: 'Seed: Specialist technical-writer — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist technical-writer'
---
