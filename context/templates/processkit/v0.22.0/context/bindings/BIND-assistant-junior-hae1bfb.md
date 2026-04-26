---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-assistant-junior-hae1bfb
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-assistant
  subject_kind: Role
  target: MODEL-anthropic-claude-haiku
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: none
    effort_ceiling: low
    rationale: Junior assistant — fast, cheap, low-latency chat
  description: "default-pack: Junior assistant — fast, cheap, low-latency chat"
---
