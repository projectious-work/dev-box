---
name: onboarding-guide
description: |
  Generates a structured onboarding guide for a new agent or human joining a project — covering codebase orientation, process context, first tasks, and key contacts. Use when asked to onboard someone new, write a getting-started guide, or create a "first week" plan for a project.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-onboarding-guide
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: product
    layer: 2
    uses:
      - skill: agent-management
        purpose: Understand context budget principles and what files to include in the always-load set.
      - skill: agent-management
        purpose: Reference the most recent session context (HANDOVER.md) as the starting point for new joiners.
---

# Onboarding Guide

## Intro

A good onboarding guide answers the questions a newcomer will have
in their first hour, day, and week — before they need to ask. For
agent onboarding specifically, it establishes what to read first,
what never to touch, and what the most common first tasks look like.
Write it once, update it at every major structural change.

## Overview

### Two modes: agent onboarding and human onboarding

**Agent onboarding** is about making the first session effective:
the agent reads a compact document and immediately knows enough to
start working safely. It targets the AGENTS.md file and the context
files a fresh agent loads at session start.

**Human onboarding** is broader: it covers environment setup, team
conventions, process workflows, key contacts, and first tasks. It
results in a longer document — a `docs/onboarding.md` or a
`context/ONBOARDING.md` — that a developer reads over their first
week.

Use the right mode for the audience. Ask if unclear.

### Agent onboarding — what to include in AGENTS.md

The `AGENTS.md` file is the canonical entry point for any agent.
It should include:

1. **Read-first trail** — the 3-4 files to read in order before
   touching anything (README, HANDOVER, this file, BACKLOG)
2. **Always-on rules** — hard constraints (never edit generated
   files, always run tests before committing, etc.)
3. **Project structure** — the key directories, what each contains,
   and what's off-limits
4. **First-session workflow** — "Here's how a typical work session
   goes" with a concrete example
5. **Gotchas** — the 3-5 things that trip up agents new to this
   project (not general programming knowledge — project-specific)

```markdown
## Read these first

1. `README.md` — what this project does and why
2. `context/HANDOVER.md` — current state and what was worked on last
3. `AGENTS.md` — this file: always-on rules
4. `context/BACKLOG.md` — what's queued, what's in progress

## Always-on rules

- Run `npm test` before every commit — the CI gate is strict
- Never edit files in `generated/` — they're regenerated on build
- Every PR needs a linked issue — use `Closes #NNN` in the PR body
- context/ files are managed by processkit — use MCP tools, not raw edits

## Project structure

src/          — application source code
context/      — project-management artifacts (backlog, decisions, logs)
docs/         — user-facing documentation
generated/    — AUTO-GENERATED, do not edit directly
scripts/      — maintenance and deployment scripts

## First-session workflow

1. Read `context/HANDOVER.md` (3 min)
2. Read `context/BACKLOG.md` (2 min) — pick the top item in "Next Up"
3. Ask the user which item to start or confirm the top item
4. Work the item to done: code → tests → PR
5. Write a session handover before ending
```

### Human onboarding — full guide structure

For a human developer joining a project, produce a guide with these
sections:

```markdown
# Onboarding Guide — [Project Name]

## Welcome and context
[2-3 sentences: what the project does, who the users are, why it matters]

## Environment setup

### Prerequisites
- [Language / runtime version]
- [Required tools]

### First-time setup
\`\`\`bash
git clone [repo]
cd [project]
[setup command]
[run tests]
\`\`\`

### Verify it works
[What a successful setup looks like]

## Codebase orientation

### Key directories
| Directory | Purpose |
|---|---|
| `src/` | Application source |
| `tests/` | Test suite |
| `docs/` | Documentation |

### Where to start reading
1. `[entry point file]` — [why it's the right starting point]
2. `[key module]` — [what it does]
3. `[tests/]` — [how the test suite is organized]

## How we work

### Branching
[Branch naming convention, e.g., `feature/`, `fix/`, `chore/`]

### Pull requests
[PR checklist: tests, description, linked issue, review requirement]

### Code style
[Linter, formatter, how to run them]

### Deployments
[Who deploys, when, how to know if something is broken]

## Your first tasks

### Day 1
- [ ] Complete environment setup and run the tests locally
- [ ] Read the last 5 git commits to understand what's been worked on
- [ ] Introduce yourself in [Slack/team channel]

### Week 1
- [ ] Pick up [specific starter ticket] and work it end-to-end
- [ ] Schedule a 30-min call with [team member] to understand [domain area]
- [ ] Read [key design doc or ADR]

## Key contacts
| Name | Role | What to ask them about |
|---|---|---|
| [Name] | Lead engineer | Architecture decisions |
| [Name] | Product | Feature priorities |

## Where to ask questions
- Quick questions: [Slack channel]
- Design/architecture: open a GitHub Discussion
- Bugs: file a GitHub Issue

## Gotchas — things that trip people up
- [Specific pitfall unique to this codebase, not generic programming advice]
- [Another project-specific quirk]
```

### Generating from existing project context

When the project uses processkit, pull from live context:

1. Read `AGENTS.md` for the existing rules
2. Read `context/HANDOVER.md` for current state
3. Read `context/BACKLOG.md` for the in-progress items
4. Scan `README.md` for setup instructions

Synthesize into the guide format. Do not duplicate content that is
already authoritative in those files — link to them instead.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Writing a generic onboarding guide that is not specific to this project.** Generic guides ("run `npm install`, then `npm test`") provide no orientation to the actual codebase or process. Every item in the onboarding guide should be something the newcomer cannot get from a template — project-specific conventions, project-specific gotchas, the actual first tasks for this project.
- **Making the guide so comprehensive it takes a day to read.** Onboarding guides that try to document everything upfront guarantee that nothing is remembered. Aim for "enough to be dangerous in the first session" — the rest is discovered through work. A good guide is read in 20-30 minutes, not 2 hours.
- **Not including a verified "first setup" command sequence.** An onboarding guide that says "install the dependencies and run the tests" without a tested step-by-step leaves newcomers guessing. Walk through the exact commands, in order, and verify they work before publishing.
- **Omitting the gotchas section.** The most valuable part of an onboarding guide is the project-specific things that will trip someone up — the generated directory that must not be edited, the test database that must be seeded, the environment variable with the non-obvious name. Generic guides omit these; effective ones lead with them.
- **Targeting only humans when the project uses AI agents.** If the project uses AI agents (Claude Code, Cursor, etc.), the agent's onboarding path is different from a human's. The AGENTS.md file is the agent's read-first document; the onboarding guide for humans should reference it but not duplicate it.
- **Not updating the guide when the project structure changes.** An onboarding guide that describes a directory that no longer exists is worse than no guide — it erodes trust. Treat the onboarding guide as a living document: update it at every major structural change, not just at the start.
- **Writing the guide in isolation without checking it against a real newcomer's questions.** The best onboarding guides are written by someone who has just onboarded, or reviewed by someone who is about to. If neither is available, walk through the guide step-by-step in a fresh environment to catch missing steps and stale instructions.

## Full reference

### Agent first-session checklist (to embed in AGENTS.md)

```markdown
## First session checklist

- [ ] Read `context/HANDOVER.md` — understand current state
- [ ] Read `context/BACKLOG.md` — identify in-progress items
- [ ] Confirm with user which item to work, or start the top "Next Up" item
- [ ] Run the test suite to confirm a clean baseline
- [ ] Work the item: implement, test, commit
- [ ] Update `context/BACKLOG.md` with any status changes
- [ ] Write a session handover note to `context/archive/sessions/`
```

### Updating AGENTS.md vs. creating a separate ONBOARDING.md

Use **AGENTS.md** for agent-specific rules and the read-first trail.
Use **ONBOARDING.md** (in `docs/` or `context/`) for human-specific
setup instructions and first-week tasks. Keep them separate: AGENTS.md
is loaded at every session start (it's in the always-load set);
ONBOARDING.md is a one-time reference that grows longer without penalty.

### Signs the onboarding guide needs updating

- A new team member asks a question that the guide should have answered
- A directory or tool referenced in the guide no longer exists
- The setup commands produce errors that didn't exist before
- A newcomer took >2 hours to complete environment setup

Treat each of these as a guide update ticket, not just a one-time
answer to one person.
