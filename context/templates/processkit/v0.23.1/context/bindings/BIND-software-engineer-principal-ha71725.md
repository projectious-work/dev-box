---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-software-engineer-principal-ha71725
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-software-engineer
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: principal
    rank: 1
    effort_floor: high
    effort_ceiling: extra-high
    rationale: Principal IC coding — reserve for architecture-scale changes
  description: "default-pack: Principal IC coding — reserve for architecture-scale changes"
---
