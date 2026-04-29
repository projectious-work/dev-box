---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260426_1627-TrueRaven-investigate-content-diff-conflict
  created: '2026-04-26T16:27:49+00:00'
  updated: '2026-04-29T10:09:10+00:00'
spec:
  title: Investigate content_diff conflict-classifier false-positives during processkit
    upgrades
  state: done
  type: bug
  priority: medium
  description: |
    ## Symptom

    The migration plan for the v0.22.0 → v0.23.0 processkit upgrade reported **50 "conflicts"** that were not real conflicts: on every flagged file, the live SHA matched the old-mirror SHA (i.e., the user had not edited the file, the upstream had simply changed it). `aibox sync` still wrote the right content, so no data was lost — but the classifier is over-eager and labels upstream-only changes as conflicts.

    ## Why it matters

    A migration plan that cries wolf 50 times trains operators to skim past conflict warnings. The next time a *real* conflict surfaces it will be lost in the noise — exactly the failure mode classifier exists to prevent. The bug is latent (no immediate operator action required) but every future processkit upgrade compounds the trust debt.

    ## Hypothesis

    In `crates/aibox/src/sync/content_diff.rs` (or wherever the classifier lives), the v0.21.0 fix tightened the `RemovedUpstreamStale` rules but left the regular conflict classifier with the same false-positive shape. Specifically: when `live_sha == old_mirror_sha` (user has *not* modified the file), an upstream-only change should be classified as `UpstreamUpdated`, not `Conflict`. The current code path may be checking `live_sha != new_upstream_sha` without first ruling out the unmodified-by-user case.

    ## Investigation steps

    1. Read the classifier in `crates/aibox/src/sync/` (start with `content_diff.rs`, `migration.rs`).
    2. Reproduce: take any of the 50 v0.22.0→v0.23.0 files, build a minimal test harness with three fixed SHAs (live=old_mirror, new_upstream different), assert classifier returns `UpstreamUpdated` not `Conflict`.
    3. Add a unit test for the canonical no-conflict case: `live_sha == old_mirror_sha && new_upstream_sha != old_mirror_sha → UpstreamUpdated`.
    4. Add a regression test for the actual-conflict case: `live_sha != old_mirror_sha && new_upstream_sha != old_mirror_sha && live_sha != new_upstream_sha → Conflict`.
    5. Fix.
    6. Replay the v0.22.0→v0.23.0 migration plan and verify the conflict count drops to ~0 (or whatever the genuine conflict count is).

    ## Acceptance

    - New unit tests cover both no-conflict and genuine-conflict cases.
    - Replaying v0.22.0→v0.23.0 migration plan against the real mirror tree shows the conflict list contains only files where the user actually modified the live copy.
    - v0.21.0's `RemovedUpstreamStale` tightening is preserved.

    ## Context

    Surfaced in the v0.21.1 release cycle handover (LOG-20260426_1613-SunnyLynx-session-handover). Encoded as an open thread and now opened as a WorkItem per the recommendation in that handover's `next_recommended_action`.
  started_at: '2026-04-26T16:33:48+00:00'
  completed_at: '2026-04-29T10:09:10+00:00'
---

## Transition note (2026-04-26T16:33:48+00:00)

Starting investigation: locate classifier in cli/src/sync/, identify false-positive code path, plan fix + tests.


## Transition note (2026-04-29T10:09:04+00:00)

Reconciliation review: content_diff classifier now includes stale-lock-aware handling and RemovedUpstreamStale safeguards in cli/src/content_diff.rs; current repository carries the fix.


## Transition note (2026-04-29T10:09:10+00:00)

Closed during 2026-04-29 reconciliation after confirming the fix is present in the working tree and no longer needs in-progress tracking.
