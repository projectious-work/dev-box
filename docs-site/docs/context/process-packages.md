---
sidebar_position: 2
title: "Process Packages"
---

# Process Packages

aibox uses a composable package system to determine which context files and skills are scaffolded for your project. **Presets** bundle multiple packages into sensible defaults; individual **packages** can be combined for custom setups.

## Presets at a Glance

| Factor | managed | software | research-project | full-product |
|--------|---------|----------|-----------------|--------------|
| Project complexity | Low–Medium | Medium | Varies | High |
| Decision tracking | Yes | Yes | Yes | Yes |
| Backlog management | Yes | Yes | Yes | Yes |
| Progress tracking | Via standups | Via standups | Via standups | Via standups |
| Development conventions | No | Yes | No | Yes |
| Research artifacts | No | No | Yes | Yes |
| Product planning | No | No | No | Yes |
| Team/ops conventions | No | No | No | Yes |

## managed

The recommended starting point. Structured tracking without product planning overhead. **Packages:** core + tracking + standups + handover.

### Files Created

```
my-project/
├── CLAUDE.md
└── context/
    ├── DECISIONS.md
    ├── BACKLOG.md
    ├── STANDUPS.md
    ├── EVENTLOG.md
    ├── OWNER.md
    ├── archive/
    │   ├── BACKLOG.md
    │   └── DECISIONS.md
    └── project-notes/
        └── session-template.md
```

### When to Use

- Ongoing projects with multiple work sessions
- Projects where architectural decisions need recording
- Team projects where context must be shared
- Open-source projects with contributors

### File Purposes

**DECISIONS.md** — Record architectural and design decisions using a lightweight ADR (Architecture Decision Record) format:

```markdown
## DEC-001: Use SQLite for local storage
- **Status:** Accepted
- **Date:** 2026-03-15
- **Context:** Need a local database; options are SQLite, embedded Postgres, flat files
- **Decision:** SQLite — zero config, single file, sufficient for our scale
- **Consequences:** No concurrent write support; acceptable for single-user app
```

**BACKLOG.md** — Prioritized list of work items:

```markdown
## High Priority
- [ ] Implement user authentication
- [ ] Add error handling for API calls

## Medium Priority
- [ ] Write integration tests
- [ ] Improve logging output
```

**STANDUPS.md** — Session-by-session progress log (newest first):

```markdown
## 2026-03-16
- Completed authentication module
- Fixed bug in API error handling
- Next: write integration tests
```

**project-notes/session-template.md** — Template for session handover notes: what was done, open items, next steps.

---

## software

For software development projects. Adds development conventions and architecture skills on top of `managed`. **Packages:** core + tracking + standups + handover + code + architecture.

### Files Created (beyond managed)

```
my-project/
└── context/
    └── work-instructions/
        └── DEVELOPMENT.md
```

Plus additional skills: `code-review`, `testing-strategy`, `software-architecture`, `refactoring`, and others.

### When to Use

- Software projects with a recurring build/test/review cycle
- Projects that benefit from documented coding conventions and architecture patterns
- Any project where `managed` is a good fit and you write code

---

## research-project

For learning, documentation, and academic projects. Adds research artifacts and documentation skills on top of `managed`. **Packages:** core + tracking + standups + handover + research + documentation.

### Files Created (beyond managed)

```
my-project/
├── experiments/
│   └── README.md
└── context/
    ├── PROGRESS.md
    ├── research/
    │   └── _template.md
    └── analysis/
```

Plus additional skills: `data-science`, `data-visualization`, `feature-engineering`, and others.

### When to Use

- LaTeX documents and academic papers
- Learning projects and study materials
- Data analysis projects
- Any project where the output is knowledge, not software

### File Purposes

**PROGRESS.md** — Track learning and research progress:

```markdown
## Chapter 3: Concurrency Patterns (In Progress)
- Read through mutex and channel sections
- Completed exercises 3.1-3.4
- Next: re-read section on pinning

## Chapter 2: Ownership (Complete)
- All exercises done
- Key insight: think of ownership as a compile-time garbage collector
```

**research/** — Directory for research notes, source summaries, literature reviews. Use `_template.md` as a starting point.

**experiments/** — Top-level directory for hands-on prototypes, benchmarks, and technical evaluations.

**analysis/** — Directory for analysis artifacts, data exploration notes, methodology documents.

---

## full-product

The most comprehensive preset. Everything from `managed` plus product planning, development conventions, and team/ops structure. **Packages:** core + tracking + standups + handover + code + architecture + design + product + security + operations.

### Files Created

```
my-project/
├── CLAUDE.md
└── context/
    ├── DECISIONS.md
    ├── BACKLOG.md
    ├── STANDUPS.md
    ├── PROJECTS.md
    ├── PRD.md
    ├── OWNER.md
    ├── work-instructions/
    │   ├── DEVELOPMENT.md
    │   └── TEAM.md
    ├── archive/
    │   ├── BACKLOG.md
    │   ├── DECISIONS.md
    │   └── PROJECTS.md
    ├── project-notes/
    │   └── session-template.md
    └── processes/
        ├── README.md
        ├── release.md
        ├── code-review.md
        ├── feature-development.md
        └── bug-fix.md
```

Plus a comprehensive skill set covering code review, testing, architecture, security, CI/CD, and more.

### When to Use

- Software products with users
- Multi-component systems
- Projects with multiple AI agents working in parallel
- Any project that benefits from formal requirements, security awareness, and team conventions

### Additional Files (beyond managed)

**PROJECTS.md** — Track multiple workstreams or sub-projects:

```markdown
## Active Projects

### Authentication Overhaul
- **Status:** In Progress
- **Lead:** Claude (agent-1)
- **Target:** v2.0 release
- **Key files:** src/auth/, tests/auth/
```

**PRD.md** — Product Requirements Document:

```markdown
## Product Vision
A CLI tool that manages AI-ready development containers.

## User Personas
1. Solo developer using Claude Code daily
2. Team lead standardizing dev environments

## Requirements
### Must Have
- Single config file (aibox.toml)
- Container lifecycle management
```

**work-instructions/DEVELOPMENT.md** — Development-specific conventions: branching strategy, testing requirements, release process.

**work-instructions/TEAM.md** — Team conventions for multi-agent or multi-developer work: who owns what, communication protocols, review processes.

---

## Individual Packages (Power Users)

Presets are the right choice for most projects. If you need a custom combination, you can specify individual packages directly via `--process <name>` on the CLI:

**13 available packages:** `core`, `tracking`, `standups`, `handover`, `code`, `architecture`, `design`, `product`, `security`, `data`, `operations`, `research`, `documentation`

```bash
# Custom combination: managed tracking + code only, no standups/handover
aibox init --process core --process tracking --process code
```

Individual packages are not shown in the interactive selection menu — pass them explicitly on the command line.

---

## Changing Process Packages

You can change the process at any time by editing `aibox.toml`:

```toml
[process]
packages = ["full-product"]  # was ["managed"]
```

Then run `aibox sync` to regenerate container files and deploy the new skill set.

However, changing packages does **not** automatically create or remove context files. To reconcile:

1. Update `packages` in the `[process]` section of `aibox.toml`
2. Run `aibox doctor` to see what files are missing or extra
3. Create missing files manually or re-run `aibox init` in a temporary directory and copy the templates

:::warning Upgrading is additive

Moving from `managed` to `full-product` means adding files. Moving from `full-product` to `managed` does not delete files — your existing context is preserved.

:::
