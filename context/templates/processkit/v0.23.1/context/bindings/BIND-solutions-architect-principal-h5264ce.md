---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-solutions-architect-principal-h5264ce
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal architect — maximum thinking budget for novel designs
  description: "default-pack: Principal architect — maximum thinking budget for novel designs"
---
