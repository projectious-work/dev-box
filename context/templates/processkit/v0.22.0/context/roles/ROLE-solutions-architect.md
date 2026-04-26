---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-solutions-architect
  created: 2026-04-22T00:00:00Z
spec:
  name: solutions-architect
  description: "Designs customer- or project-facing technical solutions end-to-end."
  responsibilities:
    - "Translate requirements into buildable architectures"
    - "Produce design artefacts and C4 diagrams"
    - "Review implementation against the design"
    - "Own trade-off decisions and escalations"
  skills_required:
    - "system-design"
    - "integration-patterns"
    - "requirements-analysis"
    - "adr-writing"
    - "diagramming"
  default_scope: permanent
  default_seniority: senior
  function_group: architecture
---
