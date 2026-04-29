---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-product-manager
  created: 2026-04-22T00:00:00Z
spec:
  name: product-manager
  description: "Owns discovery, roadmap, and prioritisation for a product area."
  responsibilities:
    - "Discover user problems through interviews and data"
    - "Define and prioritise the product backlog"
    - "Align stakeholders on goals and trade-offs"
    - "Close the loop with launch metrics and retros"
  skills_required:
    - "discovery-research"
    - "roadmap-planning"
    - "prioritization-frameworks"
    - "stakeholder-management"
  default_scope: permanent
  default_seniority: senior
  function_group: product-program
---
