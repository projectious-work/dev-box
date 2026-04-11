---
name: dependency-management
description: |
  Cross-language dependency management — lockfiles, version pinning, automated updates, security, and licenses. Use when setting up or reviewing lockfile strategy, configuring automated dependency updates, auditing dependencies for security or license issues, handling monorepo dependencies, or deciding whether to vendor.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-dependency-management
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Dependency Management

## Intro

Dependencies are the largest unmaintained surface in most projects.
Pin them, lock them, scan them, and update them on a schedule. The
defaults differ by language, but the principles are universal: commit
the lockfile, automate the updates, gate on CI, and review every
change.

## Overview

### Lockfiles

Lockfiles pin the exact resolved versions of all direct and
transitive dependencies. Always commit lockfiles for applications and
services. For libraries, commit the lockfile for CI reproducibility
but do not publish it. Never edit lockfiles by hand — regenerate them
through the package manager.

| Ecosystem | Manifest | Lockfile |
|-----------|----------|----------|
| Rust | `Cargo.toml` | `Cargo.lock` |
| Python (uv/pip) | `pyproject.toml` | `uv.lock` / `requirements.txt` |
| Node.js (npm) | `package.json` | `package-lock.json` |
| Node.js (pnpm) | `package.json` | `pnpm-lock.yaml` |
| Go | `go.mod` | `go.sum` |

### Version pinning

- **Exact pins** (`==1.2.3`): maximum reproducibility. Use for
  applications and Docker images.
- **Compatible range** (`^1.2.3` or `~=1.2`): allow patch/minor
  updates. Use for libraries to avoid conflicts with consumers.
- **Minimum bound** (`>=1.2.0`): widest compatibility, but can lead
  to untested combinations. Use sparingly.

Rule of thumb: pin tightly in applications, pin loosely in libraries.

### Automated updates

**Dependabot** is GitHub-native. Configure
`.github/dependabot.yml` with ecosystems, schedule, and reviewers.
Set `open-pull-requests-limit` (5–10) to avoid PR floods. Group
minor/patch updates to reduce noise.

**Renovate** is more flexible: monorepos, package-pattern grouping,
auto-merge for patch updates. Use `extends: ["config:recommended"]`
in `renovate.json` as a baseline. Use `matchPackagePatterns` to
group related packages (e.g. all `@aws-sdk/*`).

For both: require CI to pass before merging. Auto-merge patch
updates for well-tested projects. Review major updates manually and
read the changelog.

### Monorepos

Use the workspace features your toolchain provides — Cargo
workspaces, npm/pnpm workspaces, uv workspaces. Share common
dependency versions at the workspace root. Pin shared tooling
(linters, formatters) at the root. Let individual packages declare
their own domain-specific deps. Update everything together with
`cargo update --workspace` or the equivalent.

### Security and licenses

Integrate vulnerability scanning into CI. Block merges on
critical/high vulnerabilities and track accepted risks with ignore
rules and expiry dates. For licenses, maintain an SPDX-based
allow-list and flag copyleft licenses (GPL, AGPL) in proprietary
projects for legal review.

| Tool | Ecosystem |
|---|---|
| `cargo audit` / `cargo-deny` | Rust |
| `npm audit` / `pnpm audit` | Node.js |
| `pip-audit` / `uv pip audit` | Python |
| `govulncheck` | Go |
| `license-checker` | Node.js |
| `pip-licenses` | Python |

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Hand-editing lockfiles.** Lockfiles are generated artifacts — their format is designed for machines and manual edits are easily invalid. Regenerate them with the package manager; never edit by hand.
- **Omitting lockfiles from version control for applications.** Without a committed lockfile, different developers (and CI) resolve different transitive versions. Commit lockfiles for all applications and services; the "don't commit lockfile" rule applies only to libraries.
- **Auto-merging major version bumps.** Major bumps can contain breaking API changes. Auto-merge is appropriate for patches, not majors. Major updates require changelog review before merging.
- **Ignoring Dependabot or Renovate PRs until they pile up.** A backlog of 40 dependency PRs becomes unreviewable. Each one also causes merge conflicts with the others. Review and merge promptly; set a weekly cadence.
- **Pinning loosely in applications "to stay current".** Applications should use exact pins and a managed update workflow. Loose ranges in applications lead to unreproducible builds and silent version drift between environments.
- **Adding a 200-package dependency for a 5-line utility.** Every dependency is a maintenance, security, and license liability. Audit the size and scope of the dependency before adding it; prefer stdlib or a smaller alternative.
- **No SLA on security advisories.** "We'll get to it" means critical vulnerabilities sit in the codebase for months. Define explicit SLAs: critical/high → fix within 48 hours; medium → within the current sprint.

## Full reference

### Dependency review checklist

For every PR that changes dependencies:

1. **What changed** — new dependency, version bump, removal?
2. **Why** — feature, security fix, deprecation?
3. **License** — does it fit the project's allow-list?
4. **Size and scope** — proportional to the problem it solves?
5. **Alternatives** — is there a lighter or stdlib-based option?
6. **Security** — does the new version have known advisories?

### Vendoring

Copy dependencies into the repository for offline builds or
supply-chain control:

- Rust: `cargo vendor` plus `.cargo/config.toml` with
  `[source.vendored-sources]`
- Go: `go mod vendor` plus `-mod=vendor` at build time
- Node.js: rarely vendored; use `npm pack` plus a local tarball if
  needed

Vendor when building in air-gapped environments, pinning against
registry outages, or needing a full audit of dependency source.

### Dependabot example

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
      day: monday
    open-pull-requests-limit: 10
    reviewers:
      - "team-backend"
    groups:
      minor-and-patch:
        update-types: ["minor", "patch"]

  - package-ecosystem: npm
    directory: /frontend
    schedule:
      interval: weekly
    groups:
      react:
        patterns: ["react*", "@types/react*"]
```

### cargo-deny example

```toml
# deny.toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]
copyleft = "deny"

[[licenses.exceptions]]
allow = ["MPL-2.0"]
crates = ["webpki-roots"]

[bans]
multiple-versions = "warn"
deny = [
    { crate = "openssl", use-instead = "rustls" },
]
```

### Renovate auto-merge example

```json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": ["config:recommended"],
  "packageRules": [
    {
      "matchUpdateTypes": ["patch"],
      "automerge": true,
      "automergeType": "pr",
      "requiredStatusChecks": ["ci"]
    },
    {
      "matchPackagePatterns": ["^@aws-sdk/"],
      "groupName": "AWS SDK",
      "schedule": ["before 8am on monday"]
    },
    {
      "matchUpdateTypes": ["major"],
      "labels": ["breaking-change"],
      "automerge": false
    }
  ]
}
```

### Anti-patterns

- Hand-editing lockfiles instead of regenerating them
- Ignoring Dependabot PRs until they pile up into an unreviewable
  backlog
- Auto-merging major updates without reading the changelog
- Pinning loosely in applications "to stay current" — that is what
  the update workflow is for
- Adding a 200-package dependency for a five-line utility
- Treating a security advisory as "we'll get to it" — set an SLA
