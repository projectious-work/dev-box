# AGENTS.md

<!-- pk-compliance-contract v2 BEGIN -->
<!-- pk-compliance v2 -->

## processkit Compliance Contract

Call `route_task(task_description)` before any `create_*`,
`transition_*`, `link_*`, `record_*`, or `open_*` tool call.

If there is even a 1% chance a processkit skill applies to the current
task, consult `skill-finder` (or call `find_skill`) before acting.

When you decide to create a WorkItem, DecisionRecord, Note, or Artifact,
call the tool in the same turn — deferred entity creation is lost.

Write entities through MCP tools, not by hand-editing files under
`context/` — hand edits bypass schema validation, state-machine
enforcement, and the event-log auto-entry.

Read entities through `index-management` (`query_entities`,
`get_entity`, `search_entities`) — do not use `ls`, `grep`, or raw
filesystem walks under `context/`.

Log an event after any state change that an MCP write did not already
produce automatically.

After a cross-cutting recommendation is accepted, call `record_decision`
in the same turn.

When the last five user messages contain explicit decision language
(approved / decided / ship it / let's go / ok / yes / confirmed),
either call `record_decision` in the same turn or call
`skip_decision_record(reason=...)` to acknowledge the skip.

Do not edit any file under `context/templates/` — it is a read-only
upstream mirror used as a diff baseline.

Do not hand-edit the generated harness MCP config — edit the per-skill
`mcp-config.json` and let the installer re-merge.
<!-- pk-compliance-contract v2 END -->

## About & session start

**aibox** is a Rust CLI that manages reproducible, AI-ready dev containers.
Since v0.16.0 it has a strict two-part scope:

1. **Containers** — generates `.devcontainer/Dockerfile`, `docker-compose.yml`,
   and `devcontainer.json` from `aibox.toml`, plus a tool-bundle addon system
   (`addons/`) and themed `.aibox-home/` runtime config seed.
2. **processkit installer** — fetches a pinned release of
   [`projectious-work/processkit`](https://github.com/projectious-work/processkit)
   and installs its skills, primitives, processes, and the canonical `AGENTS.md`
   template into the consuming project under `context/`.

**MCP Permissions** — Since v0.18.7, `aibox sync` auto-generates harness-specific
permission files for all MCP servers. Configure `[mcp.permissions]` in `aibox.toml`
to eliminate repetitive permission prompts. Glob patterns expand into concrete
server names; deny patterns take precedence over allow for security. See
[Configuration / MCP Permissions](./docs-site/docs/reference/configuration.md#permission-configuration-mcppermissions).

Target users: solo developers, small teams, and consultants who want
reproducible AI-ready dev environments without manual Docker/devcontainer setup.
Success looks like: `aibox init` → working themed Zellij session with processkit
content in place in under 5 minutes.

Run `pk-resume` before acting. Provider-specific files (`CLAUDE.md`,
`CODEX.md`, `.cursor/rules`, …) are thin pointers — edit **this** file.

## Setup

```sh
# build the CLI binary
cd cli && cargo build

# run all tests (unit + integration + E2E tier 1)
cd cli && cargo test

# lint (zero warnings required)
cd cli && cargo clippy --all-targets -- -D warnings

# format check
cd cli && cargo fmt -- --check
```

For E2E tier 2 (full container lifecycle tests, requires the `aibox-e2e-testrunner`
companion service to be running alongside the devcontainer):

```sh
cd cli && cargo test --features e2e
```

See `context/work-instructions/DEVELOPMENT.md` (or `CONTRIBUTING.md`) for
the full development workflow, E2E test architecture, and cross-compile steps.

<!-- pk-commands BEGIN -->
<!--
build: "cd cli && cargo build"
test: "cd cli && cargo test"
lint: "cd cli && cargo clippy --all-targets -- -D warnings"
fmt: "cd cli && cargo fmt"
typecheck: ""
-->
<!-- pk-commands END -->

## Code style & PRs

### Code style and conventions

- **Zero clippy warnings** — always run with `-D warnings`; CI rejects any warning.
- **All tests must pass** before committing; run `cargo test` and
  `cargo clippy --all-targets -- -D warnings` before every commit.
- **`cargo audit` must be clean** before tagging a release.
- **Conventional commits** — `feat:`, `fix:`, `chore:`, `docs:`. For releases that
  include real changes beyond the version bump, use `fix(vX.Y.Z): <summary>` with a
  section-by-section body and `Refs: DEC-NNN, BACK-NNN` + `Co-Authored-By:` trailer.
- **Reference GitHub issue numbers** in commits: `fixes #N`, `refs #N`.
- **Include `Cargo.lock`** in version bump commits.
- **Never force-push to `main`.**
- **No hardcoded processkit vocabulary** in production Rust code — add constants to
  `cli/src/processkit_vocab.rs` instead.
- **No trailing summaries** in agent responses — the user can read the diff.

### Pull requests

- **Direct commits to `main`** — no PR ceremony on this repo.
- **"Ship it" means the full release ritual** end-to-end: build, test, commit, tag,
  push, GitHub release, deploy docs. Do not ask permission at each step.
- **Phase 2 is always the user's job** — macOS host builds and GHCR image pushes
  run via `./scripts/maintain.sh release-host X.Y.Z` on the host, never from the container.
- **One big change preferred over many small PRs** for breaking releases. The user
  explicitly said: "make one big change, I'll handle derived project dependencies."
- **Hold uncommitted changes when the user says "hold"** — leave in working tree,
  do not commit, do not bump version until the user says "ship".

## processkit preferences

Runtime configuration lives in per-skill config files under
`context/skills/<name>/config/settings.toml`. The agent edits these
directly; MCP servers read them on every call — no restart needed.

Config overrides in `context/skills/id-management/config/settings.toml`:

- **ID format:** `word` + `camel` + datetime prefix + slug — e.g. `BACK-20260411_0109-CalmFox-adapt-aibox-self-hosted`.
- **Directory names:** all processkit defaults (`workitems/`, `decisions/`, etc.).
- **Log sharding:** processkit default (no date-based subdirectories).

## AI agents on this project

Configured providers: **claude**, **openai**. Other agents may be working
on this project — coordinate through the entity layer
(`workitem-management`, `event-log`, `discussion-management`) rather than
assuming you are alone.

### Team

This project operates with a permanent 8-role AI-agent team defined as
processkit entities. The owner is the sole human approver; the
**project-manager** role is the single agent that speaks to the owner and
routes work to the rest of the team.

| Role | Model tier | Purpose |
|---|---|---|
| project-manager | Opus | Owner-facing lead: intake, strategy, routing, review, devil's advocate |
| senior-architect | Opus | Large features, complex bugs, cross-cutting design |
| junior-architect | Sonnet | Small/medium design, architectural questions (default architect) |
| developer | Sonnet | Implementation from plans (default execution role) |
| senior-researcher | Opus | Deep research with synthesis and judgement |
| junior-researcher | Sonnet | Bounded research and lookups (default researcher) |
| junior-developer | Haiku | Mechanical edits, bulk patterns, simple fixes |
| assistant | Haiku | Secretary: briefings, summaries, indexing, handovers |

Target orientation mix (task count, not hard budget): **~5% Opus / ~85% Sonnet
/ ~10% Haiku.** Opus costs roughly 5× Sonnet per equivalent output, so the
same mix is closer to ~20%/75%/5% by budget. PM watches actual usage and
escalates to the owner if the Opus share creeps up.

Team members are cloneable on demand (default cap 5 per role; owner approves
beyond). Clones get fresh IDs and bindings; template actor IDs are never
reused. See:

- `context/roles/` — Role responsibilities and `spec.x_aibox.model_tier`
- `context/actors/` — Template Actors (`type: agent`) with `spec.x_aibox.model`
- `context/bindings/` — Template role assignments
- `context/processes/team-task-distribution.md` — How PM routes work
- `context/decisions/DEC-20260414_1100-NobleStag-team-composition-and-model-mix.md`
  — Decision record, rationale, and alternatives considered

**Schema note (applied — MIG-20260415T093853):** the canonical processkit
team schema fields now live at the top level of `spec.*`:
- `spec.is_template` — marks a role-template actor (never reused; clones get fresh IDs)
- `spec.templated_from` — ID of the template actor a clone was created from (null for templates)
- `spec.clone_cap` — maximum simultaneous clones for this role
- `spec.cap_escalation` — escalation path when clone_cap is reached
- `spec.primary_contact` — marks the role that speaks directly to the owner (one per team)

Only `model_tier`, `model`, and `role_ref` remain under `spec.x_aibox` as
aibox-local extensions with no canonical equivalent yet. Do not move these
manually — they will be lifted when the upstream schema adds them.

**Commit to actions immediately.** If you decide to create an entity
(WorkItem, DecisionRecord, etc.), call the tool in the same turn. Do
not say "I'll track that" and move on — deferred commitments are
routinely dropped and leave the entity layer out of sync with what was
discussed.

**Check the skill catalog before acting on domain tasks.** When a
domain-specific task arrives — writing a PRD, creating a release,
reviewing a skill, designing a schema — search the processkit skill
catalog first. Use `search_entities` via index-management or check
`skill-finder` before falling back to general knowledge. A matching
skill may exist with processkit-specific conventions (entity storage
paths, workitem linking, output formats) that general knowledge does
not know. Missing a skill wastes work and produces non-standard
output.

## Project-specific notes

### Critical: `.devcontainer/` vs `images/`

**We are in a dev-container building dev-containers.** Never confuse:

- **`.devcontainer/`** — THIS project's own dev environment (Rust + Python/uv + Docusaurus).
- **`images/`** — Published images for OTHER projects (pushed to GHCR). They do NOT include Rust toolchain or MkDocs.

Changes to `.devcontainer/` affect our development. Changes to `images/` affect downstream projects.

### Project structure

| Path | Owns |
|---|---|
| `cli/` | The Rust CLI (`aibox` binary) — the only shipped artifact besides addon YAMLs |
| `addons/` | YAML addon definitions (python, rust, node, latex, …) |
| `images/` | Container image build recipes published to GHCR |
| `docs-site/` | Docusaurus documentation site |
| `context/` | This project's context (backlog, decisions, research, …) |
| `scripts/` | Release and maintenance tooling (`maintain.sh`, `record-asciinema.sh`, …) |

**Key Rust module:** `cli/src/processkit_vocab.rs` — central constants module.
All processkit-related compile-time vocabulary (path prefixes, filenames, category order,
frontmatter types) lives here. **Never hardcode processkit strings in production code.**

### aibox ⇄ processkit boundary

- **aibox owns:** containers, addons, the `[processkit]` config section, the
  install/diff/migrate machinery, the slim project skeleton at init time
  (`.aibox-version`, `.gitignore`, empty `context/`, thin provider pointer files
  like `CLAUDE.md`), and the docs site.
- **processkit owns:** every skill (`SKILL.md`), every primitive schema, every state
  machine, the canonical `AGENTS.md` template, processes, and the package YAMLs.
- **The `context/` directory** is shared territory: aibox creates it, processkit fills
  it. An immutable upstream reference lives under `context/templates/processkit/<version>/`
  for the three-way diff.

**If something process-related is missing, add it to processkit, not aibox.**
Do not re-introduce skills, processes, or primitives into aibox — that all belongs in
processkit now (DEC-027).

### Anti-patterns — stop and reconsider if you find yourself doing these

- Writing to `.claude/`, `.gemini/`, or any other provider directory from aibox
  code (off-perimeter per DEC-029)
- Hardcoding path strings, filenames, or processkit vocabulary in Rust source
  (add to `processkit_vocab.rs` instead)
- Trying to do Phase 2 of a release from inside the container (needs macOS host)
- Skipping `cargo audit`, `cargo clippy --all-targets -- -D warnings`, or `cargo test`
  before tagging a release
- Pointing users at `aibox skill` — that subcommand was removed in v0.16.0 (DEC-027)
- **Creating GitHub releases directly** with `gh release create` — always use
  `./scripts/maintain.sh release <version>` inside the container instead. It runs
  tests, cargo audit, builds Linux binaries, creates the release with assets attached,
  deploys docs, and prints the Phase 2 prompt. A bare `gh release create` produces
  a release with no binary assets.
- **Releasing when `cargo build --release` is broken** — if the build fails (e.g.
  missing linker after a container config change), stop and tell the user rather
  than creating an empty release. A release without binaries is worse than no release.
  The precondition is: `cargo build --release --target aarch64-unknown-linux-gnu`
  must succeed inside the container before `maintain.sh release` is invoked.

### Design principles (non-negotiable)

These are load-bearing. When a design call comes up, the answer is whichever option
respects more of these principles.

1. **Provider neutrality.** No file path, config field, binary, or API surface is bound
   to a specific AI provider. Skills live under `context/skills/`, never `.claude/skills/`.
   Provider-specific files (CLAUDE.md, etc.) are thin pointers to AGENTS.md.
2. **Reproducibility.** Every consumed processkit release is pinned by `(source, version,
   sha256)` in `aibox.lock`. Moving-branch consumption is a dev fallback, not production.
3. **Locality.** Everything a project needs lives inside the project directory. A fresh
   `git clone` + `aibox sync` reproduces the environment exactly.
4. **Edit-in-place.** Installed processkit content lives at editable, top-level paths.
   The immutable upstream reference under `context/templates/processkit/<version>/` is the
   diff baseline only — not a restriction on editing.
5. **Forkability.** Every reference to processkit goes through `[processkit].source`.
   Companies can fork processkit and consume the fork by changing one line.
6. **Single source of truth.** Each piece of content lives in exactly one project.
   Skills/primitives/processes/AGENTS.md template → processkit only. Container generation/
   addon management/install pipeline → aibox only. DEC-20260411_0000-JollyClover-rip-bundled-process-layer made this strict.
7. **Generic content-source machinery.** The fetcher in `content_source.rs` is content-
   source-neutral by construction. It doesn't know "processkit" specifically — it knows
   how to fetch a release-asset tarball from any GitHub-shaped source, verify it, and
   extract it. Processkit-compatible alternatives consume the same machinery with no code change.

### Provider independence

All project state must be stored in `./context/` — never in provider-specific locations
(e.g. `.claude/memory/`, `.aider/`). This ensures any AI agent (Claude, Aider, Gemini,
etc.) can pick up where another left off, and session handovers are committed to git.
Do not write to `.claude/`, `.gemini/`, or any other provider directory from aibox code.

### MCP Permissions Troubleshooting

If you're still seeing permission prompts for aibox-shipped MCP servers:

1. **Verify `[mcp.permissions]` is configured** in `aibox.toml`:
   ```toml
   [mcp.permissions]
   default_mode = "allow"
   allow_patterns = ["mcp__processkit-*"]
   ```
   
2. **Run `aibox sync`** to regenerate harness permission files. Permission configuration is applied during sync, not during runtime.

3. **Check harness-specific behavior:**
   - **Claude Code**: Checks `.claude/settings.local.json` → run `aibox sync` to populate `permissions.allow[]`
   - **OpenCode**: Reads `.opencode/config.toml [mcp]` section → verify `mode = "allow"` and `allow[]` array
   - **Continue IDE**: Per-tool `mode` in `continue/config.json` → defaults to "Ask" for safety; override with `[mcp.permissions.harness.continue] mode = "allow"`
   - **Cursor IDE**: Checks `.cursor/settings.json allowedMcpServers[]` → verify entries match expanded server names
   - **Gemini CLI**: Dual `includeTools`/`excludeTools` in `.gemini/settings.json` → intersection semantics (both must match)
   - **GitHub Copilot**: Reads environment variables `COPILOT_MCP_ALLOW_TOOLS`, `COPILOT_MCP_DENY_TOOLS` → created as `.copilot-env`
   - **Aider**: Checks `.aider/mcp-permissions.json allowed_tools` → fallback for harnesses without native MCP permission support
   - **Codex**: Uses project-level `trust_level = "trusted"` in `.codex/config.toml` → applies to all tools

4. **Verify pattern matching:**
   - `"mcp__processkit-*"` matches all processkit MCP servers (e.g., `mcp__processkit-workitem-management__create_workitem`)
   - `"bash"` matches the Bash tool fallback
   - First-match-wins: if a tool matches both allow and deny patterns, deny takes precedence
   - Check `/workspace/.aibox/aibox.log` for pattern expansion details

5. **Check for typos in `allow_patterns`** — misspelled patterns expand to zero tools. `aibox sync` logs warnings for patterns that match no servers.

### Operational gotchas

- **Podman compose** output format varies by version — always use `inspect`, never parse `ps` output.
- **Stale image cache**: if the container exits immediately after start, rebuild with `--no-cache`.
- **`.aibox-home/` must be in `.gitignore`** — it contains SSH keys and personal config.
- **Zellij version pin**: change `ARG ZELLIJ_VERSION` in `images/base-debian/Dockerfile` to upgrade.
- **`host.docker.internal`**: works on Docker Desktop and Podman pasta; bare Linux Docker may need `--add-host`.
- **OrbStack virtiofs**: files mounted from macOS may lose execute permissions — workaround: `chmod +x` inside container.
- **Claude Code OAuth in containers**: use `claude setup-token` or authenticate on host (credentials shared via `.claude` mount). See anthropics/claude-code#14528. Do NOT use `network_mode: host`.
- **OrbStack network dropout**: after ~20 minutes idle, OrbStack's VM NAT can drop connections. Fix: set `keepalive = true` in `[container]` of `aibox.toml` (adds a lightweight DNS keepalive every 2 minutes via `postStartCommand`).

### Runtime artifacts for agents (in derived projects)

When an AI agent is working inside a project that uses aibox:

| Path | Contents |
|---|---|
| `.aibox/aibox.log` | NDJSON structured log of every `aibox` command. Read to understand what aibox did recently. Rotates at 1 MB. |
| `aibox.lock` | Pinned versions of the aibox CLI and processkit last synced. |
| `context/migrations/` | Migration briefings generated when the CLI version changed. |

### GitHub organization

- **Repo:** `projectious-work/aibox`
- **GHCR:** `ghcr.io/projectious-work/aibox`
- **Docs:** `https://projectious-work.github.io/aibox/`
- **processkit upstream:** `https://github.com/projectious-work/processkit`
- **processkit releases:** `https://github.com/projectious-work/processkit/releases`

---

<sub>Scaffolded by processkit `v0.19.1` on `2026-04-23`. Re-rendered on each installer sync.</sub>
