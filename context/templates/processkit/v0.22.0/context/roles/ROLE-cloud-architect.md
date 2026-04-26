---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-cloud-architect
  created: 2026-04-22T00:00:00Z
spec:
  name: cloud-architect
  description: "Designs cloud platform topology, landing zones, and service portfolios."
  responsibilities:
    - "Design landing zones and account topologies"
    - "Set cloud service standards and guardrails"
    - "Review high-cost or high-risk designs"
    - "Mentor teams on cloud-native patterns"
  skills_required:
    - "cloud-platform"
    - "landing-zone-design"
    - "network-topology"
    - "iam-architecture"
    - "cost-governance"
  default_scope: permanent
  default_seniority: principal
  function_group: architecture
---
