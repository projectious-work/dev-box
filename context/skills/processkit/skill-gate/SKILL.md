---
name: skill-gate
description: |
  Session-start meta-skill that enforces the 1% rule: before any action
  on a processkit entity or context/ file, if there is even a 1% chance
  a processkit skill applies, you MUST check skill-finder first. Load
  this skill at the start of every session that may involve processkit
  work. Use when beginning a new task, when unsure which skill applies,
  or when you are about to edit context/, create an entity, or run an
  MCP tool.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-skill-gate
    version: "1.0.0"
    created: 2026-04-11T08:00:00Z
    category: processkit
    core: true
    layer: null
    uses:
      - skill: skill-finder
        purpose: >
          The mandatory first stop for any processkit domain task —
          skill-gate enforces that skill-finder is consulted before
          acting, not after.
    provides:
      primitives: []
      mcp_tools: []
      assets: []
---

# Skill Gate

## Intro

skill-gate is the enforcement layer that ensures agents consult the
processkit skill catalog before acting on processkit entities or
`context/` files. It implements the **1% rule**: if there is even a 1%
chance a processkit skill covers the current task, you must check
`skill-finder` before doing anything else. It does not perform work
itself — it ensures work is performed via the right skill.

## Overview

### The 1% rule

Before any of the following actions, pause and apply the decision graph:

- Editing or creating any file under `context/`
- Calling any processkit MCP tool (`create_*`, `transition_*`, `link_*`,
  `record_*`, `open_*`)
- Writing a WorkItem, Decision, Note, Artifact, or any other entity
- Running a process the user described as a processkit task

**The rule:** If there is even a **1% chance** a processkit skill
applies to this task, you MUST check `skill-finder` before proceeding.

The threshold is deliberately near-zero. Do not rationalize past it.

### Decision graph

```
Is this task related to processkit entities, context/ files,
or any process the user described as a project-management task?
│
├─ YES (or unsure) ──→ Read skill-finder and identify the matching skill.
│                       Load that skill. Work from it.
│
└─ NO ──────────────→ Proceed without skill-gate involvement.
                       (Pure code edits, data analysis, etc. that have
                       no processkit component do not need skill-gate.)
```

### What "check skill-finder" means

1. Read `context/skills/processkit/skill-finder/SKILL.md` — find the
   trigger phrase that matches the current task.
2. Load the identified skill's `SKILL.md`.
3. Confirm from its **Intro** (3 sentences) that it is the right match.
4. Work from that skill's **Overview** — not from general knowledge.

If `skill-finder` has an MCP server available (`find_skill` tool), call
it instead of reading the file — it is faster and more reliable.

### Pre-empting common rationalisations

These are the rationalizations agents use to skip skill-gate. Each one
is wrong:

| Rationalisation | Reality |
|---|---|
| "I already know how to do this." | The skill may have processkit-specific entity paths, ID formats, MCP tools, and log conventions that general knowledge does not cover. Knowing the domain is not the same as knowing the processkit conventions for it. |
| "It's a simple edit — one line." | The state machine enforces valid transitions. A hand edit that bypasses the MCP server can put an entity in an invalid state, break the index, and skip the auto-log side effect. |
| "skill-finder won't have a match for this." | The catalog has 128+ skills. Check before assuming. If there genuinely is no match, `skill-builder` is the next step — not acting from general knowledge. |
| "The user told me exactly what to do." | User instructions describe intent, not process. The skill encodes the processkit-specific conventions for realizing that intent correctly. |
| "I've done this before in this session." | Skill-gate applies to each new task type, not just the first one. If the task domain shifts, check again. |

### Escape hatch

skill-gate does NOT apply when you are already operating inside a
named processkit skill's workflow. If you loaded `workitem-management`
and are following its Overview steps, you do not need to re-check
skill-finder for each sub-action within that skill. Skill-gate is a
session-level gate, not a per-tool-call gate.

## Gotchas

- **Treating skill-gate as optional at session start.** It is not. If
  this session may involve any processkit work, load skill-gate first.
  The cost is one file read. The cost of skipping it is a hand edit
  that bypasses schema validation, state-machine enforcement, index
  sync, and the audit trail.
- **Applying the 1% rule only to obvious processkit tasks.** "Write a
  standup" and "create a ticket" are obvious. "Quick note about this
  idea" (→ `note-management`), "log that we deployed" (→ `event-log`),
  and "check what's blocking us" (→ `workitem-management`) are not
  obvious but all have matching skills. When in doubt, check.
- **Confusing skill-gate with skill-finder.** skill-gate enforces the
  checking behaviour. skill-finder does the matching. They are distinct.
  skill-gate tells you *when* to check; skill-finder tells you *what*
  to use.
- **Skipping skill-gate because the user said "just edit the file."**
  The user describes the outcome they want, not the correct processkit
  process for achieving it. Hand-editing entity files bypasses the MCP
  server's schema validation and state-machine enforcement. Use the MCP
  tool unless the MCP server is explicitly unavailable.
- **Forgetting that skill-gate applies to entity creation, not just
  edits.** Writing a new YAML file under `context/workitems/` directly
  is a hand edit. Even for new entities, use `create_workitem` (or the
  relevant `create_*` tool) so the index stays in sync and the auto-log
  fires.
- **Applying the escape hatch too broadly.** The escape hatch covers
  sub-actions *within* a skill you have already loaded and are actively
  following. It does not cover switching to a different task type or
  domain mid-session without re-checking.

## Full reference

### Why the threshold is 1%, not "when clearly applicable"

A higher threshold — "check skill-finder when the task is obviously a
processkit task" — fails because agents are systematically overconfident
about what they already know. The 1% rule removes the judgment call
entirely. If there is any possibility, check. The marginal cost of an
unnecessary skill-finder lookup is one file read (seconds). The cost of
skipping it when it was needed is an out-of-sync index, an invalid
entity state, or a missing audit trail entry.

### Provider neutrality

skill-gate is implemented entirely in prose — it makes no assumptions
about which AI harness loads it. The same behaviour applies whether the
agent is Claude Code, Codex CLI, Cursor, or any other harness that
reads SKILL.md files.

### Relationship to AGENTS.md

AGENTS.md carries the project-level instruction to load skill-gate
at session start. skill-gate carries the enforcement logic. Keep them
separate: AGENTS.md says *when* to load skill-gate; skill-gate says
*what to do* once loaded.

### When skill-gate applies to non-processkit projects

skill-gate is a processkit-specific skill. It is only meaningful in
projects that have processkit installed (i.e. that have a `context/`
directory with processkit entities). In projects without processkit,
skill-gate should not be loaded.
