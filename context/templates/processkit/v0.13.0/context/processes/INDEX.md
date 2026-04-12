# processes/

Process definitions — declarative descriptions of workflows
(code-review, release, bug-fix, ...). Processes declare WHAT happens
and in what order; skills provide HOW to do each step; context
artifacts store the results.

Each process is a Markdown file with YAML frontmatter describing:

- steps (ordered list referencing skills, with optional gates)
- roles (which actors participate)
- gates (validation points that must pass before a step completes)
- definition_of_done (the acceptance criterion for the whole process)

Processes are thin on purpose — the actual execution is delegated to
skills, agents, and humans. processkit does not ship a workflow
engine.

## Shipped processes (v0.5.0)

| File | Process | Purpose |
|---|---|---|
| `bug-fix.md` | `PROC-bug-fix` | Diagnose, reproduce, fix, and verify a defect |
| `code-review.md` | `PROC-code-review` | Review a change before it merges |
| `feature-development.md` | `PROC-feature-development` | Take a feature from backlog to merged |
| `release.md` | `PROC-release` | Cut, tag, and publish a versioned release |

Each process is a `kind: Process` entity validated by
`src/primitives/schemas/process.yaml`.

## Install paths

When aibox installs processkit into a target project, files in this
directory are copied verbatim to `<target>/context/processes/`. They
are then loaded as Process entities by the index server and may be
queried via `query_entities(kind="Process")`.

## Customizing in a consumer project

Consumers may override a shipped process by editing the installed copy
under `<target>/context/processes/<name>.md`. The next `aibox sync`
will detect the local change and surface it as a 3-way diff against
the upstream reference template.
