---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-product-manager-principal-h723e55
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-product-manager
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal PM — frontier reasoning for portfolio-scale trade-offs
  description: "default-pack: Principal PM — frontier reasoning for portfolio-scale trade-offs"
---
