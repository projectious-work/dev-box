---
id: DISC-002
title: "aibox Refocus — Core Principles and Scope"
status: active
date: 2026-04-05
participants: [owner, claude]
related: [DISC-001]
---

# DISC-002: aibox Refocus — Core Principles and Scope

## 1. Problem Statement

DISC-001 explored the context system redesign in depth: 18 primitives, tiered scaling
to enterprise, three-repo trust architectures, certificate-based RBAC, verification
manifests, and Kubernetes-inspired authorization. The exploration was valuable but
produced 74 decisions with 14 internal contradictions — a sign that the scope had
expanded beyond what aibox should be.

This discussion restarts from first principles. What IS aibox? What does it own? What
belongs elsewhere?

## 2. What aibox IS

**aibox is a CLI tool that provides consistent, containerized development environments
for working with AI coding agents.**

Analogy: uv is for Python environments. aibox is for AI work environments.

What you get when you run `aibox init`:
- A dev container configured for your project's language/stack
- Skills and process templates scaffolded into your project
- AI provider configuration (CLAUDE.md, mcp.json) ready to use
- A `context/` directory structure for project management artifacts

## 3. Core Principles

### P1: Dev container first

aibox's primary artifact is a dev container. It should be slim, configurable, and
production-ready for AI-assisted development. The container includes the toolchain
for the project's language, the AI provider's CLI, and the runtime for skill MCP servers.

### P2: No inner system fallacy

aibox does NOT re-expose Docker/docker-compose options behind its own configuration
layer. If you need a custom Docker option, you edit the Dockerfile or docker-compose.yml
directly. aibox configures what is ABOVE Docker: which skills to install, which process
template to use, which AI provider to scaffold for. aibox.toml contains aibox-specific
configuration, not Docker configuration with extra steps.

### P3: Skills are complex, multi-artifact packages

A skill is not a single markdown file. It is a package containing:
- **Instructions** — markdown following the three-level principle (§3.4)
- **Examples** — showing the agent what good output looks like (code, documents)
- **Templates** — scaffolding for new entities (YAML frontmatter templates)
- **Functionality** — Python MCP server source code that provides tool capabilities

Skills are developed and refined in the process repo, consumed by product repos.

### P4: Three-level principle

All instruction markdown follows three levels of detail:
- **Level 1 (intro):** 1-3 sentences. Enough for the agent to decide if this is relevant.
- **Level 2 (overview):** Key concepts and workflows. Enough to act in common cases.
- **Level 3 (details):** Full reference. Edge cases, examples, field-by-field specs.

Directory INDEX.md files provide **Level 0**: what lives in this directory and why.

### P5: 18 primitives as universal building blocks

The process ontology from DISC-001 identified 18 universal process primitives:

1. Work Item — unit of work (task/story/issue/ticket)
2. Log Entry / Event — immutable record of something that happened
3. Decision Record — choice with rationale
4. Artifact — any produced output
5. Role — named set of responsibilities
6. Process / Workflow — sequence of steps with decision points
7. State Machine — set of states with allowed transitions
8. Category / Taxonomy — classification system
9. Cross-Reference / Relation — typed link between entities
10. Gate — validation point
11. Metric / Measure — quantified observation
12. Schedule / Cadence — time-based trigger
13. Scope / Container — boundary grouping related items
14. Constraint — rules/limits
15. Context / Environment — ambient knowledge and conditions
16. Discussion — structured conversation producing decisions
17. Actor — person or agent (preferences, expertise, working style)
18. RoleBinding — assignment of actor to role (with scope)

These primitives are framework-agnostic. They appear in every process methodology
(SAFe, PMBOK, CMMI, Scrum, Kanban). aibox provides them as building blocks; it does
not impose a framework.

### P6: Git-based and provider-independent

Everything is versioned in git. Markdown files are the source of truth for process
artifacts. No mandatory external infrastructure (no databases, no servers, no cloud
services required). Works with any AI provider: Claude Code, GitHub Copilot, Cursor,
Aider, Gemini CLI, self-hosted models.

### P7: Simple — one repo per concern

- **aibox repo:** CLI (Rust) + container images + devcontainer scaffolding
- **Process repo:** Primitives + skills + process templates + MCP server source code

Both repos are developed using aibox dev containers (dogfooding). The process repo
releases as git tags. `aibox init` consumes a specific release version.

### P8: Skill MCP servers are Python source code

MCP server functionality within skills is shipped as Python source code. The consuming
dev container runs the code. Python chosen for: readability, modifiability (teams can
adapt skills), and universal availability in dev containers.

MCP servers use the STDIO transport (standard, most interoperable). Configuration
follows the standard `mcpServers` JSON format supported by all major AI providers.

### P9: Enterprise governance is out of scope

RBAC enforcement, multi-repo trust architectures, certificate-based authorization,
verification manifests, and compliance auditing belong to a separate platform — likely
Kubernetes-based. aibox provides the development environment and process structure.
Governance enforcement is a different concern for a different tool.

DISC-001 (preserved as-is) contains extensive research on these topics for future use.

### P10: Kubernetes-inspired object model for entities

All entity files use structured YAML frontmatter: `apiVersion`, `kind`,
`metadata` (id, timestamps, labels), `spec` (entity-specific fields). This enables
schema versioning, validation, and future migration tooling.

### P11: Slim base image + composable addons

aibox provides a single base image (`debian:trixie-slim`) with essential dev tooling
(git, zellij, yazi, vim, ripgrep, fd, bat, fzf, delta, starship, lazygit, gh).

Everything else is an **addon** — a YAML definition that composes onto the base:
- **Languages:** python, rust, node, go, latex, typst
- **Tools:** kubernetes, infrastructure, cloud-aws/azure/gcp
- **Docs:** docusaurus, hugo, mdbook, mkdocs, starlight, zensical
- **AI providers:** claude, aider, gemini, mistral

Each addon declares **opinionated versions** — a curated set of supported versions with
a sensible default (e.g., Python 3.12/3.13/3.14, default 3.13). Users override in
aibox.toml. aibox does NOT attempt to support every version — it curates.

Addons compose via Minijinja-rendered Dockerfile stages. Heavy addons (Rust, LaTeX,
Kubernetes) use parallel builder stages. The final image stays slim — only runtime
artifacts are copied from builders.

**Already implemented.** 22 addons, version-validated, template-rendered, generating
`.devcontainer/Dockerfile` + `docker-compose.yml` + `devcontainer.json`.

### P12: Binding as generalized primitive

The 18th primitive is Binding (generalized from RoleBinding). A Binding connects any
two entities with optional scope, temporality, and conditions. See §11 for full analysis.

**Rule:** If a relationship has scope, time, or its own attributes → Binding entity.
If it's just "A relates to B" → cross-reference in frontmatter.

### P13: Community process packages

Process packages are git repos with a standard structure:
```
package.yaml        # metadata, requirements, provides
context/            # files to merge into project context/
skills/             # custom skills
```

Installable via `aibox process install <git-url>`. Validated via `aibox process check`.
Enables company-to-community flow: customize → refine → export → publish.

### P14: Template originals for migration

`aibox init` stores original template copies in `context/.aibox/templates/v{version}/`.
On version updates, `aibox migrate` generates diffs (old → new template) and produces
migration prompts. The agent reviews the diff against the project's current state and
applies changes with human approval. No automatic in-place patching.

### P15: Skill hierarchy

Skills reference lower-layer skills by name. `uses:` field in frontmatter documents
dependencies. Strictly downward — lower-layer skills never reference higher-layer ones.

```
Layer 0: event-log (foundation)
Layer 1: role-management, actor-profile
Layer 2: workitem-management, decision-record, scope-management
Layer 3: process-management, gate-management, schedule-management
Layer 4: discussion-management, metrics-management
```

## 4. What aibox is NOT

- **Not a workflow engine.** aibox does not execute processes. Agents do.
- **Not an enterprise governance platform.** No RBAC enforcement, no certificates,
  no signed commits, no authorization policies. That's a separate tool.
- **Not a project management tool.** It provides primitives for project management;
  the agent and the user manage the project.
- **Not a CI/CD system.** It provides a dev environment. Build/deploy is the
  project's concern.
- **Not a Docker wrapper.** It scaffolds containers; it doesn't abstract Docker.

## 5. Two Repos

### 5.1 aibox repo

**Purpose:** CLI tool + container images for AI-assisted development.

**Contains:**
```
cli/                    ← Rust CLI source code
  src/
    main.rs
    container/          ← container lifecycle (init, start, stop)
    scaffold/           ← project scaffolding (process, skills, config)
images/                 ← container image definitions (10 flavors)
templates/              ← devcontainer templates
.devcontainer/          ← aibox's own dev environment (dogfooding)
context/                ← aibox's own process context (via aibox)
```

**Key commands:**
- `aibox init` — create dev container + scaffold context/skills from process repo release
- `aibox start` / `aibox stop` — container lifecycle
- `aibox lint` — validate context files against primitive schemas
- `aibox sync` — rebuild derived index from context files
- `aibox image build` / `aibox image push` — container image management

### 5.2 Process repo

**Purpose:** Primitives, skills, process templates — consumed by aibox and future tools.

**Contains:**
```
primitives/
  schemas/              ← YAML frontmatter schemas per primitive type
  state-machines/       ← default state machine definitions
skills/
  workitem-management/
    SKILL.md            ← 3-level instructions
    examples/           ← example outputs
    templates/          ← entity scaffolding templates
    mcp/
      server.py         ← MCP server source code
      requirements.txt  ← Python dependencies
  decision-record-management/
    ...
  event-log-management/
    ...
processes/
  code-review.md        ← process definition
  release.md
  incident-response.md
packages/
  minimal.yaml          ← which skills/primitives to activate
  managed.yaml
  software.yaml
  research.yaml
  full-product.yaml
.devcontainer/          ← own dev environment (aibox dogfooding)
context/                ← own process context (via aibox)
```

**Releases:** Git tags (semver). aibox consumes a specific release version.

## 6. Skill Anatomy

A skill is a complete package for one primitive or one cross-cutting concern:

```
skills/workitem-management/
  SKILL.md                      ← Agent instructions (3-level)
    Level 1: "Use this skill to create, update, and manage work items."
    Level 2: How to create, transition states, link items, log events
    Level 3: Full field reference, all state transitions, edge cases

  examples/
    create-feature.md           ← Example: creating a feature work item
    create-bug.md               ← Example: creating a bug report
    transition-to-review.md     ← Example: state transition with guard check

  templates/
    workitem.yaml               ← YAML frontmatter template for new work items
    workitem-bug.yaml           ← Variant template for bugs
    workitem-story.yaml         ← Variant template for user stories

  mcp/
    server.py                   ← MCP server (Python, STDIO transport)
    requirements.txt            ← Dependencies (minimal)
    mcp-config.json             ← Config snippet for consumer's mcpServers
    README.md                   ← What this server provides (tools, resources)
```

The MCP server provides tool capabilities the agent can call programmatically:
- `create_workitem(title, type, assignee)` — creates file with correct schema
- `transition_workitem(id, to_state)` — validates state machine, updates file
- `query_workitems(state, assignee)` — reads from SQLite index
- `link_workitems(from_id, to_id, relation)` — adds cross-reference

The agent can also do all of this by editing files directly (following SKILL.md
instructions). The MCP server is the mechanical-correctness path; direct editing is
the probabilistic path. Both are valid.

## 7. Open Questions

### Q1: Process repo naming (resolved)

Name: **processkit**. Lives in `projectious-work/processkit`. Owner to create.

### Q2: SQLite index — aibox or process repo concern? (resolved)

**Option B: process repo MCP servers (Python).** The index is a process concern.
The MCP server parses files, builds SQLite, serves queries. Schema changes are
self-contained in processkit. aibox CLI does basic structural validation only
(file exists, has frontmatter, has `kind` field) without full schema knowledge.

### Q3: Which DISC-001 principles to carry forward? (resolved)

Owner decisions:

**Carry forward (modified):**
- **File-per-entity storage** — yes, as-is
- **Markdown+frontmatter as source of truth, SQLite as derived index** — yes, as-is
- **ID format configurable** — user chooses in aibox.toml between UUID-based or
  word-based IDs (petname). Independent of that, user chooses whether to add content
  slugs or not. All four combinations valid. Resolves DISC-001 contradiction between
  Decision 4 and Decision 40.
- **Directory sharding** — yes, configurable per entity type
- **Process packages as activation tiers** — yes, as-is
- **Actor types (human, ai-agent, service)** — yes, as-is
- **Identity via ~/.aibox/identity.toml** — yes, as-is

**Modified:**
- **Event log: all probabilistic.** No dual event sources. Agent logs everything via
  event-log skill. Deterministic event enforcement is another project's concern.
  aibox may still write infrastructure notes (lint warnings, sync results) but these
  are informational, not an authoritative event log.

**Dropped (another project):**
- RBAC enforcement in Role definitions. Role remains a primitive (describes
  responsibilities) but has no enforcement semantics in aibox.
- All enterprise governance schemas (signed definitions, verification manifests, etc.)

**Under investigation:**
- RoleBinding as 18th primitive vs generalized Binding primitive. See §11.

### Q4: Minimal MCP execution environment (resolved)

Python >= 3.10 + uv. Official SDK + PEP 723 inline dependencies. See §8.

## 8. Minimal Python MCP Execution Environment

### The constraint

Skill MCP servers must run inside the dev container. The container should stay slim.
What is the minimum needed?

### MCP protocol basics

MCP over STDIO is just JSON-RPC 2.0 over newline-delimited streams. At the protocol
level, it needs only `json` and `asyncio` — both in Python's standard library.

### Official Python MCP SDK

The official SDK (`mcp` package, v1.27.0) requires Python >= 3.10 and pulls in heavy
dependencies:

- pydantic >= 2.12 (validation framework)
- starlette >= 0.27 (ASGI web framework — unnecessary for STDIO)
- uvicorn >= 0.31 (ASGI server — unnecessary for STDIO)
- httpx >= 0.27 (HTTP client — unnecessary for STDIO)
- opentelemetry-api >= 1.28 (observability — unnecessary for basic use)
- pyjwt, jsonschema, sse-starlette, httpx-sse, python-multipart...

**Assessment:** ~15+ transitive dependencies. Designed for all transports (HTTP, SSE,
WebSocket). For STDIO-only, most of this is dead weight. Image size: ~300-400 MB with
`python:3.10-slim` base.

### Three options, lightest to heaviest

**Option A — Raw JSON-RPC (zero dependencies):**

A minimal STDIO MCP server needs only Python stdlib (`json`, `asyncio`, `sys`).
~40-100 lines of JSON-RPC boilerplate. No pip install, no virtual environment.

Trade-off: No automatic schema validation, no tool discovery generation. You write
the protocol handling yourself. Suitable for simple tools, not for complex skill servers
with many operations.

**Option B — Pydantic only (~1 dependency):**

Use Pydantic for request/response validation + manual JSON-RPC implementation.
Gets validation and type safety without the full SDK framework stack.

Image size: ~180-220 MB on `python:3.10-slim`.

**Option C — Official MCP SDK (full):**

Use the official SDK. Accept the dependency weight. Benefit: automatic tool discovery,
schema generation, protocol compliance, community support.

Image size: ~300-400 MB on `python:3.10-slim`.

### Delivery mechanism: uv with PEP 723 inline dependencies

aibox dev containers already include `uv`. MCP servers can use PEP 723 inline script
metadata — no pyproject.toml, no virtual environment setup:

```python
#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["mcp[cli]"]
# ///

from mcp.server.fastmcp import FastMCP

server = FastMCP("workitem-management")

@server.tool()
def create_workitem(title: str, type: str = "task") -> str:
    ...

if __name__ == "__main__":
    server.run(transport="stdio")
```

First run: uv resolves and caches the environment (~5-10s). Subsequent runs: near-instant
(cached). No global Python packages polluted. Each skill's MCP server has isolated deps.

### Recommendation

**Option C (official SDK) + uv inline dependencies.** Rationale:

- The official SDK is the standard. Fighting it means maintaining custom protocol code.
- uv caching makes the dependency weight a first-run cost, not a per-invocation cost.
- The dev container already has Python and uv. No additional base image change needed.
- ~300-400 MB is acceptable for a dev container (which already has Rust, Node, etc.).
- If container size becomes critical later, Option B is the escape hatch.

**What aibox needs to provide in the container:**
- Python >= 3.10 (already present in all aibox images)
- uv (already present in all aibox images)
- That's it. No pre-installed MCP packages. uv handles everything at first run.

## 9. Relationship to DISC-001

DISC-001 is preserved as-is. It contains valuable research and exploration:

- **Carry forward to aibox (DISC-002):** Primitives, skills, three-level principle,
  file-per-entity, word-based IDs, Kubernetes object model, process packages, dual
  event sources, identity model.
- **Defer to future platform:** Three-repo trust architecture, signed process
  definitions, RoleBindings as enforcement (vs as process primitive), verification
  manifests, per-file authorization policy, CI audit pipelines, tiered scaling
  beyond Tier 0-1.
- **Drop / rethink:** aiadm/aictl binary split (resolved: one binary), OS-level
  filesystem lockdown (resolved: out of scope), daemon mode.

## 11. Investigation: Binding as a Generalized Primitive

### The question

DISC-001 introduced RoleBinding as the 18th primitive (actor + role + scope). Owner
asks: should this be generalized? The pattern of "binding two things together so one
can change without changing the other" is a fundamental principle in programming
(GoF patterns, SOLID, dependency inversion). Does it deserve to be a general primitive?

### The pattern in programming

The Gang of Four identified several patterns that are all forms of indirection:

| Pattern | What it binds | Why |
|---------|---------------|-----|
| Adapter | Client ↔ Service (incompatible interfaces) | Decouple interface from implementation |
| Bridge | Abstraction ↔ Implementation | Vary both independently |
| Strategy | Context ↔ Algorithm | Swap behavior without changing the user |
| Mediator | Colleague ↔ Colleague | Decouple many-to-many interactions |
| Dependency Injection | Consumer ↔ Dependency | Change implementation without changing consumer |

The common principle: **put an indirection between two things so either can change
independently.** In refactoring terms: "program to an interface, not an implementation."

### The pattern in process management

Where does this indirection appear with our 18 primitives?

**Currently modeled as references in frontmatter (tight coupling):**

```yaml
# Work item directly references an actor
kind: WorkItem
spec:
  assigned_to: ACTOR-alice        # Changing assignee = editing work item
```

```yaml
# Process directly references gates
kind: Process
spec:
  gates: [GATE-code-review, GATE-security-scan]   # Changing gates = editing process
```

```yaml
# Actor directly references roles
kind: Actor
spec:
  roles: [ROLE-developer, ROLE-architect]   # Changing roles = editing actor
```

**The problem with tight coupling:** To change Alice's role, you edit Alice's Actor file.
To change which gate applies to a process in a specific scope, you edit the Process
file. Every relationship change requires modifying one of the two endpoints.

**With a Binding primitive (loose coupling):**

```yaml
# Separate binding entity — neither endpoint changes
kind: Binding
spec:
  type: role-assignment
  subject: ACTOR-alice
  target: ROLE-developer
  scope: SCOPE-project-x
  valid_from: 2026-01-01
  valid_until: 2026-12-31
```

```yaml
kind: Binding
spec:
  type: process-gate
  subject: PROC-release
  target: GATE-security-scan
  scope: SCOPE-project-x
```

```yaml
kind: Binding
spec:
  type: assignment
  subject: BACK-swift-oak
  target: ACTOR-alice
  scope: SCOPE-sprint-42
```

### When bindings add value vs when they're overhead

**Bindings add value when:**

1. **The relationship is scoped.** "Alice is a developer *on project X*" — the scope
   means the same actor-role pair has different truth in different contexts. You can't
   express this in Alice's Actor file without listing every project.

2. **The relationship is temporal.** "Alice fills the tech lead role *from Jan to June*"
   — the time dimension makes it a separate fact, not a property of Alice or the Role.

3. **The relationship is many-to-many.** Multiple actors fill multiple roles on multiple
   projects. This is a junction table problem — the relationship itself has attributes
   (scope, time, conditions) that belong to neither endpoint.

4. **You want to change the relationship without touching either endpoint.** "Move the
   security gate from the release process to the deploy process" — modify one binding
   entity instead of editing two process files.

5. **You want to audit relationship changes independently.** Git history of a binding
   file shows exactly when a role assignment changed, without noise from other changes
   to the actor or role file.

**Bindings are overhead when:**

1. **The relationship is 1:1 and permanent.** "This work item is of type: bug" — just a
   field in frontmatter. No need for indirection.

2. **The relationship has no scope or time dimension.** "This decision is linked to that
   work item" — a simple cross-reference in frontmatter.

3. **Solo developer.** One person, one project, no scoping needed. References in
   frontmatter are simpler.

### The database analogy

In relational databases:

| Relationship type | Modeling | Analogy |
|---|---|---|
| 1:1 or 1:many | Foreign key in the child table | Reference in frontmatter |
| Many-to-many | Junction table | Binding entity |
| Many-to-many with attributes | Junction table with extra columns | Binding entity with scope/time/conditions |

RoleBinding is a junction table between Actor and Role. The question is: are there
enough many-to-many-with-attributes relationships to justify a general Binding primitive?

### Inventory of relationships that benefit from bindings

| Binding type | Subject | Target | Why not just a reference? |
|---|---|---|---|
| role-assignment | Actor | Role | Scoped per project, temporal, auditable |
| work-assignment | WorkItem | Actor | Temporal (sprint-scoped), re-assignable |
| process-gate | Process | Gate | Scoped (different gates per project) |
| process-scope | Process | Scope | Which process applies where |
| schedule-scope | Schedule | Scope | Which cadence applies to which project |
| constraint-scope | Constraint | Scope | Which constraints apply where |
| category-assignment | any entity | Category | Could be scoped/temporal |

At least 7 relationship types benefit from bindings. This is not a one-off pattern.

### Recommendation: generalize RoleBinding to Binding

**Rename the 18th primitive from RoleBinding to Binding.** A Binding connects any two
entities with optional scope, temporality, and conditions.

```yaml
apiVersion: aibox/v1
kind: Binding
metadata:
  id: BIND-calm-fox
spec:
  type: role-assignment           # binding type (freeform, conventions per process)
  subject: ACTOR-alice            # the entity being bound
  target: ROLE-developer          # what it's being bound to
  scope: SCOPE-project-x          # where this binding applies (optional)
  valid_from: 2026-01-01          # temporal start (optional)
  valid_until: 2026-12-31         # temporal end (optional)
  conditions:                     # arbitrary conditions (optional)
    approval_required: true
  description: "Alice fills Developer role on Project X for 2026"
```

**What stays as references:** Simple, unscoped, permanent relationships remain as
frontmatter fields. "This work item blocks that work item" = cross-reference. "This
decision relates to that work item" = cross-reference. No need to promote everything
to bindings.

**The rule:** If a relationship has **scope**, **time**, or **its own attributes** —
it's a Binding. If it's just "A relates to B" — it's a cross-reference in frontmatter.

**Impact on primitive count:** Still 18. RoleBinding is renamed to Binding, not added
alongside it.

### Open question for owner

Does this generalization feel right? The trade-off:
- **Pro:** One primitive handles all scoped/temporal relationships. Cleaner, more
  powerful, aligns with established software design principles.
- **Con:** More abstract. "Create a Binding" is less intuitive than "Create a
  RoleBinding." The skill/MCP server needs to handle multiple binding types.

## 12. Decisions (DISC-002)

1. **aibox = dev container + skills scaffolding.** Not a workflow engine, not a
   governance platform, not a Docker wrapper.
2. **Two repos from the start.** aibox repo (Rust CLI + images) and process repo
   (primitives + skills + processes). Process repo releases as git tags.
3. **Skills are multi-artifact packages.** Markdown instructions (3-level) + examples +
   templates + Python MCP servers.
4. **MCP servers use official SDK + uv PEP 723 inline dependencies.** Container needs
   only Python >= 3.10 and uv (both already present).
5. **ID format is configurable.** User chooses in aibox.toml: UUID or word-based (petname).
   Independently: with or without content slugs. Four combinations, all valid.
6. **Event log is all probabilistic.** Agent logs events via skill. No dual event sources.
   Deterministic enforcement is another project.
7. **Directory sharding included.** Configurable per entity type.
8. **RBAC enforcement is out of scope.** Role remains a primitive (describes responsibilities).
   No enforcement semantics in aibox.
9. **Enterprise governance is another project.** Likely Kubernetes-based. DISC-001 research
   preserved for future use.
10. **No inner system fallacy.** aibox.toml for aibox concerns only. Docker options edited
    directly in Dockerfile/docker-compose.

## 15. Gap Analysis — What Exists vs What's Needed

### Already implemented in aibox

| Component | Status | Location |
|---|---|---|
| CLI (Rust) | Working | `cli/` |
| Addon system (22 addons, YAML + Minijinja) | Working | `addons/` |
| Container image generation | Working | `images/`, `cli/src/generate.rs` |
| Container lifecycle (start/stop) | Working | `cli/src/container.rs` |
| 83 skills (simple markdown) | Working | `templates/skills/` |
| 4 process templates (simple markdown) | Working | `templates/processes/` |
| Package tiers (minimal/managed/research/product) | Working | `templates/` |
| aibox.toml config parsing | Working | `cli/src/config.rs` |
| CLAUDE.md scaffolding | Working | via templates |

### Gaps — need implementation

| Gap | Description | Where |
|---|---|---|
| **processkit repo** | Doesn't exist yet. Must be created, populated, scaffolded with aibox | New repo |
| **Skill format migration** | Current 83 skills are single .md files. Need to become multi-artifact packages (SKILL.md + examples/ + templates/ + mcp/) | processkit |
| **MCP servers for skills** | No MCP server code exists. Need at least foundation skills (event-log, workitem, decision) | processkit |
| **Primitive schemas** | No YAML schema definitions for the 18 primitives exist | processkit |
| **Entity file format** | The apiVersion/kind/metadata/spec frontmatter format isn't implemented in any real files | processkit |
| **SQLite index** | No indexing logic exists. Per Q2, this lives in a processkit MCP server | processkit |
| **ID generation** | Configurable UUID vs word-based (petname). Not implemented | aibox CLI |
| **aibox init → processkit consumption** | `aibox init` currently scaffolds from local templates. Needs to consume processkit git tag releases | aibox CLI |
| **aibox lint for entities** | Basic structural validation of context files (has frontmatter, has `kind`). Not implemented | aibox CLI |
| **Binding primitive** | New primitive (generalized from RoleBinding). Schema, skill, MCP server needed | processkit |
| **Three-level skill rewrite** | Existing 83 skills need review for three-level structure. Many are already close. | processkit |
| **mcp.json scaffolding** | `aibox init` should generate MCP server config entries for installed skills | aibox CLI |

### Key decision needed: what happens to existing 83 skills?

The current 83 skills in `templates/skills/` are technical coding skills (rust-conventions,
python-best-practices, database-modeling, etc.) — NOT process primitive skills (workitem-
management, decision-record, event-log, etc.).

These are two different categories:

| Category | Examples | Where they live |
|---|---|---|
| **Process skills** (primitive management) | workitem-management, event-log, decision-record | processkit |
| **Technical skills** (coding practices) | rust-conventions, python-best-practices, sql-patterns | ? |

**Options:**
- **(a)** Technical skills move to processkit too (one place for all skills)
- **(b)** Technical skills stay in aibox (they're tied to addons — rust-conventions ships with the rust addon)
- **(c)** Technical skills become a separate package/addon in aibox (not in processkit)

Option (b) makes sense: technical skills are addon-specific. The rust-conventions skill
ships when you enable the rust addon. Process skills are project-management-specific and
ship from processkit when you choose a package tier.

**Recommendation: (b)** — technical skills stay in aibox, tied to addons.

Awaiting owner input. This is the last blocking decision before the implementation plan.

## 13. Next Steps

- [x] ~~Owner review: Binding as generalized primitive (§11)~~ — approved
- [x] ~~Name the process repo (Q1)~~ — resolved: **processkit** (projectious-work/processkit)
- [x] ~~Decide where SQLite index logic lives (Q2)~~ — resolved: Option B (process repo MCP servers)
- [ ] Record formal decisions in DECISIONS.md
- [ ] Create process repo and scaffold it with aibox

## 14. SQLite Index Logic — Where Does It Live?

The index logic = code that parses markdown+frontmatter files, builds SQLite tables,
and provides query capabilities. This includes knowing primitive schemas.

**Option A — In aibox CLI (Rust):**
`aibox sync` does everything: parse files, know schemas, write SQLite, provide queries.
Fast (compiled Rust + rusqlite). But: schemas are defined in the process repo. Adding a
primitive or changing a schema requires an aibox CLI update. Tight coupling between repos.

**Option B — In process repo MCP servers (Python):**
An MCP server owns indexing: parse files, build SQLite, serve queries to agents via tools.
Schema changes are self-contained in the process repo. No coupling to aibox CLI. The MCP
server IS the query interface. `aibox lint`/`aibox sync` do basic structural validation
only (file exists, has frontmatter, has `kind` field) without full schema knowledge.

**Option C — Split: aibox generic parsing, process repo schema-aware indexing:**
`aibox sync` parses ALL frontmatter into generic key-value store. MCP server reads the
generic index and applies schema-specific logic. Loosely coupled but two-step.

**Resolved: Option B.** Process repo MCP servers own indexing. `aibox lint`/`aibox sync`
do basic structural validation only. Schema knowledge stays in processkit.
