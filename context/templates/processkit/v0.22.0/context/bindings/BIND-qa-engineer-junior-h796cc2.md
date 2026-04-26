---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-qa-engineer-junior-h796cc2
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-qa-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior QA — high-volume test-case generation
  description: "default-pack: Junior QA — high-volume test-case generation"
---
