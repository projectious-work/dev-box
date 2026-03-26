# aibox Baseline

> This file is managed by aibox. Do not edit manually — changes will be
> overwritten on `aibox sync`.

## Quick Reference

- **Process packages:** full-product, handover
- **Base image:** debian
- **Add-ons:** ai-claude, docs-docusaurus, node, python
- **CLI version:** 0.13.1

## Session Protocol

1. Check for migration documents in `context/migrations/`. If any exist with
   status "pending", discuss with the user before proceeding with other work.
2. Read this file and any work instructions in `context/work-instructions/`.
3. Follow the process-specific context files listed below.

## Safety Rules

- Never execute migration scripts automatically. Always discuss with the user.
- Never modify aibox.toml without user confirmation.
- Never delete context files.
- Always check `.aibox-version` matches expectations.

## Context Layout

```
# core package
  context/AIBOX.md
  context/OWNER.md
  context/
# tracking package
  context/BACKLOG.md
  context/DECISIONS.md
  context/EVENTLOG.md
  context/archive/BACKLOG.md
  context/archive/DECISIONS.md
  context/
  context/archive/
# standups package
  context/STANDUPS.md
  context/
# handover package
  context/project-notes/session-template.md
  context/project-notes/
  context/archive/project-notes/
# code package
  context/work-instructions/DEVELOPMENT.md
  context/work-instructions/
# architecture package
# design package
# product package
  context/PRD.md
  context/PROJECTS.md
  context/archive/PROJECTS.md
  context/
  context/archive/
# security package
# operations package
  context/work-instructions/TEAM.md
  context/work-instructions/
```
