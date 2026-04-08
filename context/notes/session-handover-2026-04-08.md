# Session handover — 2026-04-08

End of a four-release session that completed the **aibox/processkit
split** and shipped four releases (v0.16.0 → v0.16.3) bringing aibox
in line with its new responsibility perimeter. This document is the
canonical briefing for the next session.

> **Read this file first.** It supersedes any earlier `context/notes/`
> handover for everything aibox/processkit-related. Anything in
> `scratch.md` from before 2026-04-08 may be stale w.r.t. the split.

---

## 1. Where we are right now

| Item | State |
|---|---|
| Current branch | `main` |
| Working tree | clean (nothing held, nothing uncommitted) |
| Last commit | `8db063a fix(v0.16.3): 9 quality-of-life patches from real v0.16.2 use` |
| `cli/Cargo.toml` version | `0.16.3` |
| `aibox.toml` version | `0.16.3` |
| Latest tag | `v0.16.3` (pushed) |
| Latest GitHub release | https://github.com/projectious-work/aibox/releases/tag/v0.16.3 (Linux + macOS binaries attached) |
| Latest base image on GHCR | `ghcr.io/projectious-work/aibox:base-debian-v0.16.3` (Phase 2 confirmed) |
| Docs site | https://projectious-work.github.io/aibox/ (deployed from `gh-pages`) |
| processkit pin in this repo's `aibox.toml` | `version = "v0.5.1"` |
| Tests | 454/454 passing |
| Clippy | clean (`--all-targets -- -D warnings`) |
| `cargo audit` | clean |
| Memory dir | empty (no per-session memory; this handover IS the memory) |

**No work in progress.** The user is working on processkit-side
cleanups in a parallel session and will return when they have
clarifications. No CLI work is pending until then.

---

## 2. What aibox is (one paragraph)

**aibox** is a Rust CLI plus a base container image, distributed as
`github.com/projectious-work/aibox`. One-line tagline: *"uv for AI work
environments"* — reproducible AI-ready devcontainers with processkit
content pre-installed. The user runs `aibox init` in an empty
directory, picks a processkit version interactively, picks a few
addons, and gets a working `.devcontainer/` plus a project skeleton
plus all of processkit's content materialized into `context/`. From
there, `aibox start` drops them into a themed Zellij session ready for
an AI agent.

**aibox owns the *infrastructure*:** containers, addons, the
install/diff/migrate machinery, the slim project skeleton at init time
(`.aibox-version`, `.gitignore`, empty `context/`, thin provider
pointer files like `CLAUDE.md`), and the docs site.

**aibox does NOT own:** skills, primitives, processes, package
definitions, the canonical AGENTS.md *content* (it scaffolds the file
into place but processkit owns the template), schemas, state machines,
or anything about *what good process looks like* — that's processkit's
job.

---

## 3. What processkit is (one paragraph)

**processkit** is a provider-neutral, language-neutral kit of process
content that any AI-assisted software project can install. Released as
versioned `tar.gz` packages from `github.com/projectious-work/processkit`
(current: **v0.5.1**). Consumed today by aibox; designed to be
theoretically usable on its own without aibox. processkit IS the
fundamentals; aibox adds devcontainers for AI agents and convenience
for using processkit.

**processkit ships, under `src/`:**

- `skills/` — ~108 SKILL.md files (one per skill) plus per-skill
  `references/`, `mcp/` (Model Context Protocol servers, written in
  Python), and `templates/` (entity YAMLs).
- `primitives/{schemas,state-machines}/` — JSON-schema definitions and
  state-machine definitions for the 18 core primitives (WorkItem,
  DecisionRecord, Scope, Role, Gate, Binding, Constraint, Discussion,
  Process, etc.).
- `processes/` — process definition Markdown (bug-fix, code-review,
  feature-development, release).
- `packages/{minimal,managed,software,research,product}.yaml` — five
  tiers of opinionated skill bundles, composed via `extends:`.
- `scaffolding/AGENTS.md` — the canonical agent entrypoint template
  (with `{{PROJECT_NAME}}`) that lands at the project root.
- `lib/processkit/` — shared Python lib used by every MCP server.
- `PROVENANCE.toml` — file-to-version map used by the consumer-side
  three-way diff.

**processkit does NOT own:** containers, addon management, provider
config files (`CLAUDE.md`, `.gemini/`, etc.), the install/diff/migrate
machinery (that lives in the consumer), the docs site for any specific
consumer project, or any aibox-specific concept.

---

## 4. The boundary in one sentence

**aibox provisions; processkit equips.** aibox gets you a clean,
themed, AI-ready dev container with the content delivery pipeline in
place; processkit fills that pipeline with the actual process content
(skills, primitives, processes, scaffolding). The two compose via
aibox's `[processkit]` config section, which pins a processkit release
and lets the consumer override the source (forks welcome).

---

## 5. Common principles (shared across both projects)

These are non-negotiable. When a design call comes up, the answer is
whichever option respects more of these principles.

1. **Provider neutrality.** No file path, no config field, no binary,
   no API surface is bound to a specific AI provider. The canonical
   agent entrypoint is `AGENTS.md`, not `CLAUDE.md`/`.cursor/rules`/
   `.gemini/`. Provider-specific files (when they exist at all, e.g.
   `CLAUDE.md`) are *thin pointers* to `AGENTS.md`. Skills install
   under `context/skills/`, never under `.claude/skills/`.

2. **Reproducibility.** Every consumed processkit release is pinned
   by `(source, version, sha256)` in the consumer's `aibox.lock`.
   Releases are tarball-packaged and SHA256-verified before
   extraction. Moving-branch consumption is supported as a fallback
   for testing pre-release work but actively discouraged for
   production use.

3. **Locality.** Everything a project needs lives inside the project
   directory: `aibox.toml`, `.aibox-home/`, `context/` (with skills,
   schemas, state-machines, processes, templates mirror), `aibox.lock`,
   `AGENTS.md`. No external state, no central registry. A fresh
   `git clone` followed by `aibox sync` reproduces the environment
   exactly.

4. **Edit-in-place (Strawman D).** Installed processkit content lives
   at editable, top-level paths (`context/skills/<name>/SKILL.md`,
   `context/processes/release.md`, …). Users edit these files directly.
   The immutable upstream reference under
   `context/templates/processkit/<version>/` is the diff baseline —
   aibox's three-way diff at sync time detects local edits, upstream
   changes, and conflicts, surfacing them as migration documents.
   No separate "overrides" location, no rebase ritual.

5. **Forkability.** Every consumer-side reference to processkit goes
   through `[processkit].source`. Companies can fork processkit (or
   build a processkit-compatible kit from scratch) and have their
   projects consume the fork by changing one line. The release-asset
   URL template is configurable for non-GitHub hosts. No part of
   aibox special-cases the canonical processkit upstream.

6. **Single source of truth.** Each piece of content lives in exactly
   one project. Skills, primitives, processes, and the AGENTS.md
   template live in processkit and only in processkit. Container
   generation, addon management, and the install pipeline live in
   aibox and only in aibox. **DEC-027** in v0.16.0 made this strict.

7. **Generic content-source machinery.** The fetcher in
   `cli/src/content_source.rs` is content-source-neutral by
   construction — it doesn't know "processkit" specifically. It
   knows how to fetch a release-asset tarball from any GitHub-shaped
   (or template-configurable) source, verify it, extract it, and feed
   it through the install map. Processkit-compatible alternatives
   (forks, company-internal kits) consume the same machinery with no
   code change.

---

## 6. Release ritual (quick reference)

The full release process is in
**`context/work-instructions/RELEASE-PROCESS.md`**. The doc is current
as of v0.16.3 — read it for the canonical step list. Quick recap:

- **Phase 0** — dependency version check (Claude does this first).
  Check Zellij, Yazi, ripgrep, fd, bat, eza, fzf, delta, ouch,
  starship, zoxide for upstream updates. Bump in
  `images/base-debian/Dockerfile` if needed.
- **Phase 1** — in-container, by Claude:
  1. Bump `cli/Cargo.toml` and `aibox.toml`
  2. `cargo test` + `cargo clippy --all-targets -- -D warnings` +
     `cargo audit` (all must be clean)
  3. Build aarch64 native + x86_64 cross-compile (in parallel)
  4. Package both as tarballs in `dist/`
  5. Write `dist/RELEASE-NOTES.md`
  6. `git add -A` + commit + tag + push (main + tag)
  7. `gh release create` with the tarballs attached
  8. `./scripts/maintain.sh docs-deploy`
- **Phase 2** — on macOS host, by user:
  ```bash
  cd /path/to/aibox
  ./scripts/maintain.sh release-host X.Y.Z
  ```
  Builds the macOS binaries, uploads them to the existing release,
  builds container images, pushes to GHCR.

**Critical Phase 2 gotcha** (already documented in
`RELEASE-PROCESS.md`): a fresh `gh auth login` only grants the default
`repo` scope — pushing to GHCR fails with `denied: permission_denied`.
Fix: `gh auth refresh -s read:packages,write:packages,delete:packages`
on the host. The script is idempotent for the binary upload step, so
a partial Phase 2 run is fine to retry.

**Commit message convention for releases.** The
`RELEASE-PROCESS.md` doc shows the simple pattern (`chore: bump
version to vX.Y.Z`) which fits the "version bump only" case. For the
v0.16.x series we shipped multi-change releases — each commit message
was `fix(vX.Y.Z): <one-line summary>` followed by a section-by-section
body explaining every change, with `Refs: DEC-NNN, BACK-NNN` and the
`Co-Authored-By:` trailer. Follow this pattern when the release
includes real changes beyond the version bump.

---

## 7. What just shipped (v0.16.0 → v0.16.3)

Four releases in a single session, all on `main`. Net diff across the
four: ~−25,000 / +5,000 lines. Each release is a separate tag with its
own GitHub release page and its own RELEASE-NOTES.

### v0.16.0 — *the big breaking change*

Subject: **rip the bundled process layer; processkit becomes the
content source**.

Removed from aibox entirely:
- `aibox skill` CLI subcommand
- `cli/src/process_registry.rs` (705 lines)
- `cli/src/skill_cmd.rs` (~250 lines)
- The bundled `templates/` tree (~85 SKILL.md files plus
  context-doc scaffolds plus processes plus agents pointer)
- `context/AIBOX.md` (the auto-generated "universal baseline")
- `ALL_SKILL_DEFS`, all 141 `include_str!` calls,
  `scaffold_skills*`, `reconcile_skills`, `generate_aibox_md`,
  `check_agent_entry_points` — all from `cli/src/context.rs`
  (~2000 of 2368 lines gone)
- `effective_skill_includes` (config), `skills_for_addons`
  (addon-loader)
- `context/AIBOX.md` and `.claude/skills/` from the sync perimeter

Added:
- `scaffolding/` install branch in
  `content_install.rs::install_action_for` so processkit's
  `scaffolding/AGENTS.md` lands at the project root
- Inlined provider thin-pointer scaffolding (CLAUDE.md → AGENTS.md)

Linked decisions: **DEC-027**, **DEC-026**, **DEC-025** (latter two
also retroactively documented in this release).

### v0.16.1 — *sync auto-installs processkit; init picks the version*

Two complementary fixes:
- `cmd_sync` auto-installs processkit content when
  `[processkit].version != "unset"` AND (no lock OR lock disagrees).
  Pure gating fn `sync_should_install_processkit`. Closes the v0.16.0
  footgun where users edited the version and got an empty `context/`.
- `aibox init` interactive version picker via new
  `content_source::list_versions(source)`. New flags:
  `--processkit-source`, `--processkit-version`, `--processkit-branch`.

Linked decision: **DEC-028**.

### v0.16.2 — *real-run fixes from the first user test of v0.16.1*

Two more complementary fixes:
- `list_versions` falls back to `git ls-remote --tags --refs` on any
  GitHub Releases API failure (rate limit, network, JSON, empty
  result). Same fallback shape as the install.sh fix from earlier
  the same day.
- Sync perimeter catches up with v0.16.1's auto-install: adds
  `aibox.lock`, `AGENTS.md`, `context/skills/`, `context/schemas/`,
  `context/state-machines/`, `context/processes/`,
  `context/templates/` to `SYNC_PERIMETER`; removes `AGENTS.md` from
  `TRIPWIRE_SENTINELS` (sync legitimately writes it now).

Linked decision: **DEC-029**.

### v0.16.3 — *9 quality-of-life patches from real v0.16.2 use*

Container lifecycle UX:
- `cmd_start` error message names BOTH possible fixes (recreate vs
  sync), not just one
- `cmd_sync` warns about a stale running container after the build
  step (`warn_if_container_lags_image`)

Addon UX:
- `[addons.X.tools]` populated with default-enabled tools at default
  versions at init time
- Interactive per-tool version picker for tools with multiple
  `supported_versions`
- New `--addon-tool addon:tool=version` repeatable CLI flag
- Transitive `requires` expansion at both `aibox init` time AND
  `aibox addon add` time (pure helper `expand_addon_requires` shared
  between both call sites)

Privacy + headless OAuth:
- DEC-030 privacy tier `.gitignore` rule (`context/private/` and
  `context/**/private/`)
- DEC-031 `xdg-open` shim in base-debian image for headless OAuth
  flows (`gh auth login`, git credential helpers, etc. now show a
  framed copy-to-host message instead of erroring)

Documentation:
- `RELEASE-PROCESS.md` `gh auth refresh` command was missing
  `write:packages`. Fixed.

Linked decisions: **DEC-031**, **DEC-030**.

---

## 8. Open work — the transition cleanup list

aibox v0.16.0 separated processes out of aibox and made processkit
the sole owner of process content. We are still in the **transition
phase**: a few inconsistencies remain from the pre-split era. None of
them block the v0.16.x line of aibox; they're the next process-side
priorities once the next session picks up.

### 8.1 Drop the single-file Markdown skill track from processkit

processkit currently ships **two parallel tracks** for the same
conceptual artefacts:

- **Entity-sharded track** (`workitem-management`, `decision-record`,
  `scope-management`, `discussion-management`, `gate-management`, …)
  — uses MCP servers, writes per-entity YAML files lazily, follows
  the primitives/state-machines model. **This is the intended future.**
- **Single-file Markdown track** (`backlog-context`, `decisions-adr`,
  `standup-context`, `session-handover`, `context-archiving`) — the
  skill itself instructs the agent to edit `context/BACKLOG.md`,
  `context/DECISIONS.md`, etc. directly.

The single-file track is a leftover from the era when aibox contained
all processes — before the entity-sharded MCP model existed, the only
way to manage a backlog was a flat Markdown file. **Going forward,
processkit should ship only the entity-sharded track**: every primitive
has a primitive schema, a state machine, an MCP server, and per-entity
YAML files written lazily on use.

**Action (processkit-side):** delete the single-file Markdown skills
from `src/skills/` in processkit. Remove any documentation that still
treats `context/BACKLOG.md`/`context/DECISIONS.md`/`context/STANDUPS.md`
as canonical locations.

**Action (aibox-side):** none. aibox doesn't reference either track
specifically — it just installs whatever processkit ships. Once
processkit drops the single-file track, the next aibox install will
pick up the change automatically.

### 8.2 Resolve the privacy tier processkit-side

aibox v0.16.3 ships **DEC-030**, which adds `context/private/` and
`context/**/private/` to the gitignore generated by `update_gitignore`
+ `ensure_aibox_entries`. That covers the aibox-side gitignore part.

**The processkit-side decisions are still open** and tracked in
**`projectious-work/processkit#1`**:

1. **Schema-level frontmatter:** should the processkit primitive YAML
   format carry an optional `metadata.privacy: public | project | user`
   field as a per-file override on top of the directory convention?
   aibox's stated position is *no* unless a real use case forces it
   (the directory convention is observable from the path alone — every
   tool that walks the filesystem already understands it).
2. **Docs-site filtering:** when processkit ships a docs site (or when
   an aibox-managed project's docs site ingests processkit content),
   private subtrees must not be published. Define the filter rule.
3. **Per-user directory layout:** for multi-user projects, do users
   get per-user `context/private/<username>/` subdirectories? Tied to
   processkit's actor primitive. processkit should decide.

**Action:** processkit project owners respond on the issue. Once the
schema decision is made, processkit ships the relevant change; aibox
adapts only if the schema requires new install-time logic (which is
unlikely).

### 8.3 Other items NOT on the cleanup list (intentionally)

The user explicitly **stays with the v0.16.3 xdg-open shim** for the
headless OAuth flow case. A future host-forwarding feature using
**`lemonade`** (cross-platform: Linux/macOS/Windows host) was researched
and rejected for the v0.16.x line — too big for a patch release, plus
the user explicitly said *"no further implementation for v0.17.0"*.
The shim is the universal baseline. If a future v0.17+ wants to add
host forwarding, the architecture sketch (lemonade as host daemon,
shim falls through to print-URL on lemonade unavailable) is captured
in this session's discussion in `scratch.md`.

---

## 9. Open issues + processkit references

| Where | What |
|---|---|
| `projectious-work/processkit#1` | Three-tier privacy model: schema/docs/per-user (paired with aibox DEC-030) |
| processkit upstream | https://github.com/projectious-work/processkit |
| processkit current release | v0.5.1 |
| processkit release tarball URL pattern | `https://github.com/projectious-work/processkit/releases/download/vX.Y.Z/processkit-vX.Y.Z.tar.gz` (with sibling `.sha256`) |

---

## 10. Where to look for what

**Authoritative project docs:**
- `AGENTS.md` (project root) — canonical project instructions for any
  AI agent or human working on aibox itself
- `CLAUDE.md` (project root) — thin pointer to `AGENTS.md`
- `context/work-instructions/RELEASE-PROCESS.md` — full release ritual
- `context/work-instructions/DEVELOPMENT.md` — dev environment notes
- `context/DECISIONS.md` — decision log (DEC-001 … DEC-031, inverse
  chronological). The most load-bearing decisions for the v0.16.x line:
  - **DEC-031** — xdg-open shim
  - **DEC-030** — three-tier privacy model
  - **DEC-029** — list_versions GitHub fallback + sync perimeter
    catch-up
  - **DEC-028** — sync auto-install + version picker
  - **DEC-027** — rip the bundled process layer
  - **DEC-026** — cache-tracked processkit reference
  - **DEC-025** — generic content-source release-asset fetcher
- `context/BACKLOG.md` — active task registry. v0.16.x items: BACK-107
  through BACK-115, all marked done.
- `context/PROJECTS.md` — project registry
- `context/PRD.md` — product requirements

**Code (load-bearing modules for the boundary):**
- `cli/src/content_source.rs` — fetcher (release-asset, host-tarball,
  git fallback strategies); `list_versions` for the init picker
- `cli/src/content_install.rs` — install map (where each processkit
  file lands in the project tree); pure function `install_action_for`
- `cli/src/content_init.rs` — `install_content_source` orchestration;
  templates dir mirror via `copy_templates_from_cache`
- `cli/src/content_diff.rs` — three-way diff; migration document
  generation
- `cli/src/content_migration.rs` — `aibox migrate` subcommands
- `cli/src/sync_perimeter.rs` — `SYNC_PERIMETER` constant +
  `Tripwire`. **DEC-029 expanded the perimeter**; current state
  reflects v0.16.1+ auto-install behaviour.
- `cli/src/container.rs::cmd_init`,`cmd_sync`,`cmd_start` — top-level
  command orchestration; `resolve_processkit_section`,
  `populate_addon_tools`, `expand_addon_requires`,
  `sync_should_install_processkit`, `warn_if_container_lags_image`
- `cli/src/config.rs` — `[aibox]`, `[container]`, `[context]`,
  `[processkit]`, `[skills]`, `[ai]`, `[addons]`, `[customization]`,
  `[audio]`, `[agents]` section definitions
- `cli/src/context.rs` (the slim post-v0.16.0 version, ~530 lines) —
  project skeleton scaffolding, gitignore generation, provider thin
  pointers, file I/O helpers
- `cli/src/addon_cmd.rs` — `aibox addon list/add/remove/info`
- `images/base-debian/` — base container image; `Dockerfile`,
  `config/bin/xdg-open.sh` (DEC-031), `config/bin/open-in-editor.sh`,
  `config/bin/vim-loop.sh`, themes, layouts, helper scripts

**Historical / discussion material:**
- `scratch.md` — long-form session discussions. **Caution:** anything
  predating 2026-04-08 may be stale w.r.t. the aibox/processkit split.
  Useful for context on *why* a decision was made; not authoritative
  for current state.
- `context/notes/agent-personal-mcp.md` — research material on
  per-user MCP servers (related to the privacy tier discussion;
  pre-DEC-030).
- `context/notes/session-handover-2026-03-26b.md` and
  `session-handover-2026-03-27.md` — earlier session handovers.
  **Pre-split**, so the boundary descriptions in those files are
  obsolete. Read for project history only.

**External:**
- aibox repo: https://github.com/projectious-work/aibox
- aibox GHCR: https://ghcr.io/projectious-work/aibox
- aibox docs: https://projectious-work.github.io/aibox/
- aibox releases: https://github.com/projectious-work/aibox/releases
- processkit repo: https://github.com/projectious-work/processkit
- processkit releases: https://github.com/projectious-work/processkit/releases

---

## 11. User preferences observed this session

Things to know about the project owner's working style. Treat these as
defaults; the user will override when they want something different.

- **One big change > many small PRs** for breaking releases. The user
  explicitly said *"make one big change, don't care about derived
  project dependencies, I'll handle that"* for v0.16.0. PR ceremony
  is not required for releases on this repo — commits go directly to
  `main`.
- **Direct release authority.** When the user says "ship it", do the
  full release ritual end-to-end: build, test, commit, tag, push,
  GitHub release, deploy docs. Don't ask permission at each step.
- **Phase 2 is always the user's job.** Phase 1 is in-container
  (Claude); Phase 2 is on the macOS host (user). Don't try to do
  Phase 2 work from the container.
- **Hold uncommitted changes when more work is expected.** When the
  user says "hold", leave changes in the working tree, do not commit,
  do not bump version. They will say "ship" when ready.
- **Real-run testing finds real bugs.** v0.16.1, v0.16.2, and v0.16.3
  were each driven by the user actually running the previous release
  in a derived project and finding rough edges. The release cadence
  was: ship → user tests → user reports → patch → ship. Encourage
  this loop.
- **Provider neutrality is non-negotiable.** Anything that would tie
  aibox or processkit to a specific AI provider is out. The user
  pushed back hard on `.claude/skills/` paths in v0.16.0; the same
  pushback would apply to anything else provider-bound.
- **Cross-host neutrality is non-negotiable.** The container can run
  on any *nix host (Linux, macOS, BSD). Solutions that bind to one
  host OS (e.g. macOS-only `open` forwarding via launchd) are
  rejected. Universal solutions (e.g. lemonade for cross-platform
  host browser forwarding) are preferred when they're worth the
  scope.
- **The user is happy with terse, direct answers.** Lead with the
  decision, follow with the rationale, skip the preamble. Short
  sentences win. Multi-paragraph explanations only when the
  complexity actually needs them.

---

## 12. What the next session should NOT do

- Do not re-introduce skills, primitives, processes, or any
  process-related content into aibox. That all belongs in processkit
  now (DEC-027). If you find yourself adding a SKILL.md to aibox,
  stop and ask whether it should be a processkit PR instead.
- Do not write to `.claude/`, `.gemini/`, or any other provider
  directory from aibox. Provider directories are off-perimeter
  (DEC-029 sync_perimeter).
- Do not try to do Phase 2 of a release from inside the container.
  It needs the macOS host.
- Do not amend or force-push to `main`. New commits only.
- Do not bypass `cargo audit`, `cargo clippy --all-targets -- -D
  warnings`, or `cargo test` before tagging a release. All three
  must be clean. The release process documents this and the
  RELEASE-PROCESS.md text is canonical.
- Do not assume the GitHub Releases API will be available at runtime.
  v0.16.2 fixed the rate-limit footgun in `list_versions`; the
  fallback to `git ls-remote --tags --refs` is unconditional. Same
  applies to `scripts/install.sh` which uses redirect-based version
  discovery instead of the API.
- Do not point users at `aibox skill` — that subcommand was removed
  in v0.16.0 (DEC-027).

---

## 13. Quick-start commands for the next session

```bash
# Where am I?
pwd                                           # /workspace
git status                                     # should be clean
git log --oneline -5                           # latest: 8db063a (v0.16.3)
grep -E '^version' cli/Cargo.toml              # 0.16.3

# Smoke test (no network needed)
cd cli && cargo test && cargo clippy --all-targets -- -D warnings

# What did we ship most recently?
gh release list --repo projectious-work/aibox --limit 5

# What's open against processkit?
gh issue list --repo projectious-work/processkit --state open

# Where are the decisions documented?
ls context/DECISIONS.md context/BACKLOG.md context/PROJECTS.md
head -200 context/DECISIONS.md                 # latest DEC entries first
```

---

## 14. Closing note

If you are an AI agent picking up this project: read this file end to
end before doing anything. It is the most current authoritative
description of where aibox stands w.r.t. the processkit split. After
reading, do `git log --oneline main -10` to see the actual commit
history and confirm nothing has moved since this handover was written
(2026-04-08, end of v0.16.3 release session).

If anything in this file disagrees with what you find in the code or
in `context/DECISIONS.md`, **trust the code and the DEC log** — this
file is a snapshot, the code is the truth.
