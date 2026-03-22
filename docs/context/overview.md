# Context System Overview

The dev-box context system provides structured files that give AI agents the information they need to work effectively on your project.

## The Problem

AI coding agents like Claude operate best when they understand not just the code, but the project's goals, decisions, and current state. Without structure, this information ends up scattered across chat histories, stale comments, and the developer's memory.

`CLAUDE.md` alone is not enough for complex projects. It works well for instructions and preferences, but it does not provide a standard place for decisions, backlog, progress tracking, or team conventions.

## How Context Files Work

dev-box scaffolds a `context/` directory based on your chosen work process flavor. Each file has a defined purpose:

| File | Purpose |
|------|---------|
| `CLAUDE.md` | AI agent instructions and project preferences (lives at project root) |
| `DECISIONS.md` | Architectural decisions with rationale and status |
| `BACKLOG.md` | Prioritized work items |
| `STANDUPS.md` | Session-by-session progress notes |
| `PROJECTS.md` | Multi-project tracking and status |
| `PRD.md` | Product requirements document |
| `PROGRESS.md` | Learning/research progress tracking |

### CLAUDE.md vs context/

`CLAUDE.md` sits at the project root and contains instructions for AI agents: coding style, project conventions, what to avoid. It is read automatically by tools like Claude Code.

The `context/` directory contains structured project knowledge: what has been decided, what needs to be done, what happened in recent sessions. AI agents reference these files to understand project state, but the files are also useful for human developers.

## OWNER.md -- Developer Identity

`OWNER.md` captures the developer's identity and preferences. It is created as a local file in `context/OWNER.md` during `dev-box init`, with fields that help AI agents understand who they are working with:

- **Name** -- how the developer prefers to be addressed
- **Domain expertise** -- areas of knowledge and experience
- **Primary languages** -- programming languages used most often
- **Communication language** -- natural language for responses (e.g., English, German)
- **Timezone** -- for scheduling and availability context
- **Working hours** -- typical availability window
- **Current focus** -- what the developer is currently working on or learning
- **Communication preferences** -- style and conventions for AI interactions

Each project gets its own `OWNER.md`, allowing you to tailor the developer context per project (for example, different "current focus" or "domain expertise" entries for different repositories).

## Work Process Flavors

dev-box provides four process flavors that scale from simple to comprehensive:

| Flavor | Files | Best For |
|--------|-------|----------|
| `minimal` | CLAUDE.md only | Scripts, experiments, small utilities |
| `managed` | DECISIONS, BACKLOG, STANDUPS, work-instructions | Ongoing projects with decisions to track |
| `research` | PROGRESS, research/, experiments/, analysis/ | Learning, documentation, academic work |
| `product` | Everything from managed + PROJECTS, PRD, project-notes, ideas | Full product development |

See [Work Processes](work-processes.md) for detailed documentation of each flavor.

## Version Tracking

The context schema is versioned via `dev-box.toml`:

```toml
[context]
schema_version = "1.0.0"
```

A `.dev-box-version` file in the project root tracks the version that was last applied. When the schema evolves, `dev-box doctor` can detect version mismatches and generate migration artifacts.

See [Migration](migration.md) for details on how upgrades work.

## Relationship to dev-box.toml

The `process` field in `[dev-box]` determines which context files are scaffolded during `dev-box init`:

```toml
[dev-box]
process = "product"
```

Changing this field after initialization does not automatically add or remove files. Use `dev-box doctor` to identify gaps and `dev-box sync` to reconcile.

## Process Templates

dev-box ships four process templates in `context/processes/` that define standard workflows for common development activities:

| Template | Purpose |
|----------|---------|
| `release.md` | Release process steps and checklist |
| `code-review.md` | Code review workflow and standards |
| `feature-development.md` | Feature development lifecycle |
| `bug-fix.md` | Bug investigation and fix workflow |

Process templates declare **WHAT** your project does -- the steps, roles, and definition of done for each workflow. They are intentionally thin. The executable details (how an AI agent should format entries, which tools to use, integration specifics) live in **skills** (see [Skills](../skills.md)).

Process templates are scaffolded for `managed`, `research`, and `product` flavors. The `minimal` flavor does not include process templates.

You can customize process templates freely: edit them, add new ones, or remove ones you do not use.

## SKILL.md Support

Skills complement processes by providing the **HOW** -- executable instructions for AI agents. A skill is a `SKILL.md` file installed at `.claude/skills/<name>/SKILL.md` that tells the AI agent how to perform a specific task.

dev-box bundles three example skills:

| Skill | Description |
|-------|-------------|
| `backlog-context` | Manages BACKLOG.md -- adding, prioritizing, and tracking work items |
| `decisions-adr` | Manages DECISIONS.md using Architecture Decision Record format |
| `standup-context` | Manages STANDUPS.md with session progress notes |

This separation of **WHAT** (processes) from **HOW** (skills) is a core architectural decision (DEC-011). It enables swappable implementations -- for example, you could replace `backlog-context` (which manages a Markdown file) with a `backlog-github` skill that manages GitHub Issues instead, without changing your process declarations.

See [Skills](../skills.md) for full documentation on installing and using skills.

## Design Principles

**Convention over configuration.** File names and locations are standardized so AI agents can find them without special instructions.

**Human-readable first.** All context files are Markdown. They are useful without any tooling.

**Progressive complexity.** Start with `minimal` and upgrade to `managed` or `product` as the project grows.

**No lock-in.** Context files are plain Markdown in a `context/` directory. Stop using dev-box and the files remain useful.
