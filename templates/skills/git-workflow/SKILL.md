---
name: git-workflow
description: Git workflow conventions — branch naming, commit messages, PR descriptions, merge strategies. Use when the user needs guidance on git practices.
---

# Git Workflow

## When to Use

When the user asks about branch naming, commit message format, PR descriptions, merge strategy, or says "how should I organize my commits?" or "what should I name this branch?".

## Instructions

1. **Branch naming:** `<type>/<issue>-<short-description>`
   - Types: `feat/`, `fix/`, `refactor/`, `docs/`, `chore/`
   - Example: `feat/42-add-user-auth`, `fix/17-null-pointer-crash`
2. **Commit messages:** Follow Conventional Commits
   - Format: `<type>: <description>` (lowercase, imperative mood, no period)
   - Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`
   - Body: explain WHY, not WHAT (the diff shows what)
   - Footer: `fixes #N` or `refs #N` for issue references
3. **PR descriptions:**
   - Title: same format as commit messages, under 70 characters
   - Body: Summary (what and why), test plan, breaking changes
   - Link related issues
4. **Merge strategy:**
   - Squash merge for feature branches (clean history)
   - Merge commit for long-lived branches (preserve context)
   - Rebase for keeping branches up to date (before PR)
5. **General rules:**
   - Commit early and often on feature branches
   - Never force-push to shared branches
   - Keep commits atomic — one logical change per commit

## Examples

**User:** "What should I name this branch for adding search?"
**Agent:** Suggests `feat/30-add-search-functionality` (with issue number if one exists).
