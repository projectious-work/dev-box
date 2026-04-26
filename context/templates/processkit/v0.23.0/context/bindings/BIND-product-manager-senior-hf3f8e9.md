---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-product-manager-senior-hf3f8e9
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-product-manager
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior PM — discovery synthesis and roadmap writing
  description: "default-pack: Senior PM — discovery synthesis and roadmap writing"
---
