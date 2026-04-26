---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-community-manager
  created: 2026-04-22T00:00:00Z
spec:
  name: community-manager
  description: "Stewards external communities (forums, Discord/Slack, OSS contributors)."
  responsibilities:
    - "Engage with the community across channels"
    - "Identify and amplify community signal"
    - "Organise events, AMAs, and contributor programmes"
    - "Surface feedback to product and engineering"
  skills_required:
    - "community-engagement"
    - "event-planning"
    - "moderation"
    - "social-listening"
  default_scope: permanent
  default_seniority: senior
  function_group: devrel-docs
---
