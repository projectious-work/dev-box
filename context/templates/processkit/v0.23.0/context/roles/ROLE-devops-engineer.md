---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-devops-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: devops-engineer
  description: "Automates build, deploy, and operational tooling across environments."
  responsibilities:
    - "Own CI/CD pipelines and release mechanics"
    - "Codify infrastructure as code"
    - "Reduce deploy friction and mean-time-to-ship"
    - "Respond to pipeline and environment incidents"
  skills_required:
    - "ci-cd"
    - "iac-terraform-pulumi"
    - "containers-kubernetes"
    - "shell-scripting"
  default_scope: permanent
  default_seniority: senior
  function_group: platform-infra
---
