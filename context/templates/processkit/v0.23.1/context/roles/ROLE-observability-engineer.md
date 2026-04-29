---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-observability-engineer
  created: 2026-04-22T00:00:00Z
spec:
  name: observability-engineer
  description: "Instruments services and operates metrics, tracing, and logging systems."
  responsibilities:
    - "Define instrumentation standards and libraries"
    - "Operate metric/trace/log pipelines at scale"
    - "Curate dashboards and alert rules with owners"
    - "Reduce noisy alerts and signal-to-noise issues"
  skills_required:
    - "metrics"
    - "distributed-tracing"
    - "logging-pipelines"
    - "alerting"
    - "dashboard-design"
  default_scope: permanent
  default_seniority: senior
  function_group: platform-infra
---
