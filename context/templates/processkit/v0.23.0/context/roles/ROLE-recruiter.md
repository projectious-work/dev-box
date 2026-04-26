---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-recruiter
  created: 2026-04-22T00:00:00Z
spec:
  name: recruiter
  description: "Sources, screens, and closes candidates for open roles."
  responsibilities:
    - "Build and run sourcing campaigns"
    - "Screen and shortlist candidates"
    - "Coordinate interview loops"
    - "Close offers with partnership from hiring managers"
  skills_required:
    - "sourcing"
    - "candidate-experience"
    - "interview-coordination"
    - "offer-negotiation"
  default_scope: permanent
  default_seniority: senior
  function_group: people-hr
---
