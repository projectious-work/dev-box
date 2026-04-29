---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-data-scientist
  created: 2026-04-22T00:00:00Z
spec:
  name: data-scientist
  description: "Analyses data, runs experiments, and produces statistical insights for decisions."
  responsibilities:
    - "Design and run experiments with clear hypotheses"
    - "Model data to answer concrete business questions"
    - "Communicate findings with quantified uncertainty"
    - "Partner with product and engineering on follow-through"
  skills_required:
    - "statistics"
    - "python-r"
    - "experimental-design"
    - "data-visualization"
    - "causal-inference"
  default_scope: permanent
  default_seniority: senior
  function_group: data-ml
---
