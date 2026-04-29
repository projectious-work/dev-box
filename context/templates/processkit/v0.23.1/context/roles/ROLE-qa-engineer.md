---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-qa-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: qa-engineer
  description: "Designs test strategies and uncovers defects through structured testing."
  responsibilities:
    - "Design test plans that target real risk areas"
    - "Execute manual and exploratory tests"
    - "Author automation for regression coverage"
    - "Partner with devs on testability improvements"
  skills_required:
    - "test-design"
    - "exploratory-testing"
    - "test-automation"
    - "bug-reproduction"
    - "risk-analysis"
  default_scope: permanent
  default_seniority: senior
  function_group: qa
---
