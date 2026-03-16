# Work Processes

dev-box provides four work process flavors that determine which context files are scaffolded for your project. Choose the one that matches your project's complexity and workflow.

## Decision Matrix

| Factor | minimal | managed | research | product |
|--------|---------|---------|----------|---------|
| Project complexity | Low | Medium | Varies | High |
| Team size | Solo | 1-3 | Solo/duo | Any |
| Decision tracking | No | Yes | No | Yes |
| Backlog management | No | Yes | No | Yes |
| Progress tracking | No | Via standups | Dedicated | Via standups |
| Work instructions | No | Yes | No | Yes |
| Research artifacts | No | No | Yes | No |
| Product planning | No | No | No | Yes |

## minimal

The lightest option. Only `CLAUDE.md` at the project root.

### Files Created

```
my-project/
└── CLAUDE.md
```

### When to Use

- Shell scripts and small utilities
- Experiments and prototypes
- Projects where you do not need to track decisions or progress
- Quick throwaway work

### What CLAUDE.md Contains

The generated `CLAUDE.md` includes:

- Project name and description placeholder
- Reference to `OWNER.md` (if configured)
- Basic coding conventions section
- Space for project-specific instructions

---

## managed

For projects that benefit from structured tracking without full product planning overhead.

### Files Created

```
my-project/
├── CLAUDE.md
└── context/
    ├── DECISIONS.md
    ├── BACKLOG.md
    ├── STANDUPS.md
    ├── OWNER.md → (symlink or reference to ~/.config/dev-box/OWNER.md)
    └── work-instructions/
        └── GENERAL.md
```

### When to Use

- Ongoing projects with multiple work sessions
- Projects where architectural decisions need recording
- Team projects where context must be shared
- Open-source projects with contributors

### File Purposes

**DECISIONS.md** -- Record architectural and design decisions using a lightweight ADR (Architecture Decision Record) format:

```markdown
## DEC-001: Use SQLite for local storage
- **Status:** Accepted
- **Date:** 2026-03-15
- **Context:** Need a local database; options are SQLite, embedded Postgres, flat files
- **Decision:** SQLite — zero config, single file, sufficient for our scale
- **Consequences:** No concurrent write support; acceptable for single-user app
```

**BACKLOG.md** -- Prioritized list of work items:

```markdown
## High Priority
- [ ] Implement user authentication
- [ ] Add error handling for API calls

## Medium Priority
- [ ] Write integration tests
- [ ] Improve logging output

## Low Priority
- [ ] Add dark mode toggle
```

**STANDUPS.md** -- Session-by-session progress log (newest first):

```markdown
## 2026-03-16
- Completed authentication module
- Fixed bug in API error handling
- Next: write integration tests
```

**work-instructions/GENERAL.md** -- Conventions and instructions that apply across the project. More detailed than `CLAUDE.md`, focused on workflow rather than coding style.

---

## research

For learning, documentation, and academic projects where progress tracking and research artifacts matter more than backlogs.

### Files Created

```
my-project/
├── CLAUDE.md
└── context/
    ├── PROGRESS.md
    ├── OWNER.md → (reference)
    ├── research/
    └── analysis/
```

### When to Use

- LaTeX documents and academic papers
- Learning projects and study materials
- Cheatsheets and reference documents
- Data analysis projects
- Any project where the output is knowledge, not software

### File Purposes

**PROGRESS.md** -- Track learning and research progress:

```markdown
## Chapter 3: Concurrency Patterns (In Progress)
- Read through mutex and channel sections
- Completed exercises 3.1-3.4
- Struggling with: async lifetime issues
- Next: re-read section on pinning

## Chapter 2: Ownership (Complete)
- All exercises done
- Key insight: think of ownership as a compile-time garbage collector
```

**research/** -- Directory for research notes, source summaries, literature reviews:

```
research/
├── paper-notes/
│   ├── smith-2024-distributed-systems.md
│   └── jones-2025-consensus-protocols.md
└── topic-summaries/
    └── raft-vs-paxos.md
```

**analysis/** -- Directory for analysis artifacts, data exploration notes, methodology documents:

```
analysis/
├── methodology.md
├── dataset-description.md
└── preliminary-results.md
```

---

## product

The most comprehensive flavor. Everything from `managed` plus product planning tools.

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
    ├── OWNER.md → (reference)
    ├── work-instructions/
    │   ├── GENERAL.md
    │   ├── DEVELOPMENT.md
    │   └── TEAM.md
    ├── project-notes/
    └── ideas/
```

### When to Use

- Software products with users
- Multi-component systems
- Projects with multiple AI agents working in parallel
- Any project that benefits from formal requirements and planning

### Additional Files (beyond managed)

**PROJECTS.md** -- Track multiple workstreams or sub-projects:

```markdown
## Active Projects

### Authentication Overhaul
- **Status:** In Progress
- **Lead:** Claude (agent-1)
- **Target:** v2.0 release
- **Key files:** src/auth/, tests/auth/

### API v2 Migration
- **Status:** Planning
- **Blocked by:** Authentication overhaul
```

**PRD.md** -- Product Requirements Document:

```markdown
## Product Vision
A CLI tool that manages AI-ready development containers.

## User Personas
1. Solo developer using Claude Code daily
2. Team lead standardizing dev environments

## Requirements
### Must Have
- Single config file (dev-box.toml)
- Container lifecycle management
- Context scaffolding

### Nice to Have
- Auto-update mechanism
- Plugin system
```

**work-instructions/DEVELOPMENT.md** -- Development-specific conventions: branching strategy, testing requirements, release process.

**work-instructions/TEAM.md** -- Team conventions for multi-agent or multi-developer work: who owns what, communication protocols, review processes.

**project-notes/** -- Free-form directory for meeting notes, design sketches, and other project documentation that does not fit elsewhere.

**ideas/** -- Parking lot for future ideas that are not yet backlog items.

---

## Changing Process Flavor

You can change the process flavor in `dev-box.toml` at any time:

```toml
[dev-box]
process = "product"  # was "managed"
```

However, changing the flavor does not automatically create or remove files. To reconcile:

1. Update the `process` field in `dev-box.toml`
2. Run `dev-box doctor` to see what files are missing or extra
3. Create missing files manually or re-run `dev-box init` in a temporary directory and copy the templates

!!! warning "Upgrading is additive"
    Moving from `minimal` to `managed` means adding files. Moving from `product` to `minimal` does not delete files -- your existing context is preserved.
