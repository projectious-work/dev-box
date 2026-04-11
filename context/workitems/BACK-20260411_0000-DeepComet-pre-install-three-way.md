---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-DeepComet-pre-install-three-way
  created: '2026-04-10T22:36:46+00:00'
  labels:
    old_id: BACK-120
    area: cli
spec:
  title: Pre-install three-way diff to prevent edit-clobbering on version upgrade
  state: backlog
  type: task
  priority: medium
  description: 'The diff in `cmd_sync` runs AFTER `install_content_source` overwrites
    live files, so user edits to skills/processes/AGENTS.md are silently overwritten
    when crossing a processkit version boundary. Fix: run the diff BEFORE the install
    (or keep the OLD templates mirror around during the transition), generate migration
    documents for any conflicts, then run the install for non-conflicted files only.
    Touches `content_init`, `content_diff`, `cmd_sync`. Affects all install paths,
    not just templated files. Deferred to v0.17+ as it touches the core install/diff
    ordering. Old ID: BACK-120.'
---
