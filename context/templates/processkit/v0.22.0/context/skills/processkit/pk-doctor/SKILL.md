---
name: pk-doctor
description: |
  Aggregator health-check for a processkit-managed repository. Runs a fixed suite of detect-only checks over the live context/ tree plus src/context/ drift and surfaces ERROR / WARN / INFO findings in a single report. Use when the user invokes `/pk-doctor`, asks "is this repo healthy", or has just finished an upgrade and wants a sanity pass. Detect-only by default; `--fix` / `--fix-all` opt in to scoped repairs that route through existing MCP write tools only — doctor never hand-edits files under `context/`.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-pk-doctor
    version: "1.0.0"
    created: 2026-04-20T00:00:00Z
    category: processkit
    layer: 3
    uses:
      - skill: event-log
        purpose: Emit the single `doctor.report` LogEntry per run (the audit trail for every invocation).
      - skill: migration-management
        purpose: List pending migrations for the `migrations` check and apply them under `--fix`.
      - skill: index-management
        purpose: (future) surface index errors alongside doctor findings in Phase 2.
    commands:
      - name: pk-doctor
        args: "[--category=...] [--fix=... | --fix-all] [--since=<ref>] [--yes]"
        description: "Run the health-check aggregator and print a single report."
---

# pk-doctor

## Intro

`/pk-doctor` is the single user-visible health-check surface for a
processkit repository. It is the diagnostic you run after an upgrade,
before a release, or any time you need to answer the question "is this
repo healthy?" in one command. The skill pattern is deliberately modelled
on `npm doctor`, `brew doctor`, and `rustup doctor`: detect by default,
fix only when explicitly asked, and keep the report skimmable.

This skill is the Phase 1 landing. It ships four checks. Phase 2 and
Phase 3 add more checks, lift the aggregator into an MCP server, and
harden the `--fix` paths — each with their own WorkItem.

## Overview

### Invocation

```
/pk-doctor                              # detect-only; all 4 checks; report + log
/pk-doctor --category=schemas,migrations
/pk-doctor --fix=migrations             # opt-in scoped fix
/pk-doctor --fix-all                    # every fix path enabled
/pk-doctor --since=v0.18.2              # scope file-walk checks only
/pk-doctor --yes                        # non-interactive (auto-confirms safe fixes)
```

`--fix` and `--fix-all` are mutually exclusive. Exit code is `0` if no
ERRORs were produced, `1` otherwise. Every run — regardless of outcome —
writes exactly one `doctor.report` LogEntry via the event-log MCP.

### The four Phase 1 checks

1. **`schema_filename`** — walks every `context/<kind>/**/*.md`. Loads
   the matching schema from `src/context/schemas/<kind>.yaml` (the LIVE
   source, never the template mirror) and validates the file's frontmatter
   against it. Also checks that the filename stem equals `metadata.id`
   and that any date encoded in the filename matches `metadata.created`.
   Phase 1 is WARN-only for rename suggestions — no auto-fix.
2. **`sharding`** — logs must live under `context/logs/YYYY/MM/` subdirs;
   migrations must live under their `spec.state` subdir (`pending/`,
   `in-progress/`, `applied/`). WARN on any file in the wrong bucket.
3. **`migrations`** — calls
   `mcp__processkit-migration-management__list_migrations(state="pending")`.
   INFO for the pending count; WARN for any pending entry older than
   14 days. Under `--fix=migrations` or `--fix-all`, interactively
   prompts `Apply MIG-xxx? [y/N/s]` per entry and invokes
   `apply_migration` on yes.
4. **`drift`** — subprocess-invokes `scripts/check-src-context-drift.sh`
   from the repo root. Exit 0 → one INFO "trees in sync". Exit 1 →
   one WARN per offending line reported by the script.
5. **`commands_consistency`** — walks every
   `context/skills/processkit/*/SKILL.md` and verifies that each entry
   in `metadata.processkit.commands:` has a matching `commands/<name>.md`
   file alongside the SKILL.md. ERROR per missing file; WARN for
   stray `commands/*.md` files not declared in the metadata. Added
   after v0.19.1 shipped pk-doctor without its own commands/ directory.

Additional checks added after Phase 1:

- **`team_consistency`** — wraps `team-manager.check_all()`.
- **`release_integrity`** — see DEC-20260422_1348-SnowyWolf-local-only-release-bulletproofing.
- **`mcp_config_drift`** — reads `context/.processkit-mcp-manifest.json`
  (produced at release time by `scripts/generate-mcp-manifest.py`) and
  recomputes the per-skill `mcp-config.json` sha256es. A missing manifest
  or stale aggregate surfaces as WARN (run the generator). In a
  derived-project context (`aibox.lock` + `.mcp.json` both present at
  the repo root), any processkit server missing from `.mcp.json`'s
  `mcpServers` map surfaces as ERROR with a "run `aibox sync` to
  re-merge" hint. Exists because `aibox sync` currently gates
  `.mcp.json` re-merge on processkit version delta only, so per-skill
  MCP-config edits within a release cycle never reach derived projects
  until the next version bump — see DEC-20260423_2049-VastLake and
  projectious-work/aibox#54.
- **`server_header_drift`** — walks every `context/skills/processkit/*/mcp/server.py`,
  hashes its PEP 723 inline metadata block (`# /// script` ... `# ///`),
  and compares against the manifest's `per_server_header` baseline. WARN
  on any drift listing the affected skill slugs — signal that someone
  edited a dep header without regenerating the manifest, and the harness
  needs a restart to pick up the new deps. Detect-only; the fix is
  user-driven (regenerate manifest + restart harness). Per
  DEC-20260424_0127-QuickPine (SharpBrook split, RapidSwan ships the
  dep-drift half).

### What doctor will NEVER do

- Hand-edit any file under `context/`. All writes route through the
  appropriate MCP tool (e.g. `apply_migration`).
- Touch `context/templates/` (read-only diff baseline).
- Touch `.mcp.json` (merged file; pk-doctor has no MCP server in Phase 1).
- Write to `context/logs/` directly — reports go through
  `mcp__processkit-event-log__log_event`.
- Run fixes without an explicit `--fix=<category>` or `--fix-all` flag.

### Report shape

After the stdout human-readable summary, the skill emits a single
`doctor.report` LogEntry:

```json
{
  "event_type": "doctor.report",
  "summary": "/pk-doctor --fix=migrations — 0 ERROR / 2 WARN / 3 INFO",
  "details": {
    "doctor_version": "1.0.0",
    "invocation": "/pk-doctor --fix=migrations",
    "categories": {
      "schema_filename": {"ERROR": 0, "WARN": 1, "INFO": 140},
      "sharding": {"ERROR": 0, "WARN": 0, "INFO": 1},
      "migrations": {"ERROR": 0, "WARN": 1, "INFO": 1},
      "drift": {"ERROR": 0, "WARN": 0, "INFO": 1}
    },
    "top_findings": [
      {"severity": "WARN", "id": "migration.stale-pending", "entity_ref": "MIG-...", "message": "..."}
    ],
    "fixes_applied": [],
    "duration_ms": 2340
  }
}
```

`top_findings` caps at 20 to keep the log entry compact; the full
stdout report remains authoritative for the details of a specific run.

### Interactive prompts

Interactive prompts gate on `sys.stdin.isatty()`. If a `--fix` was
requested but the process isn't attached to a terminal, the skill emits
a `WARN` (`fix.non-interactive`) and skips the fix — it does not silently
prompt and hang. `--yes` auto-confirms every prompt except any marked
`data_loss=True` (none in Phase 1, but the hook is present).

### `--since` scope

`--since=<git-ref>` restricts only the file-walk checks
(`schema_filename`, `sharding`). The `migrations` and `drift` checks
always run full-scan: migrations pull from an MCP tool with its own
state, and drift compares entire trees.

### Resolved policy defaults

These were agreed with the owner during shape review:

- **YAML-datetime vs JSON-Schema string coercion** failures surface as
  `WARN schema.datetime-coercion` (not ERROR). Parser-layer quirk.
- **Slash command location** — `.claude/commands/pk-doctor.md` only.
  Not mirrored to `src/.claude/commands/`. Downstream aibox ships its
  own slash commands.
- **Filename auto-rename** — WARN only in Phase 1. Phase 2 adds
  orchestrated rename + reindex.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Running doctor with implicit fixes.** Detect-only by default. An
  agent that invokes doctor with `--fix-all` "to be safe" without
  owner approval is the opposite of safe: it applies migrations,
  potentially across unrelated upgrade cycles, with no per-entry
  review. Always confirm scope with the owner before any `--fix`.
- **Treating drift warnings as noise.** The drift check exists because
  v0.15.0–v0.18.0 shipped four releases with silent drift. If the
  drift check produces a WARN, it has found a real divergence —
  do not dismiss without investigating.
- **Hand-editing files to "clear" a doctor warning.** Doctor warnings
  are diagnostic. Fixing the underlying cause through the right MCP
  write tool (not by editing the offending file directly) preserves
  the audit trail. A WARN that reappears next run is more useful than
  a hand-fixed tree with no record of the fix.
- **Validating against `context/templates/` schemas.** The template
  mirror is a diff baseline, not the live schema source. Validation
  must read `src/context/schemas/<kind>.yaml` — the authoritative
  copy. Validating against templates will produce stale false
  positives after any schema evolution.
- **Forgetting the `doctor.report` LogEntry on dry-run.** Even a
  detect-only run must emit the report: it is the audit record of the
  invocation. A run that prints to stdout but logs nothing strands
  the next session with no record of what was checked.
- **Running doctor mid-migration.** Doctor's `migrations` check will
  list pending migrations and (under `--fix`) offer to apply them. If
  you are already in the middle of a manual migration review, running
  `--fix=migrations` can reorder work silently. Finish or pause the
  manual review first.

## Full reference

### CLI contract

```
doctor.py [--category=LIST] [--fix=LIST | --fix-all] [--since=REF] [--yes]
```

| Flag           | Meaning |
|----------------|---------|
| `--category`   | Comma list of categories to run. Default: all four. |
| `--fix`        | Comma list of categories to enable fixes for. Mutex with `--fix-all`. |
| `--fix-all`    | Enable fixes for every category that supports them. Mutex with `--fix`. |
| `--since`      | Git ref; restricts `schema_filename` + `sharding` to files changed since. |
| `--yes`        | Auto-confirm all non-`data_loss` fix prompts. |

### Adding a check in Phase 2

1. Drop a new module in `scripts/checks/<name>.py` exporting a
   `run(ctx) -> list[CheckResult]` function (and optional
   `run_fix(ctx, results) -> list[dict]`).
2. Register it in `scripts/checks/__init__.py` in `REGISTRY`.
3. Add a gotcha for that check here in SKILL.md.
4. Mirror both files into `src/context/skills/processkit/pk-doctor/`.

The aggregator in `doctor.py` does not need to change.

### What this skill does NOT do (yet)

- Validate cross-references (Phase 2).
- Check `PROVENANCE.toml` drift (Phase 2).
- Check `mcp-config.json` integrity (Phase 2).
- Smoke-import MCP servers (Phase 2).
- Detect version skew between `aibox.lock` and installed SKILL.md pins
  (Phase 2).
- Expose itself as an MCP server for machine invocation (Phase 3, only
  if demand shows up).
