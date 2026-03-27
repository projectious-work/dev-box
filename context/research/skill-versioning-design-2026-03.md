# Skill Versioning and Status Metadata Design

**Date:** 2026-03-26
**Status:** Draft

---

## 1. Problem Statement

The aibox skill library contains 85 curated SKILL.md files, each with YAML
frontmatter limited to `name`, `description`, and optionally `allowed-tools`.
There is no mechanism to track:

- **Version**: Which iteration of a skill is deployed. When a skill undergoes a
  breaking change (e.g., restructured instructions, changed tool requirements),
  users have no way to know.
- **Maturity status**: Whether a skill is experimental, under review, or
  production-ready. The quality audit (BACK-026) found a two-tier split: 45%
  "deep" skills with examples and tool declarations, 55% "thin" skills lacking
  depth. But this quality information lives only in a research report, not in the
  skills themselves.
- **Review provenance**: Who reviewed a skill and when. No audit trail exists.

This matters for three audiences:

1. **End users** choosing skills for their project need to know if a skill is
   battle-tested or freshly written.
2. **The CLI** (`aibox sync`) deploys skills into devcontainers. It should be
   able to warn when deploying immature skills.
3. **Maintainers** need to track which skills still need quality review work
   per BACK-026 criteria.

---

## 2. Status Lifecycle

### 2.1 Status Definitions

| Status       | Meaning | Entry Criteria |
|-------------|---------|----------------|
| `draft`     | Newly written, not yet reviewed. May be incomplete or structurally non-compliant. | Exists as a valid SKILL.md with `name` and `description`. |
| `alpha`     | Functionally complete but not yet validated against quality standards. | Has all required sections: "When to Use", "Instructions". Minimum 25 lines. |
| `beta`      | Passes automated quality checks. Ready for human review. | Passes all automated checks (section 6). Has `allowed-tools` declared. Has at least one code example. |
| `reviewed`  | Human-reviewed and approved. Meets the full quality bar. | Reviewed by project owner or designated reviewer. Follows three-level structure. Trigger description optimized per Anthropic guidance (slightly pushy). |
| `stable`    | Production-ready. Breaking changes require a version bump. | Has been `reviewed` for at least one release cycle without changes. |

### 2.2 Transitions

```
draft --> alpha --> beta --> reviewed --> stable
  ^                           |
  |                           v
  +-------- (breaking change triggers reset to alpha)
```

- **Forward transitions** require meeting the entry criteria of the target status.
- **Backward transitions** happen automatically when a breaking change is made
  (see section 3.2). Non-breaking changes within `stable` do not reset status.
- A `reviewed` skill that undergoes a breaking change resets to `alpha` (not
  `draft`), because the structural completeness criteria are presumably still met.

### 2.3 Who Reviews

- **Automated checks** gate the `draft` to `beta` progression. No human needed.
- **Human review** gates `beta` to `reviewed`. The project owner (see
  `context/OWNER.md`) is the reviewer. In a future community model, designated
  skill maintainers could fill this role.
- There is no committee or multi-person review. One competent reviewer is
  sufficient for a library of this size.

---

## 3. Version Semantics

### 3.1 Versioning Scheme

**Recommendation: simplified semver (`MAJOR.MINOR`).**

Full three-part semver (MAJOR.MINOR.PATCH) is excessive for markdown documents.
Skills are not libraries with API contracts; the distinction between a minor fix
and a patch is not meaningful. Two-part versioning captures the essential
information:

- **MAJOR** bump: breaking change (see 3.2).
- **MINOR** bump: non-breaking improvement (added examples, clarified wording,
  new subsection, fixed typos).

All skills start at `1.0`. The first published version is `1.0`, not `0.1`.
Pre-1.0 versioning adds confusion without value for curated content.

### 3.2 What Constitutes a Breaking Change

A "breaking change" for a skill is one that could cause an agent using the skill
to behave differently in ways that affect the user's workflow. Specifically:

| Change Type | Breaking? | Rationale |
|------------|-----------|-----------|
| Changed `allowed-tools` (added or removed tools) | **Yes** | Agent capabilities change. A skill that previously could not write files now can, or vice versa. |
| Restructured instructions that change agent behavior | **Yes** | Agent will produce different outputs for the same inputs. |
| Changed output format expectations | **Yes** | Downstream consumers may break. |
| Renamed skill (`name:` field) | **Yes** | References in `aibox.toml` break. |
| Added new examples or subsections | No | Additive, backward-compatible. |
| Improved wording for clarity | No | Same intent, better expression. |
| Added/changed `description:` trigger text | No | Changes when skill activates, but does not change what it does. |
| Added reference files in `references/` | No | Additive supporting material. |

### 3.3 Version Display Format

In frontmatter: `version: "2.1"` (string, not number, to avoid YAML float
parsing issues with values like `1.0`).

In CLI output: displayed as-is. No `v` prefix needed.

---

## 4. Metadata Storage

### 4.1 Options Evaluated

#### Option A: YAML frontmatter in SKILL.md

Add `version` and `status` directly to the existing frontmatter block:

```yaml
---
name: code-review
description: Guides structured code review...
allowed-tools: Bash Read Write
version: "1.2"
status: reviewed
---
```

**Pros:**
- Single source of truth. No file synchronization issues.
- The CLI already reads SKILL.md content (via `context::skill_content()`). Adding
  frontmatter parsing is straightforward.
- Discoverable. A human reading the file immediately sees its status.
- Compatible with the existing 100% frontmatter compliance (all 85 skills have
  valid YAML frontmatter).

**Cons:**
- Changing metadata means changing the skill file itself, which could trigger
  unnecessary diffs in version control.
- Extended metadata (reviewer name, review date, changelog) would bloat the
  frontmatter.

#### Option B: Companion metadata file

Place a `SKILL.meta.yaml` alongside each `SKILL.md`:

```
templates/skills/code-review/
  SKILL.md
  SKILL.meta.yaml
```

**Pros:**
- Clean separation of content and metadata.
- Metadata changes do not touch the skill content.
- Can grow to include rich metadata (changelog, reviewer, timestamps) without
  affecting the skill file.

**Cons:**
- 85 new files to create and keep in sync.
- Easy to forget updating the companion file when editing SKILL.md.
- CLI must read two files per skill.
- Skill directories currently contain only SKILL.md and optionally `references/`.
  Adding another file type increases cognitive load.

#### Option C: Central registry file

A single `templates/skills/registry.yaml` mapping all skills:

```yaml
skills:
  code-review:
    version: "1.2"
    status: reviewed
  debugging:
    version: "1.0"
    status: beta
```

**Pros:**
- One file to query for all skill metadata. Easy to build CLI tables.
- No changes to individual SKILL.md files.
- Easy to see the full picture at a glance.

**Cons:**
- Merge conflicts when multiple skills are updated simultaneously.
- Disconnected from the skill files -- easy to become stale.
- Loses the "single source of truth" property: you must check two places to
  fully understand a skill.
- Rust CLI would need a new parser for this registry format.

### 4.2 Recommendation: Option A (frontmatter) with an optional registry cache

**Option A is the clear winner.** Rationale:

1. **Existing infrastructure**: All 85 skills already have valid YAML
   frontmatter. The CLI already reads skill content. Adding two fields is the
   minimal change.
2. **Single source of truth**: The skill file IS the skill. Its metadata should
   live with it.
3. **No sync problem**: There is no second file to forget.
4. **The diff concern is minor**: Metadata changes are infrequent (status
   promotion, version bump). These are meaningful changes that SHOULD appear in
   version control.

For the CLI's `skill list` command, which needs to display version and status for
all 85 skills, the frontmatter is parsed at query time. This is fast enough --
reading 85 small YAML headers is negligible. If performance ever becomes a
concern, a generated cache file (`.skill-registry.cache`) can be derived from
the frontmatter, but this is not needed now.

### 4.3 Extended Frontmatter Schema

Current (minimal):
```yaml
---
name: <string>           # required
description: <string>    # required
allowed-tools: <string>  # optional
---
```

Proposed (extended):
```yaml
---
name: <string>                # required
description: <string>         # required
allowed-tools: <string>       # optional (but should become required for beta+)
version: "<MAJOR.MINOR>"      # required (default "1.0" for existing skills)
status: draft|alpha|beta|reviewed|stable  # required (default "draft")
---
```

Future fields that could be added later without breaking anything:
```yaml
reviewed-by: <string>         # who reviewed (for status >= reviewed)
reviewed-date: <YYYY-MM-DD>   # when reviewed
since: "<aibox-version>"      # which aibox release introduced this skill
tags: [<string>, ...]         # for future categorization/search
```

These future fields are explicitly NOT part of the initial implementation to keep
scope tight.

---

## 5. CLI Integration

### 5.1 `aibox skill list`

Current output format:
```
  SKILL                  PACKAGE         STATUS
  agent-management       core            active
  code-review            core            available
```

Proposed output format -- add VERSION and MATURITY columns:
```
  SKILL                  PACKAGE         VERSION  MATURITY   STATUS
  agent-management       core            1.0      stable     active
  code-review            core            1.2      reviewed   active
  data-science           analytics       1.0      alpha      available
```

The existing STATUS column (active/available) indicates deployment state. The
new MATURITY column indicates quality status. These are orthogonal: a `draft`
skill can be `active` if explicitly included, and a `stable` skill can be
`available` but not deployed.

### 5.2 `aibox skill info <name>`

Currently prints the first 20 lines of SKILL.md content. Proposed enhancement --
add a structured header before the content preview:

```
Skill:     code-review
Package:   core
Version:   1.2
Maturity:  reviewed

  ---
  name: code-review
  description: Guides structured code review...
  ...
  ... (15 more lines)
```

### 5.3 `aibox sync` Warnings

When `aibox sync` deploys skills into a devcontainer, it should warn (not block)
when deploying skills with `status: draft` or `status: alpha`:

```
  warning: skill 'my-new-skill' has maturity 'draft' — it may be incomplete
```

This is a warning only. Users who explicitly include a draft skill via
`[skills].include` have made a conscious choice. The warning ensures they are
informed, not blocked.

### 5.4 Frontmatter Parsing

The CLI currently reads skill content as raw text (`context::skill_content()`).
To extract version and status, add a lightweight YAML frontmatter parser:

```rust
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub allowed_tools: Option<String>,
    pub version: String,       // defaults to "1.0"
    pub status: SkillStatus,   // defaults to Draft
}

pub enum SkillStatus {
    Draft,
    Alpha,
    Beta,
    Reviewed,
    Stable,
}
```

The parser strips the `---` delimiters, parses the YAML block, and falls back
to defaults for missing fields. This ensures backward compatibility with skills
that have not yet been annotated.

---

## 6. Automated Quality Checks

### 6.1 Check Suite

Aligned with the BACK-026 quality criteria and the skills quality audit findings,
the following checks can be automated:

| # | Check | Validates | Gate |
|---|-------|-----------|------|
| 1 | **Frontmatter valid** | Has `name:` and `description:` in YAML frontmatter | draft |
| 2 | **Has "When to Use" section** | Contains a `## When to Use` heading | alpha |
| 3 | **Has "Instructions" section** | Contains a `## Instructions` heading | alpha |
| 4 | **Minimum length** | At least 25 lines (excluding frontmatter) | alpha |
| 5 | **Has allowed-tools** | `allowed-tools:` declared in frontmatter | beta |
| 6 | **Has code examples** | At least one fenced code block (triple backtick) or a concrete example section | beta |
| 7 | **Three-level structure** | Has at least one `###` subsection under Instructions | beta |
| 8 | **Trigger description quality** | `description:` field contains "Use when" or similar activation language | beta |
| 9 | **Not too long** | Under 250 lines | beta (warning, not blocking) |

### 6.2 Implementation: `aibox skill check`

New subcommand that runs the check suite:

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

```
$ aibox skill check --all

  85 skills checked:
    39 eligible for beta (pass all automated checks)
    29 eligible for alpha (pass structural checks only)
    17 draft-level (missing basic structure)
```

### 6.3 Integration with `aibox doctor`

The existing `aibox doctor` command checks project health. Add a skill quality
summary:

```
  [PASS] Skills: 42 active, 38 stable, 3 beta, 1 alpha
  [WARN] Skills: 1 active skill has maturity 'draft' (my-experimental-skill)
```

---

## 7. Migration Plan

### 7.1 Phase 1: Add Metadata Fields (Low Effort)

1. Add `version: "1.0"` and `status: draft` to all 85 SKILL.md frontmatter
   blocks. This is a mechanical change (scriptable).
2. Run the automated check suite to determine the actual eligible status for
   each skill.
3. Promote skills to their earned status based on check results:
   - 39 skills with code examples and `allowed-tools` can be promoted to `beta`.
   - Remaining skills stay at `alpha` or `draft` based on structural compliance.
4. Human-review the top-priority skills (core package) to promote them to
   `reviewed`.

Estimated effort: 1-2 hours for the mechanical changes, half a day for the
initial human review of core skills.

### 7.2 Phase 2: CLI Support

1. Add `SkillMeta` struct and frontmatter parser to the Rust CLI.
2. Update `cmd_skill_list` to show version and maturity columns.
3. Update `cmd_skill_info` to show structured metadata.
4. Add `cmd_skill_check` subcommand.
5. Add sync warning for draft/alpha skills.

Estimated effort: 1-2 days of Rust implementation and testing.

### 7.3 Phase 3: Ongoing Governance

1. Establish the convention that PRs modifying skills must update version
   (if applicable) and that status cannot be self-promoted past `beta`.
2. Add `aibox skill check --all` to the local pre-release checklist
   (no CI -- all builds are local per project conventions).
3. As the skill library grows beyond curated content (BACK-024: external skill
   installation), the status system becomes essential for trust signaling.

---

## 8. Summary of Recommendations

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Version scheme | `MAJOR.MINOR` (string in YAML) | Simpler than full semver; sufficient for markdown content |
| Status lifecycle | `draft` / `alpha` / `beta` / `reviewed` / `stable` | Five levels map to natural quality progression; automated checks gate the first three |
| Metadata storage | YAML frontmatter in SKILL.md (Option A) | Single source of truth; no sync issues; minimal change to existing infrastructure |
| CLI display | Add MATURITY column to `skill list`; structured header in `skill info` | Orthogonal to existing active/available STATUS |
| Automated checks | 9-check suite aligned with BACK-026 criteria | Gates `draft` through `beta`; human review gates `reviewed` and `stable` |
| Initial migration | Script `version: "1.0"` + auto-determined status into all 85 skills | Mechanical first pass, then human review for core package |
