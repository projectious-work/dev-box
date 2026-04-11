---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-GentleSea-document-structure-audit-three
  created: 2026-04-11
spec:
  title: "Document Structure Audit: Three-Level Rule Compliance"
  type: reference
  state: permanent
  tags: [documents, structure, three-level, audit, compliance, ai-consumption]
  skill: research-with-confidence
  source_file: document-structure-audit-2026-03.md
---

# Document Structure Audit: Three-Level Rule Compliance

**Date:** 2026-03-26
**Task:** BACK-027
**Purpose:** Audit all context documents and process templates for compliance with the three-level document structure rule that optimizes documents for AI agent consumption.

---

## The Three-Level Rule

1. **Level 1 (Intro):** 1-3 sentence summary — enough for an agent to decide if this document is relevant.
2. **Level 2 (Overview):** Key concepts, structure, relationships — enough to understand without reading details.
3. **Level 3 (Details):** Full reference material, examples, edge cases.

The gold standard is the SKILL.md format: frontmatter summary → "When to Use" overview → detailed instructions.

---

## Audit Summary

| Category | Audited | Compliant | Partial | Non-Compliant |
|---|:-:|:-:|:-:|:-:|
| Project context files | 6 | 2 | 3 | 1 |
| Work instructions | 8 | 3 | 3 | 2 |
| Process templates (product) | 8 | 0 | 3 | 5 |
| Process templates (managed) | 5 | 0 | 2 | 3 |
| Process templates (research) | 3 | 0 | 1 | 2 |
| CLAUDE.md templates | 4 | 0 | 2 | 2 |
| **Totals** | **34** | **5** | **14** | **15** |

**Overall compliance: 15% fully compliant, 41% partial, 44% non-compliant.**

---

## Key Insight

**The gap is almost entirely at Levels 1 and 2.** Most documents have adequate Level 3 content. The fix is additive — prepend intro paragraphs and overview sections without restructuring existing detail content.

| Aspect | SKILL.md (gold standard) | Context Documents (typical) |
|--------|---|---|
| **L1** | YAML `description` field — one line, machine-parseable | Title + maybe 1 sentence; often missing entirely |
| **L2** | "When to Use" section | Rarely present; most go straight to details |
| **L3** | Instructions with subsections, tables, code, refs | Generally adequate |

---

## Priority Restructuring List

### Critical (high-traffic)

1. **All CLAUDE.md.template files (4 files)** — Every derived project inherits this. Add real project-specific intro + Quick Reference section with key build/test commands.

2. **`context/work-instructions/DEVELOPMENT.md`** — Read at session start. Missing intro means agents must scan entire file. Add: "Build, test, and code structure reference for the aibox Rust CLI. Key commands: `cargo build`, `cargo test`, `cargo clippy -- -D warnings`."

3. **`context/PRD.md`** — Add 1-2 sentences above "## Vision" summarizing what aibox is.

### High (referenced frequently)

4. **`context/work-instructions/TEAM.md`** — Add intro about agent parallelization strategy (three agent scopes: Image Builder, CLI Developer, Documentation).

5. **`context/work-instructions/RELEASE-PROCESS.md`** — Add Phase Overview table between intro sentence and Phase 0 details.

6. **`context/DECISIONS.md`** — Add brief count and category summary.

### Medium (templates affect all derived projects)

7-10. Product/research template files: PRD.md, DEVELOPMENT.md, TEAM.md, research-note.md

---

## Authoring Checklist for Future Documents

### Level 1 (required)
- [ ] First 1-3 sentences: "What is this document about and why would an agent read it?"
- [ ] No heading before the intro
- [ ] Agent reading ONLY the intro can decide whether to continue

### Level 2 (required for documents > 20 lines)
- [ ] Key concepts summarized before details
- [ ] For registries (BACKLOG, PROJECTS): include count or summary of contents
- [ ] For process documents: include phase/step overview before detailed phases

### Anti-Patterns
- Starting with a heading and no prose
- Generic intros that apply to any document ("This file provides guidance...")
- Burying build commands or key decisions deep in the document
- Frontloading metadata (dates, refs) without a human-readable summary
