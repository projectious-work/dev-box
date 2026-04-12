---
name: ci-cd-setup
description: |
  CI/CD pipeline setup — GitHub Actions, testing, linting, deployment. Use when setting up CI from scratch, adding GitHub Actions, automating tests, wiring up deployment, or improving an existing build pipeline.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-ci-cd-setup
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
---

# CI/CD Setup

## Intro

A good CI pipeline runs the same checks a developer would run locally, in
the same order, fast enough that contributors get feedback before they
context-switch. Lint first, then test, then build, then deploy — and never
let secrets or unpinned actions sneak in.

## Overview

### Pipeline stages

Order stages from cheapest to most expensive so failures surface early:

1. **Lint** — format checking and static analysis (fastest feedback)
2. **Test** — unit tests first, then integration tests
3. **Build** — compile, package, create release artifacts
4. **Deploy** — push to staging, then promote to production

### GitHub Actions basics

Trigger on `push` to main and on `pull_request`. Pin action versions
explicitly (`actions/checkout@v4`, never `@latest`). Cache dependencies
(pip, npm, cargo) so reruns are fast. Set `timeout-minutes` on every job
so a hung step cannot eat your minutes budget.

### Testing in CI

Run the same commands as local development — if `make test` works
locally, that is what CI should call. Use matrix builds for OS or
language-version combinations only when you actually need to support
them. Use `fail-fast: true` in matrix strategies so one failure cancels
the rest.

### Security

Never put secrets in workflow files. Store them in GitHub Secrets and
inject through `${{ secrets.NAME }}`. Use the `permissions` key on each
job to limit the `GITHUB_TOKEN` scope to the minimum needed. Pin
third-party actions to a commit SHA, not just a version tag, so an
attacker who compromises a tag cannot inject code into your build.

### Best practices

- Keep PR pipelines under five minutes wherever possible.
- Make CI failures actionable — clear error messages, surfaced summaries.
- Require CI to pass before merge via branch protection rules.
- Run expensive checks (E2E suites, deploys) only on main, not on PRs.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Pinning an action to a tag instead of a commit SHA.** Tags are mutable — an attacker who compromises an action's tag can inject code into your build. Pin third-party actions to their full commit SHA, not just `@v4`.
- **Secrets leaking into build logs.** Printing or echoing an environment variable that contains a secret exposes it in the CI log. Never `echo ${{ secrets.FOO }}` — pass secrets only through inputs or masked env vars.
- **Running expensive checks on every PR.** E2E tests and deploy jobs on every pull request slow the pipeline and teach contributors to bypass CI. Gate slow jobs behind `if: github.ref == 'refs/heads/main'` or a label.
- **No `timeout-minutes` on jobs.** A hung step runs indefinitely and eats your minutes budget. Every job and every slow step needs a timeout — not just the slow ones you expect.
- **CI passes but local fails (or vice versa).** Environment divergence — OS, timezone, locale, file path casing, implicit tool versions — causes mismatch. Make CI reproduce local conditions explicitly; never assume the runner matches the developer's machine.
- **One mega-job that lints, tests, builds, and deploys.** Failures are hard to locate, reruns are expensive, and the job cannot parallelize. Split into separate jobs; each job should do one thing.
- **Branch protection not enforcing required status checks.** If CI is advisory rather than required for merge, it stops being trusted. Configure branch protection to require every status check that matters before merge.

## Full reference

### Sample workflow shape

```yaml
name: ci
on:
  push:
    branches: [main]
  pull_request:
jobs:
  lint:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
          cache: pip
      - run: pip install -e .[dev]
      - run: ruff check .
  test:
    needs: lint
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
          cache: pip
      - run: pip install -e .[dev]
      - run: pytest
```

### Caching tips by ecosystem

| Ecosystem | Cache key target |
|---|---|
| Python (pip) | `setup-python` with `cache: pip` and `cache-dependency-path` |
| Node (npm/yarn/pnpm) | `setup-node` with `cache: npm` (or yarn/pnpm) |
| Rust (cargo) | `Swatinem/rust-cache` action keyed on `Cargo.lock` |
| Go | `actions/setup-go` with `cache: true` |
| Docker layers | `docker/build-push-action` with `cache-from`/`cache-to` |

### Anti-patterns

- **`actions/checkout@latest`** — pin to a major version or SHA
- **Putting tokens in `env:` blocks at the top of a workflow** — use job
  or step scope so secrets are not over-shared
- **Running E2E tests on every PR** — gate them behind a label or run
  only on main
- **Skipping CI with `[skip ci]` to "save time"** — fix the slow step
  instead; once CI is optional, it stops being trusted
- **One mega-job that lints, tests, builds, deploys** — split into
  jobs so failures are localized and reruns are cheap

### Branch protection checklist

- Require pull request reviews before merge
- Require status checks to pass: `lint`, `test`, and any required builds
- Require branches to be up to date before merging
- Restrict who can push directly to `main`
- Require signed commits if your team has the workflow for it
