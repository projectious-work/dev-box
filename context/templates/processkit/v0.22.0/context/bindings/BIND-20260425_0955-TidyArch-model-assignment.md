---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-20260425_0955-TidyArch-model-assignment
  created: '2026-04-25T09:55:41+00:00'
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  target: MODEL-anthropic-claude-sonnet
  conditions:
    seniority: specialist
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: 'Seed: Specialist ai-research-scientist — inherits junior defaults
      pending role-specific tuning (KeenFern v0.21.0)'
  description: 'Seed: Specialist ai-research-scientist'
---
