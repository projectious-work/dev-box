---
name: dependency-audit
description: Audits project dependencies for vulnerabilities and outdated packages. Use when checking security posture or planning dependency updates.
---

# Dependency Audit

## When to Use

When the user asks to "check dependencies", "audit security", "update packages", "are my dependencies safe?", or before a release to verify dependency health.

## Instructions

1. **Identify the package manager and run its audit tool:**
   - Rust: `cargo audit` (install with `cargo install cargo-audit`)
   - Python: `pip-audit` or `safety check`
   - Node.js: `npm audit` or `pnpm audit`
   - Go: `govulncheck ./...`
2. **Review findings by severity:**
   - **Critical/High:** Fix immediately — update or find alternative
   - **Medium:** Plan fix within current sprint
   - **Low:** Track and fix when convenient
3. **Update strategy:**
   - Update one dependency at a time (easier to identify breakage)
   - Run full test suite after each update
   - Check changelogs for breaking changes before major bumps
   - Use lockfiles to pin exact versions
4. **Check for outdated packages:**
   - Rust: `cargo outdated`
   - Python: `pip list --outdated`
   - Node.js: `npm outdated`
5. **Ongoing maintenance:**
   - Schedule monthly dependency reviews
   - Enable Dependabot or Renovate for automated PRs
   - Document any pinned versions with reasons in comments

## Examples

**User:** "Are my dependencies secure?"
**Agent:** Runs the appropriate audit tool, summarizes findings by severity, and recommends specific version bumps for vulnerable packages. Flags any dependencies with no maintained alternatives.
