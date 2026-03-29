---
id: DISC-001
title: Context System Redesign — Process Ontology, Storage Architecture, Scaling
status: active
date: 2026-03-28
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
  - context/research/primitive-mapping-exercise-2026-03.md
  - context/research/primitive-skills-mapping-2026-03.md
  - context/research/personas-and-scenarios-2026-03.md
  - context/research/sector-process-structures-2026-03.md
  - context/research/scientific-research-pm-analysis.md
  - context/research/ai-provider-audit-logging-2026-03.md
  - context/research/software-dev-process-deep-dive-2026-03.md
  - context/research/ai-provider-identity-scheduling-2026-03.md
  - context/research/aiadm-aictl-architecture-2026-03.md
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

### 2.10 Primitive mapping exercise

Completed in `context/research/primitive-mapping-exercise-2026-03.md`. Mapped all 16
primitives to file locations, YAML schemas, state machines, and storage tiers. Identified
three storage patterns: file-per-entity markdown (11 primitives), JSONL append-only
(events), and YAML configuration (state machines, categories). Plus two structural
primitives with no dedicated storage (cross-references inline, context = directory itself).

### 2.11 Open questions resolved (session 2026-03-27 continued)

**Q1 — ID migration:** Full migration of all existing IDs (BACK-001..099, DEC-NNN,
PROJ-NNN) to new format. No backward compatibility, no mixed formats. Clean break.

**Q2 — People vs Roles (OWNER.md / TEAM.md):** Owner identified a gap in the ontology:
the Role primitive says "roles are not people," but OWNER.md and TEAM.md describe PEOPLE
so that AI agents know who they're working with. Two distinct concepts identified:
- **Role** = a hat (responsibilities, permissions). "Product Owner" is a role.
- **Actor** = a person or agent (preferences, expertise, working style). "@alice" is an actor.
Relationship: actors FILL roles. One actor, many roles. One role, many actors.

**Decision (tentative):** Actor becomes a 17th primitive. Justification: the separation is
real, kaits needs rich actor profiles for simulated humans/agents, and "who can fill this
role" vs "what is this person like" are fundamentally different questions. Content from
OWNER.md and TEAM.md migrates to Actor entities. OWNER.md and TEAM.md are retired — no
legacy files, full migration.

**Q3 — Event log format:** JSONL confirmed. Industry standard for log files. Owner raised
archiving implications: when events move from hot (JSONL files) to cold (tar.gz archives),
the SQLite index must be updated to point to the archive location. Agreed approach:
- Index retains event METADATA (id, timestamp, type, subject) permanently
- Storage location field updated to point to archive path
- Full event payload requires archive extraction (slow, acceptable for history queries)
- Rotating logfile practices (space savings) covered by directory sharding + archiving

**Q4 — Artifact self-description:** Owner concern: "today I go to context/research/ and
find everything. In future, do I need the index?" Resolution: TWO kinds of artifacts:
1. **Content-primary** (research, work instructions, PRD) — stay in semantic directories
   (research/, work-instructions/), gain frontmatter, index picks them up by scanning
2. **Metadata-primary** (build records, external references) — go to items/artifact/
The directory structure serves BOTH purposes: index storage AND human-browsable organization.
Not everything moves to items/. Semantic paths are preserved for human discoverability.

**Q5 — Process definitions vs instances:** Confirmed class/object analogy. Definitions
(stable, low-volume) stay in items/process/ or templates/processes/. Instances (ephemeral,
high-volume) are modeled as Work Items with `subtype: process-instance` and
`process_def: PROC-xxx` reference. Avoids duplicating Work Item lifecycle machinery.

**Q6 — ID prefix consistency:** Deferred — folded into broader ID format discussion (§2.12).

### 2.12 ID format: word-based identifiers

Owner requested investigation of human-readable alternatives to hex UUIDs. Motivated by:
readability, memorability, speakability in standups. References: zellij session naming,
what3words.

**Analysis of hex UUID minimum lengths:**
- 6 hex chars (16M combinations): ~0.03% collision at 100K items — practical minimum
- 7 hex chars: ~0.002% — comfortable
- 8 hex chars: ~0.0001% — very safe at 1M items

**Word-based alternative:** A curated wordlist of ~2,000 short English words (3-6 chars):
- 2 words: 4M combinations (≈ 6 hex chars) — `BACK-swift-oak`
- 3 words: 8B combinations (≈ 8 hex chars) — `BACK-swift.oak.bell`
- 2 words + 2-3 hex suffix: 16B combinations — `BACK-swift-oak-7f`

**Decision (tentative):** 2-word IDs from a curated wordlist. `BACK-swift-oak` is vastly
more usable than `BACK-a7f3b2c1`. Wordlist: ~10KB embedded in CLI binary, filtered for
short, common, non-offensive, unambiguously spelled words. 4M combinations sufficient for
any single project. Collision handling: if collision detected on generation, regenerate.

### 2.13 Kubernetes-inspired object model

Owner proposed adopting Kubernetes patterns: `apiVersion`, `kind`, metadata/spec separation.
Already precedent in codebase — addon YAML system uses declarative patterns, and
ARCHITECTURE.md describes a "declare desired state, let controllers reconcile" philosophy.

**Adopted patterns:**
- `apiVersion: aibox/v1` — schema versioning. Enables migration when schemas change.
  Old files declare their version, can be migrated programmatically.
- `kind: WorkItem` — unambiguous type declaration. PascalCase per Kubernetes convention.
- `metadata:` — system fields (id, timestamps, labels/tags, annotations/custom)
- `spec:` — entity-specific fields (title, state, owner, refs, etc.)

**Not adopted:**
- `namespace:` — we use `scope:` which is more general
- `status:` as separate section — agents update state directly in spec
- `metadata.managedFields` — too complex for file-based storage

**Key insight — declarative reconciliation model:**
- Entity files = declared desired state
- `aibox sync` = reconciliation loop (rebuild index, validate refs, enforce state machines)
- Events = imperative record of what actually happened
- The gap between desired state and actual state is what agents work to close

### 2.14 State machines: agent-driven vs server-driven

Owner insight: Jira executes state machines as a deterministic server automaton. In
aibox/kaits, agents are probabilistic — they INTERPRET the state machine as guidance.
This is a genuine advantage:
- State machine YAML defines what is ALLOWED, not what will happen
- Agent decides WHEN and HOW to transition based on context
- Guards are advisory checks, not hard server-side gates
- The system is more flexible/reliable than Jira's rigid automation

**Transition hooks — two mechanisms agreed:**
1. **Shell command field** (Kubernetes pod-spec `command:` style): runs a shell command
   on transition. stdout → event data, stderr → logged, exit code → success/failure.
   Enables webhooks via curl. No embedded scripting languages (too slow, hard to debug).
2. **Minijinja for guard expressions:** Already a dependency (used for Dockerfile/compose
   generation). Entity frontmatter fields become template variables. Easy to implement,
   understand, and debug — render the template to see the result.

Owner explicitly rejected: arbitrary Groovy-style scripting. Rationale: slow, complicated
to debug even for agents. Shell commands + minijinja cover the needed functionality.

### 2.15 Filesystem sharding configurability

Agreed: sharding granularity configurable per entity type in aibox.toml. Strategies:
none, yearly, monthly, weekly, daily. Changing strategy is non-destructive — existing
files stay in place, new files use new strategy, index knows actual paths. `aibox
migrate-shards` can reorganize existing files optionally.

### 2.16 Three-level rule + directory INDEX.md

All entity markdown files must follow the three-level rule from SKILL.md pattern:
YAML frontmatter → Level 1 intro (1-3 sentences) → Level 2 overview → Level 3 details.

Additionally, each directory gets an INDEX.md (or _index.md) serving as "Level 0":
- Describes the directory's content and purpose
- Lists contents with one-line descriptions
- Allows AI agents to decide whether to drill deeper before reading files
- Auto-generated/updated by `aibox sync`, with human-authored overrides supported

### 2.17 Filename conventions

Owner practices that should become standard:
1. **Inverse date prefix** for temporal sorting: `20260327-` or `20260327-1234-`
   Applies to: research reports, session notes, events — anything where "when" matters.
2. **Content slug** for human scanning: `-process-ontology-primitives.md`
   Applies to: research, discussions, work instructions.
3. **For entity files with word-IDs:** The word-ID IS the content hint.
   `BACK-swift-oak.md` is already memorable. Optionally add title slug:
   `BACK-swift-oak-implement-oauth.md` — ID + slug for browsing.

### 2.18 Template overrides (Kubernetes-inspired)

How to manage project-specific adaptations of base templates:
- **Template** (base) lives in `templates/processes/code-review.md`
- **Project override** declares `extends: templates/processes/code-review` and specifies
  only the delta (Kustomize overlay pattern)
- **Effective process** = base + overlay, computed at read time
- For agent migration briefings: diff `apiVersion: aibox/v1` vs `aibox/v2` schemas
  to produce a migration guide of what changed

### 2.19 Primitive overlaps and layering

Owner asked about overlaps between Work Item, Log Entry, Decision Record. Resolution:
not overlap but INHERITANCE. All primitives share a base schema (id, type, title,
description, state, timestamps, owner, tags, refs, custom). Each primitive extends with
type-specific fields. Distinguished by semantics and lifecycle, not data structure.

**Primitive layers (from ontology research §6):**
- Layer 0 (irreducible core): Work Item, Log Entry, State Machine, Role, Actor*
- Layer 1 (structural): Scope, Cross-Reference, Category
- Layer 2 (process): Process, Checkpoint, Artifact, Schedule
- Layer 3 (governance): Decision Record, Metric, Constraint, Context, Discussion

*Actor added as new primitive.

**Risk handling:** Not a new primitive. A risk = Work Item (subtype: risk) + Decision
Record for mitigation choice + Category dimensions for probability/impact.

**Artifact scope clarified:** Primary artifacts = project outputs (CLI binary, container
images, docs). Context documents = secondary artifacts (working documents that support the
process). Both get frontmatter, but research/work-instructions stay in semantic directories.

### 2.20 Actor primitive — multi-actor roles confirmed

Owner confirmed: a Role can be filled by multiple Actors simultaneously. "Julie and David
share PM responsibilities without exact delineation" is a valid real-world pattern. The
`filled_by:` field is an array, no sub-division required. If actors want to split
explicitly, they create sub-roles. If they share fuzzily, one role with multiple actors.

Actor types: `human` | `ai-agent`. For kaits, Actor profiles for AI agents would describe
capabilities, model, context window size, tool access — the agent equivalent of expertise
and working style.

### 2.21 Word-based IDs — petname crate and wordlist sizing

Research found the `petname` Rust crate (1.16M downloads) as the best candidate:
- Supports custom word lists, configurable separators, word count
- Default nouns are animal names (inherently safe/non-offensive)
- Adjectives curated toward positive/neutral words
- Already in Rust ecosystem, well-maintained

**Wordlist sizing with 3-8 char words:**
- ~5,000 adjectives x ~4,000 nouns = **20M combinations** with just 2 words
- Far exceeds the 4M+ target — no hex suffix or third word needed

**Decision (tentative):** Use `petname` crate with custom filtered wordlist. 2-word IDs
from 3-8 char words. Format: `BACK-swift-oak`. ~20M combinations per prefix type.

**Content slugs rejected as default:** Owner concern about slug staleness (content changes,
slug doesn't match). Word-ID IS the stable identifier. Title field in frontmatter is the
living description. INDEX.md per directory provides browsability. No slug in filename.

### 2.22 ~~Guard expression execution — aibox CLI as controller~~ (SUPERSEDED by §2.25)

Owner challenge: who evaluates minijinja guard expressions? Agents don't have minijinja.

**Resolution:** The aibox CLI is the execution point. New command:

```
aibox transition BACK-swift-oak --to in-review
```

The CLI: (1) reads entity frontmatter, (2) finds applicable state machine, (3) renders
minijinja guard with entity fields as context, (4) if guard passes: updates state, emits
event, runs on_transition shell command, (5) if guard fails: returns error with reason.

The agent doesn't need to know minijinja exists. It calls `aibox transition` and gets
yes/no. This fits the Kubernetes analogy: agent = user running `kubectl apply`, aibox CLI =
API server + controller that validates and reconciles.

Shell commands on transitions: also executed by `aibox transition` as subprocess after
state change. stdout → event data, stderr → logged, exit code → success/failure.

Fallback for agents without CLI access: guard expressions are readable as natural language
intent documentation. Agent can evaluate `{{ blocked_count == 0 }}` by reading refs and
counting blocks — probabilistic but functional.

### 2.23 ~~Template overrides — materialization, not read-time computation~~ (SUPERSEDED by §2.26)

Owner challenge: "computed at read time" assumes an on_file_read handler. Agents read raw
files. Many LLMs have <200K context. There is no middleware between the agent and the file.

**Revised model — materialization by `aibox sync`:**

```
templates/processes/code-review.md            ← base (shipped with aibox)
context/overrides/processes/code-review.md    ← project delta (user-authored)
context/items/process/PROC-swift-oak.md       ← MATERIALIZED effective process
                                                (generated by aibox sync)
```

`aibox sync` reads base + overlay, resolves, writes the effective file. The agent ONLY
reads the materialized file. This is how Kustomize works — you run `kustomize build` to
produce rendered YAML; the API server never reads kustomization.yaml.

**Override as entity (tentative):**
```yaml
apiVersion: aibox/v1
kind: Override
metadata:
  id: OVR-calm-pine
spec:
  base: templates/processes/code-review
  patches:
    - path: spec.gates
      op: add
      value: [GATE-bold-river]
```

**Is override a primitive?** No — it's an operation (`aibox sync`), not a data concept.
Kubernetes doesn't treat "overlay" as a resource kind. The Override entity is a
configuration input, not a process primitive.

**Key separation:** Authoring time (human works with overrides) vs execution time (agent
reads materialized files). The override machinery is invisible to agents at runtime.

### 2.24 Architectural boundary: aibox is infrastructure, not application

Owner's fundamental insight: aibox is like Kubernetes — it deploys and manages the context
infrastructure. It does NOT reach inside the derived project to execute process logic.

**The boundary:**
- aibox = scaffolding + schema + validation + migration (infrastructure)
- Derived project agents = process execution + state management (application)

Once scaffolded, context files belong to the derived project. Agents read, interpret, and
modify them with full autonomy. aibox doesn't enforce, it observes and validates.

**aibox commands (infrastructure):**
- `aibox init` — create context structure from templates
- `aibox sync` — rebuild index from files
- `aibox lint` — post-facto validate schema, references, state machine compliance
- `aibox validate` — check required fields, broken refs
- `aibox migrate` — generate migration diffs + prompts for schema updates
- `aibox id generate` — create new word-based IDs

**NOT in aibox (process layer):**
- Guard evaluation (agents interpret guards probabilistically)
- Transition enforcement (agents edit state directly; lint catches violations after)
- Hook/script execution (agents have their own tool access)
- Process orchestration
- RBAC enforcement (agents interpret permissions probabilistically)

### 2.25 Guards in plain English, not minijinja

Owner challenge: minijinja guards are deterministic thinking — we're building a workflow
engine when we have agents. If aibox is a differentiator to "old world" tools, we should
lean into the probabilistic model fully.

**Decision (revised):** Guard expressions are written in plain English, not minijinja
pseudo-code:

```yaml
transitions:
  - from: in-progress
    to: in-review
    guard: "Only transition when all blocking items are resolved and work is tested."
    on_transition:
      suggest: "Notify the reviewer role that a review is ready."
```

The agent reads this, evaluates it using judgment, and edits the entity file. This is:
- Simpler (no expression language to implement)
- More flexible (agents apply judgment, handle edge cases)
- Consistent with agent-driven model (§2.14)
- A real differentiator (Jira can't do fuzzy guard evaluation)

Minijinja remains in aibox for template rendering (Dockerfiles, compose files) — its
existing use case. But NOT for process logic.

### 2.26 Overrides eliminated — direct editing + migration prompts

Owner challenge: if derived project agents can edit files directly, why have an override
mechanism? It adds deterministic complexity where probabilistic simplicity suffices.

**Revised workflow:**
1. `aibox init` generates process definitions from templates into project's context/
2. Project owner tells agent: "Our code review requires 2 reviewers"
3. Agent edits the process file directly — it owns the file
4. No overlay, no base+delta, no materialization

**On aibox version updates:**
- `aibox migrate` generates a diff (old template → new template)
- Produces a migration prompt with instructions for the derived project's agent
- Agent applies changes with human approval
- Prompt includes: "Never edit without asking the user"

This mirrors real infrastructure upgrades: release notes + migration guide, not automatic
in-place patching. The derived project's customizations are respected because the agent
reviews the diff against the current state.

### 2.27 Shell commands as suggestions, not guarantees

In the probabilistic model, `on_transition.suggest` is guidance the agent MAY follow:
- Agent has autonomy — it might execute the curl, or might not
- Derived project owner may have denied bash access
- Agent can substitute an equivalent action
- The event log records what actually happened, regardless

For deterministic needs (event recording), aibox provides dumb infrastructure commands
(`aibox event append`) that agents can call — but these are infrastructure utilities, not
process enforcement.

### 2.28 ~~Bash scripts / event append~~ (SUPERSEDED by §2.30)

Earlier proposals for `aibox event append` and skill-wrapped bash scripts are superseded.
See §2.30 for the clean model.

### 2.29 RBAC in plain English

Owner raised: who is allowed to edit what? User A (admin) can change the release process,
User B (developer) cannot. Should this be deterministic or probabilistic?

**Decision (tentative):** Probabilistic RBAC via plain English in Role definitions.

```yaml
kind: Role
spec:
  name: "Developer"
  permissions:
    - "Can create and edit work items assigned to you"
    - "Can comment on any work item"
    - "Can create and edit research documents"
  restrictions:
    - "Cannot modify process definitions, state machines, or gate criteria"
    - "Cannot change role assignments or role definitions"
    - "Cannot modify constraints or scheduling"
  escalation: "Ask an Admin role holder for changes outside your permissions"
```

The derived project's agent reads this, checks the requesting user's actor profile →
roles → permissions/restrictions, and makes a judgment call. This works because:
- Real organizations are already probabilistic about authority
- Agents can handle edge cases (typo fix in process doc = probably fine)
- Deterministic RBAC can be routed around anyway (ask an admin to do it)
- The event log provides accountability for every decision

**Consequences and liability:**
- `aibox lint` flags permission anomalies post-facto ("PROC-xxx was modified by @user-b
  who holds Developer role, which restricts process modification")
- aibox assumes zero liability — RBAC definitions are guidance, not enforcement
- Derived project owner is responsible for consequences of their agents' decisions
- Event log provides full audit trail

**Permissions reference kind:** Restrictions naturally map to entity kinds:
"Cannot modify kind: Process, StateMachine, Gate, Constraint, Role"

### 2.30 Bash scripts and event recording — agents ARE the execution layer

Owner challenge: what's the point of `aibox event append`? And creating a skill per bash
script is unrealistic.

**Resolution: there is no hook execution infrastructure.** The agent IS the execution layer.

When the state machine says `suggest: "Notify the reviewer"`, the agent decides how to
act on it. Maybe it posts a comment, sends a message, runs a curl command, or does nothing.
The agent already has its own tools (bash, file editing, etc.) based on what the derived
project owner has granted. No aibox infrastructure needed.

If the suggestion says `suggest: "Run: curl -X POST https://deploy.example.com/trigger"`,
the agent either has bash access and runs it, or doesn't and tells the user to run it
manually. This is exactly what a human team member would do without the right credentials.

**Event recording — REVISED in §2.32.** The sync-based approach was wrong (deterministic
thinking). See §2.32 for the correct model.

### 2.31 Filename conventions refined

**Content-primary long-lived files** (research, decisions, discussions):
`<inverse-datetime>-<KIND>-<word-ID>-<content-slug>.md`
Example: `20260327-ART-swift-oak-process-ontology-primitives.md`
Slug acceptable because these files rarely change after creation.

**High-volume entity files** (work items, roles, schedules):
`<KIND>-<word-ID>.md`
Example: `BACK-swift-oak.md`
No slug, no date — date is in frontmatter + sharding path.

### 2.32 Event recording — agent logs via skill, aibox logs infrastructure

Owner correction: `aibox sync` detecting state changes is deterministic thinking again.
Problems: (1) nothing runs sync deterministically, (2) multiple state changes between syncs
means intermediate events are lost (draft→ready→in-progress seen as only draft→in-progress).
Sync can't reconstruct history it didn't witness.

**Correct model: two event sources.**

**Process events** (state changes, decisions, gate checks, comments):
- Agent logs these using an **event-log skill** (to be created by aibox)
- The skill is simple: append a JSONL line to the current month's event file
- The instruction to always log is placed prominently in scaffolded process documentation
- Whether the agent actually logs every time is probabilistic — derived project's responsibility
- If the derived project creates RBAC rules that trick the agent out of logging, that's on them

**Infrastructure events** (inconsistencies, lint warnings, schema errors, sync results):
- `aibox sync` / `aibox lint` write these deterministically
- "Detected broken reference: BACK-swift-oak → BACK-bold-river (not found)"
- "Schema validation failed on PROC-calm-pine: missing required field 'kind'"
- "Index rebuilt: 347 entities, 12 warnings"

**The clean separation:**

| What | Who | How |
|---|---|---|
| Process events | Agent | Event-log skill (probabilistic) |
| Infrastructure events | aibox sync/lint | Direct JSONL append (deterministic) |
| Entity file edits | Agent | Direct file editing |
| Index maintenance | aibox sync | SQLite rebuild from files |

This makes the event log richer and more auditable: you see both what the agent did (or
claims it did) AND what aibox infrastructure observed. Discrepancies between the two are
themselves informative.

### 2.33 Skills as agent API to primitives

Every primitive needs a corresponding skill — the skill is the agent's API that encodes
mechanical correctness (file naming, frontmatter schema, JSONL format, sharding path,
three-level rule) so the agent can focus on judgment.

Full research in `context/research/primitive-skills-mapping-2026-03.md`.

**17 skills mapped to 17 primitives:**

| Primitive | Skill | Status |
|---|---|---|
| Work Item | `workitem` | Rewrite of `backlog-context` |
| Log Entry | `event-log` | New (critical, foundation) |
| Decision Record | `decision` | Rewrite of `decisions-adr` |
| Artifact | `artifact-tracking` | New |
| Actor | `actor-profile` | Rewrite of `owner-profile` |
| Role | `role-management` | New |
| Process | `process-management` | New |
| State Machine | `state-machine-management` | New |
| Category | `taxonomy-management` | New |
| Cross-Reference | (embedded in other skills) | N/A |
| Checkpoint | `gate-management` | New (extends `code-review`) |
| Metric | `metrics` | New |
| Schedule | `schedule-management` | New (extends `standup-context`) |
| Scope | `scope-management` | New |
| Constraint | `constraint-management` | New |
| Context | `context-archiving` | Existing (needs update) |
| Discussion | `discussion-management` | New |

**Cross-cutting concerns (not separate skills, embedded in all entity-modifying skills):**
- RBAC checking: read actor → roles → permissions before every modification
- Event logging: log every action via `event-log` skill
- INDEX.md maintenance: update directory indexes after file changes

**Revised packages:** core (actor, role, event-log), tracking (workitem, decision,
archiving), processes (process, state-machine, gate), planning (scope, schedule,
estimation), governance (constraint, metrics, taxonomy), collaboration (discussion,
standup, handover, retro), artifacts (artifact-tracking, documentation).

**Presets:** minimal (core), managed (core+tracking+collaboration), software
(managed+processes+code+architecture), full-product (everything).

**Implementation order:** event-log → workitem → decision → actor-profile → role-management
→ process layer → planning/governance → collaboration/lifecycle.

### 2.34 Skill design refinements

**Skill naming:** Use longer descriptive names (`workitem-management` not `workitem`).
Pattern: `<noun>-management` for CRUD skills, `<noun>-<verb>` for action-specific skills.

**Skill hierarchy via instruction references:** Skills reference lower-layer skills by
name. `workitem-management` says "use event-log-management to log this" and "use
role-management to check permissions." The agent follows the chain. Dependency is
strictly downward — lower-layer skills never reference higher-layer skills. Hierarchy
documented in skill frontmatter via `uses:` field.

```
Layer 0: event-log-management (foundation)
Layer 1: role-management, actor-profile-management
Layer 2: workitem-management, decision-record-management, scope-management
Layer 3: process-management, gate-management, schedule-management
Layer 4: discussion-management, metrics-management
```

**Skill size:** Keep one skill per primitive. Target 100-200 lines. Current skills range
17-244 lines; 140 lines is the sweet spot (comparable to `agent-management`). The three-
level rule ensures agents read only what they need. Split only if a skill exceeds ~250
lines.

**Human vocabulary mapping:** Each skill's "When to Use" section must list all common human
terms that map to the primitive. "Backlog item", "task", "ticket", "issue", "bug",
"story" all map to `workitem-management`. This ensures a human saying "add a backlog item"
triggers the right skill.

### 2.35 Template originals — scaffold + keep copies

`aibox init` creates project files AND stores original template copies:

```
context/.aibox/templates/
  v1.0.0/                     # originals from scaffold time
  v1.1.0/                     # downloaded by aibox update
context/.aibox/migration/
  v1.0.0-to-v1.1.0.md         # auto-generated diff + migration instructions
```

Derived project's agent can diff originals vs current files to understand customizations.
Migration prompts generated by `aibox migrate` reference these copies. No deterministic
override mechanism — just files the agent reads and reasons about.

**Derived project skill customization:** Same principle — direct editing after scaffolding.
The originals in `.aibox/templates/` let the agent understand what was changed if analysis
is ever needed.

### 2.36 aibox / kaits logical split

**aibox = single-project infrastructure + process primitives + curated skills**
- Container environment, process ontology (17 primitives), skill library, CLI tooling
- Defines the WHAT (primitives), a little HOW (starter processes), and the vocabulary
  (apiVersion/kind, JSONL format, three-level rule, file naming)
- Single project scope: one git repo, one context directory, one team
- Terminal only

**kaits = multi-project orchestration + agent teams + company simulation**
- Orchestrates across many aibox repos (repo-per-project)
- Agent spawning, lifecycle, coordination
- Company structure (departments, OKRs, budgets, hiring)
- Cross-project database, analytics, portfolio view
- Graphical UI (dashboards, Kanban boards, agent status)
- Persistent processes (daemons for cadences, monitoring)
- Higher-level processes (PI planning, portfolio management, capacity allocation)

**Key insight:** aibox provides atoms (primitives), kaits builds molecules (company
processes) from those atoms. Solo developer uses aibox directly, never needs kaits.

### 2.37 Process packages in the new primitive-based system

Packages define which primitives and skills are active. They don't define processes —
they activate the skills that enable processes.

| Package | Primitives/Skills active | Target user |
|---|---|---|
| minimal | Actor, Role, Event log | Quick experiments, throwaway |
| managed | + Work items, Decisions, Archiving, Standups, Handover | Solo dev, ongoing project |
| software | + Processes, State machines, Gates, Code/Arch skills | Solo/small team, software |
| research | + Artifact tracking, Documentation skills | Research, writing, analysis |
| full-product | + Scopes, Schedules, Constraints, Metrics, Taxonomy, Governance | Team, product dev, kaits |

aibox ships starter processes (code-review, bug-fix, feature-dev, release) with the
software and full-product packages. These are scaffolded as Process entity files with
plain English steps. Derived project customizes them.

### 2.38 The Process Paradox — resolved

**Paradox:** If aibox is infrastructure, it shouldn't define process. But primitives without
process are useless. Having work items implies SOME workflow. Resolution:

**Three layers of process:**
1. **Primitive mechanics** (aibox — always): HOW to create/update files, log events, check
   RBAC. Skills encode this. Framework-agnostic. Like SQL is to a database.
2. **Micro-processes** (aibox — optionally): Code review, bug fix, release, feature dev.
   Small, self-contained, framework-neutral workflows. Every project needs these regardless
   of whether they call themselves "Scrum" or "Kanban."
3. **Macro-processes / frameworks** (kaits territory): SAFe, LeSS, Scrum@Scale, PMBOK,
   Disciplined Agile. Company-level operating models. Require multi-team coordination.

**Process packages are primitive activation tiers, NOT framework choices:**
- minimal = "you exist and can log" (almost no process implied)
- managed = "track work, record decisions" (no opinion on sprints vs flow)
- software = "do code review, handle bugs, manage releases" (micro-processes, not frameworks)
- full-product = all primitives active, ready for ANY framework on top

**aibox does NOT ship SAFe, Scrum, Kanban, etc.** Reasons: scope explosion, too opinionated,
wrong granularity (frameworks are organizational choices, not project infrastructure),
composability is more powerful. However, optional community-contributed framework packages
could be installed: `aibox process install scrum-basic`.

**Personas and user stories fit existing primitives:**
- Persona = Actor (subtype: persona) — fictional user profile
- User story = Work Item (subtype: story) — "As X, I want Y, to achieve Z"
- These practices are universal (XP, Design Thinking), not SAFe-specific

### 2.40 Sector analysis and process package completeness

Research completed: 15 sectors analyzed (`sector-process-structures-2026-03.md`), deep
dive on scientific research (`scientific-research-pm-analysis.md`).

**Key finding:** 17 primitives cover 70-80% of every sector. Sector differences are
primarily: constraints (regulatory), work item subtypes, and cadences.

**Revised aibox core packages (Tier 1):**
- minimal, managed, software (unchanged)
- **research** — MAJOR expansion needed (current template critically incomplete for
  real scientific work: missing publication pipeline, IRB gates, literature management,
  protocol versioning, grant tracking, data management plans)
- **editorial** — NEW: content pipeline (draft→review→approve→publish), content calendar
- **consulting** — NEW: engagement tracking, deliverable management, handoff packages
- full-product (unchanged — all primitives active)

**Community/sector packs (Tier 2-3):** healthcare-pharma, financial-services,
legal-practice, construction-eng, nonprofit-grants, government-procurement,
manufacturing-quality, data-science-ml. Installable via `aibox process install`.

### 2.41 Community process package interface

Process packages are git repos with a standard structure:

```
package.yaml        # apiVersion/kind metadata, requirements, provides
context/            # files to merge into project context/
skills/             # optional custom skills
README.md
```

Commands:
- `aibox process install <git-url>` — install a package
- `aibox process check <path>` — validate conformance
- `aibox process list` — list installed packages
- `aibox process export` — package project's process for sharing

Company-to-open-source flow: company customizes process with kaits agents → agents
iteratively improve → company exports and publishes → others install and adapt.
Like Anthropic Claude marketplace — community-driven, git-based.

### 2.42 Personas defined

6 personas created (`personas-and-scenarios-2026-03.md`):
1. **Alex** — Solo developer, freelance, managed package
2. **Dr. Priya** — Research scientist, lab lead, research package
3. **Maria** — Small team lead, startup, software/full-product package
4. **Sam** — Consultant/contractor, engagement-focused, managed package
5. **kaits** — Company simulator, programmatic usage, full-product
6. **Jordan** — Content producer, editorial workflow, editorial package

Personas ARE Actor entities (subtype: persona) — dogfooding the primitive system.

### 2.43 AI provider audit logging — hybrid event model

Research (`ai-provider-audit-logging-2026-03.md`): Most major AI providers offer some
audit logging. Best: Claude Code (21 hook events, HTTP webhooks), Gemini CLI (native
OTel), Codex CLI (OTel). Worst: Cursor, Continue.dev.

**Three logging channels (the hybrid model):**
1. **Provider hooks (deterministic):** Captures WHAT happened — every tool call, file edit.
   Configured via aibox.toml `[audit]` section. Always logs if enabled.
2. **Agent event-log skill (probabilistic):** Captures WHY it happened — state changes,
   decisions, rationale. Best-effort, agent's responsibility.
3. **aibox sync/lint (deterministic):** Infrastructure events — inconsistencies, validation.

Together: the "what" is guaranteed, the "why" is best-effort. Complete audit coverage.

Optional `[audit]` section in aibox.toml:
```toml
[audit]
provider_hooks = true
provider_destination = "context/audit/"
```

### 2.44 Software development deep dive — no new primitives needed

Research (`software-dev-process-deep-dive-2026-03.md`): Full lifecycle analyzed from
product discovery through end-of-life. **17 primitives cover everything** through
composition. No gaps requiring new primitives.

Gaps that need attention (modeled as subtypes, not new primitives):
- Environments (dev/staging/prod) → Scope subtype
- Feature flags → Work Item subtype with per-environment state
- External dependencies → Artifact subtype
- Deployment history → Event type

**4 must-have missing process templates for software package:**
1. incident-response.md
2. technical-design.md
3. spike-research.md
4. hotfix.md

**6 state machines defined:** Feature, Bug, Incident, Release, Tech Debt, PR/Code Review.

### 2.45 Naming consistency resolution

Audit found 2 inconsistencies: gate/checkpoint and scope/project. Resolved:
- **Gate** everywhere (directory `context/gate/`, kind `Gate`, prefix `GATE-`)
- **Scope** everywhere (prefix `SCOPE-` for all, not `PROJ-` vs `SCOPE-`)

**Directory naming revised:** Primitives get top-level dirs under `context/` (not nested
under `items/`). Directory name matches the primitive: `context/work-item/`,
`context/decision/`, `context/actor/`, etc.

**Filename convention revised — two patterns:**
- **Pattern A (human-named):** Low-volume, long-lived entities (Actor, Role, Process,
  Gate, Metric, Schedule, Constraint, Scope). Filename: `KIND-human-name.md`.
  Example: `ACTOR-alex-chen.md`, `ROLE-admin.md`, `PROC-code-review.md`.
  Word-ID lives in `metadata.id` inside YAML. CLI ensures name uniqueness.
- **Pattern B (auto-generated):** High-volume entities (Work Item, Decision, Discussion,
  Artifact). Filename: `KIND-word-id-content-slug.md`.
  Example: `BACK-calm-lark-dark-mode-toggle.md`, `DEC-keen-fox-use-postgresql.md`.
  Slug from title at creation, does NOT update if title changes.

### 2.46 INDEX.md — structural, not statistical

INDEX.md describes the directory's PURPOSE and SCHEMA, not its current state:
- What entity kind lives here
- What subtypes exist
- What state machine applies
- What skills manage it
- What the file naming pattern is

INDEX.md does NOT contain: item counts, state groupings, recent activity, statistics.
Those come from the SQLite index queries. INDEX.md only changes when the schema changes,
not when items are added. Auto-generated by `aibox init`, updated by `aibox migrate`.

### 2.47 SQLite index from init onwards

`aibox init` creates the SQLite index database immediately (gitignored). Not deferred
to a later `aibox sync`. The index is empty but ready from the start. `aibox sync`
keeps it current. This means queries work from the first session.

### 2.48 Human identity resolution (Kubernetes-inspired)

Research (`ai-provider-identity-scheduling-2026-03.md`): Identity varies dramatically
across providers. Claude Code uses OAuth, Copilot uses GitHub, Aider has NO identity,
self-hosted LLMs have no auth by default.

**Solution: kubeconfig-inspired local identity file.**

`~/.aibox/identity.toml` (per-user, per-machine, NEVER committed):
```toml
[identity]
name = "Bob Smith"
email = "bob@company.com"
handle = "bob"
[preferences]
communication_style = "..."
working_hours = "CET 09:00-17:00"
```

**Identity cascade:**
1. `~/.aibox/identity.toml` (most reliable, works with ANY provider)
2. Environment variable: `AIBOX_USER=bob@company.com`
3. AI provider identity (if extractable: GitHub, Google accounts)
4. Git config: `git config user.email`
5. `aibox auth whoami` — displays resolved identity + provider
6. Agent asks (last resort)

Multi-human repos: Actor files contain non-sensitive shared info only.
Personal preferences in `~/.aibox/identity.toml` (never committed).
Follows Kubernetes pattern: kubeconfig (local) vs RBAC bindings (shared).

**Full RBAC flow (3-layer model):**
1. Machine layer: `~/.aibox/identity.toml` → "I am Bob" (never committed)
2. Repository layer: `context/actor/ACTOR-bob-smith.md` → "Bob is on this project,
   fills Developer role" + `context/role/ROLE-developer.md` → permissions/restrictions
3. Runtime: Agent resolves identity → Actor → Roles → holds permissions in memory.
   On every modifying action: check permissions, if denied → explain + escalation path.

Actor types: human (identity.toml), ai-agent (env var / kaits), service (CI/CD env var).
Permission model: additive (any role granting permission wins). Restrictions checked
across all roles. Multi-role actors get union of permissions.

Detailed elaboration with diagrams and multi-actor scenarios in
`DISC-001-personas-and-scenarios.md` Scenario 5 and following sections.

### 2.49 Validation scenarios walked through (full detail)

Complete walkthroughs in `DISC-001-personas-and-scenarios.md` (appendix).
All 10 scenarios walked through in detail with 6 personas. Key findings:

Issues found across scenarios:
- "Check schedules at session start" needs prominent CLAUDE.md placement (high)
- Research package missing schedule-management (high)
- INDEX.md essential at scale >50 items, auto-generated by aibox sync (high)
- Agent identity from git config + env var override (high)
- Agent should adapt formality (no discussion entity for trivial choices) (medium)
- Migration prompts must be agent-agnostic (medium)
- `aibox id generate --count N` batch mode for kaits (medium)

### 2.50 aiadm/aictl architectural proposal (session 2026-03-28)

Owner identified that the purely probabilistic RBAC model (Decision 19) is insufficient
for enterprise: CIOs and security responsibles need deterministic enforcement and
tamper-proof audit logs, not agent goodwill.

**Proposed solution (Kubernetes-inspired):**
- Rename `aibox` -> `aiadm` (like kubeadm): infrastructure setup, images, containers, schema
- New CLI `aictl` (like kubectl): ALL context operations (create/get/describe/delete/edit/apply)
- Certificate-based authentication: each user/agent has a cert signed by project CA
- OS-level file lockdown: context/ writable only by aictl process
- Deterministic audit log: every aictl command logged automatically
- RBAC mechanically enforced: no certificate = no access

**Research conducted** (`context/research/aiadm-aictl-architecture-2026-03.md`):

1. **OS-level lockdown:** Assessed 5 mechanisms (DAC, SELinux/AppArmor, capabilities,
   container isolation, FUSE). Key finding: no intra-container mechanism is absolute
   against root shell access. Host-applied AppArmor is strongest. Recommended layered
   approach: DAC + cryptographic signing + FUSE when budget allows.

2. **kubectl->aictl mapping:** 24 commands map 1:1, 4 adapted semantically, 12 new
   aictl-specific commands (transition, lint, sync, search, board, tree, etc.), 16
   kubectl commands moved to aiadm. Total ~40 aictl commands.

3. **Decision impact:** Of 50 decisions: 14 unchanged, 17 modified, 7 superseded, 12
   strengthened. Superseded cluster around identity/RBAC (19, 43, 47, 48) and execution
   model (9, 20, 21, 35). Key boundary shift: aiadm=infrastructure, aictl=tooling,
   agents=judgment.

4. **K8s certificate flow:** Detailed analysis of CA creation, CSR workflow, kubeconfig,
   service accounts, RBAC mechanics. Key insight: enforcement works because API server is
   the single choke point to etcd. For aictl: signed files + git hook enforcement is
   the practical equivalent.

**Key architectural insight:** The probabilistic paradigm survives as a LAYER on top of
a deterministic base. aictl handles mechanical correctness (auth, RBAC, schema, logging);
agents handle judgment (what to create, when to transition, how to interpret guards).
Skills shrink by 60-70% because cross-cutting concerns become automatic.

**Open questions for owner:** (1) Does aictl govern all context/ files or only entities
with frontmatter? (2) Guard evaluation: trust agent assertion or evaluate mechanically?
(3) Solo dev certificate complexity — is `--no-auth` mode sufficient? (4) Rename timing.
(5) Structured vs plain-English permissions. (6) Daemon vs CLI-only.

## 3. Current State (as of 2026-03-29)

### 3.1 Where we are

DISC-001 has progressed through two major phases:

**Phase 1 (§2.1–§2.49): Context system design.** Complete. 17 primitives identified and
mapped to storage. File-per-entity markdown+frontmatter as source of truth, SQLite as
derived index. Word-based IDs via petname. Kubernetes-inspired object model (apiVersion/
kind/metadata/spec). 17 skills mapped to 17 primitives. 7 process packages defined.
6 personas validated across 10 scenarios. 50 tentative decisions recorded.

**Phase 2 (§2.50): aiadm/aictl proposal.** Research complete, awaiting owner review.
The purely probabilistic RBAC model (Decision 19) was identified as insufficient for
enterprise users who need deterministic enforcement and tamper-proof audit logs. Research
covered OS-level lockdown mechanisms, kubectl-to-aictl command mapping, impact on all 50
decisions, and Kubernetes certificate/RBAC mechanics.

Key research finding: the proposal is architecturally sound. Of 50 decisions, 14 are
unchanged, 17 need modification, 7 are superseded (clustered around identity/RBAC and
the execution model), and 12 are strengthened. The probabilistic paradigm survives as a
LAYER on top of a deterministic base — aictl handles mechanical correctness while agents
retain judgment authority. Skills would shrink by 60-70%.

Full research: `context/research/aiadm-aictl-architecture-2026-03.md`

### 3.2 Open questions — aiadm/aictl proposal (need owner input)

**Q-A: Scope of aictl governance.** Does aictl govern ALL files in `context/`, or only
entity files with YAML frontmatter? Research reports, work instructions, and PRD are
"narrative content" — some have frontmatter, some don't. If aictl governs everything,
agents cannot create/edit research reports without going through aictl. If aictl governs
only entities, narrative content remains unprotected but also unrestricted. The boundary
needs to be explicit: which files does the RBAC model cover?

**Q-B: Guard evaluation model.** When an agent calls `aictl transition BACK-xxx --to
in-review`, how does aictl handle the plain English guard ("all blocking items must be
resolved")? Two options:
- **(a) Trust the agent's assertion.** The agent says "I've checked the guards," aictl
  records the transition with a note that guards were self-attested. The guard text is
  logged for audit but not mechanically evaluated. Preserves the probabilistic philosophy.
- **(b) Evaluate mechanically.** aictl parses the guard into checkable conditions and
  blocks the transition if they fail. Requires a guard expression language (contradicts
  the "plain English, not minijinja" decision).
- **(c) Hybrid.** Some guards are tagged as `enforceable: true` with structured criteria
  alongside the plain English. aictl enforces the structured part, logs the English part.

**Q-C: Solo developer certificate complexity.** The Alex persona (solo dev, weekend
projects) does not need certificate-based auth. Options:
- `--no-auth` mode where aictl skips authentication entirely (simplest, no enforcement)
- Auto-generated self-signed cert on `aiadm init` (transparent to user, but adds files)
- Passphrase-based local auth (simpler than certificates, but weaker)
- Tiered: solo mode = no auth, team mode = certificates enabled via `aiadm auth init`

**Q-D: Rename timing.** The rename from `aibox` to `aiadm` is a breaking change affecting
all documentation, CLAUDE.md templates, skill references, CLI binary name, GHCR paths,
and user muscle memory. Options:
- Rename now (before v1, while user base is small — clean break)
- Rename at v1 release (bigger impact but bigger audience)
- Keep `aibox` as the umbrella brand, `aiadm`/`aictl` as subcommands (`aibox adm init`,
  `aibox ctl create workitem`) — avoids shipping two separate binaries

**Q-E: Structured vs plain-English permissions.** Decision 19 uses plain English for
role permissions. The aiadm/aictl model needs something machine-evaluable. Options:
- **(a) Structured only.** Role files use verb+kind rules (like K8s RBAC). Machine-
  enforceable but less flexible, harder for non-technical users to write.
  ```yaml
  permissions:
    - action: [create, edit] kinds: [WorkItem, Decision]
  ```
- **(b) Plain English only, parsed by aictl.** aictl uses an LLM or rule engine to
  interpret natural language permissions. Flexible but non-deterministic.
- **(c) Hybrid.** Structured rules for enforcement + plain English as documentation.
  The structured rules are the source of truth; the English is a human-readable gloss.
  ```yaml
  permissions:
    - action: [create, edit]
      kinds: [WorkItem, Decision]
      description: "Can create and edit work items and decisions"
  ```

**Q-F: Daemon vs CLI-only.** Should aictl be a long-lived background daemon or a
stateless CLI invoked per-command?
- **CLI-only:** Simpler, no process management, familiar UX. Each invocation reads certs,
  validates, writes, exits. Overhead ~5-20ms per command (negligible vs LLM inference).
  No real-time watch/subscribe capability.
- **Daemon:** Enables `aictl watch` (real-time entity change notifications), persistent
  auth session (verify cert once), incremental index updates (no `aictl sync` needed),
  file locking (prevent concurrent writes). More complex to manage but more capable.
- **Hybrid:** CLI by default, optional `aictl daemon start` for advanced scenarios
  (kaits, team environments). CLI commands connect to daemon if running, fall back to
  direct file access if not.

### 3.3 Open questions — earlier (still open)

3. **Directory structure**: Design the new `context/` layout with sharding. (Partially
   addressed in mapping exercise, needs finalization.)
4. **Migration plan**: Concrete steps to migrate from current format to file-per-entity.
   All IDs migrate, all files restructure. Need a migration script/command.
8. **Git repo as a primitive**: Owner noted that taking a git repository as granted is
   itself a precondition/primitive. Accepted for now to keep things simple.
10. **Archive indexing depth**: How deep does the SQLite index go into archived content?
    Metadata always indexed, full payload requires extraction. Need to define the boundary.
11. **INDEX.md generation**: What exactly goes into auto-generated directory index files?
    How does human override work? When does `aibox sync` regenerate them?

### 3.4 Resolved questions (for reference)

- ~~**Scaling**: resolved — see §2.7~~
- ~~**Primitive mapping**: resolved — see §2.10~~
- ~~**Event log**: resolved — JSONL, see §2.11 Q3~~
- ~~**Narrative vs structured**: resolved — content-primary vs metadata-primary, see §2.11 Q4~~
- ~~**Process templates**: resolved — packages are primitive activation tiers. See §2.38~~
- ~~**Wordlist curation**: resolved — petname crate, 3-8 char words, ~20M combos. See §2.21~~

## 4. Decisions Made (tentative, pending formal DEC-NNN)

1. **Storage**: Markdown+frontmatter as single source of truth. SQLite as derived
   runtime index (gitignored). No dual-master.
2. **Scaling**: Three-tier hot/warm/cold. Directory sharding. Sparse checkout for large repos.
3. **kaits boundary**: Repo-per-project. aibox handles per-project context (up to 100K items).
   kaits orchestrates across repos with its own database.
4. **IDs**: 2-word IDs from `petname` crate with custom wordlist (3-8 char words).
   Format: `BACK-swift-oak`. ~20M combinations per prefix type (no hex suffix needed).
   Full migration from sequential IDs. No content slugs in filenames.
5. **Discussions**: Are a primitive. Stored in `context/discussions/` (migrates to items/).
6. **Actor primitive**: 17th primitive added. Describes people/agents (preferences, expertise,
   working style). Distinct from Role (responsibilities, permissions). OWNER.md/TEAM.md
   content migrates to Actor entities.
7. **Kubernetes-inspired object model**: All entity frontmatter uses `apiVersion`, `kind`,
   `metadata`/`spec` structure. Enables schema versioning, migration, and declarative
   reconciliation.
8. **Events**: JSONL files, monthly sharded by default, configurable. Index retains metadata
   permanently; payload requires extraction from archives.
9. **State machine guards**: Plain English, not minijinja. Agents evaluate probabilistically.
   Shell commands are suggestions (`on_transition.suggest`), not guaranteed. No `aibox
   transition` command — agents edit state directly, `aibox lint` validates after the fact.
10. **Sharding**: Configurable per entity type (none/yearly/monthly/weekly/daily). Non-destructive
    strategy changes.
15. **Multi-actor roles**: A role can be filled by multiple actors simultaneously. No
    forced sub-division. `filled_by:` is an array.
16. **aibox is infrastructure, not application**: aibox inits, syncs, lints, migrates.
    It does NOT enforce process logic (guards, transitions, hooks). Agents have full autonomy.
17. **Word-IDs without content slugs for entities**: `BACK-swift-oak.md` for work items.
    Content-primary files (research, decisions) MAY keep slugs: `20260327-ART-swift-oak-process-ontology.md`.
18. **Minijinja stays for infrastructure** (Dockerfile/compose rendering), NOT for process
    logic. Guard expressions and process guidance written in plain English for agents.
19. **RBAC via plain English**: Role permissions/restrictions written as natural language in
    Role definitions. Agents interpret probabilistically. `aibox lint` flags anomalies.
    aibox assumes zero liability.
20. **Dual event sources**: Process events logged by agent via event-log skill (probabilistic).
    Infrastructure events logged by aibox sync/lint (deterministic). Both in same JSONL files.
21. **No hook execution infrastructure**: Agents ARE the execution layer. Process suggestions
    are guidance; agents use their own tool access to act on them.
22. **Event-log skill**: aibox ships a skill that agents use to append process events. Simple
    JSONL append. Instruction to use it is prominent in scaffolded process documentation.
23. **Skills as agent API**: Every primitive gets a corresponding skill. 17 skills mapped.
    Skills encode mechanical correctness; agents provide judgment. Cross-cutting concerns
    (RBAC, event logging, INDEX.md) embedded in each entity-modifying skill.
24. **Revised skill packages**: core/tracking/processes/planning/governance/collaboration/
    artifacts. Four presets: minimal, managed, software, full-product.
25. **Skill naming**: Long descriptive names (`workitem-management`, `decision-record-management`).
    Pattern: `<noun>-management` for CRUD, `<noun>-<verb>` for actions.
26. **Skill hierarchy**: Skills reference lower-layer skills by name in their instructions.
    `uses:` field in frontmatter documents dependencies. Strictly downward.
27. **Skill size**: One skill per primitive, ~100-200 lines. Split only above ~250 lines.
28. **Template originals**: `aibox init` stores originals in `context/.aibox/templates/`.
    Each `aibox update` adds new version. `aibox migrate` generates diffs + migration prompts.
    Derived project customization via direct editing — originals available for comparison.
29. **Three layers of process**: Primitive mechanics (aibox, always), micro-processes (aibox,
    optional), macro-frameworks (kaits/community). aibox does NOT ship SAFe, Scrum, etc.
    Optional community framework packages via `aibox process install`.
30. **Process packages = primitive activation tiers**: minimal/managed/software/research/
    full-product activate progressively more primitives. They are NOT framework choices.
31. **Personas and user stories**: Fit existing primitives. Persona = Actor (subtype: persona).
    User story = Work Item (subtype: story). No new primitives needed.
32. **Revised core packages**: minimal, managed, software, research (expanded), editorial
    (new), consulting (new), full-product. Seven packages total.
33. **Community process packages**: Git repos with package.yaml + context/ + skills/.
    Installed via `aibox process install <url>`. Validated via `aibox process check`.
34. **6 personas defined**: Alex (solo dev), Priya (scientist), Maria (team lead),
    Sam (consultant), kaits (orchestrator), Jordan (content producer).
35. **Hybrid audit model**: Three logging channels — provider hooks (deterministic, what),
    agent event-log (probabilistic, why), aibox sync (deterministic, infrastructure).
    Optional `[audit]` section in aibox.toml.
36. **No new primitives for software**: 17 primitives cover full software lifecycle through
    composition. Environments, feature flags, dependencies modeled as subtypes.
37. **4 additional process templates**: incident-response, technical-design, spike-research,
    hotfix. Must-have for software package v1.
38. **10 scenarios validated**: All walked through in detail. 14 issues found and resolved.
    Full walkthroughs in DISC-001-personas-and-scenarios.md appendix.
39. **Naming standardized**: Gate (not Checkpoint), Scope (not Project) everywhere.
    Primitives get top-level dirs under context/ (not nested under items/).
40. **Two filename patterns**: Human-named (Pattern A) for low-volume entities, word-id
    + content-slug (Pattern B) for high-volume. Human readability preserved.
41. **INDEX.md is structural only**: Purpose, schema, subtypes, skills. NO statistics,
    counts, or state groupings. Those come from SQLite index.
42. **SQLite index from init**: Created by `aibox init`, not deferred. Queries work
    from first session.
43. **Human identity via ~/.aibox/identity.toml**: Kubernetes kubeconfig pattern. Local
    file never committed. `aibox auth whoami` command. Cascade: identity.toml → env var →
    provider → git config → ask. Actor files non-sensitive; personal prefs in identity.toml.
44. **Open threads as sub-work-items**: Significant threads become child work items.
    Small threads noted in parent's Open Questions section.
45. **Provider-native scheduling opt-in**: Optional [scheduling] in aibox.toml. Session-
    start check is always the fallback. Provider scheduling is a bonus.
46. **kaits agents use skills**: kaits orchestrates agents, agents use aibox skills.
    Same skills as human agents — no separate kaits-specific file writing.
47. **Identity model: 3 layers**: (1) ~/.aibox/identity.toml (local, never committed),
    (2) Actor registry in context/actor/ (shared, non-sensitive), (3) Role definitions
    in context/role/ (shared, permissions/restrictions in plain English).
48. **RBAC flow**: identity.toml → match to Actor via handle/email → read Actor's roles
    → load Role permissions/restrictions → check on every modifying action. Additive
    model: any role granting permission wins. Event log provides attribution/audit.
49. **Actor types**: human (identity.toml), ai-agent (env var or kaits-assigned),
    service (CI/CD via env var). All types follow same RBAC model.
50. **`aibox auth whoami`**: Displays resolved identity, matched Actor, roles,
    permissions, restrictions, active provider. Inspired by kubectl auth whoami.

**NOTE:** Decisions 9, 19, 20, 21, 35, 43, 47, 48 are **superseded** by the aiadm/aictl
proposal (§2.50). They remain listed here for historical context. The proposal introduces
certificate-based auth, mechanical RBAC enforcement, and deterministic audit logging.
Full impact analysis: `context/research/aiadm-aictl-architecture-2026-03.md`.
Awaiting owner review — see §3.2 for the 6 open questions.
11. **Three-level rule**: All entity .md files follow Level 1 (intro) → Level 2 (overview) →
    Level 3 (details). Directory INDEX.md files provide Level 0.
12. **Filename conventions**: Inverse date prefix for temporal files + content slug for human
    browsing. Word-IDs serve as natural content hints.
13. **No override mechanism**: Derived project owns its files after scaffolding. Direct
    editing by agents. Schema updates via `aibox migrate` producing diffs + migration prompts.
14. **Content-primary artifacts**: Research, work instructions, PRD stay in semantic directories
    with added frontmatter. Only metadata-primary artifacts go to items/artifact/.

## 5. Next Steps

- [x] Research: scaling limits of file-per-entity in git repos — **done**
- [x] Map 16 primitives to storage structure — **done** (`context/research/primitive-mapping-exercise-2026-03.md`)
- [x] Resolve open questions Q1-Q5 from mapping exercise — **done** (§2.11)
- [x] Investigate word-based IDs — **done** (§2.12)
- [x] Investigate Kubernetes-inspired object model — **done** (§2.13)
- [x] Analyze state machine agent-driven model — **done** (§2.14)
- [x] Resolve Actor multi-fill, word-ID sizing, guard execution, override materialization — **done** (§2.20-2.23)
- [x] Establish infrastructure/application boundary — **done** (§2.24-2.28)
- [x] Map primitives to skills — **done** (`context/research/primitive-skills-mapping-2026-03.md`)
- [x] Research aiadm/aictl proposal — **done** (`context/research/aiadm-aictl-architecture-2026-03.md`)
- [ ] **Owner review: aiadm/aictl proposal** — 6 open questions need owner input (§2.50)
- [ ] Re-walk 10 validation scenarios under aiadm/aictl model
- [ ] Update mapping exercise document with all accumulated decisions
- [ ] Design new `context/` directory layout with sharding
- [ ] Design YAML frontmatter schemas per primitive type (now with apiVersion/kind/metadata/spec)
- [ ] Curate word list for petname-based IDs
- [ ] Implement `event-log` skill (Phase 1 foundation)
- [ ] Implement `workitem` skill (Phase 1, rewrite backlog-context)
- [ ] Implement `decision` skill (Phase 1, rewrite decisions-adr)
- [ ] Prototype: convert BACKLOG.md to file-per-entity format
- [ ] Record formal decisions in DECISIONS.md
- [ ] Session handover: capture full context for next session
