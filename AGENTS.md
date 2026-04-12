# AGENTS.md

This file is the canonical, provider-neutral entry point for any AI coding
agent (or human collaborator) working on **aibox**. It follows
the [agents.md](https://agents.md) open standard.

If your harness auto-loads a provider-specific file (`CLAUDE.md`,
`CODEX.md`, `.cursor/rules`, …), that file should be a thin pointer to
this one. Edit **this** file, not the pointers.

## About this project

**aibox** is a Rust CLI that manages reproducible, AI-ready dev containers.
Since v0.16.0 it has a strict two-part scope:

1. **Containers** — generates `.devcontainer/Dockerfile`, `docker-compose.yml`,
   and `devcontainer.json` from `aibox.toml`, plus a tool-bundle addon system
   (`addons/`) and themed `.aibox-home/` runtime config seed.
2. **processkit installer** — fetches a pinned release of
   [`projectious-work/processkit`](https://github.com/projectious-work/processkit)
   and installs its skills, primitives, processes, and the canonical `AGENTS.md`
   template into the consuming project under `context/`.

Target users: solo developers, small teams, and consultants who want
reproducible AI-ready dev environments without manual Docker/devcontainer setup.
Success looks like: `aibox init` → working themed Zellij session with processkit
content in place in under 5 minutes.

**Why it exists.** Developers working with AI agents need reproducible,
themed, AI-ready dev containers with structured work-process content
pre-installed — without having to stitch together Docker, processkit,
and AI provider configuration themselves. aibox is the "uv for AI work
environments": one binary, one config file, consistent result everywhere.

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

## Code style and conventions

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

## Pull requests

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

---

## How this project is organized: processkit content

This project uses **[processkit](https://github.com/projectious-work/processkit.git)**, pinned at
`v0.13.0`, package tier(s) `product`, to manage
process content (skills, primitives, processes, schemas). All
processkit-installed material lives under `context/`:

```
context/
├── skills/         ← skill packages (SKILL.md, mcp/, references/, templates/)
├── schemas/        ← JSON schemas for the core primitives
├── state-machines/ ← state-machine definitions
├── processes/      ← process definitions (bug-fix, code-review, release, …)
└── templates/      ← immutable upstream mirror used as a diff baseline
```

`context/templates/processkit/<version>/` is the verbatim upstream
snapshot. **Do not edit it.** Edit the live files at
`context/skills/<name>/SKILL.md`, `context/processes/<name>.md`, etc.,
directly. Local edits are detected at the next sync via three-way diff
against the templates mirror.

Every `context/` subdirectory has an `INDEX.md`. **Read those first** —
do not slurp `context/skills/` or any large directory at session start.
Load specific files only when the task demands it. This is the
three-level principle: start at Level 1 (intro), drop to Level 2
(workflows) when the task narrows, drop to Level 3 (full reference) for
edge cases.

## Working with entities

processkit models project state as **entities** — work items, decision
records, discussions, log entries, scopes, gates, bindings, and so on.
Each entity is a YAML file under `context/<kind>s/`, created lazily on
first use.

For each entity kind, processkit ships:

- **A schema** at `context/schemas/<kind>.yaml`
- **A state machine** at `context/state-machines/<kind>.yaml`
- **An MCP server** at `context/skills/<kind>-management/mcp/server.py`

### Read entities through the index

`context/skills/index-management/` exposes a SQLite-backed index over
every entity in `context/`. Call its tools — `query_entities(kind=…,
state=…)`, `get_entity(id)`, `search_entities(text)` — instead of `ls` /
`grep` / filesystem walks. The index is faster, context-cheaper, and
reflects the canonical state.

### Write entities through the per-kind MCP servers

Use the relevant MCP tool to create or transition entities —
`create_workitem` and `transition_workitem` from `workitem-management`,
`record_decision` from `decision-record`, and so on. Hand-editing entity
files works but bypasses index updates and state-machine validation, so
the index can drift and invalid transitions can slip through. Reserve
hand edits for cases the MCP tools genuinely don't cover.

### Wiring the MCP servers into your harness

Each MCP-bearing skill ships its own `mcp/mcp-config.json` declaring how
to launch the server (typically `uv run …/server.py`). Agent harnesses
(Claude Code, Codex CLI, Cursor, …) discover MCP servers by reading a
single config file at startup, so the per-skill configs need to be
**merged** into the one file your harness reads, and that file needs to
live at the path your harness expects.

If this project was set up by an installer (e.g. an aibox-managed
devcontainer), the installer is responsible for that wiring — the merged
config is generated for you and you should not need to touch it. If
processkit was installed manually, the project owner is responsible for
merging the per-skill blocks and placing the result at the
harness-specific path themselves.

Either way, MCP-bearing skills require **`uv`** and **Python ≥ 3.10** on
PATH inside the environment where the harness runs the servers — each
`server.py` is a self-contained PEP 723 script and `uv run` resolves its
dependencies on first launch.

## AI agents on this project

Configured providers: **claude**, **openai**. Other agents may be working
on this project — coordinate through the entity layer
(`workitem-management`, `event-log`, `discussion-management`) rather than
assuming you are alone.

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

### Contributing improvements upstream

When you make a behavioral or content improvement to a file in
`context/` that was installed from processkit (it has a counterpart
in `context/templates/processkit/<version>/`), ask whether the
improvement is general enough to benefit all processkit consumers.

If yes:
1. Open a Discussion entity locally with `open_discussion` —
   title it "Upstream proposal: <short description>", note the
   changed file and the improvement in the body.
   This creates an audit trail so future sessions can see what
   was proposed and what was decided.
2. File an issue at the processkit repository so maintainers can
   consider it for the upstream catalog.

Nothing is mandatory — the project owner decides what to file
upstream. The Discussion entity records the decision either way.

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

<sub>Scaffolded by processkit `v0.13.0` on `2026-04-12`. Re-rendered on each installer sync.</sub>
