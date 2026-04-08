# Decisions Log

Inverse chronological. Each decision has a rationale and alternatives considered.

## DEC-029 — list_versions GitHub fallback + sync perimeter catch-up (2026-04-08)

**Decision:** v0.16.2 closes two real-run footguns from v0.16.1, both surfaced by the first end-to-end `aibox init` → `aibox sync` user run after the v0.16.1 release shipped.

1. **`content_source::list_versions` falls back to `git ls-remote` on any GitHub Releases API failure.** The unauthenticated GitHub API is capped at 60 requests/hour per IP, and the v0.16.1 implementation called `list_github_releases` with no fallback — when the limit was hit, the picker silently fell through to `unset` with a warning, leaving the user with the same v0.16.0 footgun the v0.16.1 release was supposed to fix. The new implementation tries the API first (gives nicer release metadata when it works) and falls through to `git ls-remote --tags --refs <https url>` on any failure, including HTTP 403 / network / JSON parse / empty result. Same fallback shape as the install.sh fix from earlier today.

2. **The sync perimeter catches up with the v0.16.1 auto-install behavior.** v0.16.1 wired `cmd_sync` to call `install_content_source` when `[processkit].version != "unset"` AND the lock disagreed (BACK-110). The install correctly fetched processkit content and copied 262 files including `AGENTS.md` at the project root, `aibox.lock`, and the `context/{skills,schemas,state-machines,processes,templates}/` subtrees. But `cli/src/sync_perimeter.rs::SYNC_PERIMETER` and `TRIPWIRE_SENTINELS` were not updated to match — the perimeter still reflected v0.16.0's narrower contract ("sync never installs processkit; only init does"). The result was a runtime tripwire firing on the very first sync that materialized processkit content, with the misleading message *"these out-of-perimeter paths were modified during sync, which is a bug: AGENTS.md (absent → present, 4719 bytes)"*. The auto-install path was right; the perimeter was stale. v0.16.2 brings the perimeter into agreement:

   - **`SYNC_PERIMETER` additions:** `aibox.lock`, `AGENTS.md`, `context/skills/`, `context/schemas/`, `context/state-machines/`, `context/processes/`, `context/templates/`.
   - **`TRIPWIRE_SENTINELS` removal:** `AGENTS.md` is no longer a sentinel — sync legitimately writes it on the first install. `README.md`, `CLAUDE.md`, `LICENSE`, `CHANGELOG.md`, `.gitignore`, and the user-owned `context/{BACKLOG,DECISIONS,PRD,PROJECTS,STANDUPS,OWNER}.md` remain in the sentinel set.
   - Module doc-comment in `sync_perimeter.rs` rewritten to spell out the v0.16.1 perimeter expansion and explain *why* the install destinations are now sync-managed (the three-way diff in `content_diff.rs` catches local edits and surfaces them as migration documents — they are never silently clobbered).
   - `all_known_sync_write_targets_are_in_perimeter` extended with 9 new install-time write targets (aibox.lock, AGENTS.md, the five `context/` subtree samples, plus two `context/templates/processkit/v0.5.1/` samples).

**Rationale:**

These are both *specification catching up with implementation* fixes. The implementations that shipped in v0.16.1 (the auto-install, the version picker) were correct in spirit; the bug was that two adjacent layers (the API fallback contract, the sync perimeter contract) had not been updated together. The right fix is to update those layers so the implementation no longer surprises them.

For (1) specifically, the fact that I shipped the same bug in both `scripts/install.sh` and `content_source.rs::list_versions` on the same day suggests the underlying lesson is "any unauthenticated GitHub API call is a footgun; default to the smart-HTTP git path". I'm leaving the API as the *primary* path (it gives explicit release metadata that's nicer than raw tags), but the fallback now means users will never be stuck because of rate limits.

For (2), the fact that the tripwire fired on the *first* successful auto-install is a sign that the sync_perimeter unit tests should have been updated together with the cmd_sync change in v0.16.1 — the static `is_within_perimeter` test for the install destinations was missing. v0.16.2 adds it (`processkit_install_destinations_are_in_perimeter`, `processkit_templates_mirror_is_in_perimeter`, `aibox_lock_is_in_perimeter`, `agents_md_is_in_perimeter`) so any future regression of this kind will fail at `cargo test` time, not at the user's first sync.

**Alternatives:**

- *For (1): switch to GitHub API authenticated calls (e.g. honor a `GH_TOKEN` env var).* Rejected: introduces a credentials path that the rest of aibox doesn't have, complicates the install story, and still rate-limits authenticated calls (5000/hour) — just to a higher ceiling. The git fallback is unconditional and works for everyone.
- *For (1): drop the API path entirely and always use git ls-remote.* Considered. The argument for keeping the API as primary is purely future-facing — if we ever want release metadata (publication date, draft/prerelease flags, asset list) we'll already have the code. If that future never comes, the API path is dead code we can remove in a follow-up. The fallback architecture means the user sees no behavioral difference when the API works vs when it doesn't.
- *For (2): instead of expanding the perimeter, gate the install path with a "tripwire suppression" flag.* Rejected: tripwires that can be locally suppressed defeat their own purpose. The clean answer is to make the perimeter accurately reflect what sync legitimately writes.
- *For (2): keep `AGENTS.md` in the tripwire and exempt it only when an install is in progress.* Rejected: a stateful tripwire is harder to reason about and harder to test. A simpler invariant — "sentinels are files sync NEVER writes; everything else is in-perimeter or doesn't matter" — is the right shape.

**Implementation:**

- `cli/src/content_source.rs::list_versions` now matches on `list_github_releases` result, falling through to `list_git_tags` on `Err` or empty `Ok`. `tracing::debug!` records the fallback so test runs and `RUST_LOG=debug` users can see why the git path was taken.
- `cli/src/sync_perimeter.rs`: 7 new entries in `SYNC_PERIMETER`, `AGENTS.md` removed from `TRIPWIRE_SENTINELS`, module doc-comment rewritten. 6 new tests, the existing tripwire-fires-on-AGENTS.md test inverted to assert it does NOT fire, a new positive control `tripwire_fires_when_readme_is_created`. Full sync_perimeter test count: 33 (was 27).
- 443/443 total tests pass; clippy clean; cargo audit clean.

**Migration impact:** **Backwards compatible.** No config schema changes, no breaking API changes. Existing v0.16.1 projects pick up the fixes on the next `aibox sync`. If a project hit the v0.16.1 tripwire and aborted, upgrading to v0.16.2 and re-running sync fixes it.

**Source:** Session 2026-04-08, reported by user after the v0.16.1 install completed and they ran `aibox init` (which fell through to `unset` because of the API rate limit), then edited aibox.toml to v0.5.1 and ran `aibox sync` (which installed correctly but tripped the perimeter tripwire on AGENTS.md). Both bugs reported in the same session message; both fixed in the same release.

## DEC-028 — aibox sync auto-installs processkit; init offers a version picker (2026-04-08)

**Decision:** v0.16.1 closes a v0.16.0 bug and adds two related ergonomics.

1. **`aibox sync` auto-installs processkit content** when `[processkit].version != "unset"` AND (no `aibox.lock` yet OR the lock disagrees with the current `aibox.toml` on `(source, version)`). The decision is a pure function `container::sync_should_install_processkit(config_version, config_source, lock_pair)` so it can be unit-tested without I/O. Five tests cover: unset sentinel, no-lock + pinned, lock matches, lock version stale, lock source changed.

2. **`aibox init` offers an interactive `processkit.version` picker.** It calls `content_source::list_versions(source)` and presents a `dialoguer::Select` with the latest at the top and an explicit `unset — skip processkit install (configure later)` escape hatch at the bottom. Non-interactive mode picks the latest. If listing fails (network, no semver tags), it falls back to `unset` with a warning — preserving v0.16.0 behavior in the failure case.

3. **Three new CLI flags on `aibox init`**: `--processkit-source`, `--processkit-version`, `--processkit-branch`. The first two are independent; the third tracks a moving branch and wins over the version at fetch time per the existing fetcher contract. The version is still recorded in `aibox.toml` so the project can drop the branch later and have a sensible pin to fall back to.

4. **New `content_source::list_versions(source) -> Result<Vec<String>>`** API. GitHub-hosted sources (host == `github.com`) use the GitHub Releases API; everything else uses `git ls-remote --tags --refs <source>`. Filtering: only tags that parse as semver (optional leading `v`) are kept. Sort descending by semver, dedupe by the stripped form. Six unit tests cover the filter/sort/dedupe helpers without network.

**Rationale:**

The reported bug was a real footgun: a brand-new project with `[processkit].version = "unset"` (the v0.16.0 default) had no obvious recovery path. The user edited `aibox.toml` to pin a version, ran `aibox sync`, and got an empty `context/` with no error. The only call site for `install_content_source` in v0.16.0 was `cmd_init`, and `cmd_init` errors out if `aibox.toml` already exists, so there was no in-place way to fix it.

The picker addresses the upstream cause: there was no good reason to default to `unset` in the first place. The default was a relic of the v0.16.0 plan, where I knew aibox couldn't yet auto-pick a version because there was no version-listing helper. With `list_versions` in place, the resolver can do the right thing — pick the latest by default, let the user pick a specific version interactively, and only fall back to `unset` when listing genuinely fails.

The branch flag is for symmetry with `[processkit].branch` and serves the "test against pre-release work" use case the existing fetcher already supports.

**Alternatives:**

- *Always re-install on every sync, regardless of lock state.* Rejected: would re-fetch the tarball on every sync (the fetcher is cached by `(source, version)` so the network cost is zero, but the templates dir wipe-and-recopy is wasteful and the install report is noisy). The lock-pair gate is a one-line cost in the steady state.
- *Only auto-install when there's no lock at all (skip the source/version mismatch check).* Rejected: would mean `aibox sync` doesn't react to a version bump in `aibox.toml`. The user would have to delete the lock manually. That's the same class of footgun as the original bug, just one step removed.
- *Default to a hard-coded latest at compile time.* Rejected: every aibox release would need to know the current processkit release, creating a coupling and a release-ordering hazard. The runtime list keeps the projects independent.
- *Use the GitHub tags API (`/repos/<org>/<name>/tags`) instead of releases.* Rejected: tags include lightweight tags, draft refs, and feature-branch markers. Releases are explicitly published artifacts and map cleanly to `[processkit].version`. Falls back to `git ls-remote` for non-GitHub hosts where the releases concept doesn't exist.
- *Make `list_versions` async / parallelize the GitHub call.* Rejected: it's one HTTP GET; the existing `ureq` blocking client is fine and adds no async runtime.

**Implementation:**

- `cli/src/content_source.rs::list_versions` (+ private `list_github_releases`, `list_git_tags`, `filter_and_sort_semver_tags`, `parse_loose_semver`)
- `cli/src/container.rs::sync_should_install_processkit` (pure gating fn) and the wired call in `cmd_sync` before the existing three-way diff
- `cli/src/container.rs::resolve_processkit_section` (interactive picker + flag resolution)
- New CLI flags `--processkit-source`, `--processkit-version`, `--processkit-branch` in `cli/src/cli.rs::Init`, threaded through `main.rs` and `container::InitParams`
- 5 + 6 = 11 new unit tests; full suite 438/438 passing

**Migration impact:** **Backwards compatible.** No config schema changes, no breaking API changes. Existing v0.16.0 projects pick up the new behavior on the next `aibox sync` (which will install content if a version is now pinned).

**Source:** Session 2026-04-08, user reported `aibox init` + manual edit + `aibox sync` left `context/` empty; user requested both the sync auto-install AND a version picker (with optional source/branch overrides) in init. Both shipped together.

## DEC-027 — aibox v0.16.0: rip the bundled process layer (2026-04-08)

**Decision:** v0.16.0 removes every process-related artefact from the aibox repo and reduces aibox to two responsibilities: (1) managing AI-ready devcontainers, (2) installing a pinned `processkit` release into the consuming project. Specifically, this release deletes:

- `cli/src/process_registry.rs` (the 705-line `ProcessPackage`/`ContextFileDef`/`ProcessPreset` registry)
- `cli/src/skill_cmd.rs` and the entire `aibox skill` CLI subcommand
- The bundled `templates/` directory (~85 `templates/skills/<name>/SKILL.md` files plus `templates/{minimal,managed,product,research}/` context-doc scaffolds plus `templates/processes/` plus `templates/agents/`)
- The `ALL_SKILL_DEFS` array, all 141 `include_str!` calls, `scaffold_skills*`, `reconcile_skills`, `generate_aibox_md`, `check_agent_entry_points`, `expected_context_files`, `template_content_for_key`, `setup_owner_md` from `cli/src/context.rs` (~2000 of 2368 lines gone)
- `context/AIBOX.md` and the in-repo and per-project copies of it (the auto-generated "universal baseline" file)
- `context::reconcile_skills` and `context::generate_aibox_md` calls in `cmd_sync` and `cmd_update`
- The `effective_skill_includes` config helper and the `skills_for_addons` addon-loader helper
- The `[skills]` section's runtime semantics (the section still parses, but `include`/`exclude` are reserved for a future release; today aibox installs every skill processkit ships)
- `context/AIBOX.md` from the sync perimeter; `.claude/skills/` from the sync perimeter (provider directories are no longer aibox territory)

The new shape:

- `aibox init` writes a slim project skeleton (`.aibox-version`, `.gitignore`, empty `context/`, `CLAUDE.md` thin pointer per `[ai].providers`, `.devcontainer/{Dockerfile.local,docker-compose.override.yml}` placeholders) and then calls `crate::content_init::install_content_source` to fetch the configured processkit release into `context/templates/processkit/<version>/` and copy the live install into `context/skills/`, `context/schemas/`, `context/state-machines/`, `context/processes/`, plus the canonical `AGENTS.md` at the project root (newly handled by a `scaffolding/` branch in `content_install::install_action_for`).
- `aibox sync` no longer reconciles skills, generates `AIBOX.md`, or checks agent entry points. It runs the seed/generate path, then the three-way processkit diff, then the build.
- The `[ai].providers` list is the lead for provider-specific entry files. When `claude` is listed, `aibox init` writes a thin `CLAUDE.md` at the project root pointing at `AGENTS.md`. Nothing else lands under `.claude/`. Other providers use config files (handled in `seed.rs` / addons) and do not get a markdown pointer.
- Initial `--process` interactive selection now offers the five processkit packages (`minimal`, `managed`, `software`, `research`, `product`); the non-interactive default is `managed`.

**Rationale:** Two reasons.

1. *Single source of truth.* The previous arrangement carried duplicate copies of the same skills in two repos (aibox's `templates/skills/` and processkit's `src/skills/`), drifting out of sync at every release. processkit now ships 108 skills (a strict superset of aibox's 85), the package YAMLs (`packages/{minimal,managed,software,research,product}.yaml` with `extends:` composition), the primitive schemas, the state machines, the canonical `scaffolding/AGENTS.md`, and shared MCP lib. There is nothing left in aibox that processkit doesn't ship more comprehensively. Keeping the local copies would mean curating two registries and resolving conflicts on every processkit release.

2. *Conceptual boundary.* aibox is to AI work environments what `uv` is to Python — infrastructure provisioning. The previous mixing of "process management" with "container management" tied aibox to a particular process model and made the CLI surface area schizophrenic (`aibox skill add` next to `aibox addon add` next to `aibox start`). After v0.16.0 the boundary is enforceable: aibox knows about devcontainers and consumes processkit as an opaque content source via the same release-asset machinery that any future content source will use (DEC-025).

**Alternatives:**
- *Keep the bundled skills as a "fallback" for when `[processkit].version = "unset"`.* Rejected: doubles maintenance, blurs the boundary, encourages drift.
- *Filter installs by selected package at install time.* Considered, deferred. The `packages/*.yaml` schema is straightforward to walk (`extends:` chain → union of `includes.skills`), but skipping ~80 skills out of 108 to save disk space is not worth the additional Rust code and YAML dependency in v0.16.0. Agents read SKILL.md files lazily; extra files on disk cost nothing at session time. Will revisit in a follow-up release if there is a real reason (e.g. license filtering, MCP server start cost).
- *Scaffold context-doc templates from aibox and request processkit add them.* Initially planned. Then realised processkit deliberately ships **two tracks** for the same artefacts: an entity-sharded track (`workitem-management`, `decision-record`, `scope-management`, …) that creates per-item YAML files on demand via MCP, and a single-file track (`backlog-context`, `decisions-adr`, `standup-context`, `session-handover`, `context-archiving`) where the skill itself instructs the agent to create/edit `context/BACKLOG.md` etc. directly. Neither track needs upfront scaffolding from aibox — the entity track creates files lazily, and the single-file track has the skill *as* the documentation. The aibox `templates/{minimal,managed,product,research}/` scaffolds were pre-split-era relics from before either track existed. Rejected the issue against processkit; deleted the scaffolds outright.
- *Stage v0.16.0 as several smaller PRs.* Rejected. The cleanup is interlocking: half-removing the registry leaves the build broken for several callers in `container.rs`, `update.rs`, `sync_perimeter.rs`, and `doctor.rs`. The user explicitly authorised one big breaking change.

**Implementation:** Branch `release/v0.16.0`. New `scaffolding/` install branch in `content_install.rs` mapping `scaffolding/AGENTS.md` → project-root `AGENTS.md` (skipping `scaffolding/INDEX.md`). Test fixtures and the static perimeter table updated to match the new contract. `aibox.toml` in this repo now pins `[processkit].version = "v0.5.1"`. Container deps queued from v0.15.0 bumped: zellij `0.44.0 → 0.44.1`, fzf `0.70.0 → 0.71.0`, delta `0.19.1 → 0.19.2`, ouch `0.5.1 → 0.6.1`. Docusaurus docs updated to reflect the new boundary; the `docs-site/docs/skills/` tree (~17 files) is collapsed to a single index page pointing at processkit.

**Migration impact for derived projects:** **Breaking.** Any project on aibox v0.15.x:
- Loses `aibox skill` — use the SKILL.md files under `context/skills/` directly, edit-in-place per Strawman D.
- Loses `context/AIBOX.md` — the file is gone; replace any references with `AGENTS.md` (now installed by processkit).
- Will see `[skills] include = [...]` parse but have no effect at install time. Today's installer ignores it; a future release may re-introduce filtering. Document removal is fine.
- Must set `[processkit].version` to a real release tag (e.g. `"v0.5.1"`) for skills/processes/AGENTS.md to land. The default is `"unset"`, which is safe but produces an empty install.
- Will get an `AGENTS.md` written into the project root on the next `aibox init` if processkit is pinned. If the user already had a hand-written `AGENTS.md`, the install is `write_if_missing` so it is preserved.

The user has confirmed they will handle their derived project (processkit itself) manually.

**Source:** Session 2026-04-07/2026-04-08, "remove all process-related content from aibox now". Picks up where DEC-025 (release-asset fetcher) and DEC-026 (cache-tracked processkit reference) left off.

## DEC-026 — Cache-tracked processkit reference under context/templates/ (2026-04-08)

**Decision:** The fetched processkit release is mirrored verbatim into `<project>/context/templates/processkit/<version>/` and **git-tracked** alongside the live install at `<project>/context/{skills,schemas,state-machines,processes}/` and `<project>/AGENTS.md`. This is the "immutable upstream reference" used by the three-way diff in `content_diff.rs` to detect upstream-vs-local edits at sync time. Strawman D applies: derived projects edit live files in place; the templates dir is the diff baseline.

**Rationale:** The user explicitly wants derived projects to be able to "edit processes/skills/state-machines etc in place but always have the original as reference." Tracking the templates dir in git gives that reference at every commit, makes upstream changes inspectable in PR diffs, and lets `aibox sync` compute the three-way merge without re-fetching from the network. The cache is reproducible from `aibox.lock` (source + version + commit + sha256), so it's not a secret and not a build artefact — it's documentation. Per BACK-106 / DEC-025 the cache is a verbatim copy of `<src_path>/` modulo `.git`/`__pycache__`/dotfiles/`*.pyc`.

**Alternatives:**
- *Cache outside the project under `~/.cache/aibox/processkit/<version>/`.* Rejected: would require every collaborator to re-fetch on first checkout, would make CI runs harder to debug, and would defeat the "in-PR-diff visibility" goal.
- *Cache inside the project but `.gitignore`'d.* Rejected for the same reasons.
- *Cache only the package YAMLs and rely on the live install for everything else.* Rejected: the live install can drift (Strawman D says edit in place); without the immutable templates dir there is nothing to diff against.

**Implementation:** Already in `cli/src/content_init.rs::copy_templates_from_cache` (BACK-106). The path is `context/templates/processkit/<version>/`. The cache is created on `aibox init` and on `aibox sync` when the version moves. `aibox sync` reads `aibox.lock` to find the current version and walks the templates dir for the diff.

**Source:** Session 2026-04-07/2026-04-08, recap of last session's discussion. Recorded here so the load-bearing layout is in the decision log rather than only in the content_init module-doc.

## DEC-025 — Generic content-source release-asset fetcher (2026-04-07)

**Decision:** aibox consumes content sources (today: processkit; tomorrow: any processkit-compatible producer) via a **generic release-asset fetcher** that prefers a purpose-built `<name>-<version>.tar.gz` attached to the producer's release, with bit-exact reproducibility via a sibling `.sha256` checksum file. Host auto-tarball and `git clone` remain as fallback paths. The fetcher is content-source-neutral by construction — processkit gets no special treatment, just a default URL template that happens to point at the canonical processkit repo when no consumer override is set.

**Rationale:** The previous "fetch the git auto-tarball, walk the entire repo, skip `.git`/`__pycache__`/`tests`/dotfiles" approach pushed the "what counts as shippable?" decision into aibox's walker. That logic belongs to the producer. A purpose-built release artifact gives an explicit shippable contract, dramatically smaller downloads (no `tests/`, `docs-site/`, `.git/`), no skip rules in the consumer on the happy path, and a clean place to attach checksums (and, future, signatures). The templates dir at `context/templates/processkit/<version>/` becomes a verbatim mirror of *exactly* what the producer chose to ship — nothing more, nothing less — which is the "transparent reference" property the consumer relies on.

**Alternatives:**
- *Keep git-tarball-only* — works today, but couples consumer correctness to producer repo hygiene; every new top-level file in the producer risks landing in consumer projects unless aibox's walker is updated.
- *Replace git fetch entirely* — loses the ability to test against `version = "main"` or a SHA before a release is cut. Rejected: dev/test ergonomics are worth the modest fetcher complexity.
- *Pin via Cargo-style registry* — too heavyweight for the current scale. The release-asset path is the lightest-weight thing that provides the explicit-contract guarantee.
- *Make the fetcher processkit-specific* — would violate the "no special treatment" principle. Rejected in favor of a generic fetcher with a configurable URL template.

**What is generic vs processkit-specific:**
- **Generic** (lives in `cli/src/content_*.rs`): the fetch strategy ladder (branch → release-asset → host-tarball → git-clone), the URL template expansion (`{source}`, `{version}`, `{org}`, `{name}`), checksum verification, the wrapped-vs-flat tarball auto-detect, the install path mapping, the 3-way diff, the Migration document lifecycle.
- **Processkit-specific** (lives in `[processkit]` config + `ensure_processkit_section_in()` migration): the consumer-side configuration block. Today's only content source. When multi-source support lands, additional `[content_sources.*]` blocks will sit alongside it.

**Implementation:** BACK-106 (landed in v0.15.0). Hybrid fetcher with the four-step ladder, `release_asset_sha256` recorded in `aibox.lock` when the asset path was used, `release_asset_url_template` configurable in `[processkit]` for non-GitHub hosts. SHA256 mismatch is a hard error (no fallback), since it indicates either tampering or a producer bug — both situations the user must be told about.

**Source:** Discussion 2026-04-07 following the full-templates rework (commit `4c8bde3`) and the user pushback "handle processkit like any other derived project, no special treatment".

## DEC-024 — Directory sharding per entity type (2026-04-06)

**Decision:** Projects may shard entity directories by time, state, or other axes on a per-primitive basis. Default is flat (one directory per primitive kind). Sharding is configured in `aibox.toml` under `[context.sharding.<kind>]`.

**Rationale:** Flat directories become unwieldy past ~500 files. Large projects benefit from date-based sharding for logs (`context/logs/2026/04/`), state-based sharding for work items (`context/workitems/done/`, `context/workitems/active/`), or flat for small projects. Making this configurable rather than imposed avoids premature organization.

**Alternatives:** Always flat (breaks at scale), always sharded (overhead for small projects), per-repo fixed scheme (less flexible).

**Source:** DISC-002 Q3 carry-forward from DISC-001.

## DEC-023 — Binding as generalized primitive (replaces RoleBinding) (2026-04-06)

**Decision:** Rename the 18th primitive from RoleBinding to **Binding**. A Binding connects any two entities with optional scope, temporality, and conditions — not just Actor-to-Role. Rule: if a relationship has scope, time, or its own attributes, use a Binding; if it is just "A relates to B," use a cross-reference in frontmatter.

**Rationale:** The indirection pattern (put a third thing between two things so either can change independently) is a fundamental software design principle (GoF patterns, dependency injection, junction tables with attributes). Inventory of processkit relationships shows ≥7 types that benefit from this pattern: role-assignment, work-assignment, process-gate, process-scope, schedule-scope, constraint-scope, category-assignment. One generalized Binding primitive handles all of them without multiplying primitives.

**Alternatives:** Keep RoleBinding specific and add more specific bindings as needed (rejected — grows primitive count without benefit), no bindings, just references everywhere (rejected — cannot express scope/time on relationships without editing endpoints).

**Source:** DISC-002 §11.

## DEC-022 — Configurable ID format (word/uuid × with/without slug) (2026-04-06)

**Decision:** ID format is configurable in `aibox.toml`. Two independent axes: base format (`id_format = "word"` via petname crate, or `"uuid"`) and slug (`id_slug = true` or `false`). All four combinations are valid. The kind prefix (`BACK-`, `LOG-`, `DEC-`, ...) is not configurable. Default: word without slug.

**Rationale:** Solo developers prefer short memorable IDs (`BACK-calm-fox`); larger teams may want uniqueness guarantees (`BACK-550e8400-e29b`); projects with lots of IDs in prose benefit from slugs (`BACK-calm-fox-add-lint`). None of these choices affects interop between projects — the prefix and structure are constant. Making the format configurable resolves the DISC-001 contradiction between Decision 4 (word-based) and Decision 40 (UUID).

**Alternatives:** Fixed word-based (excludes teams needing uniqueness), fixed UUID (unfriendly for solo work), per-entity-type configuration (overkill).

**Source:** DISC-002 §3 Q3 resolution.

## DEC-021 — SQLite index lives in processkit MCP servers (2026-04-06)

**Decision:** Entity indexing (parse markdown+frontmatter, build SQLite tables, serve queries) lives in a processkit MCP server (`skills/index-management/mcp/server.py`), not in the aibox CLI. aibox CLI performs basic structural validation only (`apiVersion`, `kind`, `metadata.id` present); it is schema-agnostic.

**Rationale:** Schemas live in processkit. Putting schema-aware code in aibox CLI creates tight coupling — every primitive or schema change would require an aibox release. Putting the indexer in processkit makes schema evolution self-contained. The MCP server becomes the canonical query interface for agents, which is where the queries are actually issued from.

**Alternatives:** Option A — index in aibox CLI (rejected: tight coupling, release friction). Option C — generic parsing in aibox + schema-aware overlay in processkit (rejected: two-step, unclear ownership).

**Source:** DISC-002 Q2 resolution.

## DEC-020 — MCP servers = official Python SDK + uv PEP 723 inline deps (2026-04-06)

**Decision:** Skill MCP servers are Python source code using the official `mcp` SDK, delivered as standalone scripts with PEP 723 inline dependency metadata. No `pyproject.toml`, no manual venv. `uv` (already present in all aibox containers) handles resolution and caching on first run. STDIO transport only. Container requirements: Python ≥ 3.10 and `uv` — both already present.

**Rationale:** The official SDK is the standard; avoiding it means maintaining custom JSON-RPC code. PEP 723 + uv eliminates per-skill environment setup. First-run cost (~5–10s dependency resolution) is amortized by uv's cache. ~300–400 MB added is acceptable for a dev container that already carries Rust/Node/LaTeX toolchains. Option B (pydantic-only) is the documented escape hatch if container size becomes critical.

**Alternatives:** Option A — raw JSON-RPC with zero dependencies (rejected: reimplements the protocol, fragile for complex servers). Option B — pydantic-only minimal server (kept as escape hatch; not default). Pre-install MCP packages in the base image (rejected: couples aibox image to SDK version, breaks skill independence).

**Source:** DISC-002 §8.

## DEC-019 — Skills are multi-artifact packages (2026-04-06)

**Decision:** A skill is no longer a single `SKILL.md` file. It is a directory containing `SKILL.md` (three-level instructions), `examples/`, `templates/` (YAML frontmatter entity scaffolds), and optionally `mcp/` (Python MCP server source + `mcp-config.json` snippet). Skills declare `uses:` dependencies in frontmatter; the dependency graph is strictly downward (Layer 0 → 4).

**Rationale:** Markdown alone cannot deliver what a capable skill needs: examples of good output, parametric templates for new entities, and programmatic tool capabilities. Bundling these as a package keeps them versioned together and makes skills composable via explicit `uses:` references. The three-level principle (Level 1: 1-3 sentences, Level 2: key workflows, Level 3: full reference) keeps `SKILL.md` scannable for agents.

**Alternatives:** Keep single-file skills (rejected: no way to ship deterministic tool capabilities or templates). Separate examples repo (rejected: breaks versioning and discovery). Skills as code-only (rejected: loses the instructional markdown that agents read first).

**Source:** DISC-002 §3, §6, P3, P4, P15.

## DEC-018 — Two-repo split: aibox + processkit (2026-04-06)

**Decision:** Split content from infrastructure into two repos. **aibox** (`projectious-work/aibox`) holds the Rust CLI, container images, and devcontainer scaffolding. **processkit** (`projectious-work/processkit`) holds primitives, schemas, all 85 skills + new process-primitive skills, process templates, packages, and MCP servers. processkit releases as git tags; aibox consumes a specific tag via `aibox init`. Both repos dogfood aibox for their own dev environments.

**Rationale:** The DISC-001 exploration conflated content (what gets scaffolded into projects) with infrastructure (how the container runs). The conflation made both harder to evolve — every skill change required an aibox CLI release, every CLI change risked destabilizing skills. Splitting by concern lets skills evolve at their own pace, enables community skill packages (via `aibox process install <git-url>`), and gives users a clear mental model: aibox = infra, processkit = content. The bootstrap loop (aibox needs processkit for content, processkit needs aibox for its devcontainer) is resolved by version pinning on both sides.

**Alternatives:** Keep everything in aibox (rejected: couples release cycles, bloats repo). Three+ repos, e.g. separating technical skills from process skills (rejected: the distinction is blurry and splitting creates friction). Skills in aibox but primitives in processkit (rejected: arbitrary boundary).

**Source:** DISC-002 §5, P7, Q1 (name resolution), §15 (all 85 skills in processkit).

## DEC-017 — aibox scope refocus: dev container + skills scaffolding (2026-04-06)

**Decision:** Refocus aibox around one job: **provide consistent, containerized development environments for AI-assisted work.** Analogy: uv is for Python environments, aibox is for AI work environments. Drop from scope: RBAC enforcement, enterprise governance, multi-repo trust architectures, certificate-based authorization, verification manifests, deterministic event logging, workflow execution, Docker wrapping. These are either another project's concern (governance → likely a Kubernetes-based platform) or things aibox should not do (inner-system fallacy — re-exposing Docker behind its own config layer).

**Rationale:** DISC-001 explored enterprise scenarios in depth and produced 74 decisions with 14 internal contradictions — a sign the scope had expanded beyond what aibox should be. Tightening scope to the dev-environment job removes the contradictions, clarifies the product pitch, and lets the remaining work (processkit content, MCP servers, CLI polish) proceed without governance coupling. DISC-001 research is preserved for whatever governance platform eventually needs it.

**Alternatives:** Keep enterprise governance in scope (rejected: scope creep produced contradictions and no clear product). Build both the environment and the governance platform together (rejected: violates single-responsibility, delays both). Drop processkit content too, just ship containers (rejected: users would build the same context/skills scaffolding by hand every time).

**Source:** DISC-002 §1-4, §12, P1, P2, P9.

## DEC-016 — Declarative config + minimal base images (2026-03-23)

**Decision:** Redesign aibox around a single published base image (base-debian), unified add-on system with per-tool version selection, 13 composable process packages replacing 4 monolithic levels, and declarative skill management. No backward compatibility — clean break.

**Rationale:** The 10 pre-compiled image architecture creates maintenance burden (TeX Live duplicated 3x across 3 Dockerfiles), limits composability (can't combine Node+Go without a dedicated image), and gives users no control over which skills they deploy. The 4 monolithic process levels (minimal/managed/research/product) don't fit non-software projects (document, research, data). Moving everything to add-ons + composable process packages gives users full control while reducing our maintenance surface from 10 images to 1.

**Key decisions within:**
- Abstract base contract (Debian now, Alpine later) — not tied to specific distro
- LaTeX becomes an add-on with multi-stage builder (no dedicated base-latex image)
- Add-ons have internal recipe versioning; users select per-tool versions from curated lists
- 13 atomic process packages + 4 convenience presets, freely composable
- Core package (always present): agent-management + owner-profile skills, AIBOX.md + OWNER.md
- Content-addressed skill updates on sync
- AI providers: Claude, Aider, Gemini, Mistral (bring-your-own-model deferred)

**Alternatives:** Keep base-latex image for build speed (rejected — Docker layer caching + future GHCR cache image sufficient), keep 4 monolithic processes (rejected — too rigid for non-software projects), maintain backward compat (rejected — too few users, baggage not worth carrying).

## DEC-015 — Dogfood the product process template (2026-03-23)

**Decision:** Align aibox's own `context/` with the product process template it ships to users. Adopt BACK-NNN IDs in BACKLOG.md, add PROJECTS.md and PRD.md, install 8 product-relevant skills in `.claude/skills/`, close 13 completed GitHub issues, and update the public roadmap.

**Rationale:** aibox promotes structured work processes but wasn't fully following its own product template. Eating our own dogfood validates the template and reveals friction. The existing context/ was close but used a different backlog format (checkboxes vs BACK-NNN table) and lacked structured project tracking. GitHub had 16 open issues, 13 of which were already done — creating a false impression of outstanding work.

**Deviations from template:** STANDUPS.md omitted — session handovers in `project-notes/session-*.md` are more detailed and serve the same purpose. OWNER.md kept (not in product template but useful). Extra work-instructions kept (DOCKERFILE-PRACTICES.md, SCREENCASTS.md) as project-specific extensions. `backlog-context` skill customized for table format with BACK-NNN IDs.

**Alternatives:** Full template adoption including STANDUPS.md (redundant with session handovers), keep current format (misses dogfooding opportunity), automated migration tool (over-engineering for a one-time task).

## DEC-014 — Skills Library: curated quality over marketplace quantity (2026-03-22)

**Decision:** Ship 83 curated skills with reference files rather than providing a marketplace integration or a smaller "starter" set. Skills are embedded in the binary via `include_str!` and scaffolded on `aibox init`. No external download step.

**Rationale:** Marketplace research (SkillsMP: 97K skills, Skills.sh: 40K, ClawHub: 13.7K) revealed that 46.3% of publicly available skills are duplicates or near-duplicates (HuggingFace analysis). The ecosystem's #1 problem is quality, not quantity. A curated library with progressive disclosure (SKILL.md < 150 lines, reference files for depth) differentiates aibox from "skill slop." Embedding in the binary ensures skills work offline and are version-locked to the CLI.

**Categories chosen (14):** Process, Development, Language, Infrastructure, Architecture, Design & Visual, Data & Analytics, AI & ML, API & Integration, Security, Observability, Database, Performance, Framework & SEO. Based on marketplace demand analysis: infrastructure and data skills are vastly underserved relative to frontend/framework skills.

**Alternatives:** Marketplace-first (ClawHub/Skills.sh integration — deferred to backlog), smaller starter set with `aibox skill install` (adds complexity, network dependency), external file download during init (fragile, no offline support).

## DEC-013 — Granular vim mounts preserve image colorschemes (2026-03-22)

**Decision:** The docker-compose template mounts `.vim/vimrc` and `.vim/undo` individually instead of the entire `.vim/` directory. This preserves the image-baked `~/.vim/colors/` and `~/.vim/pack/` directories.

**Rationale:** The base image downloads 6 vim colorscheme files (gruvbox, catppuccin-mocha, catppuccin-latte, tokyonight, nord, dracula) into `/root/.vim/colors/` during Docker build. When the entire `.vim/` was mounted from the host (`.aibox-home/.vim/`), the image's `colors/` directory was shadowed. Result: `E185: Cannot find color scheme 'gruvbox'` in derived projects. Mounting only the two files we actually persist (vimrc and undo/) leaves the image's baked-in directories visible.

**Alternatives:** Copy colorschemes into `.aibox-home/.vim/colors/` during seed (duplicates files, maintenance burden), embed colorschemes via `include_str!` in the binary (bloat, version drift), post-create command to copy (fragile).

## DEC-012 — Reference file scaffolding via SkillDef type (2026-03-22)

**Decision:** Extend `scaffold_skills()` to deploy reference files alongside SKILL.md. Changed the skills data structure from `(&str, &str)` to a `SkillDef` type alias: `(&str, &str, &[(&str, &str)])` = `(name, content, [(ref_filename, ref_content)])`. Reference files go in `.claude/skills/<name>/references/`.

**Rationale:** 8 of the original 26 skills had reference files on disk (11 files total) but they were never deployed to derived projects — `scaffold_skills()` only wrote SKILL.md. With the expansion to 83 skills and 57 reference files, fixing this was a prerequisite. The `SkillDef` type alias satisfies clippy's type_complexity lint while keeping the flat `include_str!` embedding pattern.

**Alternatives:** Struct-based SkillDef (heavier, overkill for static data), dynamic file discovery at runtime (fragile, no compile-time guarantees), separate scaffolding function for references (unnecessary split).

## DEC-011 — Skills + Processes architecture: separate WHAT from HOW (2026-03-22)

**Decision:** Process declarations in context/ define WHAT processes exist ("there shall be backlog management"). Skills (SKILL.md standard) define HOW they're executed. Context stores the resulting artifacts (BACKLOG.md, DECISIONS.md, etc.). Skills come in flavors (e.g., backlog-context vs backlog-github) that users choose.

**Rationale:** Today's process presets bake both "what" and "how" into context template files. This makes them rigid — you can't swap from a context-file backlog to GitHub Issues without restructuring. By separating concerns: process declarations become thin ("there shall be X"), skills become the executable implementation, and artifacts remain in context/. This enables: swappable implementations, testable skills (via SKILL.md eval framework), thinner aibox scaffolding, and a clear boundary between aibox (infrastructure + curated skills) and derived projects (tailoring + execution).

**Relationship to SKILL.md standard:** The open standard at agentskills.io/specification provides the skill format. aibox provides curated, vetted skills. External marketplaces (ClawHub) are user responsibility.

**Implications:** Process presets (minimal/managed/research/product) become skill compositions. aibox.toml gains a [skills] section mapping processes to skill flavors. aibox doctor checks consistency between declared processes and installed skills.

**Alternatives:** Keep current monolithic process templates (simpler but rigid). Full framework integration like SAFe/PMBOK (too heavy for aibox scope — that's kaits territory).

---

Older decisions (DEC-001 through DEC-010): [archive/DECISIONS.md](archive/DECISIONS.md)
