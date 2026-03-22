---
name: release-semver
description: Manages semantic versioning releases — version bumps, changelogs, tags, publishing. Use when preparing a new release.
---

# Semantic Versioning Release Process

## When to Use

When the user says "prepare a release", "bump the version", "publish", "cut a release", or asks about versioning.

## Instructions

1. **Determine version bump:**
   - **Patch** (0.1.0 → 0.1.1): Bug fixes, no API changes
   - **Minor** (0.1.0 → 0.2.0): New features, backward-compatible
   - **Major** (0.1.0 → 1.0.0): Breaking changes
   - Pre-1.0: minor = breaking, patch = features+fixes
2. **Pre-release checklist:**
   - All tests pass on the main branch
   - No uncommitted changes
   - Dependencies are up to date
   - CHANGELOG or release notes drafted
3. **Prepare the changelog:**
   - Group changes: Added, Changed, Fixed, Removed
   - Reference issue/PR numbers
   - Note breaking changes prominently
4. **Bump version** in all relevant files (Cargo.toml, pyproject.toml, package.json, etc.)
5. **Commit and tag:**
   - Commit message: `chore: bump version to vX.Y.Z`
   - Tag: `vX.Y.Z`
6. **Publish** according to the project's distribution channel

## Examples

**User:** "Let's release — we fixed two bugs and added a feature"
**Agent:** Recommends a minor bump (new feature), drafts changelog entries, updates version files, creates the tagged commit.
