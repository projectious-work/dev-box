---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-software-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: software-engineer
  description: "Generalist IC who ships features end-to-end across the stack."
  responsibilities:
    - "Implement WorkItems against a written plan"
    - "Write and update tests alongside behavioural changes"
    - "Keep changes scoped; flag drive-by refactors"
    - "Review code and surface risks with actionable fixes"
    - "Escalate unclear specs to the PM rather than guessing"
  skills_required:
    - "code-generation"
    - "testing-strategy"
    - "tdd-workflow"
    - "code-review"
    - "git-workflow"
  default_scope: permanent
  default_seniority: senior
  function_group: engineering-software
---
