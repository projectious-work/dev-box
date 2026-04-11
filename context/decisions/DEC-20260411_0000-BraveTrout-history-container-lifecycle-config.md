---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-BraveTrout-history-container-lifecycle-config
  created: '2026-04-10T22:35:31+00:00'
spec:
  title: 'History: container lifecycle, config, and content-install decisions (DEC-001
    – DEC-024 summary)'
  state: accepted
  decision: 'Condensed record of DEC-001 through DEC-024 from the pre-v0.16.0 era.
    Key choices that shaped current architecture: (DEC-001–010) Docker/Podman abstraction
    layer in `runtime.rs`; named environment management; `aibox.toml` as the single
    source of truth for all generated files; `.aibox-home/` as a gitignored runtime
    seed directory; 8 base image flavors (base, python, latex, typst, rust, and combinations);
    6 Zellij themes (nord, dracula, catppuccin-mocha, catppuccin-latte, tokyo-night,
    gruvbox); three IDE layouts (dev, focus, cowork). (DEC-011–020) Addon YAML format;
    `[addons.X.tools]` for tool version management; transitive `requires` expansion
    at init and `addon add` time; `aibox env` for named environment management; `aibox
    sync` as the config reconciliation command; asciinema-based visual testing in
    `scripts/test-screencasts.sh`. (DEC-021–024) `aibox update` version checking via
    GHCR + GitHub Releases API with `git ls-remote` fallback; structured NDJSON log
    at `.aibox/aibox.log` rotating at 1 MB; `aibox reset`/`aibox backup` lifecycle
    with versioned backup directories under `.aibox/backup/`; `aibox doctor` diagnostic
    framework.'
  context: These decisions were made across early development sessions (2026-03 through
    early 2026-04) before the aibox/processkit split. They are implementation decisions
    already reflected in the current codebase. Recorded here for historical completeness
    during the processkit v0.8.0 reset migration.
  rationale: All these decisions are stable and reflected in the code. Git history
    (`git log`, `git blame`) and the codebase itself are the authoritative sources;
    this record captures the design intent for context only.
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:35:31+00:00'
---
