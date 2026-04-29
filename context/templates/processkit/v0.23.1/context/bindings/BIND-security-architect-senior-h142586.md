---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-security-architect-senior-h142586
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-security-architect
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: "Senior security architect — rigorous threat analysis; G:5"
  description: "default-pack: Senior security architect — rigorous threat analysis; G:5"
---
