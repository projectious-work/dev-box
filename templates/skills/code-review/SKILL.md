---
name: code-review
description: Guides structured code review with a checklist approach. Use when reviewing PRs, diffs, or code changes before merging.
---

# Code Review

## When to Use

When the user asks to review code, a PR, a diff, or says "review this", "check my changes", or "is this ready to merge".

## Instructions

1. Read the diff or changed files
2. Evaluate against this checklist:
   - **Correctness:** Does the code do what it claims? Are edge cases handled?
   - **Clarity:** Are names descriptive? Is logic easy to follow?
   - **Tests:** Are new behaviors tested? Are existing tests still valid?
   - **Security:** Any hardcoded secrets, SQL injection, or unsafe input handling?
   - **Performance:** Any unnecessary allocations, N+1 queries, or O(n^2) loops?
   - **Style:** Consistent with the project's existing conventions?
3. Categorize findings as:
   - **Must fix:** Bugs, security issues, broken tests
   - **Should fix:** Unclear naming, missing tests, code smells
   - **Nit:** Style preferences, minor suggestions
4. Provide specific line references and concrete suggestions
5. Acknowledge what was done well

## Examples

**User:** "Review my changes"
**Agent:** Reads the diff, then provides structured feedback:

> **Must fix:** `parse_input()` on line 42 doesn't handle empty strings — this will panic.
>
> **Should fix:** The variable `x` on line 15 could be renamed to `retry_count` for clarity.
>
> **Nit:** Consider extracting lines 30-45 into a helper function.
>
> Overall: Good separation of concerns. The error handling pattern is clean.
