---
name: release-semver
description: |
  Semantic versioning releases — version bumps, changelogs, tags, publishing. Use when preparing a new release: deciding the version bump, drafting the changelog, tagging the commit, and publishing to the project's distribution channel.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-release-semver
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
    layer: 3
    commands:
      - name: release-semver-prepare
        args: "bump-type"
        description: "Prepare a release: bump version, draft changelog, create tag (bump-type: major, minor, patch, or auto)"
      - name: release-semver-publish
        args: ""
        description: "Execute the publish phase of the prepared release"
---

# Semantic Versioning Release Process

## Intro

Releases are routine, not heroic. Decide the version bump from the
nature of the changes, draft a changelog, update version files in
one commit, tag with `vX.Y.Z`, and publish. The goal is a release
anyone on the team can perform from the same checklist.

## Overview

### Determine the version bump

Semantic versioning has three numbers: MAJOR.MINOR.PATCH.

- **Patch** (0.1.0 → 0.1.1): bug fixes only, no API changes.
- **Minor** (0.1.0 → 0.2.0): new features, backward-compatible.
- **Major** (0.1.0 → 1.0.0): breaking changes.

Pre-1.0 projects bend the rules: minor bumps are allowed to break
compatibility, and patch bumps cover both features and fixes. Once
you ship 1.0, the rules become strict.

### Pre-release checklist

- All tests pass on the main branch.
- No uncommitted changes in the working tree.
- Dependencies reviewed and up to date where appropriate.
- CHANGELOG or release notes drafted.

### Draft the changelog

Group entries under **Added**, **Changed**, **Fixed**, and
**Removed**. Reference issue and PR numbers. Note breaking changes
prominently — call them out in their own section if a major bump
is involved.

### Bump, commit, tag, publish

Update the version in every relevant file (`Cargo.toml`,
`pyproject.toml`, `package.json`, lockfiles). Commit with a message
like `chore: bump version to vX.Y.Z`. Tag the commit `vX.Y.Z`.
Publish according to the project's distribution channel
(crates.io, PyPI, npm, GitHub Releases, container registry, etc.).

### Example

User says: "Let's release — we fixed two bugs and added a feature."
The agent recommends a minor bump (because of the new feature),
drafts changelog entries grouped Added/Fixed, updates version
files, creates the bump commit, and tags it.

This skill also provides the `/release-semver-prepare` slash command for direct invocation — see `commands/release-semver-prepare.md`. This skill also provides the `/release-semver-publish` slash command for direct invocation — see `commands/release-semver-publish.md`.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Bundling a breaking change into a patch or minor release.** Consumers rely on semver guarantees to control when they take breaking changes. A breaking change in a patch release violates the contract and causes silent runtime failures in consumer code.
- **Skipping the changelog because "git log has it".** Git log is not a changelog. It lacks context, grouping, and the human judgment that makes a changelog useful. Every release needs a curated changelog with Added/Changed/Fixed/Removed sections.
- **Bumping all version files in separate commits.** A brief window where `Cargo.toml` says `1.2.0` but `pyproject.toml` still says `1.1.0` is a broken state. Update all version files in a single atomic commit.
- **Tagging before CI passes.** A tag should point to a commit that has been fully built and tested. Tagging a commit that later fails CI means the published artifact may not match what was verified.
- **Publishing from a developer machine without a clean checkout.** Uncommitted local changes — WIP files, debug flags, `console.log` statements — may sneak into the artifact. Publish from CI with a clean checkout or from a fresh clone.
- **Re-tagging a published version to fix a mistake.** Package registries (crates.io, PyPI, npm) prohibit or block re-publishing to an existing version. Cut a new patch version (e.g. `1.2.1`) to fix a mistake in `1.2.0`; never overwrite a tag.
- **Pre-1.0 projects that promise stability they cannot keep.** If the API is still moving, stay below 1.0 and say so. Cutting `1.0.0` too early creates a compatibility obligation that slows future improvements.

## Full reference

### When pre-1.0 ends

Cut 1.0 when the public API is something you are willing to
support without breaking. Until then, keep the version below 1.0
honestly — there is no shame in 0.17.0, and it signals to users
that the surface may still move.

### Version bumps for special changes

| Change | Bump |
|---|---|
| Bug fix that does not change behavior contract | patch |
| Performance improvement, no API change | patch |
| New feature behind a feature flag | minor |
| New optional parameter with a default | minor |
| Removing or renaming a public symbol | major |
| Changing default behavior of an existing API | major |
| Tightening input validation that rejects previously valid input | major |
| Dependency bump that surfaces in your public API | major |

### Anti-patterns

- Bundling breaking changes into a patch release
- Skipping the changelog "because git log has it"
- Tagging a commit that was not built and tested
- Publishing from a developer machine without a clean checkout
- Re-tagging a published version to fix a mistake (cut a new
  patch instead)
- Pre-1.0 projects that promise stability they cannot keep
