---
name: team-manager
description: |
  Create and maintain TeamMember entities — humans and named AI personas that participate in the project with persistent identity, personality, and tiered memory. Use when adding a new persistent team-member (human or named AI), managing the name pool, scaffolding memory trees, or exporting/importing a team-member bundle. Replaces the deprecated actor-profile skill (DEC-20260422_0233-SpryTulip).
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-team-manager
    version: "1.0.0"
    created: 2026-04-22T00:00:00Z
    category: processkit
    layer: 1
    replaces:
      - SKILL-actor-profile
    uses:
      - skill: event-log
        purpose: Log events to keep the audit trail accurate after every write.
      - skill: index-management
        purpose: Query existing entities and keep the SQLite index fresh after writes.
      - skill: id-management
        purpose: Allocate unique entity identifiers via central ID generation.
      - skill: role-management
        purpose: Resolve default_role references against the Role catalog.
    provides:
      primitives: [TeamMember]
      mcp_tools:
        - create_team_member
        - get_team_member
        - list_team_members
        - update_team_member
        - deactivate_team_member
        - reactivate_team_member
        - reserve_name
        - release_name
        - list_available_names
        - suggest_name
        - init_memory_tree
        - export_team_member
        - import_team_member
        - check_consistency
        - check_all_consistency
      templates: [team-member-human, team-member-ai-agent]
---

# Team Manager

## Intro

TeamMembers are the persistent participants in the project — humans, named
AI personas, and services — that accumulate identity, personality, and
tiered memory over time. This skill owns their lifecycle: creation, memory
tree scaffolding, a curated international name pool, A2A-compatible Agent
Cards, export/import bundles, and 10 consistency checks.

Unlike ephemeral worker invocations (role+seniority dispatches resolved on
the fly and never persisted), TeamMembers live as directory trees on disk:
`persona.md` + `card.json` + `team-member.md` (entity) + tiered memory
subdirectories + optional `private/` (gitignored).

> **MCP server.** This skill ships a self-contained MCP server at
> `mcp/server.py` (PEP 723 script — requires `uv` and Python ≥ 3.10 on
> PATH). Agent harnesses reach its tools by reading a single MCP config
> file at startup, so the contents of `mcp/mcp-config.json` must be merged
> into the harness's MCP config and placed at the harness-specific path
> before this skill is usable.

## ID convention

TeamMember IDs follow a single shape:

    TEAMMEMBER-<slug>

where `<slug>` is the team-member's canonical kebab-case name (e.g.
`atlas`, `alice-chen`, `thrifty-otter`). The slug starts with a letter and
uses only `[a-z0-9-]`. It is also the directory name under
`context/team-members/<slug>/`.

For `type=ai-agent`, the slug must be drawn from the team-manager name
pool (see **Name pool** below). For `type=human` and `type=service`, any
valid kebab-case slug is accepted.

## TeamMember types

| type         | Example                                     | When to create |
|--------------|---------------------------------------------|----------------|
| `human`      | Project owner, contributors, reviewers      | The first time the person is assigned work, makes a decision, or gets a Binding |
| `ai-agent`   | Atlas, Aria, Kai (named personas)           | A persistent AI persona you want to grow over time — with memory, personality, and accumulated lessons |
| `service`    | GitHub Actions, CI bots, deploy pipelines   | Automated agents that perform repeat actions and need a stable identity for audit trails |

Do **not** promote every mention of a person into a TeamMember. And do
**not** create a named AI persona for a one-off dispatch — use an
ephemeral role+seniority invocation instead.

## Memory tiers

Every team-member directory contains six tier subdirectories under
`context/team-members/<slug>/`:

| Tier       | What lives here | Cadence |
|------------|-----------------|---------|
| `working/` | Task-local scratchpad, active session context | Ephemeral |
| `journal/` | Daily journal, episodic entries | Daily consolidation |
| `knowledge/` | Semantic knowledge, stable facts, domain notes | Weekly promotion from journal |
| `skills/`  | Procedural recipes, working style, patterns | As learned |
| `relations/` | Per-teammate relationship notes (one file per known teammate) | On interaction |
| `lessons/` | Reflected lessons; A-MemGuard-style consensus defense against memory poisoning | Per-task |

Plus `private/` (optional, developer-local, gitignored) and three
top-level files: `persona.md` (identity + personality prompt),
`card.json` (A2A Agent Card), `team-member.md` (the entity itself).

Every memory file under a tier subdirectory must carry a frontmatter
block validated against `assets/memory-file-header.schema.json` —
required keys: `tier`, `source`, `sensitivity`, `created`.

## Name pool & international naming policy

The name pool lives at `data/name-pool.yaml` (managed by this skill).
It contains three gender-bucket lists (feminine, masculine, neutral) of
international, short, human-friendly first names — ~60 total. Names
from the pool are the canonical identifier source for AI personas.

**Reservation lifecycle**

- `suggest_name(kind?)` — pick a random available name, optionally
  filtered by gender bucket. Does not reserve.
- `reserve_name(name, team_member_slug)` — mark a name taken by a
  specific slug. Writes to `data/name-pool.yaml` atomically.
- `release_name(name)` — remove the reservation (e.g. on team-member
  deletion or re-export).
- `list_available_names(kind?)` — enumerate non-reserved names.

For `create_team_member(type="ai-agent", ...)`, the `slug` must
correspond to a name currently in the pool. The tool auto-reserves on
success.

## Consistency checks

`check_consistency(slug)` returns findings for a single team-member;
`check_all_consistency()` aggregates across every team-member.

Each finding is `{severity, code, team_member, path, message}`. Codes:

| Code | Severity | What it checks |
|------|----------|----------------|
| `team.drift.schema` | error | `team-member.yaml` frontmatter validates against `SCHEMA-team-member` |
| `team.drift.tier_missing` | error | Expected subdirs (knowledge, journal, skills, relations, lessons) exist |
| `team.drift.dangling_ref` | error | `default_role` resolves to an existing Role; `relationships[].with` resolves to known slugs/IDs |
| `team.name_collision` | error | No two active team-members share `name` or `slug` |
| `team.name.off_pool` | warning | For `type=ai-agent`, the name must be in the name pool |
| `team.drift.orphan_file` | warning | No files outside expected tier subdirs / persona.md / card.json / team-member.md / private/ |
| `team.sensitivity.leak_risk` | warning | Memory files with `sensitivity: confidential` or `pii` must live under `private/` |
| `team.private.not_ignored` | warning | Each `private/` directory is covered by a `.gitignore` rule |
| `team.memory.bad_header` | warning | Memory files under tier subdirs carry required frontmatter keys |
| `team.card.stale` | warning | `card.json` name/role/seniority match `team-member.yaml` |

## Export / import

`export_team_member(slug, output_path?)` builds a tar.gz bundle at
`<slug>-export-<YYYY-MM-DD>.tar.gz` containing:

- `persona.md`
- `card.json` (A2A Agent Card, conforms to `assets/agent-card.schema.json`)
- `team-member.md` (the entity)
- `knowledge/`, `skills/`, `lessons/`

Excludes `journal/`, `relations/`, `private/` by default. During the
memory-file walk, any file whose frontmatter declares
`sensitivity: confidential` or `sensitivity: pii` is redacted (skipped).

`import_team_member(tarball_path)` extracts to a temp dir, validates the
`team-member.yaml` against `SCHEMA-team-member`, validates `card.json`
against the A2A Agent Card schema, and checks that a `signature` field
is present on the card (crypto verification is deferred — signature
field presence is a hand-off marker for future work). On success it
copies the tree into `context/team-members/<slug>/` and logs
`team_member.imported`.

## Deprecation note

This skill **replaces** `actor-profile` (DEC-20260422_0233-SpryTulip).
The `ACTOR-<slug>` ID class and the `create_actor`/`get_actor`/etc.
tools are deprecated in v0.19.0. A migration on `context/actors/` runs
in a separate phase; do not modify actor-profile from here.

## Gotchas

- **Creating a TeamMember for every mention.** Reserved for persistent
  participants — the same rule as Actor. If they have no upcoming
  WorkItem/Binding/decision, don't promote them.
- **Drawing an AI persona name outside the pool.** `team.name.off_pool`
  will warn. Use `suggest_name` or `list_available_names` first.
- **Putting confidential info outside `private/`.** `team.sensitivity.leak_risk`
  catches it. The tier subdirectories are git-tracked; only `private/`
  is safe for developer-local sensitive context.
- **Hand-editing `data/name-pool.yaml`.** Use `reserve_name`/
  `release_name` — the tool preserves the document frontmatter and
  writes atomically.
- **Forgetting `init_memory_tree` after create.** `create_team_member`
  writes the entity but does not scaffold the tier subdirectories.
  Call `init_memory_tree(slug)` right after (or pass
  `init_memory=True` at create time, if implemented).
- **Putting role+seniority dispatches here.** Ephemeral worker
  invocations are NOT TeamMembers — they are resolved on the fly via
  `task-router`. Only persistent personas live in `team-members/`.
