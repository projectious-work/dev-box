---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-TallQuail-track-aider-native-mcp
  created: '2026-04-10T22:36:51+00:00'
  labels:
    old_id: BACK-121
    area: cli
spec:
  title: Track Aider native MCP client support upstream
  state: backlog
  type: task
  priority: low
  description: 'Aider has no native MCP client today (third-party mcpm-aider bridge
    is explicitly experimental). When Aider adds native MCP client support, add a
    writer to `cli/src/mcp_registration.rs` and remove the v0.16.5 sync-time warning.
    Watch `paul-gauthier/aider` releases for MCP support. Old ID: BACK-121.'
---
