---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-product-owner
  created: 2026-04-22T00:00:00Z
spec:
  name: product-owner
  description: "Steward of a Scrum backlog; bridges PM vision and delivery teams."
  responsibilities:
    - "Refine user stories with acceptance criteria"
    - "Prioritise the sprint backlog"
    - "Accept or reject completed work"
    - "Protect team focus from mid-sprint churn"
  skills_required:
    - "backlog-refinement"
    - "user-story-writing"
    - "scrum-practices"
    - "acceptance-criteria"
  default_scope: permanent
  default_seniority: senior
  function_group: product-program
---
