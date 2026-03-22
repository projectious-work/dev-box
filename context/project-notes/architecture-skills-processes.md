# Architecture: Skills + Processes + Context

Session 2026-03-22. Documents the design conversation about the relationship between CLAUDE.md, context/, and skills.

## Core Insight

Separate WHAT (processes) from HOW (skills). Context declares requirements and stores artifacts. Skills provide the executable implementation.

## Layered Model

```
Layer 1: CLAUDE.md (thin pointer)
  "Read context/ for project rules. Follow declared processes. Use installed skills."

Layer 2: context/ — process declarations (meta-process, thin)
  "There SHALL be backlog management."
  "There SHALL be decision tracking."
  "There SHALL be release management."
  NOT: how to format a backlog entry, how to write an ADR.

Layer 3: skills/ — executable implementation (SKILL.md standard)
  backlog-context/SKILL.md  → manages BACKLOG.md in context/
  backlog-github/SKILL.md   → manages GitHub Issues instead
  decisions-adr/SKILL.md    → ADR format in DECISIONS.md
  release-semver/SKILL.md   → semver release process
  User picks which flavor.

Layer 4: context/ — artifacts (outputs of skills)
  BACKLOG.md, DECISIONS.md, STANDUPS.md, PROGRESS.md, etc.
  These are the actual project state, written by skills.
```

## What Changes from Today

Today (v0.4.x): Process presets (minimal/managed/research/product) bake both the "what" AND "how" into context files. BACKLOG.md template includes formatting conventions, DECISIONS.md includes ADR format, etc.

New model: Process presets declare WHICH processes exist. Skills handle HOW they're executed. Context stores the artifacts.

### Process preset files become thinner:

**Before (today):**
```markdown
# Backlog
Prioritized work items in format:
- [ ] **Title** — Description (#issue-ref)
## Next Up
...
```

**After (with skills):**
```markdown
# Process Configuration

## Required Processes
- backlog-management (skill: backlog-context OR backlog-github)
- decision-tracking (skill: decisions-adr)
- release-management (skill: release-semver)

## Active Skills
See .claude/skills/ for installed skills.
```

The actual BACKLOG.md still exists in context/ — but it's created and maintained by the skill, not by the process template.

## Skill Flavors

Skills come in flavors that implement the same process differently:

| Process | Flavor A (context-based) | Flavor B (tool-integrated) |
|---------|-------------------------|---------------------------|
| Backlog | backlog-context (BACKLOG.md) | backlog-github (GitHub Issues) |
| Decisions | decisions-adr (DECISIONS.md) | decisions-github (GitHub Discussions) |
| Standup | standup-context (STANDUPS.md) | standup-slack (Slack channel) |
| Release | release-semver (git tags) | release-github (GitHub Releases) |
| Code review | review-context (REVIEWS.md) | review-pr (GitHub PRs) |

User chooses flavors. dev-box ensures consistency between declared processes and installed skills.

## dev-box Responsibilities

1. **Scaffold process declarations** — thin context files saying "these processes exist"
2. **Provide curated skill library** — vetted, secure, tested skills
3. **Install skills** — `dev-box skill install backlog-context`
4. **Consistency checking** — `dev-box doctor` verifies declared processes have matching skills
5. **Migration support** — when switching skill flavors, generate migration artifacts

## What Stays in Context

- Process declarations (thin: "there shall be X")
- Artifacts (BACKLOG.md, DECISIONS.md — written by skills)
- Project-specific info (OWNER.md, work-instructions/)
- Shared cross-environment files (context/shared/)

## What Moves to Skills

- Formatting conventions (how a backlog entry looks)
- Workflow instructions (how to triage, prioritize, close)
- Tool integration (where to create issues, how to tag)
- Quality criteria (what makes a "good" decision record)

## Consistency Model

dev-box CLI ensures consistency:
1. Process preset declares required processes
2. Each process maps to one or more skill categories
3. `dev-box doctor` checks that required skills are installed
4. If missing, suggests: `dev-box skill install <skill-name>`
5. Migration: switching from backlog-context to backlog-github generates migration artifacts for the project's AI agent

## Implications for dev-box.toml

```toml
[dev-box]
version = "0.5.0"
image = "python"
process = "product"

[skills]
# Which skill flavors to use for each process
backlog = "backlog-context"       # or "backlog-github"
decisions = "decisions-adr"
release = "release-semver"
code-review = "review-pr"

# Additional skills
additional = ["python-best-practices", "docker-review"]
```

## Relationship to kaits

- dev-box: provides infrastructure + curated skills + meta-processes
- kaits: provides agent orchestration + XP tracking + governance
- SKILL.md is the shared contract
- dev-box skills work standalone with Claude Code; kaits adds team coordination

## Security

- Only skills from trusted sources (dev-box curated library, or user-verified)
- Each skill has a SKILL.md with declared tool permissions (allowed-tools field)
- External marketplaces (ClawHub) are user responsibility — dev-box does not manage them
- `dev-box doctor` could check skill integrity (hash verification)
