---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-controller
  created: 2026-04-22T00:00:00Z
spec:
  name: controller
  description: "Runs accounting operations, close, and audit relationships."
  responsibilities:
    - "Own the monthly close process end-to-end"
    - "Ensure GAAP/IFRS compliance"
    - "Manage audit relationships and evidence"
    - "Maintain and improve internal controls"
  skills_required:
    - "accounting-close"
    - "gaap-ifrs"
    - "internal-controls"
    - "audit-management"
  default_scope: permanent
  default_seniority: principal
  function_group: finance
---
