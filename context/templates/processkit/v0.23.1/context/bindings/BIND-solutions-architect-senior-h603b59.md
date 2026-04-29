---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-solutions-architect-senior-h603b59
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-solutions-architect
  subject_kind: Role
  target: MODEL-anthropic-claude-opus
  target_kind: Model
  conditions:
    seniority: senior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Senior architect — frontier reasoning for system-design trade-offs
  description: "default-pack: Senior architect — frontier reasoning for system-design trade-offs"
---
