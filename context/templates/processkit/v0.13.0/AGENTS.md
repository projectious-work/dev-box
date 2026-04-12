# AGENTS.md

This file is the canonical, provider-neutral entry point for any AI coding
agent (or human collaborator) working on **aibox**. It follows
the [agents.md](https://agents.md) open standard.

If your harness auto-loads a provider-specific file (`CLAUDE.md`,
`CODEX.md`, `.cursor/rules`, …), that file should be a thin pointer to
this one. Edit **this** file, not the pointers.

## ⚠ First-time setup (delete this section once done)

This file ships from
[processkit](https://github.com/projectious-work/processkit) as a generic
template. Some placeholders are filled in by the project's installer at
render time; the rest require human input or codebase analysis.

**This is your first task on any new project.** Treat it as the implicit
first backlog entry — until the file is fully customized, no other work
can confidently proceed.

### Markup conventions

This template uses two markup conventions you need to recognize:

- **`{{TOKEN}}`** — a placeholder. Replace it with the real value during
  the fill protocol below.
- **`<!-- PLACEHOLDER:TOKEN class=X ... -->`** — an HTML comment
  immediately above a placeholder, declaring its class and how to fill
  it. Delete the comment along with the token once the value has been
  written in.

### Placeholder classes

Scan this file for any remaining `{{TOKEN}}` strings. For each, look at
the comment immediately above it (when present) for the class label, then
follow the matching protocol:

| Class | Source | What you do |
|---|---|---|
| **A — installer-rendered** | The project's installer (e.g. an aibox-managed devcontainer) substitutes these from its config at render time. Names like `aibox`, `v0.13.0`, `2026-04-12`. | If you see one still literal, the file was installed manually, by an installer that doesn't recognise the name, or the installer failed. Fall back to: read the project's installer-config file (e.g. `aibox.toml`) directly, or ask the owner. |
| **B — owner-supplied** | Information only the project owner knows: description, purpose, code-style preferences, conventions, gotchas. | Interview the owner one question at a time. Do **not** guess. Write each answer back into the file immediately, then move on. |
| **C — discoverable + confirmable** | Information you can extract from the codebase (build/test/lint commands from `package.json` / `Makefile` / `Cargo.toml` / `pyproject.toml` / `justfile` / etc.). | Propose a value to the owner with a one-line justification. Owner confirms or corrects. Then write the agreed value into the file. |

Placeholders **without** a `<!-- PLACEHOLDER:... -->` annotation default to
**Class B** — owner-supplied.

### Steps

1. Walk every `{{TOKEN}}` in this file in document order.
2. For each, follow the protocol above. Replace the token (and its
   `<!-- PLACEHOLDER:... -->` annotation, if any) with the value.
3. Once the file is fully populated, **delete this entire `## ⚠
   First-time setup` section** so future agents do not re-read
   instructions that no longer apply.
4. Then — and only then — proceed with whatever the user actually asked
   you to do.

Do not skip this step in the name of being helpful. An AGENTS.md full of
placeholders is worse than no AGENTS.md at all: it tells every future
agent that the project's contract has not been established, and they will
have no shared ground to work from.

## About this project

<!-- PLACEHOLDER:PROJECT_DESCRIPTION class=B
     One paragraph: what aibox does, who it is for, and what
     success looks like. Anything longer belongs in README.md. -->
{{PROJECT_DESCRIPTION}}

<!-- PLACEHOLDER:PROJECT_PURPOSE class=B
     One or two sentences stating the problem this project exists to
     solve — the "why does this exist". -->
**Why it exists.** {{PROJECT_PURPOSE}}

## Setup

<!-- PLACEHOLDER:BUILD_COMMAND class=C
     The command(s) to install dependencies and build the project.
     Sources: package.json scripts, Makefile, Cargo.toml, justfile,
     Taskfile.yml, pyproject.toml [project.scripts] / [tool.poetry.scripts],
     go.mod / Makefile, etc. -->
<!-- PLACEHOLDER:TEST_COMMAND class=C
     The command(s) to run the test suite. Same sources. -->
<!-- PLACEHOLDER:LINT_COMMAND class=C
     The command(s) to run the linter / formatter check. Same sources. -->

```sh
# install dependencies and build
{{BUILD_COMMAND}}

# run tests
{{TEST_COMMAND}}

# run linter / formatter check
{{LINT_COMMAND}}
```

## Code style and conventions

<!-- PLACEHOLDER:CODE_STYLE_NOTES class=B
     Project-specific conventions that aren't already enforced by the
     linter: commit message format, branch naming, test naming, forbidden
     patterns, anything an agent could get wrong without being told. -->
{{CODE_STYLE_NOTES}}

## Pull requests

<!-- PLACEHOLDER:PR_CONVENTIONS class=B
     How PRs are reviewed and merged on this project: required CI checks,
     who reviews, squash vs merge, branch protection, anything an agent
     should know before opening a PR. -->
{{PR_CONVENTIONS}}

## processkit preferences

Runtime configuration lives in per-skill config files under
`context/skills/<name>/config/settings.toml`. The agent edits these
directly; MCP servers read them on every call — no restart needed.

<!-- PLACEHOLDER:PROCESSKIT_PREFS class=B
     Ask the owner the following questions, then edit the relevant
     skill config files directly with their answers:

     1. ID format — "word" (e.g. BACK-calm-fox) or "uuid"?
        Edit: context/skills/id-management/config/settings.toml
        Key: format = "word" | "uuid"

     2. ID slug — append a content-derived slug from the entity title?
        (e.g. BACK-calm-fox-add-fts5-search)
        Edit: context/skills/id-management/config/settings.toml
        Key: slug = true | false

     3. Directory overrides — any custom subdirectory names under context/?
        (e.g. rename "workitems" to "tasks")
        Edit: context/skills/index-management/config/settings.toml
        Section: [directories]

     4. Log sharding — date-based subdirectories for log entries?
        Edit: context/skills/index-management/config/settings.toml
        Section: [sharding.LogEntry]

     After editing the config files, replace this block with a short
     bullet list of the choices made (see processkit's own AGENTS.md
     for an example). -->
{{PROCESSKIT_PREFS}}

---

## How this project is organized: processkit content

This project uses **[processkit](https://github.com/projectious-work/processkit.git)**, pinned at
`v0.13.0`, package tier(s) `product`, to manage
process content (skills, primitives, processes, schemas). All
processkit-installed material lives under `context/`:

```
context/
├── skills/         ← skill packages (SKILL.md, mcp/, references/, templates/)
├── schemas/        ← JSON schemas for the core primitives
├── state-machines/ ← state-machine definitions
├── processes/      ← process definitions (bug-fix, code-review, release, …)
└── templates/      ← immutable upstream mirror used as a diff baseline
```

`context/templates/processkit/<version>/` is the verbatim upstream
snapshot. **Do not edit it.** Edit the live files at
`context/skills/<name>/SKILL.md`, `context/processes/<name>.md`, etc.,
directly. Local edits are detected at the next sync via three-way diff
against the templates mirror.

Every `context/` subdirectory has an `INDEX.md`. **Read those first** —
do not slurp `context/skills/` or any large directory at session start.
Load specific files only when the task demands it. This is the
three-level principle: start at Level 1 (intro), drop to Level 2
(workflows) when the task narrows, drop to Level 3 (full reference) for
edge cases.

## Working with entities

processkit models project state as **entities** — work items, decision
records, discussions, log entries, scopes, gates, bindings, and so on.
Each entity is a YAML file under `context/<kind>s/`, created lazily on
first use.

For each entity kind, processkit ships:

- **A schema** at `context/schemas/<kind>.yaml`
- **A state machine** at `context/state-machines/<kind>.yaml`
- **An MCP server** at `context/skills/<kind>-management/mcp/server.py`

### Read entities through the index

`context/skills/index-management/` exposes a SQLite-backed index over
every entity in `context/`. Call its tools — `query_entities(kind=…,
state=…)`, `get_entity(id)`, `search_entities(text)` — instead of `ls` /
`grep` / filesystem walks. The index is faster, context-cheaper, and
reflects the canonical state.

### Write entities through the per-kind MCP servers

Use the relevant MCP tool to create or transition entities —
`create_workitem` and `transition_workitem` from `workitem-management`,
`record_decision` from `decision-record`, and so on. Hand-editing entity
files works but bypasses index updates and state-machine validation, so
the index can drift and invalid transitions can slip through. Reserve
hand edits for cases the MCP tools genuinely don't cover.

### Wiring the MCP servers into your harness

Each MCP-bearing skill ships its own `mcp/mcp-config.json` declaring how
to launch the server (typically `uv run …/server.py`). Agent harnesses
(Claude Code, Codex CLI, Cursor, …) discover MCP servers by reading a
single config file at startup, so the per-skill configs need to be
**merged** into the one file your harness reads, and that file needs to
live at the path your harness expects.

If this project was set up by an installer (e.g. an aibox-managed
devcontainer), the installer is responsible for that wiring — the merged
config is generated for you and you should not need to touch it. If
processkit was installed manually, the project owner is responsible for
merging the per-skill blocks and placing the result at the
harness-specific path themselves.

Either way, MCP-bearing skills require **`uv`** and **Python ≥ 3.10** on
PATH inside the environment where the harness runs the servers — each
`server.py` is a self-contained PEP 723 script and `uv run` resolves its
dependencies on first launch.

## AI agents on this project

Configured providers: **claude**. Other agents may be working
on this project — coordinate through the entity layer
(`workitem-management`, `event-log`, `discussion-management`) rather than
assuming you are alone.

**Commit to actions immediately.** If you decide to create an entity
(WorkItem, DecisionRecord, etc.), call the tool in the same turn. Do
not say "I'll track that" and move on — deferred commitments are
routinely dropped and leave the entity layer out of sync with what was
discussed.

**Check the skill catalog before acting on domain tasks.** When a
domain-specific task arrives — writing a PRD, creating a release,
reviewing a skill, designing a schema — search the processkit skill
catalog first. Use `search_entities` via index-management or check
`skill-finder` before falling back to general knowledge. A matching
skill may exist with processkit-specific conventions (entity storage
paths, workitem linking, output formats) that general knowledge does
not know. Missing a skill wastes work and produces non-standard
output.

### Contributing improvements upstream

When you make a behavioral or content improvement to a file in
`context/` that was installed from processkit (it has a counterpart
in `context/templates/processkit/<version>/`), ask whether the
improvement is general enough to benefit all processkit consumers.

If yes:
1. Open a Discussion entity locally with `open_discussion` —
   title it "Upstream proposal: <short description>", note the
   changed file and the improvement in the body.
   This creates an audit trail so future sessions can see what
   was proposed and what was decided.
2. File an issue at the processkit repository so maintainers can
   consider it for the upstream catalog.

Nothing is mandatory — the project owner decides what to file
upstream. The Discussion entity records the decision either way.

## Project-specific notes

<!-- PLACEHOLDER:NONOBVIOUS_GOTCHAS class=B
     Anything else an agent or human must know that isn't visible from
     the code: out-of-scope reminders, local quirks, deployment notes,
     decisions that aren't yet in a DecisionRecord. Keep it short —
     long-form context belongs in entity files under context/. -->
{{NONOBVIOUS_GOTCHAS}}

---

<sub>Scaffolded by processkit `v0.13.0` on `2026-04-12`. Re-rendered on each installer sync.</sub>
