---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260426_1534-SnappyTrout-release-shipped
  created: '2026-04-26T15:34:43+00:00'
spec:
  event_type: release.shipped
  timestamp: '2026-04-26T15:34:43+00:00'
  summary: 'aibox v0.21.0 released: sync content-diff data-loss fix (closes aibox#57),
    same-version sync short-circuit (closes aibox#56), multi-harness slash-command
    scaffolding (Codex/Cursor/Gemini/OpenCode). 654 tests passing. Phase 1 (Linux
    binaries + GitHub release) complete; Phase 2 (macOS binaries + GHCR images) pending
    user macOS-host run.'
  details:
    version: v0.21.0
    tag: v0.21.0
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.21.0
    commits:
    - '34b9a83 chore: housekeep v0.20.0 release artifacts'
    - 'b1b57cf fix(sync): repair content-diff data loss and same-version baseline'
    - '5204969 feat(harness): scaffold processkit slash commands beyond Claude Code'
    - 'e88ad2f chore: bump CLI version to 0.21.0'
    issues_closed:
    - aibox#55
    - aibox#56
    - aibox#57
    tests_passing: 654
    phase1_complete: true
    phase2_pending: user runs ./scripts/maintain.sh release-host 0.21.0 on macOS host
---
