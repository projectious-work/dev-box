---
schema_version: "1.0.0"
dev_box_version: "0.1.0"
---

# Dev-Box Context Schema v1.0.0

## AI Migration Agent Instructions

You are a project structure migration agent. When given this schema document
along with a project's current context files, your task is to:

1. Read this schema to understand the required structure
2. Examine the project's current context/ directory
3. Identify missing files, missing sections, or structural deviations
4. Propose specific, non-destructive changes
5. NEVER delete or overwrite existing content
6. Adapt the structure to the project's specific domain

Generate a migration plan as a markdown document with:
- A summary of what needs to change
- For each change: the file, what to add/modify, and the specific content
- Mark each change as "required" or "recommended"

### Migration Principles

- **Additive only.** Never remove content the user has written.
- **Preserve formatting.** Match the existing style of the file.
- **ID continuity.** When adding BACKLOG or PROJECTS entries, read the
  "Next ID" field and increment from there.
- **Section order.** New sections go at the end unless the schema specifies
  a position.
- **Placeholder vs content.** If a section exists but is empty (only has
  the HTML comment placeholder), leave it alone — the user hasn't filled
  it in yet and that's fine.

---

## Work Process Flavors

### minimal

The lightest process. Suitable for scripts, experiments, and throwaway projects.

**Required files:**
- `CLAUDE.md` (project root)
- `.dev-box-version`
- `.gitignore` (with dev-box entries)

**No context/ directory is created.**

### managed

A structured process for solo or small-team projects that benefit from
decision tracking and a task backlog.

**Required files:**
- `CLAUDE.md` (project root)
- `context/OWNER.md` (symlink or placeholder)
- `context/DECISIONS.md`
- `context/BACKLOG.md`
- `context/STANDUPS.md`
- `context/work-instructions/GENERAL.md`
- `.dev-box-version`
- `.gitignore` (with dev-box entries)

### research

Designed for research, writing, and document-production projects (papers,
books, reports). Tracks progress across sections rather than a task backlog.

**Required files:**
- `CLAUDE.md` (project root)
- `context/OWNER.md` (symlink or placeholder)
- `context/PROGRESS.md`
- `context/research/.gitkeep`
- `context/analysis/.gitkeep`
- `.dev-box-version`
- `.gitignore` (with dev-box entries)

**Required directories:**
- `context/research/` — research notes and source material
- `context/analysis/` — structural analysis and design documents

### product

Full project management for products and applications with multiple
collaborators (human or AI). Includes everything from managed plus
project tracking, PRD, and team collaboration rules.

**Required files:**
- `CLAUDE.md` (project root)
- `context/OWNER.md` (symlink or placeholder)
- `context/DECISIONS.md`
- `context/BACKLOG.md`
- `context/STANDUPS.md`
- `context/PROJECTS.md`
- `context/PRD.md`
- `context/work-instructions/GENERAL.md`
- `context/work-instructions/DEVELOPMENT.md`
- `context/work-instructions/TEAM.md`
- `context/project-notes/.gitkeep`
- `context/ideas/.gitkeep`
- `.dev-box-version`
- `.gitignore` (with dev-box entries)

**Required directories:**
- `context/project-notes/` — deep research on specific topics
- `context/ideas/` — future direction research

---

## File Specifications

### CLAUDE.md (project root)

**Purpose:** Primary context file for Claude Code. This is the first file
Claude reads when starting work on a project.

**Required sections:**

| Section | Required for | Description |
|---------|-------------|-------------|
| Project Overview | all flavors | What this project is and does |
| Building | all flavors | How to build the project |
| Testing | product | How to run tests |
| Verifying Output | research | How to verify build results |
| Project Structure | all flavors | Key directories and files |
| Context Files | managed, research, product | Lists context/ directory contents |

**Template variable:** `{{project_name}}` is replaced with the project name
from dev-box.toml during scaffolding.

**Example (managed):**
```markdown
# CLAUDE.md — my-api

This file provides guidance to Claude Code when working with this repository.

## Project Overview

A REST API for managing user accounts, built with Python/FastAPI.

## Building

    pip install -e ".[dev]"
    uvicorn main:app --reload

## Project Structure

- `src/` — application source code
- `tests/` — test suite
- `context/` — project management files

## Context Files

The `context/` directory contains structured project management files:

- `context/OWNER.md` — Profile of the project owner
- `context/DECISIONS.md` — Decision log (inverse chronological)
- `context/BACKLOG.md` — Task registry with unique IDs
- `context/STANDUPS.md` — Daily standup template and log
- `context/work-instructions/GENERAL.md` — General rules for AI agents
```

---

### context/OWNER.md

**Purpose:** Profile of the project owner. Shared across projects via
symlink to `~/.config/dev-box/OWNER.md`.

**Setup:** If `~/.config/dev-box/OWNER.md` exists at init time, a symlink
is created. Otherwise a placeholder is written with instructions.

**Required sections:**

| Section | Description |
|---------|-------------|
| About | Name, role, contact |
| Preferences | Communication style, code style, review preferences |

---

### context/DECISIONS.md

**Purpose:** Inverse chronological log of key technical and process decisions.
Provides historical context for why things are the way they are.

**Used by:** managed, product

**Required format:** Each entry must include Date, Decision, Rationale,
and Alternatives considered.

**ID scheme:** No formal IDs — entries are identified by date and title.

**Example of a well-formed entry:**
```markdown
### 2025-12-15 — Use SQLite instead of PostgreSQL

- **Date:** 2025-12-15
- **Decision:** Use SQLite as the primary database for v1
- **Rationale:** The application is single-user and doesn't need concurrent
  write access. SQLite requires zero configuration and the database file
  can be backed up by simply copying it. Migration to PostgreSQL later
  is straightforward since we use an ORM.
- **Alternatives considered:**
  - PostgreSQL: more capable but adds deployment complexity
  - JSON files: simpler but no query capability or ACID guarantees
```

---

### context/BACKLOG.md

**Purpose:** Central task registry with unique IDs for cross-referencing
in commits, PRs, and discussions.

**Used by:** managed, product

**ID scheme:** `BACK-NNN` (zero-padded three digits, monotonically increasing).
The "Next ID" field at the top tracks the next available ID.

**Status values:** `todo`, `in-progress`, `done`, `blocked`, `archived`

**Priority values:** `must`, `should`, `could`, `wont`

**Required sections:**
- Next ID header
- Format table
- Active Items
- Archive

**Example of well-formed entries:**
```markdown
## Next ID: BACK-005

## Active Items

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
| BACK-001 | Set up CI pipeline | done | must | GitHub Actions, runs on push |
| BACK-002 | Add user authentication | in-progress | must | Using JWT tokens |
| BACK-003 | Write API documentation | todo | should | OpenAPI spec |
| BACK-004 | Add rate limiting | todo | could | Consider nginx vs app-level |

## Archive

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
```

**Cross-referencing:** Use BACK-NNN in git commit messages and branch names:
- Branch: `back-002/user-authentication`
- Commit: `feat: add login endpoint (BACK-002)`

---

### context/PROJECTS.md

**Purpose:** Project registry for tracking higher-level initiatives that
span multiple backlog items.

**Used by:** product

**ID scheme:** `PROJ-NNN` (zero-padded three digits, monotonically increasing).

**Status values:** `active`, `paused`, `complete`, `archived`

**Example of a well-formed entry:**
```markdown
## Next ID: PROJ-003

## Active Projects

| ID | Name | Status | Description |
|----|------|--------|-------------|
| PROJ-001 | User Management | active | Authentication, authorization, user profiles |
| PROJ-002 | API v2 | paused | REST to GraphQL migration |
```

---

### context/STANDUPS.md

**Purpose:** Daily standup log capturing what was done, what's planned,
and any blockers. Useful for handoffs between AI agent sessions.

**Used by:** managed, product

**Required format:** Each entry has Done, Planned, and Blockers sections.

**Example of a well-formed entry:**
```markdown
### 2025-12-16

**Done:**
- Implemented JWT token generation and validation (BACK-002)
- Added login and register endpoints
- Wrote integration tests for auth flow

**Planned:**
- Add password reset flow
- Set up email service for verification

**Blockers:**
- Need SMTP credentials for email service — waiting on owner
```

---

### context/PROGRESS.md

**Purpose:** Track completion status across project sections or chapters.
Used for research and document-production projects.

**Used by:** research

**Status values:** `not-started`, `in-progress`, `review`, `complete`

**Example of well-formed content:**
```markdown
## Overall Status

60% complete, 3 of 5 chapters done.

## Sections

| Section | Status | Notes |
|---------|--------|-------|
| Chapter 1: Introduction | complete | Reviewed 2025-12-10 |
| Chapter 2: Literature Review | complete | 45 sources cited |
| Chapter 3: Methodology | complete | Peer reviewed |
| Chapter 4: Results | in-progress | Data analysis 80% done |
| Chapter 5: Conclusion | not-started | Blocked on Ch 4 |
```

---

### context/PRD.md

**Purpose:** Product requirements document defining vision, users, core
requirements, non-goals, and success metrics.

**Used by:** product

**Required sections:**

| Section | Description |
|---------|-------------|
| Vision | What the product is and why it exists |
| Target Users | Who the product serves |
| Core Requirements | Must-have features |
| Non-Goals | What the product explicitly will NOT do |
| Success Metrics | How success is measured |

---

### context/work-instructions/GENERAL.md

**Purpose:** General rules for all AI agents. Managed by dev-box and
updated via `dev-box doctor` and `dev-box update`.

**Used by:** managed, product

**Required sections:** Communication, Code Quality, Git Workflow, Context Management

**Note:** This file is machine-managed. User customizations should go in
separate instruction files.

---

### context/work-instructions/DEVELOPMENT.md

**Purpose:** Development-specific rules: tech stack, build/test commands,
code conventions, branching strategy.

**Used by:** product

**Required sections:** Tech Stack, Build & Test, Code Conventions, Branching Strategy

---

### context/work-instructions/TEAM.md

**Purpose:** Team collaboration rules for multi-agent workflows.

**Used by:** product

**Required sections:** Agent Roles, Handoff Protocol, Communication

---

## .dev-box-version

**Purpose:** Records the dev-box CLI version that created or last updated
the context structure. Used by `dev-box doctor` to detect schema drift.

**Format:** Plain text, single line, semver string. Example: `0.1.0`

---

## .gitignore entries

The following entries must be present in `.gitignore`:

```
# dev-box generated
.devcontainer/Dockerfile
.devcontainer/docker-compose.yml
.devcontainer/devcontainer.json
.root/
.dev-box-version
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-01-01 | Initial schema release |
