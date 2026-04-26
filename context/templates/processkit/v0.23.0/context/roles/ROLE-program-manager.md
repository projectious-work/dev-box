---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-program-manager
  created: 2026-04-22T00:00:00Z
spec:
  name: program-manager
  description: "Coordinates cross-team delivery of multi-quarter initiatives."
  responsibilities:
    - "Plan and sequence cross-team dependencies"
    - "Run steering rituals and status reporting"
    - "Identify and unblock cross-team risks"
    - "Close out programmes with learnings"
  skills_required:
    - "dependency-management"
    - "program-planning"
    - "risk-management"
    - "cross-team-coordination"
  default_scope: permanent
  default_seniority: senior
  function_group: product-program
---
