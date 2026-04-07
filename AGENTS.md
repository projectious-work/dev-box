# AGENTS.md — aibox

Canonical, provider-neutral instructions for any AI agent (or human)
working on this project. Provider-specific files (`CLAUDE.md`, etc.)
are thin pointers to this document.

## What aibox is

**aibox** is a Rust CLI that manages reproducible AI-ready dev
containers. Since v0.16.0 it has a strict scope:

1. **Containers** — generates `.devcontainer/Dockerfile`,
   `docker-compose.yml`, and `devcontainer.json` from `aibox.toml`,
   plus a tool-bundle addon system (`addons/`) and themed
   `.aibox-home/` runtime config seed.
2. **processkit installer** — fetches a pinned release of
   [`projectious-work/processkit`](https://github.com/projectious-work/processkit)
   and installs its skills, primitives, processes, and the canonical
   `AGENTS.md` template into the consuming project under `context/`.

Everything process-related (skills, work-item entities, decision
records, state machines, packages) is **owned by processkit**, not
aibox. The split is the whole point of v0.16.0.

## Critical: `.devcontainer/` vs `images/`

We are in a dev-container building dev-containers. **Never confuse
these two:**

- **`.devcontainer/`** — THIS project's own dev environment
  (Rust + Python/uv + Docusaurus).
- **`images/`** — Published images for OTHER projects (pushed to GHCR).

## Building and Testing

```bash
cd cli && cargo build                     # Build CLI
cd cli && cargo test                      # Run all tests (unit + integration)
cd cli && cargo clippy --all-targets -- -D warnings   # Lint (zero warnings required)
```

## Project structure

| Path             | Owns                                                              |
|------------------|-------------------------------------------------------------------|
| `cli/`           | The Rust CLI (the only shipped artifact besides addon YAMLs)      |
| `addons/`        | YAML addon definitions (python, rust, node, latex, …)             |
| `images/`        | Container image build recipes published to GHCR                   |
| `docs-site/`     | Docusaurus documentation                                          |
| `context/`       | Project context (backlog, decisions, projects, research, …)      |
| `schemas/`       | Embedded context schema versions                                  |

The `context/` directory follows the **product** process template.
Single chronological file → root, multiple files of same kind →
subdirectory.

- `context/BACKLOG.md` — Active task registry (BACK-NNN IDs)
- `context/DECISIONS.md` — Decision log (inverse chronological, DEC-NNN)
- `context/PROJECTS.md` — Project registry (PROJ-NNN)
- `context/PRD.md` — Product requirements document
- `context/OWNER.md` — Profile of the project owner
- `context/work-instructions/` — Reference docs (DEVELOPMENT, GENERAL,
  TEAM, RELEASE-PROCESS, ARCHITECTURE, …)
- `context/research/` — Research reports
- `context/brand/` — Brand package (may migrate to company repo)
- `context/archive/` — Archived items, mirrored structure

## aibox ⇄ processkit boundary

Read this carefully — the boundary is the load-bearing decision of
v0.16.0:

- **aibox owns:** containers, addons, the `[processkit]` config
  section, the install/diff/migrate machinery, the project skeleton at
  init time (`.aibox-version`, `.gitignore`, `context/` directory,
  `CLAUDE.md` thin pointer), and the docs site.
- **processkit owns:** every skill (`SKILL.md`), every primitive
  schema, every state machine, the canonical `AGENTS.md` template,
  the processes (`bug-fix.md`, `code-review.md`, …), and the package
  YAMLs (`packages/{minimal,managed,software,research,product}.yaml`).
- **The user-side `context/` directory** is shared territory: aibox
  creates it, processkit fills it, the user can edit it in place.
  An immutable upstream reference is kept under
  `context/templates/processkit/<version>/` for the three-way diff.

If something process-related is missing, add it to processkit, not
aibox.

## GitHub organization

- **Repo:** `projectious-work/aibox`
- **GHCR:** `ghcr.io/projectious-work/aibox`
- **Docs:** `https://projectious-work.github.io/aibox/`
- **processkit upstream:** `https://github.com/projectious-work/processkit`
