---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260426_1538-ProudHeron-release-shipped
  created: '2026-04-26T15:38:40+00:00'
spec:
  event_type: release.shipped
  timestamp: '2026-04-26T15:38:40+00:00'
  summary: aibox v0.21.0 Phase 2 complete — macOS aarch64 + x86_64 binaries uploaded;
    GHCR container images pushed. Release is fully shipped across both phases.
  details:
    version: v0.21.0
    phase: phase2_complete
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.21.0
    assets_verified:
    - aibox-v0.21.0-aarch64-apple-darwin.tar.gz
    - aibox-v0.21.0-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.21.0-x86_64-apple-darwin.tar.gz
    - aibox-v0.21.0-x86_64-unknown-linux-gnu.tar.gz
---
