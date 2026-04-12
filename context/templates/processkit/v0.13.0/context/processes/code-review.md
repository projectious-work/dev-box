---
apiVersion: processkit.projectious.work/v1
kind: Process
metadata:
  id: PROC-code-review
  version: "1.0.0"
  created: 2026-04-07T00:00:00Z
spec:
  name: code-review
  description: "Review a change before it merges into the main branch."
  triggers:
    - pr.opened
    - pr.review-requested
  roles:
    - developer
    - reviewer
  steps:
    - name: author-self-check
      role: developer
      description: >
        The author runs the self-review checklist before requesting
        review: lint clean, tests passing, scope clear, no unrelated
        changes bundled, docstrings/comments updated.
      uses_skill: code-review
    - name: peer-review
      role: reviewer
      description: >
        A different actor reads the diff and leaves comments addressing
        correctness, readability, test coverage, and architectural
        consistency.
      uses_skill: code-review
      gates:
        - GATE-no-blocking-comments
    - name: address-feedback
      role: developer
      description: >
        The author addresses all blocking comments. Non-blocking nits
        are addressed at the author's discretion.
      on_failure: retry
    - name: approval
      role: reviewer
      description: "The reviewer approves the change."
      gates:
        - GATE-code-review-passed
    - name: merge
      role: developer
      description: "The author merges after approval and CI green."
      gates:
        - GATE-tests-green
        - GATE-code-review-passed
  definition_of_done: >
    PR merged with at least one approval, all blocking comments
    resolved, and all CI gates passed.
  retryable: true
---

# Code Review Process

A change moves from "the author thinks it's done" to "merged into
main" via at least one independent reviewer. processkit defines the
shape; the actual review happens in whatever tool the project uses
(GitHub, GitLab, Gerrit, etc.).

## Author and reviewer must differ

A code review by the author of the change is not a code review.
processkit does not enforce this — it is a social rule — but it is
the entire point of the process.

## Blocking vs non-blocking comments

A reviewer may mark comments as blocking ("this must be addressed
before merge") or non-blocking ("nit", "consider", "fyi"). Only the
blocking comments gate the merge. The convention prevents review
threads from spiraling into infinite back-and-forth on stylistic
preferences.
