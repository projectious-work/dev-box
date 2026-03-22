---
name: ci-cd-setup
description: CI/CD pipeline setup — GitHub Actions, testing, linting, deployment. Use when setting up or improving continuous integration and deployment.
---

# CI/CD Setup

## When to Use

When the user asks to "set up CI", "add GitHub Actions", "automate tests", "add deployment", or wants to improve their build pipeline.

## Instructions

1. **Pipeline stages** (in order):
   - **Lint:** Format checking, static analysis (fastest feedback)
   - **Test:** Unit tests, then integration tests
   - **Build:** Compile, package, create artifacts
   - **Deploy:** Push to staging, then production
2. **GitHub Actions basics:**
   - Trigger on `push` to main and `pull_request`
   - Use specific action versions: `actions/checkout@v4` (not `@latest`)
   - Cache dependencies (pip, npm, cargo) for faster builds
   - Set `timeout-minutes` to prevent hung jobs
3. **Testing in CI:**
   - Run the same commands as local development
   - Use matrix builds for multiple OS/version combos only when needed
   - Fail fast: `--fail-fast` in matrix strategies
4. **Security:**
   - Never put secrets in workflow files — use GitHub Secrets
   - Use `permissions` key to limit token scope
   - Pin third-party actions to SHA, not just version tag
5. **Best practices:**
   - Keep CI fast (under 5 minutes for PRs)
   - Make CI failures actionable (clear error messages)
   - Require CI pass before merge (branch protection rules)
   - Run expensive checks (E2E, deploy) only on main branch

## Examples

**User:** "Set up CI for my Rust project"
**Agent:** Creates `.github/workflows/ci.yml` with lint (clippy), test (cargo test), and build steps, with cargo caching for faster runs.
