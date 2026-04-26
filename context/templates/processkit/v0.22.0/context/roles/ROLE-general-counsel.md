---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-general-counsel
  created: 2026-04-22T00:00:00Z
spec:
  name: general-counsel
  description: "Leads legal strategy and advises the exec team across matters."
  responsibilities:
    - "Own legal strategy and risk posture"
    - "Advise on commercial and strategic transactions"
    - "Manage external counsel relationships"
    - "Shape policy and governance"
  skills_required:
    - "corporate-law"
    - "commercial-contracts"
    - "litigation-management"
    - "governance"
  default_scope: permanent
  default_seniority: principal
  function_group: legal-compliance
---
