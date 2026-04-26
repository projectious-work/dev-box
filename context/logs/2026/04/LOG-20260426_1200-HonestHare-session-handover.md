---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260426_1200-HonestHare-session-handover
  created: '2026-04-26T12:00:24+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-04-26T12:00:24+00:00'
  summary: Session handover — aibox v0.20.0 fully shipped (Phase 1 + Phase 2); 13
    CLI defects fixed; clean main branch
  actor: claude-code-opus-4-7
  details:
    session_date: '2026-04-26'
    current_state: 'aibox v0.20.0 is fully released across both phases. Phase 1 (Linux
      binaries, container build, GitHub release) and Phase 2 (macOS aarch64 + x86_64
      binaries, GHCR images) are confirmed live at https://github.com/projectious-work/aibox/releases/tag/v0.20.0.
      The release closes the multi-agent investigation that began from a single user-visible
      symptom (/pk-doctor and /pk-resume diverging from the reference processkit repo)
      and ended in a deployment-pipeline overhaul: 13 distinct CLI defects identified,
      12 actively fixed in v0.20.0, 1 deferred (CLI-11 Codex parity, tracked separately).
      Test count rose 670 → 709. Branch is on main, working tree is clean except for
      one untracked release-event log (LOG-20260426_0037-CalmRobin-release-shipped.md).
      No in-progress or blocked WorkItems.'
    open_threads:
    - CLI-11 (Codex allowed_tools) deferred per aibox#55 — file follow-up issue when
      Codex ships a permissions surface upstream.
    - Untracked log file context/logs/2026/04/LOG-20260426_0037-CalmRobin-release-shipped.md
      from the release.shipped event — not yet committed; safe to commit on next sync
      or leave as event-log artifact.
    - processkit v0.22.0 ships PROVENANCE.toml mirror with generated_for_tag=v0.21.0
      (upstream stamping bug). aibox tolerates it via tracing::warn! — file an upstream
      issue against processkit if not already tracked.
    - WS-0 PR-B's --no-container mode is gated only on init/sync; reset.rs, update.rs,
      doctor.rs still call Runtime::detect(). Documented limitation; revisit if needed.
    next_recommended_action: No active workstream. The next session should start by
      querying recent events and running /pk-resume to orient. If the user wants to
      file the deferred CLI-11 issue or the upstream processkit PROVENANCE bug, those
      are the only loose threads from v0.20.0.
    branch: main
    commit: '9854284'
    uncommitted_changes:
    - context/logs/2026/04/LOG-20260426_0037-CalmRobin-release-shipped.md (untracked
      release event log)
    stashes: none
    behavioral_retrospective:
    - 'Parallel WS-2/WS-4/WS-6 background agents reverted WS-0 PR-A''s clap changes
      (no_container field, parse_truthy_flag helper) when their isolated worktrees
      merged back. Encoded by manually restoring + verifying via cargo test before
      next batch — ongoing rule: when batching parallel impl agents over overlapping
      files, run a quick git diff sweep after each batch lands and before the next
      batch fires.'
    - 'Initial cmd_sync signature had no_container in 4th position but main.rs passed
      it 5th, causing AIBOX_NO_CONTAINER=1 to set the wrong bool. Reordered signature
      to (config_path, no_cache, no_build, fix_compliance_contract, no_container)
      and updated all 3 call sites. Lesson: when adding a new flag through a long
      signature, audit ALL call sites in one pass, not iteratively.'
    - Several background subagents returned suspicious 'monthly usage limit' / 'safety
      classifier unavailable' messages but their file changes were substantive and
      verified. Flagged as potential prompt-injection vector at the time; no action
      required since changes were independently verified by build/test, but worth
      keeping in mind for future multi-agent sessions.
---
