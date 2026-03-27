---
id: DISC-001
title: Context System Redesign — Process Ontology, Storage Architecture, Scaling
status: active
date: 2026-03-27
participants: [owner, claude]
related: [BACK-097, BACK-082, BACK-090, BACK-091, BACK-099]
research:
  - context/research/process-ontology-primitives-2026-03.md
  - context/research/context-database-architecture-2026-03.md
  - context/research/process-frameworks-research.md
  - context/research/rag-context-layer-2026-03.md
  - context/research/event-log-design-2026-03.md
  - context/research/document-structure-audit-2026-03.md
  - context/research/issue-handling-design-2026-03.md
  - context/research/file-per-entity-scaling-2026-03.md
  - context/research/competitive-landscape-2026-03.md
  - context/research/competitive-tools-2026-03.md
---

# DISC-001: Context System Redesign

## 1. Problem Statement

aibox's context system uses markdown tables (BACKLOG.md, DECISIONS.md, PROJECTS.md) as
the source of truth for structured data. This approach has reached its limits:
- Editing structured records in markdown tables is fragile and error-prone
- A single BACKLOG.md file with 70+ rows creates merge conflicts for collaborators
- No efficient query/search capability (grep is the only option)
- Growing research corpus (27+ reports) is hard to navigate
- No way to enforce state machines, validate references, or compute metrics

Meanwhile, process framework research (SAFe, PMBOK, CMMI, IPMA) reveals that all
frameworks share universal primitives — and aibox should provide these as building blocks.

## 2. Train of Thought

### 2.1 Starting point: Process model retrospective (BACK-097)

We began by asking: do our process templates (minimal/managed/research/product) still
make sense? This led to the deeper question: what are the universal building blocks that
ALL process frameworks share?

### 2.2 Process ontology discovery

Research identified **15 universal process primitives** that appear across 7 domains
(software, manufacturing, healthcare, legal, supply chain, knowledge management, quality):

1. Work Item — unit of work (task/story/issue/ticket/card/case)
2. Log Entry / Event — immutable record of something that happened
3. Decision Record — choice with rationale
4. Artifact — any produced output
5. Role — named set of responsibilities
6. Process / Workflow — sequence of steps with decision points
7. State Machine — set of states with allowed transitions
8. Category / Taxonomy — classification system
9. Cross-Reference / Relation — typed link between entities
10. Checkpoint / Gate — validation point
11. Metric / Measure — quantified observation
12. Schedule / Cadence — time-based trigger
13. Scope / Container — boundary grouping related items
14. Constraint — rules/limits that restrict degrees of freedom
15. Context / Environment — ambient knowledge and conditions

**Key insight:** Every primitive shares the same meta-structure: identifier, name,
description, state, timestamps, owner, categories, cross-references. This is a
universal schema.

### 2.3 Storage architecture debate

**Question:** Should structured data move from markdown tables to a database?

**Owner's core principle:** Data must have ONE source of truth. Dual-master never works —
it leads to divergence, contradictions, merge conflicts, and becomes impossible to manage.

**Arguments for database (SQLite):**
- Efficient queries (SQL vs parsing markdown)
- RAG integration (vector embeddings in same storage)
- State machine enforcement
- Cross-reference validation
- Metrics computation

**Arguments against database:**
- NOT git-native (binary blob, no meaningful diffs)
- Not human-readable without tooling
- Single source of truth principle violated if both markdown and DB exist
- Git delta compression doesn't work well for SQLite (page-based format)
- Size estimate: even small project database could reach 5-50MB per commit

**Git and binary files — the facts** (from `context-database-architecture-2026-03.md`):
Git does NOT delta-compress binary files efficiently in loose object storage. Packfiles
do apply binary delta compression, but SQLite's page-based format means even small changes
shuffle many pages — deltas are large. A 5MB SQLite file changed 100 times could consume
200-500MB of git history. SQLite as committed source of truth is not viable.

**Arguments for markdown+frontmatter (file-per-entity):**
- Git-native (perfect diffs, blame, merge)
- Human-readable
- Flexible schema via YAML `custom:` field
- Already proven pattern (SKILL.md files)
- Each entity = own file → minimal merge conflicts
- Single source of truth = the .md file

**NoSQL / document store exploration** (owner-initiated):
Owner asked: would NoSQL be better than SQL for flexible schemas (user-defined fields)?
Jira uses EAV (Entity-Attribute-Value) for custom fields — flexible but notoriously slow
at scale. Options researched:
- JSON-per-entity files (git-native, but less readable than markdown)
- TinyDB, LowDB, UnQLite (embedded document stores — not git-native)
- SurrealDB embedded (multi-model, Rust-native — promising but immature)
- SQLite with JSON columns (SQL + flexible fields — good query, bad git)
- Markdown+frontmatter with `custom:` map (best of all worlds)

**Jira comparison:** Jira's power is per-issue-type field configuration and configurable
workflows (state machines). Its pain is vendor lock-in and performance at scale. Linear
solved this by being opinionated with fewer custom fields but faster queries. Our
markdown+frontmatter approach gets Jira's flexibility via the `custom:` YAML map without
the EAV performance tax.

**Resolution:** Markdown+frontmatter as source of truth, SQLite as DERIVED runtime
index (gitignored). Rebuilt on `aibox sync`. This gives git-native storage + fast
queries without dual-master problems.

**Two kinds of content identified:**
- **Narrative content** stays as markdown body: research reports, decision rationale,
  work instructions, session notes, SKILL.md instructions
- **Structured records** get YAML frontmatter: IDs, states, priorities, dates, categories,
  cross-references, custom fields

### 2.4 Scaling concerns

**The scaling question:** Can file-per-entity scale to very large projects? This matters
because aibox is intended as the BASIS for kaits (multi-agent company simulator), which
could generate many thousands of artifacts per project.

**Current estimate (needs validation):**

| Scale | Files | Git | Filesystem | Index rebuild |
|-------|-------|-----|------------|---------------|
| Small (<1K) | trivial | trivial | trivial | <1s |
| Medium (1K-10K) | fine | fine | needs subdirs | 1-10s |
| Large (10K-50K) | slow status | needs sharding | slow | 10-60s |
| Very large (50K+) | breaks | breaks | breaks | minutes |

**Concern (owner):** 50K files already being "shaky" is worrying. kaits could easily
generate 50K+ artifacts for a single simulated company. We can't just throw this problem
over the fence to kaits — if aibox is the basis, aibox needs to handle it.

**Hot/cold archiving:** Move completed/old items to compressed archives. Reduces active
file count. But how far does this scale? Research needed.

**Open questions:**
- Is there a middle ground between "all files" and "all database"?
- Could sharding (year/month subdirs) + hot/cold push the limit to 100K+?
- What do large git monorepos actually do? (Google, Microsoft — but they use custom VFS)
- Could git sparse checkout help? (only check out active items)
- Should kaits use a different storage layer entirely while maintaining aibox-compatible import/export?

### 2.5 ID generation

**Options discussed:**
- Sequential (BACK-001, BACK-002) — current approach, simple, human-friendly
- UUID-based (BACK-a7f3b2c1) — no coordination needed, scales to multi-collaborator
- Prefix-based (BACK-BG-042) — brittle, not scalable
- Lock file — doesn't scale well for concurrent collaborators

**Leaning:** UUID-based. Owner can live with `BACK-a7f3b2c1` format. Lock files and
prefixes are brittle. Sequential is nice for small teams but breaks with concurrent
contributors.

**Alternative:** `aibox id --type backlog` as a central ID generator. But this is just
a local sequential counter — same coordination problem in distributed setting.

**Decision pending.** Need to resolve as part of the storage architecture.

### 2.6 Mapping primitives to storage

**Not yet done.** Need to take the 15 identified primitives and map each to:
- File location in `context/` directory
- YAML frontmatter schema (required fields, optional fields, custom fields)
- State machine definition (allowed states and transitions)
- Relationships to other primitives
- Whether it's hot (filesystem) or could become cold (archive)

This is the next step in the discussion.

### 2.7 Scaling resolution

Research (`file-per-entity-scaling-2026-03.md`) resolved the scaling concern:

**Git handles 50K files comfortably** with three mitigations:
1. Directory sharding (`items/2026/03/BACK-xxx.md`) — keeps each dir under 1K files
2. Git fsmonitor + `feature.manyFiles` — daemon tracks changes, avoids stat-ing all files
3. Sparse checkout — all files in git, only "hot" ones on disk

**Three-tier architecture:**
- Hot (filesystem, git-tracked): individual .md files for active items
- Warm (SQLite, gitignored): derived index for queries + RAG embeddings
- Cold (compressed archives, git-lfs): completed items older than threshold

**kaits boundary:** Repo-per-project. Each simulated company project = own git repo
with own aibox context (max ~50K active files). kaits orchestrates across repos and
maintains a cross-project database for analytics. aibox markdown = interchange format.

**Ultimate mitigation (owner):** The underlying filesystem can be changed from disk-based
to RAM-based (tmpfs/ramfs). This eliminates all I/O bottlenecks but is the nuclear option
— only for extreme performance-critical scenarios.

**Decision (tentative):** File-per-entity with sharding + sparse checkout + hot/cold
archiving. Scales to 100K+ items per project. kaits scales beyond via repo-per-project.

### 2.8 Discussion as a primitive

A Discussion IS a process primitive (this document is proof). It has: ID, title,
participants, status (active/concluded), related items, research references, and
produces decisions. Added as 16th primitive alongside the 15 from the ontology research.

### 2.9 UUID for identifiers

**Decision (tentative):** Use short UUIDs (first 8 hex chars of UUID4) for all entity IDs.
Format: `BACK-a7f3b2c1`, `DEC-f290e4b3`, `DISC-001` (discussions keep sequential for now
since they're rare and human-authored).

Rationale: no coordination needed for multi-collaborator, collision probability negligible
at 100K items (~0.0002%), human-readable enough. Lock files and prefixes rejected as brittle.

## 3. Open Questions (for continued discussion)

1. ~~**Scaling**: resolved — see §2.7~~
2. **Primitive mapping**: Map each of the 16 primitives to concrete storage decisions.
3. **Directory structure**: Design the new `context/` layout with sharding.
4. **Migration**: How do we migrate from current BACKLOG.md table format to file-per-entity?
5. **Event log**: JSONL append-only vs individual event files?
6. **Narrative vs structured**: Where exactly is the boundary? (Some primitives like
   Decision Records are mostly narrative with light structure; others like Events are
   mostly structured with light narrative.)
7. **Process templates**: How do the 4 presets (minimal/managed/research/product) map
   to the new primitive-based system?
8. **Git repo as a primitive**: Owner noted that taking a git repository as granted is
   itself a precondition/primitive. Accepted for now to keep things simple.

## 4. Decisions Made (tentative, pending formal DEC-NNN)

1. **Storage**: Markdown+frontmatter as single source of truth. SQLite as derived
   runtime index (gitignored). No dual-master.
2. **Scaling**: Three-tier hot/warm/cold. Directory sharding. Sparse checkout for large repos.
3. **kaits boundary**: Repo-per-project. aibox handles per-project context (up to 100K items).
   kaits orchestrates across repos with its own database.
4. **IDs**: Short UUID (8 hex chars). No coordination needed.
5. **Discussions**: Are a primitive. Stored in `context/discussions/`.

## 5. Next Steps

- [x] Research: scaling limits of file-per-entity in git repos — **done**
- [ ] Map 16 primitives to storage structure
- [ ] Design new `context/` directory layout with sharding
- [ ] Design YAML frontmatter schemas per primitive type
- [ ] Prototype: convert BACKLOG.md to file-per-entity format
- [ ] Record formal decisions in DECISIONS.md
- [ ] Session handover: capture full context for next session
