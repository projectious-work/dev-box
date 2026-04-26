---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-data-architect
  created: 2026-04-22T00:00:00Z
spec:
  name: data-architect
  description: "Designs enterprise data models, flows, and governance."
  responsibilities:
    - "Model logical and canonical data schemas"
    - "Design pipelines and lineage at org scale"
    - "Set data contracts and quality standards"
    - "Govern master data and reference data"
  skills_required:
    - "data-modeling"
    - "data-governance"
    - "data-lineage"
    - "pipeline-design"
    - "master-data-management"
  default_scope: permanent
  default_seniority: senior
  function_group: architecture
---
