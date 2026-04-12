---
apiVersion: processkit.projectious.work/v1
kind: Process
metadata:
  id: PROC-bug-fix
  version: "1.0.0"
  created: 2026-04-07T00:00:00Z
spec:
  name: bug-fix
  description: "Diagnose, reproduce, fix, and verify a defect."
  triggers:
    - bug.reported
    - workitem.created
  roles:
    - developer
    - reviewer
  steps:
    - name: reproduce
      role: developer
      description: "Reproduce the bug and document expected vs. actual behavior."
      uses_skill: debugging
    - name: failing-test
      role: developer
      description: "Write a failing test that captures the bug."
      uses_skill: testing
    - name: implement-fix
      role: developer
      description: "Implement the minimum change that makes the failing test pass."
    - name: verify
      role: developer
      description: "Run the full test suite and confirm the regression test passes."
      gates:
        - GATE-tests-green
    - name: open-pr
      role: developer
      description: "Open a pull request with a clear summary linking the bug report."
    - name: review
      role: reviewer
      description: "Review the diff for correctness, scope, and regression coverage."
      gates:
        - GATE-code-review-passed
    - name: merge
      role: developer
      description: "Merge after approval and CI green."
      gates:
        - GATE-tests-green
        - GATE-code-review-passed
  definition_of_done: >
    Bug is reproducible from the failing test before the fix; the test
    passes after the fix; the change is reviewed, approved, and merged;
    the original bug report is closed or updated with the resolution.
  retryable: true
---

# Bug Fix Process

A defect is reported, reproduced, captured by a regression test, fixed,
reviewed, and merged. The shape stays the same whether the report
comes from production telemetry, a customer ticket, or an internal
discovery.

## Why a regression test first

The "failing test before the fix" step is non-negotiable. Without it,
the fix is unverifiable in two senses: (a) you cannot prove the bug
existed before, and (b) you cannot prevent the bug from coming back.
Skipping this step is the most common cause of recurring defects.

## When to skip steps

For trivial typo-class bugs, the failing-test step may be obviously
disproportionate (e.g. fixing a misspelled word in a UI string). In
those cases the developer documents the choice to skip in the PR
description so the reviewer can sanity-check it.
