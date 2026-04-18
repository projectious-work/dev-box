# Changelog

All notable changes to processkit are documented here.
Versions follow [Semantic Versioning](https://semver.org/).

---

## [v0.18.2] — 2026-04-18

### Fixed

- **Stale `mcp-config.json` paths block MCP server startup in derived
  projects** ([#8](https://github.com/projectious-work/processkit/issues/8)).
  12 of the 16 per-skill `mcp-config.json` files under
  `context/skills/processkit/<skill>/mcp/` shipped a `script` path that
  omitted the `processkit/` category segment introduced in v0.17.0's
  nested layout (`context/skills/<skill>/mcp/server.py` instead of
  `context/skills/processkit/<skill>/mcp/server.py`). Harnesses merged
  the broken path verbatim and then failed to launch the affected MCP
  servers on startup, forcing agents back to bash/python workarounds
  for `context/` writes. Latent since v0.17.0; masked until v0.18.6 of
  aibox because earlier aibox walker bugs prevented `.mcp.json` from
  being written at all. Mechanical fix — prepend `processkit/` to the
  script arg for: `actor-profile`, `binding-management`,
  `decision-record`, `discussion-management`, `event-log`,
  `gate-management`, `id-management`, `index-management`,
  `model-recommender`, `role-management`, `scope-management`,
  `workitem-management`. Mirrored into `src/context/`.
- **`AGENTS.md` Session start block points at `pk-resume`** (was stale
  `morning-briefing-generate`, renamed in v0.17.0).

### Added

- **`skip_decision_record(reason, session_id?)` MCP tool on
  `skill-gate`** — acknowledges the v2 contract's decision-language
  window without writing a record, when the agent concludes no decision
  needs recording (e.g. the user withdrew a proposal). Writes a
  `session-<id>.decision-skip` marker next to the existing `.ack` /
  `.decision-observed` markers (24 h TTL, empty reason rejected).
  `check_decision_captured.py` honours skip markers in block mode.
- **`compliance-contract.md` asset bumped to v2.** Adds the clause:
  "When the last five user messages contain explicit decision language
  (approved / decided / ship it / let's go / ok / yes / confirmed),
  either call `record_decision` in the same turn or call
  `skip_decision_record(reason=...)` to acknowledge the skip." Resolves
  the v1 asset vs v2 `AGENTS.md` inconsistency called out in the
  v0.18.1 known-issues block. Existing session markers (which embed
  the old contract hash) automatically invalidate; callers re-ack on
  next hook invocation.
- **`scripts/check-src-context-drift.sh` — release-time drift guard.**
  Runs `diff -rq context/ src/context/` with an allowlist for
  dogfood-only directories (`actors/`, `artifacts/`, `bindings/`,
  `decisions/`, `discussions/`, `logs/`, `migrations/`, `notes/`,
  `roles/`, `team/`, `workitems/`, root `INDEX.md`), runtime paths
  (`.cache/`, `.state/`, `__pycache__/`, `.gitkeep`), and `templates/`.
  Wired into `scripts/build-release-tarball.sh` as a pre-build step;
  non-zero exit fails the release before the tarball is built.
  Structural guard against the v0.15.0–v0.18.0 drift class.
- **Session-start skill-check checklist (SnappyTrout).** `AGENTS.md`
  and `session-handover/SKILL.md` now name six task classes (research
  ingestion, artifact creation, discussion management, decision
  recording, backlog item creation, quality audits) that must be
  preceded by a `search_entities(kind=skill, ...)` / `find_skill(...)`
  / `list_skills(...)` call. Extends, does not duplicate, the
  compliance-contract 1 % rule.
- **`/pk-standup` and `/pk-status` differentiated.** `/pk-standup` is
  now the brief daily standup (done / doing / next / blockers);
  `/pk-status` is the fuller status snapshot. Both invoke
  `standup-context`, different argument framing. Prior identical
  bodies were a v0.17.0 authoring glitch.

### Closed work

- `BACK-20260410_1049-SnappyTrout` — session-start skill-check
  (shipped here).
- `BACK-20260411_0802-LivelyTulip` — skill-finder MCP server
  (`find_skill`, `list_skills` already ship; verified by
  `scripts/smoke-test-servers.py`).

---

## [v0.18.1] — 2026-04-17

### Fixed

- **Release tarball now actually contains v0.15.0–v0.18.0 content**
  ([#7](https://github.com/projectious-work/processkit/issues/7)). The
  build script (`scripts/build-release-tarball.sh`) packages from
  `src/context/`, but content added since v0.14.0 had been landing only
  in the dogfooded `context/` tree. v0.18.0 and earlier tarballs were
  therefore missing features announced in their own CHANGELOG entries.
  v0.18.1 is a pure-sync release: no behavioural changes beyond bringing
  `src/context/` up to parity with what was already running in the
  dogfood.
- **`skill-gate/scripts/emit_compliance_contract.py` now emits
  `hookEventName`** in the `hookSpecificOutput` envelope, as required by
  Claude Code 2.1+. Reads `hook_event_name` from stdin (sent by every
  Claude Code hook invocation) and echoes it back as `hookEventName`.
  Falls back to `"UserPromptSubmit"` if stdin is empty or malformed. The
  Codex / plain-stdout path is unchanged. Resolves the
  `UserPromptSubmit hook error: invalid hookSpecificOutput` that was
  blocking every prompt in Claude Code 2.1+ consumer projects.
- **`check_route_task_called.py`** — marker-lookup decoupled from
  `session_id` (scans `.state/skill-gate/` for any valid marker with
  matching contract_hash + 12h TTL) so Claude Code sessions whose id
  shifts mid-session no longer lock out of MCP tools.
- **`test_hooks.py`** — adds coverage for SessionStart / UserPromptSubmit
  event-name echo, stale marker (> 12h), and hash-mismatch cases.

### Added (now actually shipped — were missing from v0.15.0–v0.18.0 tarballs)

- `team-creator` skill (SKILL.md + commands + references) — entire
  directory tree. Claimed in v0.15.0 CHANGELOG, previously only in the
  dogfood tree.
- All 26 `/pk-*` slash-command files across 13 skills — the rename from
  `<skill>-<verb>.md` to `pk-<verb>.md` happened in v0.17.0 and was
  extended in v0.18.0, but legacy names lingered in `src/context/`.
- Skill-gate Rail 5 scripts and fixtures:
  `check_decision_captured.py`, `decision_markers.py`,
  `decision_sweeper.py`, `record_decision_observer.py`,
  plus transcript and sessionend fixtures.
- `discussion-management/commands/`, `id-management/config/`,
  `index-management/config/` directories.
- Three `owner-profiling` assets (`OWNER-20260409_2054-*`).
- `actor.yaml` / `role.yaml` schema updates for CapabilityProfileRouting
  (`Role.model_profiles`, `Actor.model_profile_override`).
- Updated `AGENTS.md` template: compliance contract v2 marker + decision-
  language clause + Session start block.

### Known non-blocking inconsistency

- `AGENTS.md` template carries `pk-compliance v2` marker and the
  decision-language clause, but the `skill-gate/assets/compliance-
  contract.md` asset (printed by the hook) is still v1. Clauses remain
  a strict superset in v2; a follow-up release will reconcile both to v2
  and ship the `skip_decision_record` MCP tool referenced by the clause.
  Tracked in the dogfood backlog.

## [v0.18.0] — 2026-04-17

### Added

- **CapabilityProfileRouting — three-layer model-selection architecture**
  (DEC-20260417_1800). Replaces the single `preferences.model` string
  pin per Actor with:
  - **Layer A — Model catalog** (*what exists*): curated model registry
    inside `model-recommender` skill, refreshed via `/pk-model-refresh`.
  - **Layer B — Project/owner preferences** (*what this project can use*):
    subscription tier, API keys, cost bias, preferred ordering, owned by
    `model-recommender` skill's `set_config` / `get_config` MCP tools
    (`/pk-model-setup`).
  - **Layer C — Role standard sets** (*what this role needs*): each
    `Role` now carries a ranked `model_profiles` array (provider +
    family + default_version + default_effort + rationale, primary +
    fallbacks). `Actor` gains optional `model_profile_override` for rare
    per-actor deviations.
  - All 8 permanent team roles populated with ranked profiles covering
    Anthropic / OpenAI / Google / xAI / DeepSeek fallbacks. PM routing
    algorithm wired in `context/team/roster.md`.
- **13 additional `/pk-*` slash-command promotions** (total 26 /pk-*
  commands, 0 legacy processkit commands): `/pk-work`, `/pk-dec`,
  `/pk-dec-find`, `/pk-route`, `/pk-model-setup`, `/pk-model-refresh`,
  `/pk-groom`, `/pk-note-promote`, `/pk-note-review`,
  `/pk-owner-bootstrap`, `/pk-observe`, `/pk-skill-new`,
  `/pk-skill-audit`.
- **Actor schema**: optional `model_profile_override` field — identical
  shape to a single `model_profiles` entry, used only when a specific
  actor deviates from its role's primary.
- **Role schema**: required-on-new-roles `model_profiles` array with
  `rank`, `provider`, `family`, `default_version`, `default_effort`
  (enum: minimal / low / medium / high / xhigh / none), `rationale`.

### Fixed

- **Skill-gate PreToolUse hook decoupled from `session_id`** — the hook
  used to key marker-file matching on the harness-provided session id,
  but the processkit MCP server wrote markers under its own pid, so the
  two identifiers never matched and file edits silently blocked once
  the first legacy marker drifted. Hook now scans the marker directory
  for any marker whose `contract_hash` matches the current contract
  hash and whose `acknowledged_at` is within the 12 h TTL.
- **`emit_compliance_contract.py` echoes `hookEventName`** in its
  `hookSpecificOutput` JSON envelope — Claude Code 2.1+ rejects the
  envelope without it (`Hook JSON output validation failed —
  hookSpecificOutput is missing required field 'hookEventName'`). The
  script now reads `hook_event_name` from stdin and echoes it back.
- **`test_hooks.py`** — two new failure-mode tests (stale marker beyond
  TTL, marker with mismatched `contract_hash`). 13 cases total, all
  green.

### Changed

- **Team roster** rewritten with three-layer architecture section, PM
  routing algorithm (5 steps), Primary default + Fallbacks columns,
  effort-tier attribution for session handovers.
- **`preferences.model` string on Actor is deprecated** (no breaking
  migration — roles carry the routing intent now; actors stay identity-
  only unless overriding via `model_profile_override`).

### Notes

- `DEC-20260417_1800-CapabilityProfileRouting` was hand-written via a
  file Write (not the `record_decision` MCP tool) because the installer
  did not merge the per-skill `mcp-config.json` files into a harness-
  readable `.mcp.json` (filed upstream as aibox#53). The file carries a
  `CLEANUP-REQUIRED` marker; re-record through the MCP tool once the
  wiring ships.

---

## [v0.17.0] — 2026-04-17

### Added

- **13 `/pk-*` ergonomic slash commands** — the `/pk-<verb>` namespace
  is processkit's canonical command surface, provider-neutral across
  Claude Code, Codex CLI, Cursor, OpenCode (Aider falls back to
  AGENTS.md instructions). Three delivery surfaces:
  - **Skill-driven (9):** `/pk-resume` (session start),
    `/pk-status` + alias `/pk-standup` (mid-session status),
    `/pk-wrapup` (session end), `/pk-note` (fleeting capture with
    Zettelkasten qualified-link suggestion at capture time),
    `/pk-discuss` (structured multi-turn engagement),
    `/pk-research` (investigate with confidence labels),
    `/pk-release` (prepare a release), `/pk-publish` (push + publish).
  - **AGENTS.md-driven (3):** `/pk-test`, `/pk-build`, `/pk-lint` —
    read a structured `<!-- pk-commands -->` YAML block in AGENTS.md
    and execute the project's declared command. Fills the cross-harness
    gap (4 of 5 harnesses ship zero build/test coverage).
  - **Cross-cutting skill-driven (1):** `/pk-review` — processkit-
    opinionated code review against AGENTS.md conventions + compliance
    contract.
- **`<!-- pk-commands -->` YAML block in AGENTS.md** — machine-readable
  project-specific build/test/lint/fmt/typecheck declarations.
- **OpenWeave: team-creator 4-layer override surface** (FEAT-OpenWeave)
  — landscape artifact override (3-level precedence), DEC-*-TeamWeights
  (tag-based weight + threshold override), `role-archetypes.yaml`
  (project-level role→class pin override with delta/replace semantics
  and eager validation). team-creator bumped v1.1.0 → v1.2.0.
- **QuietLedger Rail 5: auto-capture of decisions** (FEAT-QuietLedger)
  — Lever 1 PreToolUse decision-language gate (shadow-mode-ON default;
  `--mode=block` available but NOT recommended until calibrated) +
  Lever 2 SessionEnd sweeper (writes Note artifact tagged
  `decision-candidates` for owner async review). New MCP tool:
  `skip_decision_record(reason, session_id)` on skill-gate.
  Compliance contract bumped v1 → v2 (new Rail 5 clause).
- **ShadowCount calibration** (RES-ShadowCount) — 9-session corpus,
  precision 6/19 = 0.316, recall 6/6 = 1.00. Verdict: NO-GO on
  `--mode=block` with current marker list. Marker tightening proposed.
- **DEC-CommandNexus** — strategic decision: `/pk-<verb>` namespace,
  processkit ships commands for any lifecycle phase not uniformly
  built-in across all major harnesses. Build/test/lint exclusion
  reversed.
- **CommandCompass research artifact** — cross-harness built-in command
  matrix + external landscape scan + gap analysis + proposal.

### Fixed

- **tool_use transcript filter** — PreToolUse gate + SessionEnd sweeper
  now filter `tool_use`, `tool_result`, `isCompactSummary`,
  `isSidechain`, and `<local-command-*>` entries from transcript reads.
  Prevents false-positive gate fires on agent write payloads that
  contain decision-language text (e.g., "ship it" in WorkItem YAML).

### Changed

- **Compliance contract v1 → v2** — new clause: "When the last five
  user messages contain explicit decision language, either call
  `record_decision` in the same turn or call `skip_decision_record`."
  Existing session acknowledgements re-prompt once on upgrade (contract
  hash changes).
- **team-creator v1.1.0 → v1.2.0** — 4-layer override docs + new
  `--threshold-overrides` CLI flag + agent-driven discovery section
  with trigger phrases for all 4 layers.
- **skill-finder** — 13 new trigger-phrase entries for `/pk-*` commands
  + v0.17.0 ergonomic shortcuts category section.

---

## [v0.16.0] — 2026-04-15

### Added

- **Canonical team-composition schema fields** — closes the aibox field
  report in issue #6, which requested that processkit land a canonical
  team primitive in place of their temporary `x_aibox.*` namespaced
  extension. Three new optional fields on `Role` (`primary_contact`,
  `clone_cap`, `cap_escalation`) and two on `Actor` (`is_template`,
  `templated_from`):
  - `Role.primary_contact` (bool, default `false`) — true for the role
    that is the single point of contact with the human owner. Exactly
    zero or one role per team may set this.
  - `Role.clone_cap` (int, default `5`) — per-role parallelism
    ceiling. The role with `primary_contact: true` must set
    `clone_cap: 1` (never cloned).
  - `Role.cap_escalation` (string, default `"owner"`) — who must
    approve exceeding `clone_cap`. Literal `"owner"` or an actor-ref.
  - `Actor.is_template` (bool, default `false`) — true for the
    canonical seed Actor that defines a role binding; clones derive
    from it. Enables index queries that separate seed team members
    from task-specific clones.
  - `Actor.templated_from` (Actor-ref, default `null`) — back-
    reference from a clone to its template Actor.
- **`team-creator` skill bumped 1.0.0 → 1.1.0** — emits the five new
  canonical fields on every `team-create` / `team-rebalance` run.
  Deactivation logic now uses `is_template: true` as the canonical
  seed identifier instead of the prior same-model-same-role heuristic.
- **Two new applied migrations** — `MIG-20260415T085311` (Role schema
  + data) and `MIG-20260415T095000` (Actor schema + data). Back-fill
  every existing Role/Actor entity so v0.15.0-installed teams carry
  the new fields automatically on upgrade.

### Changed

- **`role-management` skill bumped to v1.0.1** — documents the three
  new Role fields and their validation invariants.
- **`actor-profile` skill bumped to v1.0.1** — documents the two new
  Actor fields and their validation invariants.

### Upstream migration note

Consumers carrying the aibox-style `spec.x_aibox.*` extension can lift
those fields into the canonical names with a one-to-one mapping:
`x_aibox.is_template` → `is_template`, `x_aibox.clone_of` →
`templated_from`, `x_aibox.default_clone_cap` → `clone_cap`,
`x_aibox.escalate_cap_to` → `cap_escalation`, plus a new field
`primary_contact` set true on the PM-equivalent role.

---

## [v0.15.0] — 2026-04-15

### Added

- **`team-creator` skill** — provider-neutral team composition. Tiers
  available models into heavy / medium / light using a weighted
  formula over Capability (0.60), Cost-efficiency (0.20), Latency
  (0.10), and Governance (0.10), then maps eight role archetypes
  (PM, sr-architect, jr-architect, developer, sr-researcher,
  jr-researcher, jr-developer, assistant) onto the tiered models.
  Three commands: `team-create` (full derivation, writes 24 entities
  + roster + DecisionRecord), `team-review` (read-only diff vs latest
  landscape), `team-rebalance` (targeted re-tier of named roles).
  Composes `model-recommender`, `role-management`, `actor-profile`,
  `binding-management`, `decision-record-write` — no new primitives,
  no MCP server. Defaults reproduce the v0.14.0 manually-composed
  team 8/8 on Anthropic Max 5×. Validated by Phase 3 dogfood
  (`ART-20260415_1545-TeamWeaver-team-creator-dogfood-diff`).
- **Session-orientation wiring** — AGENTS.md gains a "Session start"
  section under the compliance-contract block: agents run
  `morning-briefing-generate` before acting (provider-neutral, every
  AGENTS.md-aware harness picks it up). `emit_compliance_contract.py`
  hook script extended with `--include-session-start` flag for
  Claude Code / Cursor reinforcement.
- **6 new artifacts under `context/artifacts/`** — TeamWeaver Phase 1
  design, OpenWeave Phase 1 design, landscape snapshot (HTML +
  structured summary), Phase 3 dogfood diff, Rail 5 research.
- **3 new follow-up WorkItems filed** — `FEAT-OpenWeave` (4-layer
  override design done; implementation queued), `FEAT-QuietLedger`
  (Rail 5 L1+L2 implementation), `RES-GapScout` (research closed).

### Changed

- **`morning-briefing` skill bumped 1.0.0 → 1.1.0** — adds
  `context/migrations/pending/` to "Sources to read" and emits a
  one-line token-budget-share snapshot (Opus / Sonnet / Haiku
  actuals vs ≈5/85/10 target, flagging drift > ±10pp per
  `DEC-20260414_0900-TeamRoster`).

### Closed

- **`ARCH-20260414_1245-FirmFoundation`** — enforcement
  implementation plan closed. All four originally-blocking FEAT
  items (Rails 1–4) plus the two reconstructed follow-ups
  (PathFinder, TeamWeaver) shipped.

---

## [v0.14.0] — 2026-04-14

### Added

- **Compliance contract (Rail 1)** — canonical
  `context/skills/processkit/skill-gate/assets/compliance-contract.md`
  is the single source of truth for the eight enforceable rules. AGENTS.md
  carries a verbatim copy between `<!-- pk-compliance-contract v1 BEGIN -->`
  and `<!-- pk-compliance-contract v1 END -->` markers at the top of the
  file (primacy header), so every harness that reads AGENTS.md surfaces
  the contract before the rest of the project narrative.
- **`acknowledge_contract()` MCP tool (Rail 3)** — added to the
  `skill-gate` MCP server. Records a per-session acknowledgement marker
  under `context/.state/skill-gate/session-<id>.ack`. Paired with the
  PreToolUse hook script (Rail 2) so writes under `context/` block until
  the contract has been acknowledged once per session. The server also
  ships `read_contract()` and `check_acknowledged()`.
- **Provider-neutral hook scripts (Rail 2)** — stdlib-only Python scripts
  shipped under `context/skills/processkit/skill-gate/scripts/`:
  `emit_compliance_contract.py` (SessionStart / UserPromptSubmit) and
  `check_route_task_called.py` (PreToolUse). Harness wiring (Claude Code
  `settings.json`, Codex CLI `hooks.json`, etc.) is aibox's responsibility;
  fixtures captured from a real Claude Code 2.1+ session ship as golden
  files alongside the scripts. Includes `test_hooks.py` for CI.

### Changed

- **1% rule sentence in 8 MCP tool descriptions (Rail 4)** — the
  entity-mutating tools on `artifact-management`, `decision-record`,
  `discussion-management`, `event-log`, `skill-finder`, and
  `workitem-management` MCP servers now carry a one-line reminder in
  their docstring: `route_task()` first if there is even a 1% chance a
  processkit skill applies. Always-on, zero-dependency rail visible in
  the tool schema every turn.

### Notes

- v0.14.0 is the **processkit (Bucket A) half** of the enforcement
  rollout. The matching aibox-side wiring (merged MCP config per
  harness, hook installation, drift report) is tracked as upstream
  issues #43–#51 on `projectious-work/aibox`. Until aibox ships those,
  the rails are present in the source tree but only Rail 1 (prose) and
  Rail 4 (tool descriptions) are visible to derived projects.

---

## [v0.13.0] — 2026-04-11

### Added

- **`task-router` skill and MCP server** — primary routing entry point
  for processkit agents. `route_task(task_description)` returns the
  matching skill, project-specific process override (from
  `context/processes/`), and recommended MCP tool in a single call
  without an LLM call. Two-phase heuristic routing: keyword match
  against 13 domain groups (Phase 1), token-overlap scoring within
  the group's tools (Phase 2), skill-finder trigger-table fallback for
  cross-domain tasks. Return shape includes `skill_description_excerpt`
  (first 150 chars), `tool_qualified` (`{server}__{tool}` collision-safe
  naming), `confidence`, `routing_basis`, and `candidate_tools[]`.
- **`skill-finder` MCP server** — new MCP server for the existing
  skill-finder skill. Tools: `find_skill(task_description)` (trigger-
  phrase table lookup + token-overlap scoring) and `list_skills(category?)`
  (catalog browser). Both tools are read-only with `readOnlyHint=true`.
- **`skill-gate` meta-skill** — provider-neutral prose skill that
  enforces the 1% rule: if there is even a 1% chance a processkit skill
  covers the task, check the router first. Includes decision graph,
  rationalization pre-emption table (5 entries), and escape hatch for
  agents already operating inside a named skill workflow.

### Changed

- **MCP tool prerequisite prompts (Track C)** — all 20 entity-mutating
  MCP tools (`create_*`, `transition_*`, `link_*`, `record_*`, `open_*`)
  now carry a prerequisite sentence in their docstring: call
  `route_task()` or confirm you are inside a named skill workflow before
  using the tool. These prompts appear in the tool schema every turn.
- **`skill-builder` and `skill-reviewer` updated (Track E)** — skill
  `description:` field convention changed from "Use when…" summaries to
  one-sentence imperatives (verb-noun, ≤150 chars). Both skills enforce
  this: skill-builder's step 4 template uses the new format; skill-
  reviewer's Category 6 and Skill Killer #1 check for violations.
- **`AGENTS.md` routing reference** — `find_skill()` replaced by
  `route_task()` as the primary 1% rule entry point. `task-router` added
  to the mandatory MCP server table.
- **`context/processes/release.md`** — fleshed out from generic stub to
  processkit-specific: added `breaking-change-audit`, `update-docs-site`,
  `stamp-provenance`, `push`, `build-and-upload-release`, `deploy-docs`
  steps; docs-deploy WildButter/aibox#42 blocker documented.

---

## [v0.12.0] — 2026-04-11

### Added

- **`artifact-management` skill and MCP server** — new Layer 2
  processkit primitive for registering and retrieving completed
  deliverables (documents, datasets, builds, diagrams, URLs, etc.).
  Supports two usage patterns: self-hosted (Markdown body in the
  entity file) and pointer (external URL or path via `location`).
  MCP tools: `create_artifact`, `get_artifact`, `query_artifacts`,
  `update_artifact`. Artifact has no state machine (`state_machine:
  null`) — it is a catalogue record, not a work-tracking entity.
- **`skill-finder` updated** — new trigger phrases for
  `artifact-management` (`"register an artifact"`, `"catalog this
  document"`, `"store this deliverable"`, `"link this design file"`)
  and a one-liner in the Process category.
- **`context/skills/INDEX.md`** — `processkit/` skill count updated
  to 31; Layer 2 entry updated to include `artifact-management **MCP**`.

---

## [v0.11.1] — 2026-04-11

### Fixed

- **`ids.py`: `pascal` and `camel` are now distinct word styles** —
  `pascal` produces PascalCase (`BoldVale`, every word capitalised);
  `camel` now correctly produces true camelCase (`boldVale`, first word
  lowercase). Previously `camel` incorrectly generated PascalCase.
  `id-management/config/settings.toml` updated to `word_style = "pascal"`.

### Changed

- **`context/` dogfood mirror synced** — `resolve_entity()` and
  partial-ID lookup now live in the installed `context/` copies of
  `index.py` and all MCP servers, consistent with the `src/`
  implementation shipped in v0.10.0.
- **`session-handover` SKILL.md** — log-entry writing steps updated
  with `generate_id` call, date-sharded path derivation, and word-pair
  ID template.
- **`context/skills/INDEX.md`** — expanded with skill package layout
  section and `_lib/` note.
- **`aibox.lock`** — updated to aibox v0.17.12, processkit v0.10.0.

---

## [v0.11.0] — 2026-04-11

### Added

- **Note schema: `links` field** — qualified Zettelkasten note-to-note
  links with `target`, `relation` (enum: elaborates, contradicts,
  supports, is-example-of, see-also, refines, sourced-from), and a
  required `context` sentence explaining *why* the connection matters.
  Tags group notes by topic; links build arguments.
- **note-management SKILL.md: linking section** — new "Linking notes"
  section with a full relation table and usage guidance; note template
  updated with links example.

### Changed

- **Note schema: type descriptions aligned with Luhmann/Ahrens taxonomy**
  — `insight` = permanent note (never discarded, part of the knowledge
  base); `reference` = literature note; `fleeting` = fleeting note.
  Schema description clarifies that permanent notes are not ephemeral.
  `type` descriptions and note-management SKILL.md type table updated
  accordingly.
- **Artifact schema: self-hosted and pointer patterns both documented** —
  description updated to acknowledge two valid usage patterns:
  self-hosted (Markdown body in the file, `location` optional) and
  pointer (external URL/path, `location` required). `location` removed
  from the `required` array. Note vs Artifact distinction added.
- **`config.py` `_skill_config_dir`** — now tries the `processkit/`
  category subdirectory before falling back to the flat layout, fixing
  ID generation and config loading after the v0.10.0 skills
  reorganization.
- **`.mcp.json`** — all 12 MCP server paths corrected from
  `context/skills/<name>/` to `context/skills/processkit/<name>/` after
  the v0.10.0 SteadyLeaf reorganization.
- **`AGENTS.md`** — added AGENTS.md lean-scope principle; expanded
  `context/` layout table to all 15 directories including `artifacts/`,
  `decisions/`, `discussions/`, `notes/`, `workitems/`, `logs/`,
  `migrations/`, `owner/`, `actors/`, `roles/`.

---

## [v0.10.0] — 2026-04-10

### Added

- **`index.resolve_entity()`** — new shared library function with 3-stage ID
  resolution: exact match → prefix match (missing slug) → word-pair match (bare
  word-pair like `StoutCrow`). Enables agents and humans to look up entities by
  the colloquial word-pair shorthand rather than needing the full canonical ID.

### Changed

- **Skills directory reorganized into 7 category subdirectories** (SteadyLeaf):
  `processkit/` (30), `engineering/` (46), `devops/` (15), `data-ai/` (11),
  `product/` (11), `documents/` (8), `design/` (5). All 126 skills moved in
  both `src/context/skills/` and `context/skills/`. All SKILL.md files updated
  with a consistent `category:` frontmatter field.
- **`skill-finder`** updated with a "By directory" table mapping the 7 categories
  to their on-disk subdirectory names.
- **`get_workitem`**, **`get_decision`**, **`get_entity`** (index-management) now
  use `resolve_entity()` and return `{"error": "..."}` instead of `null`/`None`
  when the entity is not found, and `{"error": "ambiguous..."}` with a candidate
  list when multiple entities match.
- **All `_load_*` helpers** (`_load_scope`, `_load_gate`, `_load_actor`,
  `_load_role`, `_load_discussion`, `_load_decision`, `_load_workitem`) updated
  to use `resolve_entity()` so word-pair lookup also works in write tools
  (transition, link, etc.).
- **`smoke-test-servers.py`** updated to find MCP server files in the new
  category subdirectory layout.

### Fixed

- `get_workitem` and `get_decision` previously returned `null` for not-found
  IDs, making it impossible for callers to distinguish "not found" from
  "found but empty". Both now return a structured error dict (SnappyCrane).
- Agents looking up entities by word-pair shorthand (e.g. `StoutCrow` instead
  of the full `BACK-20260410_1050-StoutCrow-create-brand-design-skill`) now
  resolve correctly via the new `resolve_entity()` fallback chain (SteadyPeak).

---

## [v0.9.0] — 2026-04-09

`src/` → `context/` mirror restructure (GrandLily). All processkit content
now lives under `src/context/` in the source tree and is mirrored to
`context/` in the target project root on `aibox sync`.

---

## [v0.8.0] — 2026-04-09

Auto-log side effects in all entity-mutating MCP servers. Every
`create_*`, `transition_*`, and `link_*` call now appends a `LogEntry`
without the caller doing anything extra.

---

## [v0.7.0] — 2026-04-09

Initial processkit v0.7.0 release. Core entity MCP servers, state machines,
schemas, and 85+ skills migrated from the original aibox templates.
