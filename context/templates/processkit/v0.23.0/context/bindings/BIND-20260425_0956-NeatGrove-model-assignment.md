---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0956-NeatGrove-model-assignment
  created: '2026-04-25T09:56:04+00:00'
spec:
  type: model-assignment
  subject: ROLE-security-architect
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Specialist security-architect — inherits junior defaults pending
      role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist security-architect'
---
