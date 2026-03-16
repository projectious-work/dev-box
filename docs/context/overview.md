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

## OWNER.md -- Shared Identity

`OWNER.md` captures the developer's identity and preferences that apply across all projects:

- Name and working style
- Communication preferences
- Common conventions
- Timezone and availability

This file lives outside the project, typically at `~/.config/dev-box/OWNER.md`, and is referenced via `dev-box.toml`:

```toml
[context]
owner = "~/.config/dev-box/OWNER.md"
```

During context scaffolding, the `OWNER.md` path is included in generated `CLAUDE.md` templates so AI agents know where to find it.

!!! tip "One OWNER.md for all projects"
    Updating your preferences in one place propagates to every project. No more duplicating the same instructions across repositories.

## Work Process Flavors

dev-box provides four process flavors that scale from simple to comprehensive:

| Flavor | Files | Best For |
|--------|-------|----------|
| `minimal` | CLAUDE.md only | Scripts, experiments, small utilities |
| `managed` | DECISIONS, BACKLOG, STANDUPS, work-instructions | Ongoing projects with decisions to track |
| `research` | PROGRESS, research/, analysis/ | Learning, documentation, academic work |
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

Changing this field after initialization does not automatically add or remove files. Use `dev-box doctor` to identify gaps and `dev-box generate` to reconcile.

## Design Principles

**Convention over configuration.** File names and locations are standardized so AI agents can find them without special instructions.

**Human-readable first.** All context files are Markdown. They are useful without any tooling.

**Progressive complexity.** Start with `minimal` and upgrade to `managed` or `product` as the project grows.

**No lock-in.** Context files are plain Markdown in a `context/` directory. Stop using dev-box and the files remain useful.
