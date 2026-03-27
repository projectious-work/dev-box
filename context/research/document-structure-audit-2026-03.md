# Document Structure Audit: Three-Level Rule Compliance

**Date:** 2026-03-26
**Backlog ref:** BACK-027
**Purpose:** Audit all context documents and process templates for compliance with the three-level document structure rule (intro, overview, details) that optimizes documents for AI agent consumption.

---

## The Three-Level Rule

1. **Level 1 (Intro):** 1-3 sentence summary at the top -- enough for an agent to decide if this document is relevant.
2. **Level 2 (Overview):** Key concepts, structure, and relationships -- enough to understand without reading details.
3. **Level 3 (Details):** Full reference material, examples, edge cases.

The gold standard is the SKILL.md format: frontmatter summary, "When to Use" overview, then detailed instructions and references.

---

## Summary

| Category | Documents Audited | Compliant | Partial | Non-Compliant |
|----------|:-:|:-:|:-:|:-:|
| Project context files | 6 | 2 | 3 | 1 |
| Work instructions | 8 | 3 | 3 | 2 |
| Process templates (product) | 8 | 0 | 3 | 5 |
| Process templates (managed) | 5 | 0 | 2 | 3 |
| Process templates (research) | 3 | 0 | 1 | 2 |
| CLAUDE.md templates | 4 | 0 | 2 | 2 |
| **Totals** | **34** | **5** | **14** | **15** |

**Overall compliance: 15% fully compliant, 41% partially compliant, 44% non-compliant.**

---

## Good Examples (Models to Follow)

These documents demonstrate the three-level rule well and should be used as references when restructuring others.

### 1. SKILL.md (e.g., `templates/skills/api-design/SKILL.md`) -- GOLD STANDARD

- **Level 1:** YAML frontmatter with `description` field provides a one-line summary an agent can evaluate without reading the body.
- **Level 2:** "When to Use" section lists scenarios, giving structural overview.
- **Level 3:** Detailed instructions with tables, examples, and references to external files.

### 2. `context/work-instructions/GENERAL.md` -- COMPLIANT

- **Level 1:** "Machine-managed by aibox. These rules apply to all AI agents working in this repo." -- immediately tells the agent scope and relevance.
- **Level 2:** Bold architectural distinction ("We are in a dev-container building dev-containers") front-loads the most critical concept.
- **Level 3:** Detailed sections (Communication, Code Quality, Git Workflow, Known Issues) provide reference depth.

### 3. `context/work-instructions/ARCHITECTURE.md` -- COMPLIANT

- **Level 1:** "Session 2026-03-22. Documents the design conversation about the relationship between CLAUDE.md, context/, and skills." -- clear scope statement.
- **Level 2:** "Core Insight" section with one-line thesis, then a layered model diagram.
- **Level 3:** Detailed breakdown of what changes, with before/after comparisons.

### 4. `context/work-instructions/SCREENCASTS.md` -- COMPLIANT

- **Level 1:** Heading is descriptive.
- **Level 2:** "Overview" paragraph in 2 sentences tells you purpose (docs visuals + visual smoke tests) and approach (asciinema).
- **Level 3:** Architecture diagram, tools table, file layout, script documentation.

### 5. `context/research/competitive-landscape-2026-03.md` -- COMPLIANT

- **Level 1:** Title + dense opening paragraph summarizes scope, date, four research streams, and outputs.
- **Level 2:** Sources table provides orientation before diving in.
- **Level 3:** Detailed per-stream analysis with subsections.

---

## Detailed Per-Document Audit

### A. Project Context Files (`context/`)

| Document | L1 Intro | L2 Overview | L3 Details | Verdict | Issues |
|----------|:--------:|:-----------:|:----------:|---------|--------|
| BACKLOG.md | Yes (2 sentences) | Yes (format table, status values) | Yes (item table) | Partial | Intro is adequate but generic. No sentence explaining what kinds of items are tracked or how an agent should use this file. |
| DECISIONS.md | Yes (1 sentence) | No | Yes (entries) | Partial | Missing overview of decision categories or how many decisions exist. An agent must read all entries to understand scope. |
| PROJECTS.md | Yes (1 sentence) | No | Yes (table) | Partial | Same issue as DECISIONS. No overview of what kinds of projects exist. |
| PRD.md | No | Partial (sections serve as overview) | Yes | Non-Compliant | Jumps straight to "## Vision" with no intro paragraph. An agent cannot determine relevance without reading the Vision section. Needs 1-2 sentences above the first heading. |
| OWNER.md | Yes (name, role, contact) | Yes (structured profile) | Yes (preferences, context) | Compliant | The bullet-list profile format naturally satisfies all three levels. Short and scannable. |
| AIBOX.md | Yes (blockquote + quick reference) | Yes (session protocol, safety rules) | Yes (context layout, full details) | Compliant | Well-structured. Quick Reference section is excellent L2 material. |

### B. Work Instructions (`context/work-instructions/`)

| Document | L1 Intro | L2 Overview | L3 Details | Verdict | Issues |
|----------|:--------:|:-----------:|:----------:|---------|--------|
| GENERAL.md | Yes | Yes | Yes | Compliant | See "Good Examples" above. |
| DEVELOPMENT.md | No | Partial (project structure) | Yes | Non-Compliant | No intro paragraph. Starts with "## Project Structure" immediately. An agent reading this cannot tell it contains build commands, test architecture, and config spec without scanning all headings. |
| ARCHITECTURE.md | Yes | Yes | Yes | Compliant | See "Good Examples" above. |
| TEAM.md | No | Partial (agent roles) | Yes | Non-Compliant | No intro. Starts with "## Agent Strategy" -- an agent cannot determine this file is about parallelizing work across multiple agents without reading the section. |
| RELEASE-PROCESS.md | Yes (1 sentence) | No | Yes (detailed phases) | Partial | "When asked to release version X.Y.Z, follow ALL steps in order." is a good action-oriented intro, but there is no overview of the phases (0-4) before diving into Phase 0 details. An agent cannot estimate effort or scope. |
| DOCKERFILE-PRACTICES.md | Yes (1 sentence) | Yes (section headings serve as overview) | Yes | Compliant | "Reference for reviewing aibox image Dockerfiles. Apply during every image change." is concise and tells the agent exactly when to use this document. |
| PROCESS-ARCHITECTURE.md | Partial (metadata block) | No | Yes | Partial | Has a metadata block (Status, Date, Decision refs, Author) which is useful but not a summary. Dives into SAFe 6.0 details immediately without summarizing what this research covers or why it matters. |
| SCREENCASTS.md | Yes | Yes | Yes | Compliant | See "Good Examples" above. |

### C. Process Templates -- Product (`templates/product/`)

| Document | L1 Intro | L2 Overview | L3 Details | Verdict | Issues |
|----------|:--------:|:-----------:|:----------:|---------|--------|
| BACKLOG.md | Yes | Yes | Minimal (empty) | Partial | Template. Same structure as project BACKLOG.md. Adequate for a template. |
| DECISIONS.md | Yes | Yes | Minimal (empty) | Partial | Template. Adequate. |
| PROJECTS.md | Yes | Yes | Minimal (empty) | Partial | Template. Adequate. |
| PRD.md | No | No | Structure only | Non-Compliant | Empty template with no intro. Jumps to "## Vision" placeholder. Should include a 1-2 sentence intro explaining what a PRD is and how agents should use it. |
| STANDUPS.md | Yes (2 sentences) | Yes (format template) | Minimal | Compliant-for-template | Acceptable for a template file. |
| CLAUDE.md.template | No | No | Partial | Non-Compliant | "This file provides guidance to Claude Code" is too generic to count as an intro. No overview of what sections exist or how to use them. This is the most critical template since every derived project gets it. |
| work-instructions/GENERAL.md | Yes (2 sentences) | No | Yes | Partial | Has intro but no overview before diving into Communication, Code Quality, etc. |
| work-instructions/DEVELOPMENT.md | Yes (1 sentence) | No | Structure only | Non-Compliant | "Development-specific rules for AI agents" is a good intro, but the rest is empty placeholders with no overview. |
| work-instructions/TEAM.md | Yes (1 sentence) | No | Partial | Non-Compliant | Same issue -- intro exists but no overview of what collaboration patterns are covered. |

### D. Process Templates -- Managed (`templates/managed/`)

| Document | L1 Intro | L2 Overview | L3 Details | Verdict | Issues |
|----------|:--------:|:-----------:|:----------:|---------|--------|
| BACKLOG.md | Yes | Yes | Minimal | Partial | Same as product template. |
| DECISIONS.md | Yes | Yes | Minimal | Partial | Same as product template. |
| STANDUPS.md | Yes | Yes | Minimal | Compliant-for-template | Same as product template. |
| CLAUDE.md.template | No | No | Partial | Non-Compliant | Same issue as product CLAUDE.md.template. |
| work-instructions/GENERAL.md | Yes | No | Yes | Partial | Identical to product version. |

### E. Process Templates -- Research (`templates/research/`)

| Document | L1 Intro | L2 Overview | L3 Details | Verdict | Issues |
|----------|:--------:|:-----------:|:----------:|---------|--------|
| PROGRESS.md | Yes (1 sentence) | Partial (table structure) | Minimal | Partial | "Track completion status across project sections/chapters." is a good intro for a template. |
| research-note.md | No | Yes (YAML frontmatter) | Yes (sections) | Non-Compliant | Template has good structure (Objective, Method, Findings, Conclusions) but no intro paragraph explaining what a research note is and when to create one. |
| CLAUDE.md.template | No | No | Partial | Non-Compliant | Same issue as other CLAUDE.md templates. |

---

## Priority Restructuring List

Ranked by impact (how often agents encounter the document times how much the missing structure costs).

### Critical (high-traffic documents)

1. **All CLAUDE.md.template files** (4 files) -- Every derived project inherits this. Should model the three-level rule since it is the first file agents read. Add a real intro paragraph and a brief overview of what the project does.

2. **`context/work-instructions/DEVELOPMENT.md`** -- Read at session start. Missing intro means agents must scan the entire file to find build commands. Add: "Build, test, and code structure reference for the aibox Rust CLI. Key commands: `cargo build`, `cargo test`, `cargo clippy -- -D warnings`."

3. **`context/PRD.md`** -- Product requirements document. Add 1-2 sentences above "## Vision" summarizing what aibox is and linking to this document's purpose.

### High (referenced frequently)

4. **`context/work-instructions/TEAM.md`** -- Add intro: "Defines agent parallelization strategy for this repository -- three agent scopes (Image Builder, CLI Developer, Documentation) that can work concurrently."

5. **`context/work-instructions/RELEASE-PROCESS.md`** -- Add a phase overview (numbered list of phases with one-line descriptions) between the intro sentence and Phase 0 details.

6. **`context/DECISIONS.md`** -- Add a brief overview count and category summary so agents know the decision landscape without reading every entry.

### Medium (templates affect all derived projects)

7. **`templates/product/PRD.md`** -- Add intro paragraph to the template.

8. **`templates/product/work-instructions/DEVELOPMENT.md`** -- Add a brief "what this file covers" intro.

9. **`templates/product/work-instructions/TEAM.md`** -- Add intro.

10. **`templates/research/research-note.md`** -- Add intro explaining when to create a research note.

### Low (less frequently accessed)

11. **`context/work-instructions/PROCESS-ARCHITECTURE.md`** -- Add a summary paragraph after the metadata block.

12. **`context/PROJECTS.md`** -- Add a brief overview of active project count and themes.

---

## Recommendations Per Non-Compliant Document

### CLAUDE.md.template (all 4 variants)

**Current:** "This file provides guidance to Claude Code when working with this repository."
**Recommended intro:**
```
# CLAUDE.md -- {{project_name}}

[1-3 sentence description of what this project is, what it does, and its primary
technology stack. This is the most important paragraph in the repository -- it tells
AI agents whether this project is relevant to their current task.]
```
**Recommended overview:** Add a "Quick Reference" section (like AIBOX.md has) with key build/test commands before the detailed sections.

### context/work-instructions/DEVELOPMENT.md

**Add before "## Project Structure":**
```
Build, test, and structural reference for the aibox Rust CLI and its supporting
artifacts (images, templates, docs). This is the primary development reference
-- start here for build commands, test architecture, and project layout.
```

### context/PRD.md

**Add before "## Vision":**
```
Product requirements for aibox -- a CLI tool for reproducible, containerized
development environments with built-in AI context structure. This document defines
target users, core requirements, non-goals, and success metrics.
```

### context/work-instructions/TEAM.md

**Add before "## Agent Strategy":**
```
Agent parallelization and collaboration rules for this repository. Defines three
agent scopes (Image Builder, CLI Developer, Documentation) that can work concurrently,
plus release process handoff between agents and the human operator.
```

### context/work-instructions/RELEASE-PROCESS.md

**Add after the intro sentence:**
```
## Phase Overview

| Phase | Owner | Summary |
|-------|-------|---------|
| 0 | Claude | Dependency version check (upstream tools) |
| 1 | Claude | Version bump, tests, clippy, audit, Linux binaries |
| 2 | Human | macOS cross-compile, image builds, GitHub release |
| 3 | Human | Image push to GHCR, docs deploy |
```

### templates/research/research-note.md

**Add after frontmatter, before "# [Research Topic]":**
```
<!-- This template is for structured research investigations. Create a new
research note when exploring a question that may lead to a project decision.
Each note should be self-contained: state the question, method, findings,
and conclusions. -->
```

---

## Authoring Checklist for Future Documents

Use this checklist when creating or reviewing any document in `context/` or `templates/`.

### Level 1: Intro (required for every document)

- [ ] First 1-3 sentences answer: "What is this document about and why would an agent read it?"
- [ ] An agent reading ONLY the intro can decide whether to continue reading
- [ ] No heading before the intro (intro is the first content after the title)
- [ ] For templates: explain what the template is for, not just what it contains

### Level 2: Overview (required for documents longer than 20 lines)

- [ ] Key concepts, structure, or relationships are summarized before details
- [ ] An agent reading intro + overview understands the document's scope without reading details
- [ ] For registries (BACKLOG, PROJECTS): include a count or summary of contents
- [ ] For process documents: include a phase/step overview before detailed phases
- [ ] For reference documents: include a "When to Use" or "Applies to" section

### Level 3: Details (the bulk of the document)

- [ ] Most important details come first within each section
- [ ] Tables, code blocks, and examples are used for scannable reference
- [ ] Cross-references to related documents use relative paths
- [ ] Long sections (50+ lines) have their own internal structure (sub-headings)

### Anti-Patterns to Avoid

- Starting with a heading and no prose (e.g., `# Title\n## First Section`)
- Burying critical information (build commands, key decisions) deep in the document
- Generic intros that apply to any document ("This file provides guidance...")
- Mixing overview and detail levels (jumping between high-level and low-level in the same section)
- Frontloading metadata (dates, refs, authors) without a human-readable summary

---

## Comparison: SKILL.md vs. Context Documents

The SKILL.md format is the gold standard in this project. Here is how its structure maps to the three-level rule, and where context documents fall short:

| Aspect | SKILL.md | Context Documents (typical) |
|--------|----------|---------------------------|
| **L1** | YAML `description` field -- one line, machine-parseable | Title + maybe 1 sentence. Often missing entirely. |
| **L2** | "When to Use" section with bullet list of scenarios | Rarely present. Most documents go straight from title to details. |
| **L3** | "Instructions" with subsections, tables, code, refs | Generally good. The detail level of most documents is fine. |
| **Progressive disclosure** | Agent loads frontmatter first, reads body only if relevant | No equivalent mechanism. Agent must read the full document to assess relevance. |

**Key insight:** The gap is almost entirely at Levels 1 and 2. Most documents have adequate Level 3 content. The fix is additive -- prepend intro paragraphs and overview sections without restructuring the existing detail content.
