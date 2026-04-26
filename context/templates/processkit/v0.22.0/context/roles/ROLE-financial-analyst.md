---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-financial-analyst
  created: 2026-04-22T00:00:00Z
spec:
  name: financial-analyst
  description: "Builds models and analyses that guide financial decisions."
  responsibilities:
    - "Build forecasts and variance analyses"
    - "Partner with functions on business-case modelling"
    - "Produce board and exec financial narratives"
    - "Monitor KPIs and surface deviations"
  skills_required:
    - "financial-modeling"
    - "variance-analysis"
    - "business-case-modeling"
    - "excel-modeling"
  default_scope: permanent
  default_seniority: senior
  function_group: finance
---
