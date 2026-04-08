# aibox v0.16.2 — close two real-run footguns from v0.16.1

Patch release. Both fixes came from the first real `aibox init` →
`aibox sync` user run after v0.16.1 shipped.

## Bug fix #1 — `list_versions` falls back when GitHub API rate-limits

`aibox init`'s interactive picker called the GitHub Releases API to
list available processkit versions. The unauthenticated GitHub API is
capped at **60 requests/hour per IP** — a real footgun on shared
NATs, CI runners, and developer machines that already use other
GitHub-using tooling. When the limit was hit the picker silently fell
through to `unset` with a warning, leaving the user with the same
v0.16.0 footgun the v0.16.1 release was supposed to fix.

`list_versions` now tries the API first (it gives nicer release
metadata if/when we want it) and **falls back to `git ls-remote
--tags --refs <source>` on any failure** — including 403, network,
JSON parse, and empty result. The git smart-HTTP protocol has a much
higher rate limit and is the canonical source of truth for tags
anyway. Same fallback behavior the install script (`scripts/install.sh`)
got in v0.16.1.

Identical fix shape to the install.sh fix from earlier today, just
applied to the in-CLI version listing path.

## Bug fix #2 — sync perimeter tripwire fired on AGENTS.md install

The sync auto-install path landed in v0.16.1 (BACK-110). It correctly
fetched processkit content and copied 262 files including `AGENTS.md`
at the project root. But the **sync perimeter** — the documented and
runtime-enforced set of files sync may write — was not updated to
match. The result: a runtime tripwire fired on the very first sync
that materialized processkit content, with the misleading error
"these out-of-perimeter paths were modified during sync, which is a
bug: AGENTS.md (absent → present, 4719 bytes)".

The auto-install path was right; the perimeter was stale. v0.16.2
brings the perimeter into agreement with v0.16.1's install behavior:

**`SYNC_PERIMETER` additions:**
- `aibox.lock` (top-level pin file written by the installer)
- `AGENTS.md` (canonical agent entrypoint installed by processkit)
- `context/skills/` (live install destination)
- `context/schemas/` (primitive schemas)
- `context/state-machines/` (state machines)
- `context/processes/` (process definitions)
- `context/templates/` (immutable upstream cache mirror used by the
  three-way diff)

**`TRIPWIRE_SENTINELS` removal:**
- `AGENTS.md` is no longer a sentinel — sync legitimately writes it
  on the first install. The tripwire still watches `README.md`,
  `CLAUDE.md`, `LICENSE`, `CHANGELOG.md`, `.gitignore`, and the
  user-owned `context/{BACKLOG,DECISIONS,PRD,PROJECTS,STANDUPS,OWNER}.md`.

Module doc-comment in `cli/src/sync_perimeter.rs` rewritten to spell
out the v0.16.1 perimeter expansion. Migration impact: **none**, this
matches the implementation that already shipped — it's the
specification catching up.

## Tests

- 443/443 passing (was 438 in v0.16.1)
- 6 new sync_perimeter tests covering the new in-perimeter
  paths and the new tripwire behavior:
  - `aibox_lock_is_in_perimeter`
  - `agents_md_is_in_perimeter`
  - `processkit_install_destinations_are_in_perimeter` (5 paths)
  - `processkit_templates_mirror_is_in_perimeter`
  - `tripwire_does_not_fire_when_agents_md_is_created`
  - `tripwire_fires_when_readme_is_created` (positive control)
- Updated `all_known_sync_write_targets_are_in_perimeter` to include
  the 9 new install-time write targets (aibox.lock, AGENTS.md, the
  five context/ subtree samples, plus two templates/ samples).

`cargo audit`: clean.
`cargo clippy --all-targets -- -D warnings`: clean.

## Migration impact

**Backwards compatible.** No config schema changes. Existing v0.16.1
projects pick up the fixes on the next `aibox sync`. If you got stuck
on the v0.16.1 tripwire, just upgrade to v0.16.2 and re-run sync.

## Linked decisions

- **DEC-029** — list_versions GitHub fallback + sync perimeter
  catch-up (this release)
- **DEC-028** — sync auto-installs processkit, init picks the version
  (v0.16.1)
- **DEC-027** — aibox v0.16.0: rip the bundled process layer
- **DEC-026** — Cache-tracked processkit reference
- **DEC-025** — Generic content-source release-asset fetcher
