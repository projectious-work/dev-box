---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260426_1958-SureLeaf-release-shipped
  created: '2026-04-26T19:58:54+00:00'
spec:
  event_type: release.shipped
  timestamp: '2026-04-26T19:58:54+00:00'
  summary: aibox v0.21.2 Phase 2 complete — macOS arm64 + x86_64 binaries uploaded
    to the GH release; GHCR container images pushed. v0.21.2 (processkit v0.23.1 integration)
    is fully shipped across both phases.
  actor: claude-opus-4-7
  details:
    version: 0.21.2
    processkit_version: v0.23.1
    phase: 2
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.21.2
    tag: v0.21.2
    phase_2_artifacts:
    - macOS arm64 binary
    - macOS x86_64 binary
    - GHCR container images
---
