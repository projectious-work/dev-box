---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-KeenCliff-regular-code-audit-process
  created: '2026-04-10T22:38:07+00:00'
  labels:
    old_id: BACK-071
    area: process
spec:
  title: Regular code audit process — simplification, security, and performance
  state: backlog
  type: task
  priority: medium
  description: 'Establish a recurring audit practice for the CLI codebase. Scope:
    (1) Simplification — dead code, over-abstractions, duplicated logic, unnecessary
    dependencies. (2) Security — OWASP top 10 applicability, input validation coverage,
    supply chain (cargo audit, dependency review). (3) Performance — hot paths, unnecessary
    allocations, slow tests. (4) Process — frequency (per release? monthly?), checklist
    template, tooling (clippy lints, cargo-deny, cargo-bloat, cargo-udeps). Consider
    making this a skill that can be triggered per session. Related to BACK-002 (security
    review). Old ID: BACK-071.'
---
