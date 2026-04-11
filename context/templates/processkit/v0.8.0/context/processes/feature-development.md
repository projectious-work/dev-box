---
apiVersion: processkit.projectious.work/v1
kind: Process
metadata:
  id: PROC-feature-development
  version: "1.0.0"
  created: 2026-04-07T00:00:00Z
spec:
  name: feature-development
  description: "Take a feature from backlog item to merged code."
  triggers:
    - workitem.transitioned
    - feature.requested
  roles:
    - developer
    - reviewer
  steps:
    - name: pick-up-workitem
      role: developer
      description: >
        Pick a WorkItem from the backlog or accept assignment. Move it
        from backlog to in-progress via workitem-management.
    - name: clarify-requirements
      role: developer
      description: >
        Read the workitem, related discussions, and any linked
        decisions. Open clarifying questions before writing code.
    - name: implement
      role: developer
      description: >
        Implement the feature with tests. Decompose into smaller commits
        if the change is large.
      uses_skill: testing
    - name: open-pr
      role: developer
      description: "Open a pull request linking back to the workitem."
    - name: code-review
      role: reviewer
      description: "Run the code-review process on the PR."
      gates:
        - GATE-code-review-passed
    - name: merge
      role: developer
      description: "Merge after approval and CI green."
      gates:
        - GATE-tests-green
        - GATE-code-review-passed
    - name: close-workitem
      role: developer
      description: >
        Move the WorkItem to done via workitem-management. The
        transition emits a LogEntry that closes the audit trail.
  definition_of_done: >
    Feature works as described in the WorkItem; tests cover the new
    behavior; the change is reviewed, approved, merged; the WorkItem
    is in the done state.
  retryable: true
---

# Feature Development Process

The default workflow for taking a tracked piece of work from backlog
to done. Feature here is broad: stories, tasks, chores, spikes — any
WorkItem subtype follows the same shape.

## Why "clarify requirements" is its own step

Skipping this step is the second most common cause of wasted work.
The point is not to gate progress on perfect specs — it is to surface
the unknowns before they become surprises three days into
implementation.

## Linking back to the WorkItem

Every commit message and the PR description should reference the
WorkItem ID (e.g. `BACK-vast-falcon`). This lets the index server
join the diff to the entity history and answer questions like "what
work did we ship in sprint 42?".
