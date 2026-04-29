---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-compliance-officer
  created: 2026-04-22T00:00:00Z
spec:
  name: compliance-officer
  description: "Ensures the organisation meets regulatory and internal compliance obligations."
  responsibilities:
    - "Maintain the compliance programme and policies"
    - "Run compliance training and attestations"
    - "Manage audits and regulator interactions"
    - "Investigate and remediate compliance issues"
  skills_required:
    - "regulatory-frameworks"
    - "policy-writing"
    - "audit-management"
    - "training-design"
  default_scope: permanent
  default_seniority: senior
  function_group: legal-compliance
---
