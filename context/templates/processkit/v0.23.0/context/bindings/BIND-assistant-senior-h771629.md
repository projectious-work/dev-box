---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-assistant-senior-h771629
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Senior assistant — still fast, slightly more thinking budget
  description: "default-pack: Senior assistant — still fast, slightly more thinking budget"
---
