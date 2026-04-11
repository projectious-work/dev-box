---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-SureMeadow-history-aibox-processkit-integration
  created: '2026-04-10T22:35:45+00:00'
spec:
  title: 'History: aibox/processkit integration decisions (DEC-025 – DEC-029 summary)'
  state: accepted
  decision: 'Condensed record of DEC-025 through DEC-029 covering the v0.16.x integration
    machinery. (DEC-025) Generic content-source release-asset fetcher — architecture
    decision recorded separately as a load-bearing decision. (DEC-026) Cache-tracked
    processkit reference: the full processkit source tree is cached locally under
    `.aibox/cache/processkit/<version>/` so the install pipeline works offline and
    the three-way diff has a stable reference. (DEC-027) Rip bundled process layer
    — recorded separately as load-bearing. (DEC-028) `cmd_sync` auto-installs processkit
    when `[processkit].version != "unset"` AND (no lock OR lock disagrees with `(source,
    version)`). `aibox init` gets an interactive version picker via `content_source::list_versions()`
    with GitHub Releases API + `git ls-remote` fallback. New flags: `--processkit-source`,
    `--processkit-version`, `--processkit-branch`. (DEC-029) `list_versions` falls
    back to `git ls-remote --tags --refs` on any GitHub API failure. Sync perimeter
    expanded to cover `aibox.lock`, `AGENTS.md`, `context/skills/`, `context/schemas/`,
    `context/state-machines/`, `context/processes/`, `context/templates/`. `AGENTS.md`
    removed from `TRIPWIRE_SENTINELS` (sync legitimately writes it now).'
  context: 'Post-v0.16.0 integration work to make the processkit install pipeline
    robust: auto-install on sync, interactive version picker, API fallback, and correct
    sync perimeter. All shipped in v0.16.1 and v0.16.2 within a week of the v0.16.0
    split.'
  rationale: Recorded for historical completeness during the processkit v0.8.0 reset
    migration. All decisions are stable and reflected in the current codebase.
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:35:45+00:00'
---
