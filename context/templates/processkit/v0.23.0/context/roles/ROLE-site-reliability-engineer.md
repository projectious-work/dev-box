---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-site-reliability-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: site-reliability-engineer
  description: "Defends service reliability via SLOs, incident response, and toil reduction."
  responsibilities:
    - "Define and enforce SLOs and error budgets"
    - "Lead postmortems and drive follow-up actions"
    - "Automate repetitive operational work"
    - "Improve observability and on-call ergonomics"
  skills_required:
    - "slo-design"
    - "incident-management"
    - "observability"
    - "capacity-planning"
    - "automation"
  default_scope: permanent
  default_seniority: senior
  function_group: platform-infra
---
