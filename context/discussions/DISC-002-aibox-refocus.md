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

### Q1: Process repo naming

What is the process repo called? Options:
- `aibox-processes` (tied to aibox brand)
- `process-kit` (generic)
- `ai-process-kit` (descriptive)
- Something else?

### Q2: SQLite index — aibox or process repo concern?

The SQLite derived index (gitignored, rebuilt by `aibox sync`) was designed in DISC-001.
Does the index logic live in aibox CLI (Rust) or in the process repo (Python MCP)?
- If in aibox CLI: fast, compiled, but couples aibox to the schema
- If in MCP server: flexible, schema changes don't require CLI updates, but slower

### Q3: Which DISC-001 principles to carry forward?

Candidate principles from DISC-001 not yet covered in DISC-002:

- **File-per-entity storage** (DISC-001 §2.3): Each entity = own markdown file. Minimal
  merge conflicts. Git-native diffs.
- **Word-based IDs** (DISC-001 §2.12, Decision 4): `BACK-swift-oak` format via petname
  crate. ~20M combinations per prefix type.
- **Two filename patterns** (DISC-001 §2.45, Decision 40): Pattern A (human-named) for
  low-volume entities, Pattern B (word-id + slug) for high-volume.
- **Markdown+frontmatter as source of truth, SQLite as derived index** (Decision 1):
  No dual-master.
- **Directory sharding** (DISC-001 §2.15, Decision 10): Configurable per entity type.
- **Process packages as activation tiers** (DISC-001 §2.38, Decision 30): Packages
  activate progressively more primitives. Not framework choices.
- **Dual event sources** (DISC-001 §2.32): Agent logs process events (probabilistic),
  aibox logs infrastructure events (deterministic).
- **Actor types** (DISC-001 §2.20, Decision 49): human, ai-agent, service. All use same
  process model.
- **Identity via ~/.aibox/identity.toml** (DISC-001 §2.48): Local, never committed.
  Cascade: identity.toml → env var → provider → git config.

Owner to review: which of these carry forward as-is, which need modification, which
are dropped.

### Q4: Minimal MCP execution environment

What is the smallest Python runtime needed in the dev container for skill MCP servers?
See §8.

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

## 10. Next Steps

- [ ] Owner review: Q3 (which DISC-001 principles carry forward)
- [ ] Research: minimal Python MCP environment for dev containers (Q4)
- [ ] Name the process repo (Q1)
- [ ] Decide where SQLite index logic lives (Q2)
- [ ] Record formal decisions
- [ ] Create process repo and scaffold it with aibox
