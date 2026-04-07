---
sidebar_position: 2
title: "Process Packages"
---

# Process Packages

Process packages are **upstream-defined skill bundles** owned by
[processkit](https://github.com/projectious-work/processkit). They live as YAML
files under `packages/{minimal,managed,software,research,product}.yaml` in the
processkit repository and compose via an `extends:` field, so each package can
build on the one below it without restating its contents.

aibox itself does not define packages anymore — it only consumes them. The
package you pick in `aibox.toml`'s `[context].packages` is forwarded to
processkit content and read by AI agents to decide which skills are *relevant*
for the project.

```toml
[context]
packages = ["managed"]   # or "minimal", "software", "research", "product"
```

> **Heads-up:** in **v0.16.0**, package selection is declarative metadata only.
> aibox installs every processkit skill into `context/skills/` regardless of the
> selected package. The package list is what agents read to filter; it does not
> change which files land on disk. Future releases may make installation
> package-aware — do not depend on "this skill is not installed" as a guarantee.

## The Five Packages

| Package | Extends | Adds | Best for |
|---------|---------|------|----------|
| `minimal` | — | Bare minimum: agent management, owner profile | Scripts, experiments, small utilities |
| `managed` | `minimal` | Backlog, decisions, standups, session handover, archiving | The recommended default for most projects |
| `software` | `managed` | Code review, testing, debugging, refactoring, TDD, error handling, git workflow, integration testing, software architecture | Software projects with a recurring build/test/review cycle |
| `research` | `managed` | Data science, data visualisation, feature engineering, documentation, LaTeX | Learning, documentation, academic work |
| `product` | `software` | Design, infographics, security, secure coding, threat modelling, secrets, dependency audit, CI/CD, container orchestration, logging, metrics, alerting, incident response, performance profiling, estimation, retrospectives, PRD/PROJECTS skills | Full product development |

The exact composition is defined upstream and may evolve from release to
release. For the canonical, version-pinned source of truth, look at:

- `https://github.com/projectious-work/processkit/tree/main/src/packages` (HEAD)
- `context/templates/processkit/<version>/packages/` in your project (after
  `aibox sync`)

## Where the Content Lands

After `aibox init` and `aibox sync` with `[processkit].version` pinned:

```
context/
├── skills/                              # Editable copies of every processkit skill
│   ├── code-review/SKILL.md
│   ├── backlog-context/SKILL.md
│   └── ... (108 skills in v0.5.1)
├── processes/                           # release, code-review, feature-development, bug-fix
├── primitives/
│   ├── schemas/
│   └── state-machines/
└── templates/
    └── processkit/
        └── v0.5.1/
            ├── skills/
            ├── packages/                # The package YAMLs themselves
            │   ├── minimal.yaml
            │   ├── managed.yaml
            │   ├── software.yaml
            │   ├── research.yaml
            │   └── product.yaml
            ├── processes/
            ├── primitives/
            └── scaffolding/
                └── AGENTS.md            # The canonical entry-point template
```

The version path (`v0.5.1` above) is whatever `[processkit].version` is pinned
to in `aibox.toml`.

## Single-File vs Entity-Sharded Tracks

Every processkit release ships **two tracks** for the same conceptual artefact:

| Track | Examples | Storage |
|-------|----------|---------|
| **Single-file** | `backlog-context`, `decisions-adr`, `standup-context`, `session-handover`, `context-archiving` | One Markdown file per artefact (`context/BACKLOG.md`, `context/DECISIONS.md`, …). The skill creates the file in place on first use — there is **no** starter template. |
| **Entity-sharded** | `workitem-management`, `decision-record`, `scope-management`, … | Per-item YAML files with IDs, slugs, and state machines. Backed by an MCP server. |

Both tracks are installed in every project, regardless of selected package.
Pick the one that fits the project; nothing forces you to use one over the
other.

## Changing Packages

You can change the package list at any time by editing `aibox.toml`:

```toml
[context]
packages = ["product"]   # was ["managed"]
```

Then run `aibox sync`. Because all processkit skills are already on disk, no
file movement happens — only the metadata changes. Agents will start preferring
the broader skill set on their next session.

## Upstream Source

The package YAMLs and the skills they reference are owned by processkit. To
inspect or fork them:

- Repository: https://github.com/projectious-work/processkit
- Releases: https://github.com/projectious-work/processkit/releases
- Local copy in your project: `context/templates/processkit/<version>/packages/`

To consume a fork or a private mirror, point `[processkit].source` at it (see
[`[processkit]` configuration](../reference/configuration.md#processkit)).
