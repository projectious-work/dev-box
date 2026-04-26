---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-security-operations-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: security-operations-engineer
  description: "Runs detection, triage, and response for security events."
  responsibilities:
    - "Build and maintain detection rules across platforms"
    - "Triage alerts and escalate per runbooks"
    - "Lead containment and eradication on incidents"
    - "Post-incident: close gaps with preventive controls"
  skills_required:
    - "siem"
    - "ir-playbooks"
    - "threat-detection"
    - "forensics"
    - "scripting"
  default_scope: permanent
  default_seniority: senior
  function_group: security
---
