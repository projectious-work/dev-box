---
sidebar_position: 1
title: "Context Overview"
---

# Context System Overview

The aibox context system gives AI agents the structured, file-based information
they need to work effectively on your project — backlog, decisions, standups,
processes, primitives, and skills.

As of **v0.16.0**, the system is split across two cleanly separated projects:

- **aibox** owns the **container** — devcontainers, addons, the CLI, the
  install/sync/migrate machinery, and the slim project skeleton (`.aibox-version`,
  `.gitignore`, an empty `context/`, and a thin `CLAUDE.md` pointer).
- **[processkit](https://github.com/projectious-work/processkit)** owns the
  **content** — every skill, every primitive schema, every state machine, the
  canonical `AGENTS.md` template, the processes, and the package YAMLs that
  compose them.
- **The user-side `context/` directory is shared territory.** aibox creates it,
  processkit fills it, and the user edits in place. An immutable upstream
  snapshot is kept under `context/templates/processkit/<version>/` for the
  three-way diff that `aibox sync` uses to detect upstream changes versus
  local edits.

## The Problem

AI coding agents like Claude operate best when they understand not just the
code, but the project's goals, decisions, and current state. Without structure,
this information ends up scattered across chat histories, stale comments, and
the developer's memory.

A single root-level instructions file is not enough for non-trivial projects.
It works well for instructions and preferences, but it does not provide a
standard place for decisions, backlog, progress tracking, or team conventions.

## How Context Files Work

After `aibox init` and `aibox sync` (with a real `[processkit].version` pinned),
your project looks something like this:

```
my-project/
├── AGENTS.md                       # Canonical agent entry — rendered from processkit scaffolding
├── CLAUDE.md                       # Thin pointer to AGENTS.md (provider entry file)
├── aibox.toml
├── .devcontainer/
└── context/
    ├── BACKLOG.md                  # Created by the agent on first use
    ├── DECISIONS.md                # Created by the agent on first use
    ├── STANDUPS.md                 # Created by the agent on first use
    ├── OWNER.md                    # Profile of the project owner
    ├── skills/                     # Editable skill copies (108 in v0.5.1)
    ├── processes/                  # release, code-review, feature-development, bug-fix
    ├── primitives/                 # schemas, state-machines
    └── templates/
        └── processkit/
            └── v0.5.1/             # Immutable upstream snapshot — base of three-way diffs
```

Notable: the single-file context tracks (`BACKLOG.md`, `DECISIONS.md`,
`STANDUPS.md`, …) are **not** scaffolded as starter files. The corresponding
processkit skill — `backlog-context`, `decisions-adr`, `standup-context` —
instructs the agent to create the file in place the first time it needs to
write to it. There is deliberately no template; the file is born by being
used.

### AGENTS.md, CLAUDE.md, and provider files

`AGENTS.md` at the project root is the **canonical** agent entry document. It
is rendered from the processkit scaffolding template (`src/scaffolding/AGENTS.md`)
during `aibox init` (write-if-missing — never overwrites). The
[agents.md](https://agents.md/) ecosystem convention is to read this file from
any AI harness.

When `[ai].providers` includes `claude`, aibox also writes a thin `CLAUDE.md`
at the project root that just points at `AGENTS.md`. **No content is written
under `.claude/skills/` or any other provider-specific directory** as of
v0.16.0 — that path is gone. Other providers (Aider, Gemini, Mistral) use
config files (`.aider.conf.yml`, `.gemini/settings.json`, `.mistral/config.json`)
which are scaffolded by the addon system; they do not get a markdown pointer.

## OWNER.md — Developer Identity

`OWNER.md` captures the developer's identity and preferences. It is created
during `aibox init` (or by the `owner-profile` skill the first time the agent
asks), with fields that help AI agents understand who they are working with:

- **Name** — how the developer prefers to be addressed
- **Domain expertise** — areas of knowledge and experience
- **Primary languages** — programming languages used most often
- **Communication language** — natural language for responses (e.g., English, German)
- **Timezone** — for scheduling and availability context
- **Working hours** — typical availability window
- **Current focus** — what the developer is currently working on or learning
- **Communication preferences** — style and conventions for AI interactions

## Two Tracks for the Same Artefact

processkit deliberately ships **two tracks** for context-management. Both are
installed in every release; pick the one that fits each project.

| Track | Skills | How it works |
|-------|--------|--------------|
| **Single-file** | `backlog-context`, `decisions-adr`, `standup-context`, `session-handover`, `context-archiving` | The skill maintains a single Markdown file in place (e.g. `context/BACKLOG.md`). No starter template — the file is born when the agent first writes to it. |
| **Entity-sharded** | `workitem-management`, `decision-record`, `scope-management`, … | Per-item YAML files with IDs, slugs, and state machines. Backed by an MCP server. |

## Process Packages

processkit ships **five packages** that compose skill sets via `extends:` in
upstream YAMLs (`packages/{minimal,managed,software,research,product}.yaml`).
You select packages declaratively in `aibox.toml`:

```toml
[context]
packages = ["managed"]
```

| Package | Best for |
|---------|----------|
| `minimal` | Scripts, experiments, small utilities |
| `managed` | Recommended default — backlog, decisions, standups, handover |
| `software` | Software projects with a recurring build/test/review cycle |
| `research` | Learning, documentation, academic work |
| `product` | Full product development with security, ops, design, planning |

In v0.16.0, package selection is **declarative metadata**. aibox installs every
processkit skill regardless; the package list tells agents which subset to
prefer. See [Process Packages](process-packages.md) for the full breakdown.

## Version Tracking

Two pieces track the version:

```toml
[aibox]
version = "0.16.0"

[context]
schema_version = "1.0.0"

[processkit]
version = "v0.5.1"
```

`.aibox-version` in the project root records the aibox CLI version that was
last applied. When the schema evolves, `aibox doctor` flags version mismatches
and `aibox sync` runs the relevant migrations. See [Migration](migration.md)
for details.

## Relationship to aibox.toml

The `[context]` section in `aibox.toml` declares which processkit packages are
in scope. The `[processkit]` section pins which version of the content
repository this project consumes:

```toml
[context]
packages = ["managed"]

[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.5.1"
```

Changing `[context].packages` after initialisation does not move files around —
the install set is always the full processkit skill catalogue. Run `aibox sync`
after editing `[processkit].version` to pull a new release.

## Design Principles

**Convention over configuration.** File names and locations are standardised
so AI agents can find them without special instructions.

**Human-readable first.** All context files are Markdown. They are useful
without any tooling.

**Editable in place.** Everything under `context/skills/`, `context/processes/`,
and `context/primitives/` is yours to edit. The immutable snapshot under
`context/templates/processkit/<version>/` exists only as the base of
`aibox sync`'s three-way diff.

**No lock-in.** Context files are plain Markdown in a `context/` directory.
Stop using aibox and the files remain useful.

**Clean boundary between container and content.** aibox owns the box;
processkit owns what goes in it. Each ships on its own cadence.
