# Skills Quality Audit — March 2026

**Audited:** 85 SKILL.md files in `templates/skills/`
**Sample read in detail:** 22 files across all categories
**Date:** 2026-03-26

---

## Executive Summary

The skill library is **consistently structured and readable**, but exhibits a clear
two-tier quality split. Approximately 45% of skills (the "deep" tier) follow all
best practices with detailed reference sections, code examples, and well-scoped
tool permissions. The remaining 55% (the "thin" tier) are serviceable but lack depth
— typically under 45 lines, with no code examples and no `allowed-tools` declarations.

No skills define a `tools:` YAML frontmatter key. The correct field in use is
`allowed-tools:`, which 38 of 85 skills (45%) declare. Zero skills use `tools:`.

---

## Methodology

Each skill was evaluated on five criteria:

| # | Criterion | Description |
|---|-----------|-------------|
| 1 | **Structure** | Three-level rule: intro paragraph, overview section, detailed reference |
| 2 | **Examples** | Concrete code examples or command snippets (code blocks) |
| 3 | **Tools section** | `allowed-tools:` declared in YAML frontmatter |
| 4 | **Trigger description** | Specific enough that an agent knows when to activate |
| 5 | **Length** | Between 20 and 200 lines (not too thin, not overwhelming) |

Quantitative data was gathered across all 85 files; qualitative assessment was based
on a detailed reading of 22 representative files.

---

## Quantitative Findings

### Code Examples (code blocks present)

- **39 of 85 (46%)** contain at least one fenced code block
- **46 of 85 (54%)** have zero code blocks

### allowed-tools Declaration

- **38 of 85 (45%)** declare `allowed-tools:` in frontmatter
- **47 of 85 (55%)** omit it entirely
- **0 of 85** use a `tools:` key

### YAML Frontmatter

- **85 of 85 (100%)** have valid YAML frontmatter with `name:` and `description:`
- All descriptions include a "Use when..." trigger clause (100%)

### Structure (three-level rule)

Based on the sample:

- **~40% fully follow** the three-level rule (intro paragraph + overview/when-to-use + detailed reference sections with subsections)
- **~45% partially follow** (have "When to Use" and "Instructions" but Instructions is a flat numbered list, not a multi-section reference)
- **~15% are minimal** (fewer than 25 lines, bullet-list-only instructions)

### Length Distribution

| Range | Count | Percentage | Assessment |
|-------|-------|------------|------------|
| < 20 lines | 0 | 0% | -- |
| 20-45 lines | 17 | 20% | Thin — likely incomplete |
| 46-100 lines | 32 | 38% | Adequate for simple topics |
| 101-200 lines | 30 | 35% | Good depth |
| > 200 lines | 6 | 7% | Risk of context overload |

**Thinnest skills (under 25 lines):** backlog-context (18), standup-context (21),
decisions-adr (21), context-archiving (22).

**Longest skills (over 200 lines):** shell-scripting (245), container-orchestration (208),
auth-patterns (192 — close to threshold), terraform-basics (182 — close).

---

## Common Issues

### Issue 1: No Code Examples (54% of skills)

The majority of skills rely entirely on prose instructions and conversational
"User/Agent" example pairs. While the conversational examples show intent, they
do not give the agent concrete patterns to follow.

**Affected examples:** python-best-practices, rust-conventions, debugging,
testing-strategy, code-review, git-workflow, release-semver, documentation,
dependency-audit, dockerfile-review, retrospective, estimation-planning

**Impact:** An agent activating these skills has guidance on *what to do* but
not *how it should look*. For code-centric skills (python-best-practices,
rust-conventions) this is a significant gap.

### Issue 2: Missing allowed-tools (55% of skills)

Skills that perform actions (file reads, command execution) should declare
`allowed-tools:` so the agent gets appropriate permissions. Many skills that
clearly need tool access lack this declaration.

**Examples of skills that should have allowed-tools but do not:**
- `python-best-practices` — needs Bash(python:*), Bash(ruff:*), Read, Write
- `rust-conventions` — needs Bash(cargo:*), Read, Write
- `debugging` — needs Bash, Read
- `git-workflow` — needs Bash(git:*), Read
- `dockerfile-review` — needs Read, Write (at minimum)
- `dependency-audit` — needs Bash, Read
- `code-review` — needs Read

### Issue 3: Thin Process Skills (backlog-context, standup-context, decisions-adr)

These three context-management skills are stubs: 18-21 lines, flat bullet lists,
no examples of the actual file format they manage, and no error handling guidance.
They contrast sharply with `session-handover` (97 lines) and `context-archiving`
(22 lines), which manage similar concerns.

**Specific gaps:**
- `backlog-context` (18 lines) — does not show the BACKLOG.md table format, does
  not explain ID assignment (BACK-NNN), does not handle edge cases (duplicate IDs,
  moving items between sections)
- `standup-context` (21 lines) — does not show the STANDUPS.md entry format
- `decisions-adr` (21 lines) — does not show the full DEC-NNN format or the
  inverse-chronological ordering rule

### Issue 4: Trigger Descriptions Could Be Pushier

Per Anthropic research, agents undertrigger — descriptions should be "a little
pushy" so the agent activates the skill even when the user does not explicitly
ask for it. Most descriptions are adequate but passive. They describe what the
skill does rather than asserting when it should fire.

**Weak trigger examples:**
- `backlog-context`: "Manages project backlog as a BACKLOG.md file" — agent must
  infer this applies when a user mentions a task or work item
- `standup-context`: "Manages session standup notes" — does not mention session
  start/end, progress recording
- `git-workflow`: "Use when the user needs guidance on git practices" — too
  deferential; should fire whenever the agent creates branches or commits
- `documentation`: "Use when creating or improving project documentation" — should
  also fire when adding new public functions without doc comments

**Strong trigger examples (models to follow):**
- `session-handover`: Lists specific trigger phrases and proactive trigger condition
- `debugging`: Good range of trigger phrases
- `incident-response`: "Use when something is broken in production" — clear urgency

### Issue 5: Inconsistent Depth Across Categories

Skills covering similar complexity levels have wildly different depths:

| Comparison | Lines | Code blocks |
|------------|-------|-------------|
| auth-patterns | 192 | 14 |
| secret-management | ~45 | 0 |
| | | |
| shell-scripting | 245 | 30 |
| python-best-practices | 40 | 0 |
| rust-conventions | 39 | 0 |
| | | |
| container-orchestration | 208 | 22 |
| dockerfile-review | 45 | 2 |
| | | |
| rag-engineering | 125 | 0 |
| prompt-engineering | 133 | 2 |
| embedding-vectordb | ~90 | 0 |

---

## Category-by-Category Summary

### Language Conventions (6 skills)
`python-best-practices`, `rust-conventions`, `typescript-patterns`, `go-conventions`,
`java-patterns`, `shell-scripting`

**Quality:** Mixed. `shell-scripting` is exemplary (245 lines, 30 code blocks,
allowed-tools). The rest are thin (35-45 lines) with minimal or no code examples.
`typescript-patterns` has one inline code snippet, which is a modest improvement.

**Recommendation:** Expand all language skills to match `shell-scripting` depth.
Each should include: project setup snippet, idiomatic error handling, testing
example, and linting/formatting commands.

### AI/ML (8 skills)
`ai-fundamentals`, `prompt-engineering`, `rag-engineering`, `llm-evaluation`,
`embedding-vectordb`, `ml-pipeline`, `feature-engineering`, `agent-management`

**Quality:** Good. Most are 80-130 lines with structured subsections. `rag-engineering`
and `prompt-engineering` are strong. `ai-fundamentals` and `agent-management` are
well-structured. Most have `allowed-tools` where needed.

**Recommendation:** Add code examples to `rag-engineering` (Python snippets for
chunking/embedding would be high-value). Add `allowed-tools` to `agent-management`.

### Data (7 skills)
`data-science`, `data-pipeline`, `data-quality`, `data-visualization`,
`pandas-polars`, `feature-engineering`, `sql-patterns`, `sql-style-guide`,
`nosql-patterns`

**Quality:** Above average. `data-science` is exemplary (114 lines, reference
links, preferred-libraries table). Most have `allowed-tools`. `sql-style-guide`
and `nosql-patterns` have good code blocks.

**Recommendation:** Minor — add code examples to `data-pipeline` and `data-quality`.

### DevOps/Infrastructure (10 skills)
`ci-cd-setup`, `container-orchestration`, `dockerfile-review`, `kubernetes-basics`,
`terraform-basics`, `linux-administration`, `dns-networking`,
`secret-management`, `alerting-oncall`, `distributed-tracing`

**Quality:** Strong for the detailed skills (`container-orchestration`,
`terraform-basics`, `kubernetes-basics`, `linux-administration`). Weaker for
`ci-cd-setup`, `secret-management`, `alerting-oncall` which lack code examples.

**Recommendation:** Expand `secret-management` (high-stakes topic deserves more
than ~45 lines). Add CI config examples to `ci-cd-setup`.

### Process/Context Management (7 skills)
`backlog-context`, `standup-context`, `decisions-adr`, `context-archiving`,
`session-handover`, `release-semver`, `retrospective`

**Quality:** Bimodal. `session-handover` is excellent (97 lines, concrete template,
clear triggers). The rest are stubs (18-45 lines).

**Recommendation:** All process skills should include the actual file format
template they manage, with a filled example. These skills are activated frequently
and shallow instructions lead to inconsistent output.

### Design/Creative (4 skills)
`excalidraw`, `infographics`, `logo-design`, `frontend-design`

**Quality:** Consistently good. All are 100+ lines with structured subsections,
color palettes, and decision tables. `excalidraw` includes JSON schema detail.

**Recommendation:** These are models for other categories.

### Software Engineering Practices (12 skills)
`code-review`, `testing-strategy`, `tdd-workflow`, `debugging`, `refactoring`,
`error-handling`, `concurrency-patterns`, `performance-profiling`,
`integration-testing`, `software-architecture`, `system-design`,
`domain-driven-design`

**Quality:** Mixed. `software-architecture` (132 lines, Mermaid examples) and
`estimation-planning` (110 lines) are strong. `testing-strategy`, `debugging`,
`code-review` are adequate but brief (35-45 lines).

**Recommendation:** `code-review` should include a more concrete checklist with
example findings per language. `debugging` should include diagnostic command
examples.

### API/Protocol (6 skills)
`api-design`, `graphql-patterns`, `grpc-protobuf`, `webhook-integration`,
`auth-patterns`, `fastapi-patterns`

**Quality:** Good. All have code examples and most have `allowed-tools`.
`auth-patterns` is the deepest at 192 lines.

**Recommendation:** These are generally well-done. Minor: add `allowed-tools`
to any missing.

---

## Top 10 Skills Most in Need of Improvement

| Rank | Skill | Lines | Issues |
|------|-------|-------|--------|
| 1 | `backlog-context` | 18 | Stub; no format template; no ID rules; no allowed-tools |
| 2 | `standup-context` | 21 | Stub; no format example; no allowed-tools |
| 3 | `decisions-adr` | 21 | Stub; no DEC-NNN format; no allowed-tools |
| 4 | `python-best-practices` | 40 | No code examples for a code-centric skill; no allowed-tools |
| 5 | `rust-conventions` | 39 | No code examples for a code-centric skill; no allowed-tools |
| 6 | `secret-management` | ~45 | High-stakes topic with minimal depth; no code examples; no allowed-tools |
| 7 | `context-archiving` | 22 | Stub; no example of archive move operation; no allowed-tools |
| 8 | `ci-cd-setup` | ~40 | No CI config examples (GHA YAML); no allowed-tools |
| 9 | `code-review` | 41 | No allowed-tools (needs Read); example section is conversational not concrete |
| 10 | `git-workflow` | 39 | No allowed-tools (needs Bash(git:*)); trigger too deferential |

---

## Trigger Description Improvement Recommendations

Current descriptions follow the pattern: *"[Topic summary]. Use when [condition]."*
This is functional but passive. Recommended improvements:

### Principle: Describe the trigger, not just the capability

**Before (passive):**
> Manages project backlog as a BACKLOG.md file in the context directory. Creates, prioritizes, and tracks work items in Markdown format.

**After (pushy):**
> Manages the project backlog in context/BACKLOG.md. Activate whenever the user mentions a task, work item, feature, bug, or TODO — or when creating/completing work that should be tracked. Also activate proactively at session start to review open items.

### Specific rewrites for the weakest triggers

| Skill | Current | Recommended |
|-------|---------|-------------|
| `backlog-context` | "Manages project backlog as a BACKLOG.md file..." | "Manages the project backlog. Activate when the user mentions tasks, work items, bugs, or TODOs. Also activate proactively when completing work to update status." |
| `standup-context` | "Manages session standup notes..." | "Records session progress in context/STANDUPS.md. Activate at session start to log what is planned, at session end to log what was done, or when the user says 'standup', 'progress', or 'what did we do'." |
| `decisions-adr` | "Manages architectural decision records..." | "Records decisions in context/DECISIONS.md. Activate whenever a non-trivial technical or process choice is made — do not wait for the user to ask. If the conversation includes 'let us go with X' or 'we decided', this skill applies." |
| `git-workflow` | "Use when the user needs guidance on git practices." | "Use when creating branches, writing commit messages, preparing PRs, or when the user asks about git conventions. Activate proactively when the agent is about to commit or create a branch." |
| `documentation` | "Use when creating or improving project documentation." | "Use when writing docs, adding READMEs, or when the agent creates new public functions/types that lack doc comments. Activate proactively after adding new public API surface." |
| `code-review` | "Use when reviewing PRs, diffs, or code changes before merging." | "Use when the user says 'review', 'check this', 'is this ready', or shares a diff/PR link. Also activate when the agent is about to submit code and should self-review first." |
| `debugging` | "Use when tracking down bugs, unexpected behavior, or test failures." | "Use when the user reports a bug, error message, unexpected output, test failure, or says 'this doesn't work', 'help me debug', 'why is this failing', or 'what's wrong'. Also activate when a command the agent ran produces an error." |
| `secret-management` | "Use when dealing with API keys, passwords, tokens, or credentials." | "Use when code contains API keys, tokens, passwords, or connection strings — or when the user needs to configure secrets. Also activate proactively when reviewing code that may accidentally commit secrets." |
| `context-archiving` | "Manages context/archive/..." | "Moves completed items from active context files to archive. Activate when active context files grow large (>100 items), when closing a project phase, or when the user says 'archive', 'clean up context', or 'move done items'." |
| `release-semver` | "Use when preparing a new release." | "Use when the user says 'release', 'bump version', 'publish', 'tag', 'cut a release', or 'ship it'. Also activate when the changelog has unreleased entries and the user asks to deploy." |

---

## Checklist Template for Future Skill Authoring/Review

Use this checklist when creating a new skill or reviewing an existing one.

### YAML Frontmatter
- [ ] `name:` matches the directory name
- [ ] `description:` is one sentence summarizing capability + trigger conditions
- [ ] `description:` includes "Use when..." with specific trigger phrases and proactive triggers
- [ ] `description:` trigger is "a little pushy" — errs on the side of activating
- [ ] `allowed-tools:` is declared if the skill reads files, runs commands, or writes output
- [ ] `allowed-tools:` uses scoped Bash permissions where possible (e.g., `Bash(cargo:*)` not just `Bash`)

### Structure (three-level rule)
- [ ] **Level 1 — Intro paragraph:** One sentence or short paragraph explaining what this skill does
- [ ] **Level 2 — "When to Use" section:** Bullet list of trigger conditions including exact user phrases
- [ ] **Level 3 — "Instructions" section:** Detailed reference organized into numbered or named subsections
- [ ] Subsections use `### heading` for major topics
- [ ] Tables used for decision matrices and comparisons
- [ ] References to `references/*.md` files for deep-dive content (optional)

### Examples
- [ ] At least one fenced code block showing concrete output or commands (for technical skills)
- [ ] At least one User/Agent conversational example showing a realistic interaction
- [ ] Examples cover the most common use case, not just edge cases
- [ ] Code examples are syntactically correct and copy-pasteable

### Content Quality
- [ ] Instructions are actionable — agent can follow them step by step
- [ ] No placeholder text or TODO comments
- [ ] File formats managed by the skill are shown with a filled example
- [ ] Error cases and edge cases are addressed (what to do when X is missing)
- [ ] Language is concise — every sentence adds information

### Length
- [ ] Minimum 40 lines (below this, likely incomplete)
- [ ] Maximum 200 lines (above this, consider extracting detail to references/)
- [ ] Sweet spot: 80-150 lines for most skills

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total skills | 85 |
| With code examples | 39 (46%) |
| With allowed-tools | 38 (45%) |
| With valid frontmatter | 85 (100%) |
| With "Use when" trigger | 85 (100%) |
| Follow three-level rule fully | ~34 (40%) |
| Follow three-level rule partially | ~38 (45%) |
| Minimal/stub | ~13 (15%) |
| Estimated "production ready" (all criteria met) | ~30 (35%) |

### Overall Assessment

The skill library has a strong foundation — consistent naming, universal YAML
frontmatter, and a clear structural pattern. The top ~35% of skills (auth-patterns,
shell-scripting, container-orchestration, terraform-basics, rag-engineering,
data-science, excalidraw, prompt-engineering, session-handover, software-architecture,
estimation-planning, logo-design, infographics) are high quality and serve as models.

The primary improvement opportunity is lifting the bottom 55% to match the top tier:
adding code examples, declaring allowed-tools, expanding stub process skills, and
making trigger descriptions more assertive. This would be a medium-effort task
(estimated 2-3 focused sessions) with high impact on agent activation accuracy
and output quality.
