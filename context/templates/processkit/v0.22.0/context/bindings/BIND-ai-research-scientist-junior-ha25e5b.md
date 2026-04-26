---
apiVersion: processkit.projectious.work/v1
kind: Binding
metadata:
  id: BIND-ai-research-scientist-junior-ha25e5b
  created: 2026-04-22T00:00:00Z
spec:
  type: model-assignment
  subject: ROLE-ai-research-scientist
  subject_kind: Role
  target: MODEL-anthropic-claude-sonnet
  target_kind: Model
  conditions:
    seniority: junior
    rank: 1
    effort_floor: medium
    effort_ceiling: high
    rationale: Junior AI researcher — familiar with agentic+tool-use patterns
  description: "default-pack: Junior AI researcher — familiar with agentic+tool-use patterns"
---
