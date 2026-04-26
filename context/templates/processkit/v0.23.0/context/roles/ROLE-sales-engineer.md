---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-sales-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: sales-engineer
  description: "Provides technical validation, demos, and POCs during the sales cycle."
  responsibilities:
    - "Run scoped demos and technical POCs"
    - "Answer deep technical questions"
    - "Partner with AE to remove technical friction"
    - "Feed product gaps back to PM"
  skills_required:
    - "technical-demos"
    - "poc-scoping"
    - "solutioning"
    - "objection-handling"
  default_scope: permanent
  default_seniority: senior
  function_group: sales-customer
---
