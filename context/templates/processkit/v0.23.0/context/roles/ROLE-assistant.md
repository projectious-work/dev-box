---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-assistant
  created: 2026-04-22T00:00:00Z
spec:
  name: assistant
  description: "General-purpose helper for small, scoped, cross-cutting tasks."
  responsibilities:
    - "Take on short tasks that don't fit specialist roles"
    - "Summarise, draft, and triage quickly"
    - "Escalate work that requires a specialist"
    - "Keep the team unblocked on small items"
  skills_required:
    - "general-reasoning"
    - "summarization"
    - "drafting"
    - "triage"
  default_scope: permanent
  default_seniority: junior
  function_group: support
---
