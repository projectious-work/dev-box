---
sidebar_position: 3
title: "Migration"
---

# Migration

When the aibox context schema evolves between versions, existing projects may need to update their context files. The `aibox doctor` command helps identify gaps and produces migration artifacts.

:::warning v0.16.0 — `context/AIBOX.md` is gone

Pre-v0.16 releases generated a `context/AIBOX.md` "universal baseline" file
on every `aibox sync`. That file has been **removed** as part of the
aibox⇄processkit split. The canonical agent entry document is now `AGENTS.md`
at the project root, owned by processkit and rendered at `aibox init` time
(write-if-missing — never overwritten).

Existing projects upgrading to v0.16.0 can safely delete `context/AIBOX.md`.
Anything you wrote into it by hand should be moved into `AGENTS.md`,
`context/DECISIONS.md`, or one of the work-instructions files, depending on
its nature.

The `[skills]` section in `aibox.toml` still parses but is **reserved / no-op**
in v0.16.0. Every project gets every processkit skill installed under
`context/skills/` regardless of `include`/`exclude`.

:::

## How Version Tracking Works

Two pieces track the version:

1. **`aibox.toml`** contains the target schema version:
   ```toml
   [context]
   schema_version = "1.0.0"
   ```

2. **`.aibox-version`** in the project root records the version that was last applied. This file is created during `aibox init` and updated after successful migrations.

When `aibox doctor` detects a mismatch between these two values, it flags the project as needing migration.

## Running Doctor

```bash
aibox doctor
```

Doctor performs the following checks:

- Validates `aibox.toml` syntax and field values
- Detects the container runtime (podman or docker)
- Checks for `.aibox-home/` and `.devcontainer/` directories
- Compares `.aibox-version` against `context.schema_version`
- Validates that expected context files exist for the chosen process flavor

Example output when migration is needed:

```
==> Running diagnostics...
 ✓ Config version: 0.1.0
 ✓ Image: python
 ✓ Process: product
 ✓ Container name: my-app
 ✓ Container runtime: podman
 ✓ .aibox-home/ directory exists at .aibox-home
 ✓ .devcontainer/ directory exists
 ! Schema version mismatch: .aibox-version says 0.9.0, config says 1.0.0
 ✓ Diagnostics complete
```

## Migration Artifacts

When a version mismatch is detected, `doctor` generates files in `.aibox/migration/`:

```
.aibox/
└── migration/
    ├── schema-diff.md        # What changed between versions
    ├── migration-prompt.md   # Instructions for an AI agent to apply the migration
    └── checklist.md          # Manual checklist of required changes
```

### schema-diff.md

Documents the differences between the old and new schema versions:

- New files that should be created
- Files that have been renamed or moved
- Structural changes to existing files
- Removed files (if any)

### migration-prompt.md

A ready-to-use prompt for an AI agent (like Claude Code) that describes exactly what needs to change. You can paste this into a conversation to have the agent apply the migration:

```markdown
The project needs to migrate from context schema v0.9.0 to v1.0.0.

Changes required:
1. Create context/work-instructions/TEAM.md with the following template: ...
2. Rename context/NOTES.md to context/project-notes/README.md
3. Add the `schema_version` field to the [context] section of aibox.toml
```

### checklist.md

A human-readable checklist for manual migration:

```markdown
- [ ] Create context/work-instructions/TEAM.md
- [ ] Move context/NOTES.md to context/project-notes/README.md
- [ ] Update [context] section in aibox.toml
- [ ] Update .aibox-version to 1.0.0
```

## Applying a Migration

### With an AI agent (recommended)

1. Run `aibox doctor` to generate migration artifacts
2. Open `.aibox/migration/migration-prompt.md`
3. Paste its contents into a Claude Code session
4. Review the changes the agent makes
5. Update `.aibox-version` to the new version

### Manually

1. Run `aibox doctor` to generate migration artifacts
2. Follow `.aibox/migration/checklist.md`
3. Update `.aibox-version` to the new version

:::warning Review before applying

Migration artifacts describe structural changes. They do not migrate content. If a file is renamed, the artifact tells you to create the new file -- but you need to move the content yourself (or have an AI agent do it thoughtfully).

:::

## Best Practices

**Never auto-migrate content.** Structural changes (new files, renames) can be automated. Content changes (rewriting sections, reformatting entries) should always be reviewed by a human or guided AI session.

**Commit before migrating.** Always commit your current state before applying migration changes. This gives you a clean rollback point.

**Run doctor after migrating.** After applying changes, run `aibox doctor` again to confirm everything is clean.

**Keep .aibox-version in version control.** This file should be committed so all team members know which schema version the project uses.

## Schema Document Format

Schema documents in the `schemas/` directory define the expected structure for each version. They specify:

- Which files each process flavor should contain
- Required sections within each file
- File naming conventions
- Directory structure requirements

These schemas are used by `doctor` to validate the project and by migration tooling to compute diffs between versions.
