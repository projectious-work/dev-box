---
sidebar_position: 1
title: "Skills (via processkit)"
---

# Skills

As of aibox **v0.16.0**, skills are no longer bundled with aibox. They live in
**[processkit](https://github.com/projectious-work/processkit)** — a separate,
versioned content repository that ships skills, primitives, processes, packages,
and the canonical `AGENTS.md` template.

aibox owns the **container** (devcontainers, addons, the CLI, the install/sync
machinery). processkit owns the **content** (skills, packages, processes,
state machines). The boundary is deliberate and load-bearing: it lets the two
projects move at their own velocity without dragging each other through breaking
changes.

## Where to find skills

After running `aibox init` and `aibox sync` in your project, processkit content
is materialised under your `context/` directory:

```
context/
├── skills/                          # Active, editable skill copies
│   ├── code-review/SKILL.md
│   ├── backlog-context/SKILL.md
│   ├── decisions-adr/SKILL.md
│   └── ... (108 skills total in v0.5.1)
└── templates/
    └── processkit/
        └── v0.5.1/                  # Immutable upstream snapshot, git-tracked
            ├── skills/
            ├── packages/            # minimal.yaml, managed.yaml, software.yaml,
            │                        # research.yaml, product.yaml
            ├── primitives/
            ├── processes/
            └── scaffolding/
                └── AGENTS.md
```

The version in the path (`v0.5.1`) is whatever you pinned in `aibox.toml`:

```toml
[processkit]
source  = "https://github.com/projectious-work/processkit.git"
version = "v0.5.1"
```

The `context/skills/` copies are yours to edit. The `context/templates/processkit/<version>/`
copies are the immutable upstream snapshot — `aibox sync` uses them as the base
side of a three-way diff to detect upstream changes that should be pulled into
your local edits.

## Browsing skills

The full skill catalogue lives upstream:

- **Source:** https://github.com/projectious-work/processkit/tree/main/src/skills
- **Releases:** https://github.com/projectious-work/processkit/releases
- **In your project, after `aibox sync`:** `context/skills/` and
  `context/templates/processkit/<version>/skills/`

Every skill is a directory with at least a `SKILL.md` (the agent-readable
instructions) and may include `references/`, `mcp/`, and `templates/` siblings.
Skills follow the open [Agent Skills specification](https://agentskills.io/specification).

## Two tracks for the same artefact

processkit deliberately ships **two tracks** for context-management skills, and
both are installed into every project regardless of selected package:

| Track | Examples | How it works |
|-------|----------|--------------|
| **Single-file** | `backlog-context`, `decisions-adr`, `standup-context`, `session-handover`, `context-archiving` | The skill instructs the agent to create and maintain a single Markdown file in place (e.g. `context/BACKLOG.md`). No starter template — the file is born when the agent first writes to it. |
| **Entity-sharded** | `workitem-management`, `decision-record`, `scope-management`, … | Per-item YAML files with IDs, slugs, and state machines. Backed by an MCP server. |

You pick the track that fits the project; both tracks coexist peacefully and
nothing forces you to use one over the other.

## Packages

Five processkit packages compose skill sets via `extends:` in upstream YAML:

| Package | Purpose |
|---------|---------|
| `minimal` | Bare-minimum skill set for scripts and experiments |
| `managed` | Recommended default — backlog, decisions, standups, handover |
| `software` | `managed` + code review, testing, debugging, refactoring, architecture |
| `research` | `managed` + data science, documentation, research artefacts |
| `product` | Everything — `software` + design, security, operations, product planning |

You select packages declaratively in `aibox.toml`:

```toml
[context]
packages = ["managed"]   # or ["software"], ["research"], ["product"], ["minimal"]
```

In **v0.16.0**, package selection is metadata that agents read to decide which
skills are *relevant* for the project — but aibox installs **all** processkit
skills regardless. The full set is always available; the package list tells
agents which subset to prefer. The package YAMLs themselves live in
`context/templates/processkit/<version>/packages/`.

## Custom skills

To add a project-specific skill, drop a directory under `context/skills/`:

```
context/skills/my-custom-skill/
└── SKILL.md
```

Local skills are not touched by `aibox sync`. They are also not part of any
processkit package — they exist purely for the local project.

## Why this split?

- **Independent release cadence.** processkit can ship a new skill or fix a
  prompt without forcing an aibox CLI release.
- **Reusable content.** Other tools can consume processkit directly without
  taking a dependency on aibox or its container stack.
- **Forkable content.** A team can fork processkit, point `[processkit].source`
  at the fork, and ship a private skill catalogue without forking aibox itself.
- **Smaller aibox.** The aibox binary stays focused on container lifecycle and
  the install/diff/migrate machinery.

See [`[processkit]` configuration](../reference/configuration.md#processkit) for
the full set of fields, including release-asset URL templates and SHA256
verification.
