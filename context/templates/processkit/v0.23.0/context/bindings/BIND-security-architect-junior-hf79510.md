---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-security-architect-junior-hf79510
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-security-architect
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: "Junior security architect — threat-model drafting; G:5 required"
  description: "default-pack: Junior security architect — threat-model drafting; G:5 required"
---
