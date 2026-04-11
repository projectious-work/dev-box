---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-ProudWolf-skill-versioning-and-status
  created: 2026-04-11
spec:
  title: "Skill Versioning and Status Metadata Design"
  type: reference
  state: permanent
  tags: [skills, versioning, metadata, quality, maturity]
  skill: research-with-confidence
  source_file: skill-versioning-design-2026-03.md
---

# Skill Versioning and Status Metadata Design

**Date:** 2026-03-26
**Status:** Draft

---

## 1. Problem Statement

The aibox skill library contains 85 curated SKILL.md files with no mechanism to track:
- **Version**: Which iteration of a skill is deployed.
- **Maturity status**: Whether a skill is experimental, under review, or production-ready.
- **Review provenance**: Who reviewed a skill and when.

Three audiences need this:
1. **End users** choosing skills need to know if they're battle-tested or freshly written.
2. **The CLI** (`aibox sync`) should warn when deploying immature skills.
3. **Maintainers** need to track which skills still need quality review work.

---

## 2. Status Lifecycle

| Status | Meaning | Entry Criteria |
|--------|---------|----------------|
| `draft` | Newly written, not yet reviewed | Valid SKILL.md with `name` and `description` |
| `alpha` | Functionally complete but not validated | Has "When to Use" + "Instructions" sections, min 25 lines |
| `beta` | Passes automated quality checks | All automated checks pass, `allowed-tools` declared, code example exists |
| `reviewed` | Human-reviewed and approved | Reviewed by project owner, three-level structure, trigger description optimized |
| `stable` | Production-ready | `reviewed` for at least one release cycle without changes |

Transitions: `draft → alpha → beta → reviewed → stable`
Breaking changes in `stable` reset to `alpha`.

---

## 3. Version Semantics

**Recommendation: simplified semver (`MAJOR.MINOR`).**

Full three-part semver is excessive for markdown documents.
- **MAJOR** bump: breaking change
- **MINOR** bump: non-breaking improvement

All skills start at `1.0`.

### Breaking vs Non-Breaking Changes

| Change Type | Breaking? |
|------------|-----------|
| Changed `allowed-tools` | **Yes** |
| Restructured instructions that change agent behavior | **Yes** |
| Changed output format expectations | **Yes** |
| Renamed skill (`name:` field) | **Yes** |
| Added new examples or subsections | No |
| Improved wording for clarity | No |
| Changed `description:` trigger text | No |
| Added reference files | No |

---

## 4. Metadata Storage — Recommendation: YAML frontmatter (Option A)

**Proposed extended frontmatter:**
```yaml
---
name: <string>                # required
description: <string>         # required
allowed-tools: <string>       # optional (required for beta+)
version: "<MAJOR.MINOR>"      # required (default "1.0")
status: draft|alpha|beta|reviewed|stable  # required (default "draft")
---
```

**Rationale for frontmatter over companion file or registry:**
- Single source of truth — the skill file IS the skill
- No sync problem — no second file to forget
- Existing infrastructure — all 85 skills already have valid YAML frontmatter

---

## 5. CLI Integration

### `aibox skill list` — add VERSION and MATURITY columns

```
  SKILL                  PACKAGE         VERSION  MATURITY   STATUS
  agent-management       core            1.0      stable     active
  code-review            core            1.2      reviewed   active
  data-science           analytics       1.0      alpha      available
```

### `aibox skill check` — automated quality check suite

```
$ aibox skill check code-review

  code-review (version 1.2, status: reviewed)
  [PASS] Valid frontmatter
  [PASS] Has "When to Use" section
  [PASS] Has "Instructions" section
  [PASS] Minimum length (41 lines)
  [PASS] Has allowed-tools
  [PASS] Has code examples
  [PASS] Three-level structure
  [PASS] Trigger description quality
  [PASS] Length under 250 lines

  Result: eligible for status 'beta' or higher
```

### `aibox sync` warnings

```
  warning: skill 'my-new-skill' has maturity 'draft' — it may be incomplete
```

### Automated Check Suite

| # | Check | Gate |
|---|-------|------|
| 1 | Frontmatter valid | draft |
| 2 | Has "When to Use" section | alpha |
| 3 | Has "Instructions" section | alpha |
| 4 | Minimum length (25 lines, excluding frontmatter) | alpha |
| 5 | Has allowed-tools | beta |
| 6 | Has code examples | beta |
| 7 | Three-level structure | beta |
| 8 | Trigger description quality | beta |
| 9 | Not too long (under 250 lines) | beta (warning) |

---

## 6. Migration Plan

### Phase 1: Add Metadata Fields (Low Effort)
1. Script `version: "1.0"` + `status: draft` into all 85 SKILL.md frontmatter blocks.
2. Run automated check suite to determine eligible status for each skill.
3. Promote skills to their earned status: ~39 skills with code examples and `allowed-tools` can go to `beta`.
4. Human-review core package skills to promote to `reviewed`.

### Phase 2: CLI Support
1. Add `SkillMeta` struct and frontmatter parser to Rust CLI.
2. Update `cmd_skill_list` to show version and maturity columns.
3. Add `cmd_skill_check` subcommand.
4. Add sync warning for draft/alpha skills.

### Phase 3: Ongoing Governance
1. PRs modifying skills must update version.
2. Add `aibox skill check --all` to pre-release checklist.

---

## 7. Summary of Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Version scheme | `MAJOR.MINOR` (string in YAML) | Simpler than full semver; sufficient for markdown |
| Status lifecycle | draft/alpha/beta/reviewed/stable | 5 levels map to natural quality progression |
| Metadata storage | YAML frontmatter (Option A) | Single source of truth; minimal change |
| CLI display | Add MATURITY column to `skill list` | Orthogonal to existing STATUS column |
| Initial migration | Script version+status, auto-determine, human review core | Mechanical first pass |
