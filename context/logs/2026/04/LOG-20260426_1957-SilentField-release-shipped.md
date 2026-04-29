---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260426_1957-SilentField-release-shipped
  created: '2026-04-26T19:57:05+00:00'
spec:
  event_type: release.shipped
  timestamp: '2026-04-26T19:57:05+00:00'
  summary: aibox v0.21.2 Phase 1 shipped — processkit v0.23.1 integration. Linux binaries
    (aarch64 + x86_64) attached to GH release; docs deployed to gh-pages. 720 tests
    passing (654 unit + 50 e2e + 16 integration); clippy + cargo audit clean. Phase
    2 (macOS binaries + GHCR images) pending user macOS-host run of `./scripts/maintain.sh
    release-host 0.21.2`.
  actor: claude-opus-4-7
  details:
    version: 0.21.2
    processkit_version: v0.23.1
    phase: 1
    release_url: https://github.com/projectious-work/aibox/releases/tag/v0.21.2
    tag: v0.21.2
    commit: d49980d
    test_counts:
      unit: 654
      e2e: 50
      integration: 16
      total: 720
    binaries_uploaded:
    - aibox-v0.21.2-aarch64-unknown-linux-gnu.tar.gz
    - aibox-v0.21.2-x86_64-unknown-linux-gnu.tar.gz
    docs_url: https://projectious-work.github.io/aibox/
    phase_2_pending: macOS binaries + GHCR images
---
