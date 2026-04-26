---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-software-engineer-junior-h92feb2
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: low
    effort_ceiling: medium
    rationale: Junior IC coding — fast, cheap, sufficient for routine tickets
  description: "default-pack: Junior IC coding — fast, cheap, sufficient for routine tickets"
---
