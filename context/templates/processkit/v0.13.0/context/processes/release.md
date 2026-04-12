---
apiVersion: processkit.projectious.work/v1
kind: Process
metadata:
  id: PROC-release
  version: "1.0.0"
  created: 2026-04-07T00:00:00Z
spec:
  name: release
  description: "Cut, tag, and publish a versioned release."
  triggers:
    - release.requested
    - milestone.completed
  roles:
    - release-manager
    - developer
  steps:
    - name: pre-release-tests
      role: developer
      description: "Run the full test suite and any smoke tests."
      gates:
        - GATE-tests-green
    - name: update-changelog
      role: release-manager
      description: >
        Add the new version section to CHANGELOG.md (or equivalent).
        List notable changes user-facing first.
    - name: bump-version
      role: release-manager
      description: >
        Bump the version in whatever files declare it (package.json,
        pyproject.toml, Cargo.toml, etc.). Commit the bump separately
        from feature work.
    - name: tag
      role: release-manager
      description: >
        Create an annotated git tag matching the new version. Push it
        to the remote.
    - name: publish-artifacts
      role: release-manager
      description: >
        Build and upload release artifacts (tarballs, container images,
        package-registry uploads, documentation deploy). Verify each
        artifact downloads cleanly.
      gates:
        - GATE-artifacts-verified
    - name: announce
      role: release-manager
      description: >
        Announce the release in the channels the project uses. Link to
        the changelog entry.
  definition_of_done: >
    All tests pass; CHANGELOG updated; version bumped; tag pushed;
    artifacts published and verified; release announced in the
    project's communication channels.
  retryable: false
---

# Release Process

Versioned releases follow semver and are gated on tests and artifact
verification. The release manager role can be the same person as the
developer in small projects.

## Why retryable: false

A failed release attempt that has already pushed a tag and uploaded
some artifacts cannot simply be re-run. Retrying means: investigate
what was done, decide whether to roll back or roll forward, then
either bump to the next patch version (roll forward) or delete the
tag and re-do with corrections (roll back, only safe if nothing has
been downloaded yet).

## Building release artifacts

For processkit itself, see `scripts/build-release-tarball.sh` — the
script produces a reproducible tarball + sha256 sidecar that aibox
prefers as the consumption path (DEC-025 / aibox BACK-106).
