---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-technical-writer
  created: 2026-04-22T00:00:00Z
spec:
  name: technical-writer
  description: "Produces the reference docs, tutorials, and release notes developers rely on."
  responsibilities:
    - "Write and maintain accurate developer docs"
    - "Produce tutorials and getting-started guides"
    - "Partner with engineering on API reference quality"
    - "Run docs QA: links, samples, accuracy"
  skills_required:
    - "technical-writing"
    - "docs-as-code"
    - "api-reference"
    - "tutorial-design"
    - "editorial-review"
  default_scope: permanent
  default_seniority: senior
  function_group: devrel-docs
---
