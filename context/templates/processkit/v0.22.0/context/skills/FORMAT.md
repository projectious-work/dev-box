---
apiVersion: processkit.projectious.work/v1
kind: Documentation
metadata:
  id: SKILL-FORMAT
  title: "Skill Package Format Specification"
  version: "2.0.0"
spec:
  level: 3
---

# Skill Package Format

## Intro

A **skill** in processkit is a directory, not a single file. It bundles
agent instructions (`SKILL.md`), supporting documentation (`references/`),
output assets (`assets/`), executable scripts (`scripts/`), example
outputs (`examples/`), and optionally a Python MCP server (`mcp/`) into
one composable package. Skills are the primary way processkit gives
agents new capabilities.

processkit's skills follow the **[Agent Skills](https://agentskills.io)**
open standard (donated by OpenAI and Anthropic to the Linux Foundation's
Agentic AI Foundation in December 2025) so they load identically in
Claude.ai, Claude Code, Cursor, Codex, Gemini CLI, and any other
harness that consumes the standard. processkit-specific metadata
(category, layer, dependencies, primitives, MCP tools) lives under an
optional `metadata.processkit:` sub-block that other harnesses ignore.

## Overview

### Directory layout

```
src/skills/<skill-name>/
  SKILL.md              ← REQUIRED — the only file Claude reads automatically
  scripts/              ← optional — executable code (Python, Bash); may be empty
  references/           ← optional — deep-dive reference docs, loaded on demand
  assets/               ← optional — templates, fonts, icons used in output
  examples/             ← optional — example outputs (kept; not in Anthropic canonical layout)
  commands/             ← optional — slash-command adapter files; one per user-invocable workflow
    <skill-name>-<workflow>.md  ← provider-specific frontmatter + one-liner invoking the skill
  mcp/                  ← optional — Python MCP server (processkit-specific)
    server.py           ← PEP 723 inline deps, STDIO transport, official SDK
    mcp-config.json     ← config fragment merged by the installer into the harness
    SERVER.md           ← what tools / resources / env / limitations the server provides
```

`<skill-name>` is a kebab-case identifier unique across processkit (and
any community skill packages a project installs).

**Two prohibitions:**

- **No `README.md` inside the skill folder** — Anthropic's hard rule. All
  documentation belongs in `SKILL.md` or `references/`. The MCP server
  documentation lives at `mcp/SERVER.md` instead.
- **No `INDEX.md` inside the skill folder** — the parent
  `src/skills/INDEX.md` indexes the whole catalog; per-skill indexing
  would be redundant.

### SKILL.md frontmatter (Agent Skills standard)

Every `SKILL.md` starts with YAML frontmatter following the open
[Agent Skills](https://agentskills.io) standard:

```yaml
---
name: workitem-management
description: |
  Create, transition, query, and link WorkItems — tasks, stories, bugs,
  spikes, epics, chores. Use when the user asks to add, update, query,
  or link work items, or says things like "add to backlog", "mark this
  done", "what's blocked", "create a bug for X", or "what's in review".
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-workitem-management
    version: "1.0.0"
    created: 2026-04-06T00:00:00Z
    category: process
    layer: 2
    uses:
      - skill: event-log
        purpose: "Log workitem events to keep the audit trail accurate after every write."
      - skill: index-management
        purpose: "Query existing workitems before proposing new ones; reindex after writes."
      - skill: id-management
        purpose: "Allocate unique BACK-<id> identifiers via central ID generation."
    provides:
      primitives: [WorkItem]
      mcp_tools: [create_workitem, transition_workitem, query_workitems, link_workitems]
      assets: [workitem, workitem-bug, workitem-story]
---
```

### Top-level (Agent Skills) fields

| Field         | Required | Purpose |
|---------------|----------|---------|
| `name`        | yes | Kebab-case identifier — must match the directory name. No spaces, no capitals, no underscores. |
| `description` | yes | What the skill does **and** when to use it. Anthropic's loader uses this to decide whether to load the skill. Must include trigger phrases users would actually say. Under 1024 characters. No XML angle brackets (`<` `>`). |
| `license`     | no  | Optional. Use if making the skill open source (MIT, Apache-2.0, …). |
| `compatibility` | no | Optional, 1-500 chars. Indicates environment requirements. |
| `metadata`    | no  | Optional. Free-form custom key/value pairs. processkit puts everything under `metadata.processkit:` (see below). |

### `metadata.processkit:` fields (processkit-specific)

| Field             | Required | Purpose |
|-------------------|----------|---------|
| `apiVersion`      | yes | Always `processkit.projectious.work/v1` at v0.x. |
| `id`              | yes | `SKILL-<skill-name>` — used by the index. |
| `version`         | yes | Per-skill semver. Independent of processkit's release version. |
| `created`         | yes | ISO 8601 UTC. |
| `category`        | yes | One of the registered categories — see below. |
| `layer`           | depends | Integer 0–4 for entity-management/process skills (see hierarchy). Omit (or use `null`) for category=`language`/`framework`/etc. |
| `uses`            | no  | List of objects with `skill` + `purpose` — see below. Strictly lower-layer for layered skills. |
| `provides`        | no  | What the agent gains: primitives, MCP tools, asset templates, processes. |
| `commands`        | no  | List of user-invocable command descriptors; enables `commands/` adapter files. See `commands/` below. |
| `replaces`        | no  | For customized community forks: ID of the skill this overrides. |

### `uses:` — list of objects with explicit purpose

`uses` is a list of skills this skill **delegates to** at runtime. Each
entry is an object with two fields:

- `skill` — the kebab-case name of the dependency
- `purpose` — one sentence explaining **why** this skill uses the other

The `purpose` field is mandatory. The Zettelkasten community's principle
applies here too: *"If you just add links without any explanation you
will not create knowledge."* Without `purpose`, the agent has to guess
why the dependency exists, and the index has no way to surface the
relationship meaningfully.

```yaml
uses:
  - skill: event-log
    purpose: "Log decision.recorded events to keep the audit trail accurate."
  - skill: index-management
    purpose: "Query existing decisions before recording a new one to surface duplicates."
```

The `uses` field is also a **convention-only delegation marker**:
when an agent reads skill A and sees `uses: [B]`, it knows it can
read skill B's `SKILL.md` for the underlying methodology. Skill A's
SKILL.md should explicitly tell the agent to do so when the
delegation matters (e.g., *"To create a workitem, follow
`workitem-management/SKILL.md`'s create_workitem flow."*). The
harness does not auto-load skill B; the agent does, when skill A
tells it to.

This makes `uses` the **knowledge delegation** field — the
processkit-specific innovation that lets one skill defer methodology
and MCP-tool usage to another instead of duplicating it.

### SKILL.md body structure

After the frontmatter, every `SKILL.md` follows this section structure:

```markdown
# <Skill Title>

## Intro

1-3 sentences. What this skill is and when it applies. Just enough for
the agent to decide whether the skill is relevant to the current task.

## Overview

Key concepts, workflows, and the most common operations. Enough for the
agent to act in typical cases without consulting the Full reference or
the bundled `references/` files.

## Gotchas

REQUIRED. Failure modes specific to AI agents — recurring across providers
(Claude, Gemini, ChatGPT, etc.). Pause-and-self-check items, NOT general
practitioner anti-patterns. The Skills Master Class calls this "the
highest-signal content" — where the model goes wrong. Provider-neutral:
what is obvious to Claude may not be obvious to Gemini.

## Full reference

Edge cases, complete field specs, rare workflows, error modes, anti-patterns
that apply to humans and agents alike. May link to deeper material in
`references/<topic>.md`.
```

The historical "Level 1 / Level 2 / Level 3" headings have been retired.
"Level" now refers to Anthropic's file-level progressive disclosure model
(see below), not section markers inside SKILL.md.

### Progressive disclosure (Anthropic's model)

processkit follows Anthropic's three-level progressive disclosure pattern,
which is **file-level**, not section-level:

| Level | Lives in | Always loaded? |
|-------|----------|----------------|
| 1 | YAML frontmatter (`name`, `description`) | YES — in the harness's system prompt |
| 2 | `SKILL.md` body (Intro / Overview / Gotchas / Full reference) | Loaded when the description triggers |
| 3 | Files in `references/`, `scripts/`, `assets/` | Loaded only when SKILL.md tells the agent to fetch them |

This minimizes token usage while keeping deep material available on
demand. **Keep `SKILL.md` under 5000 words** — Anthropic's recommendation.
For longer skills, push detail into `references/<topic>.md` and link
to it from `SKILL.md`.

### The Gotchas section is mandatory

Every SKILL.md MUST have a `## Gotchas` section. This is processkit's
version of the Skills Master Class's "highest-signal content" rule.

A good Gotchas section:

- Lists **agent-specific** failure modes — things the model gets wrong, not
  things humans would.
- Is **provider-neutral** — what's obvious to Claude may not be obvious to
  Gemini, ChatGPT, or whatever harness loads the skill.
- Is **specific** — "Don't be sloppy" is useless; "Always check call sites
  before approving a function signature change" is useful.
- Does NOT duplicate the general anti-patterns subsection in Full reference.
  Anti-patterns are practitioner pitfalls that apply to humans and agents
  alike; Gotchas are agent-only.
- Cross-references the Full reference's Anti-patterns when relevant.

### Skill hierarchy (process category only)

Layered skills reference lower-layer skills via `uses:`. The graph is a
DAG; cycles are validation errors. Only skills in the `process` category
have a layer; technical/language/framework skills (e.g.
`python-best-practices`, `fastapi-patterns`) have no layer.

| Layer | Role                      | Examples                                                 |
|-------|---------------------------|----------------------------------------------------------|
| 0     | Foundation                | event-log, index-management, id-management               |
| 1     | Primitive management      | role-management, actor-profile                           |
| 2     | Core entity management    | workitem-management, decision-record, scope-management   |
| 3     | Process orchestration     | process-management, gate-management, schedule-management |
| 4     | Cross-cutting concerns    | discussion-management, metrics-management                |

### Categories

The `category` field is declared at `metadata.processkit.category` and is
the canonical machine-readable grouping for tooling consumers (docs site,
`aibox kit skill list`, skill-finder). Read it with:

```
skill_frontmatter → metadata → processkit → category
```

The registered vocabulary (closed set — propose additions via processkit
issue before using a new value):

| Value | Typical skills |
|---|---|
| `process` | workitem-management, decision-record, event-log, note-management, release-semver |
| `meta` | skill-builder, skill-reviewer, skill-finder, research-with-confidence, status-briefing |
| `architecture` | software-architecture, system-design, api-design, software-modularization, microservice-creation |
| `language` | python-best-practices, typescript-patterns, go-conventions, rust-conventions |
| `framework` | fastapi-patterns, graphql-patterns, grpc-protobuf, flutter-development |
| `infrastructure` | kubernetes-basics, terraform-basics, ci-cd-setup, dockerfile-review |
| `data` | data-science, data-pipeline, pandas-polars, data-quality |
| `ai` | rag-engineering, llm-evaluation, prompt-engineering, embedding-vectordb |
| `api` | api-design, webhook-integration, grpc-protobuf |
| `security` | secure-coding, threat-modeling, secret-management, auth-patterns |
| `observability` | logging-strategy, metrics-monitoring, distributed-tracing, alerting-oncall |
| `database` | database-modeling, database-migration, sql-patterns, nosql-patterns |
| `performance` | performance-profiling, load-testing, caching-strategies |
| `design` | excalidraw, logo-design, mobile-app-design, infographics, frontend-design |

`category` is optional. Consumers fall back to `"uncategorized"` when absent.
Tooling that wants coarser display groups should map from this vocabulary
rather than asking processkit to coarsen it — processkit is the source of truth.

## Full reference

### The `provides` block

`metadata.processkit.provides` is a promise to consumers. It lists:

```yaml
provides:
  primitives:           # primitive kinds this skill creates/manages
    - WorkItem
  mcp_tools:            # tool names exposed by the MCP server (if any)
    - create_workitem
    - transition_workitem
    - query_workitems
  assets:               # asset names shipped in assets/ (templates, fonts, icons, ...)
    - workitem
    - workitem-bug
    - workitem-story
  processes:            # process names this skill implements (if any)
    - backlog-grooming
```

A consumer's `aibox lint` cross-checks `provides.primitives` against the
`kind` values in the directory's `assets/`, and `provides.mcp_tools`
against the server's `list_tools()` response.

### `commands/`

When a skill has workflows that benefit from direct user invocation (without
relying on context-based auto-loading), ship slash-command adapter files in
`commands/`. Each file corresponds to one entry in `metadata.processkit.commands`.

**Frontmatter schema** (`metadata.processkit.commands` list):

```yaml
commands:
  - name: <skill-name>-<workflow>   # namespaced: skill-name prefix is mandatory
    args: "arg-shape"               # provider-neutral argument hint (no angle brackets)
    description: "One sentence: what this command does"
  - name: <skill-name>-<workflow-2>
    args: ""                        # empty string if the command takes no arguments
    description: "..."
```

The `name` field must be prefixed with the skill name (e.g. `model-recommender-profile`,
not just `profile`) to avoid collision across skills.

**Adapter file format** (Claude Code-specific; other providers use their own format):

```markdown
---
argument-hint: arg-shape
allowed-tools: []
---

Use the <skill-name> skill, Workflow X, for $ARGUMENTS.
```

Keep the body to one line — all logic lives in `SKILL.md`, not in the adapter.
`allowed-tools` should be scoped as narrowly as possible (`Bash(scripts/*)` not
`Bash(*)`); use `[]` if no tools beyond reading are needed.

**How commands are discovered:**

- SKILL.md declares the `commands:` list → any agent reading the skill at runtime
  knows the commands exist. This is the provider-neutral baseline.
- The `commands/*.md` adapter files are read on demand when the agent (or user)
  invokes the command — same Level 3 loading as `references/`.
- For harness-level UI registration (tab-complete `/command` in Claude Code), copy
  the adapter files to `.claude/commands/` in the project root. For standalone use
  this is a manual step; the skill works without it.

SKILL.md should mention the available commands in its Overview section so agents
learn about them even before a user triggers them.

### `examples/`

Each example is a Markdown file demonstrating a complete good output for a
realistic task. Examples are **not** tests — they are patterns. SKILL.md
points the agent at the nearest example when Overview gives a hint like
"see the create-feature example for the full flow."

Naming convention: `<action>-<subject>.md` — e.g. `create-feature.md`,
`transition-to-review.md`, `link-related-decisions.md`.

processkit keeps `examples/` even though it's not in Anthropic's
canonical layout. It's additive — Anthropic's loader ignores
unrecognized subdirectories.

### `assets/`

Assets are anything the skill ships for use in its output: templates,
fonts, icons, reference images, brand kits, JSON schemas, prompt
fragments. Most processkit skills use `assets/` for entity YAML
scaffolds (the original `templates/` use case).

A YAML entity template is a complete frontmatter block (no body) used as
a scaffold for new entities. Templates should include sensible defaults
and leave `TODO` markers where the agent must fill in context-specific
values:

```yaml
# assets/workitem.yaml
---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-TODO           # MCP server replaces with generated ID
  created: TODO           # MCP server replaces with current timestamp
spec:
  title: TODO             # agent fills in
  state: backlog
  type: task
  priority: medium
---
```

### `references/`

Deep-dive reference material that doesn't fit in SKILL.md's body without
exceeding the 5000-word limit. Each file documents one focused topic.
SKILL.md points at them by relative path:

```markdown
For the full state-machine transition table, see
`references/transitions.md`.
```

Anthropic's loader treats files under `references/` as Level 3 — they
are loaded only when SKILL.md tells the agent to fetch them. This is the
right place for: complete schemas, edge-case workflows, troubleshooting
playbooks, integration notes, historical decisions.

### `scripts/`

Executable code that ships with the skill — Python, Bash, or any other
interpreter the environment supports. Anthropic's canonical layout
includes `scripts/` at the same level as `references/` and `assets/`.

processkit creates an empty `scripts/` (with a `.gitkeep`) in every
skill, even if the skill doesn't currently ship any. The convention
makes the dir easy to find when the skill author wants to add one
later, and it surfaces the option to skill-builder/skill-reviewer.

Use `scripts/` when:

- The skill needs a deterministic operation that's easier to verify in
  code than to instruct the agent to perform.
- The operation involves parsing, validation, formatting, or any task
  where language-model interpretation is unreliable.
- The Anthropic guide's "advanced technique" applies: *"For critical
  validations, consider bundling a script that performs the checks
  programmatically rather than relying on language instructions."*

For Python scripts, use PEP 723 inline metadata so they are
self-contained (`#!/usr/bin/env -S uv run` shebang + `# /// script` block
declaring dependencies). Same pattern as MCP servers.

### `mcp/`

When a skill ships an MCP server:

- `server.py` uses the official Python MCP SDK via PEP 723 inline deps:
  ```python
  #!/usr/bin/env -S uv run
  # /// script
  # requires-python = ">=3.10"
  # dependencies = ["mcp[cli]>=1.0"]
  # ///
  from mcp.server.fastmcp import FastMCP
  server = FastMCP("processkit-<skill-name>")
  ...
  if __name__ == "__main__":
      server.run(transport="stdio")
  ```
- `mcp-config.json` is the fragment a consumer-side installer merges
  into the agent harness's MCP config:
  ```json
  {
    "mcpServers": {
      "processkit-<skill-name>": {
        "command": "uv",
        "args": ["run", "context/skills/<skill-name>/mcp/server.py"]
      }
    }
  }
  ```
- `SERVER.md` documents tools provided, resources exposed, required
  environment, known limitations. **Not `README.md`** — Anthropic's
  no-README rule applies inside the skill folder, including
  subdirectories.

#### Tool annotations are mandatory

Every tool exposed by a processkit MCP server **must** carry explicit annotations.
The MCP specification defines four boolean hints:

| Annotation | Meaning | Default |
|---|---|---|
| `readOnlyHint` | Tool does not modify state — safe to call without confirmation | `false` |
| `destructiveHint` | Tool may overwrite or delete data — harness should prompt user | `false` |
| `idempotentHint` | Calling twice has the same effect as once | `false` |
| `openWorldHint` | Tool makes external network calls (HTTP, APIs) | `true` |

In FastMCP, set annotations on the tool function:

```python
from mcp.types import Tool
from mcp.server.fastmcp import FastMCP

server = FastMCP("processkit-example")

@server.tool(
    annotations={"readOnlyHint": True, "destructiveHint": False,
                 "idempotentHint": True, "openWorldHint": False}
)
def query_items(filter: str) -> str:
    ...

@server.tool(
    annotations={"readOnlyHint": False, "destructiveHint": True,
                 "idempotentHint": True, "openWorldHint": False}
)
def delete_item(item_id: str) -> str:
    ...
```

**Rules of thumb:**
- Query / list / get / profile tools → `readOnlyHint: true`
- Create / update / set-config tools → `destructiveHint: false` (additive, not
  destructive), unless they overwrite without merge
- Delete / reset / overwrite tools → `destructiveHint: true`
- Any tool making HTTP requests → `openWorldHint: true`
- Tools safe to retry → `idempotentHint: true`

Missing annotations are a **must-fix** finding in `skill-reviewer` (Category 11).

#### MCP server naming: the `processkit-` prefix is mandatory

The MCP server name — the string passed to `FastMCP(...)` and the key
under `mcpServers` in `mcp-config.json` — **must** be prefixed with
`processkit-`. The skill directory name itself stays unprefixed
(`src/skills/workitem-management/`), as does the `name` field in
`SKILL.md`. The prefix applies only to the MCP wire identifier.

The reason is collision avoidance with user-added MCP servers.
Consumer-side installers merge processkit-shipped MCP servers into the
agent harness's single MCP config file alongside any servers the project
owner adds themselves. The merge is **non-destructive**: the installer
owns and re-renders the entries whose names start with `processkit-`
(the managed set), and leaves any other entries strictly alone. Without
the prefix, a user-added server that happened to share a name with a
processkit-shipped one would be silently overwritten on every sync.

Skill name, directory name, and MCP server name are three distinct
identifiers:

| Identifier               | Example                            | Used by                                                  |
|--------------------------|------------------------------------|----------------------------------------------------------|
| Skill directory          | `src/skills/workitem-management/`  | processkit source layout, install paths                  |
| `name` (SKILL.md)        | `workitem-management`              | skill listings, `uses:` references between skills        |
| MCP server name          | `processkit-workitem-management`   | `FastMCP(...)`, `mcp-config.json`, harness config files  |

Documentation referring to the **skill** uses the unprefixed form ("the
`workitem-management` skill provides the `create_workitem` tool").
Documentation referring to the **MCP server registration** uses the
prefixed form ("the harness loads `processkit-workitem-management` from
`.mcp.json`").

### Versioning

`metadata.processkit.version` is per-skill semver, independent of
processkit's overall release version. A skill's version bumps when its
behavior or interface changes. The processkit release tag (e.g.
`v0.5.1`) is a coordinated snapshot of all skill versions at a point in
time.

### Overriding skills in community packages

A community skill package (installed via `aibox process install
<git-url>`) can override a processkit-shipped skill by setting
`metadata.processkit.replaces: <skill-name>`. The consumer project sees
the community skill at `<skill-name>`; the original is shadowed. This
is the extension mechanism for forking or customizing opinionated
defaults without monkey-patching.

### Migration from the legacy K8s-style frontmatter

processkit skills before v0.6.0 used a Kubernetes-style frontmatter
(`apiVersion` / `kind` / `metadata` / `spec` at the top level). That
format was incompatible with Anthropic's Agent Skills loader, which
reads top-level `name` and `description`. The migration to the current
flat format was performed mechanically by
`scripts/migrate-skill-frontmatter.py` — see the script for the exact
transformation. The legacy fields are preserved verbatim under
`metadata.processkit.*`; nothing is lost.
