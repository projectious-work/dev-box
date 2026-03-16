# Migration

When the dev-box context schema evolves between versions, existing projects may need to update their context files. The `dev-box doctor` command helps identify gaps and produces migration artifacts.

## How Version Tracking Works

Two pieces track the version:

1. **`dev-box.toml`** contains the target schema version:
   ```toml
   [context]
   schema_version = "1.0.0"
   ```

2. **`.dev-box-version`** in the project root records the version that was last applied. This file is created during `dev-box init` and updated after successful migrations.

When `dev-box doctor` detects a mismatch between these two values, it flags the project as needing migration.

## Running Doctor

```bash
dev-box doctor
```

Doctor performs the following checks:

- Validates `dev-box.toml` syntax and field values
- Detects the container runtime (podman or docker)
- Checks for `.root/` and `.devcontainer/` directories
- Compares `.dev-box-version` against `context.schema_version`
- Validates that expected context files exist for the chosen process flavor

Example output when migration is needed:

```
==> Running diagnostics...
 ✓ Config version: 0.1.0
 ✓ Image: python
 ✓ Process: product
 ✓ Container name: my-app
 ✓ Container runtime: podman
 ✓ .root/ directory exists at .root
 ✓ .devcontainer/ directory exists
 ! Schema version mismatch: .dev-box-version says 0.9.0, config says 1.0.0
 ✓ Diagnostics complete
```

## Migration Artifacts

When a version mismatch is detected, `doctor` generates files in `.dev-box/migration/`:

```
.dev-box/
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
3. Add the `schema_version` field to the [context] section of dev-box.toml
```

### checklist.md

A human-readable checklist for manual migration:

```markdown
- [ ] Create context/work-instructions/TEAM.md
- [ ] Move context/NOTES.md to context/project-notes/README.md
- [ ] Update [context] section in dev-box.toml
- [ ] Update .dev-box-version to 1.0.0
```

## Applying a Migration

### With an AI agent (recommended)

1. Run `dev-box doctor` to generate migration artifacts
2. Open `.dev-box/migration/migration-prompt.md`
3. Paste its contents into a Claude Code session
4. Review the changes the agent makes
5. Update `.dev-box-version` to the new version

### Manually

1. Run `dev-box doctor` to generate migration artifacts
2. Follow `.dev-box/migration/checklist.md`
3. Update `.dev-box-version` to the new version

!!! warning "Review before applying"
    Migration artifacts describe structural changes. They do not migrate content. If a file is renamed, the artifact tells you to create the new file -- but you need to move the content yourself (or have an AI agent do it thoughtfully).

## Best Practices

**Never auto-migrate content.** Structural changes (new files, renames) can be automated. Content changes (rewriting sections, reformatting entries) should always be reviewed by a human or guided AI session.

**Commit before migrating.** Always commit your current state before applying migration changes. This gives you a clean rollback point.

**Run doctor after migrating.** After applying changes, run `dev-box doctor` again to confirm everything is clean.

**Keep .dev-box-version in version control.** This file should be committed so all team members know which schema version the project uses.

## Schema Document Format

Schema documents in the `schemas/` directory define the expected structure for each version. They specify:

- Which files each process flavor should contain
- Required sections within each file
- File naming conventions
- Directory structure requirements

These schemas are used by `doctor` to validate the project and by migration tooling to compute diffs between versions.
