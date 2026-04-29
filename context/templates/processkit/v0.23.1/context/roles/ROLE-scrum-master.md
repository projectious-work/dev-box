---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-scrum-master
  created: 2026-04-22T00:00:00Z
spec:
  name: scrum-master
  description: "Facilitates agile rituals and removes team-level impediments."
  responsibilities:
    - "Facilitate standups, planning, review, and retros"
    - "Surface and remove impediments"
    - "Coach the team on agile practices"
    - "Protect the team from outside interrupts"
  skills_required:
    - "scrum-facilitation"
    - "agile-coaching"
    - "conflict-resolution"
    - "retro-design"
  default_scope: permanent
  function_group: product-program
---
