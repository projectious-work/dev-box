---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-LoyalTide-docusaurus-code-walkthrough-site
  created: '2026-04-10T22:38:14+00:00'
  labels:
    old_id: BACK-072
    area: docs
spec:
  title: Docusaurus code walkthrough site for Rust newcomers
  state: backlog
  type: task
  priority: medium
  description: 'Documentation site (or subsite under docs-site/) explaining the CLI
    codebase for someone new to Rust but experienced in programming. Scope: (1) Architecture
    overview — module graph, data flow from aibox init → generated files, key abstractions.
    (2) Per-module walkthrough — config parsing, template rendering, addon system,
    seed/sync lifecycle, runtime detection. (3) Rust idioms used — Result/Option chains,
    serde patterns, builder pattern, error handling. (4) How to contribute — adding
    an addon, adding a command, writing tests. (5) Maintenance strategy — auto-generate
    from doc comments vs hand-written? Old ID: BACK-072.'
---
