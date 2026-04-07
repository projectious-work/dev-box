# aibox v0.15.0

CLI minor release. The "processkit consumption layer" that landed in v0.14.x
gets finalised: provider-neutral install paths, a documented & enforced sync
perimeter, AGENTS.md as the canonical agent entry file, and a generic
release-asset fetcher with bit-exact reproducibility. Plus archive previews
in yazi and a fix for emoji rendering in LuaLaTeX.

This is the release that lets **processkit dogfood aibox** to manage its own
devcontainer.

## Highlights

### 1. Generic content-source release-asset fetcher (BACK-106 / DEC-025)

`aibox init` (and `aibox sync`'s diff path) now fetch processkit content via
a four-step strategy ladder, in priority order:

1. **Branch override** — `git clone --branch <name>` for testing pre-release work.
2. **Release-asset tarball** — downloads a purpose-built `.tar.gz` attached
   to the release (e.g.
   `https://github.com/projectious-work/processkit/releases/download/v0.5.1/processkit-v0.5.1.tar.gz`).
   When a sibling `<asset>.sha256` file is present, the tarball bytes are
   verified against it BEFORE extraction. The verified SHA256 is recorded
   in `aibox.lock` as `release_asset_sha256` for bit-exact reproducibility.
   A SHA256 mismatch is a hard error — does NOT fall through.
3. **Host auto-tarball** — falls back to GitHub / GitLab's auto-generated
   `archive/refs/tags/<version>.tar.gz` when no release asset is available.
4. **Git clone** of the tag — last resort for hosts that serve neither
   tarball form (typical for self-hosted git over SSH).

The fetcher is **content-source-neutral**: it doesn't know or care that
the only currently-configured content source is processkit. The new
`[processkit] release_asset_url_template` field lets non-GitHub hosts
(Gitea, GitLab, self-hosted) override the URL pattern. Placeholders:
`{source}` (`.git` stripped), `{version}`, `{org}`, `{name}`.

The Rust modules in `cli/src/` were renamed accordingly:

| Before                       | After                  |
|------------------------------|------------------------|
| `processkit_source.rs`       | `content_source.rs`    |
| `processkit_install.rs`      | `content_install.rs`   |
| `processkit_init.rs`         | `content_init.rs`      |
| `processkit_diff.rs`         | `content_diff.rs`      |
| `processkit_migration.rs`    | `content_migration.rs` |

The `[processkit]` config block stays processkit-named — it's the
consumer-side configuration of the (currently sole) content source.

### 2. Documented & enforced `aibox sync` perimeter (closes #34)

Downstream projects (like processkit, when it dogfoods this release) need
a hard guarantee that `aibox sync` will never touch their hand-written
files. The sync perimeter is now both **documented** and **enforced**:

```
aibox.toml                          (one-time schema migrations)
.aibox-version                      (CLI version tracking)
.aibox-home/**                      (runtime config seed; gitignored)
.devcontainer/Dockerfile            (regenerated)
.devcontainer/docker-compose.yml    (regenerated)
.devcontainer/devcontainer.json     (regenerated)
.claude/skills/**                   (skill deployment, write-if-missing)
context/AIBOX.md                    (universal baseline, regenerated)
context/migrations/**               (additive migration documents)
```

Anything else (`README.md`, `AGENTS.md`, `src/`, `tests/`,
`context/BACKLOG.md`, `context/skills/` install destinations, …) is **out
of perimeter and will never be touched**, in any release, under any
configuration.

Two layers of enforcement:

- **Static**: a unit test (`all_known_sync_write_targets_are_in_perimeter`)
  walks every known sync write site and asserts each is in perimeter.
  Adding a new sync write outside the perimeter fails CI immediately.
- **Runtime**: `aibox sync` snapshots a set of representative
  out-of-perimeter sentinel files (`README.md`, `AGENTS.md`, `CLAUDE.md`,
  `context/BACKLOG.md`, `LICENSE`, …) before running, and verifies after
  the sync work that none were touched. A violation aborts with an error
  naming the offending path BEFORE the slow image build runs.

See `aibox sync --help` and `docs-site/docs/reference/cli-commands.md`.

### 3. AGENTS.md as canonical, providers as thin pointers (closes #33)

`aibox init` now scaffolds `AGENTS.md` as the canonical, provider-neutral
agent entry document, with provider-specific files (`CLAUDE.md`, future
`CODEX.md`, …) as **thin pointers** by default — matching the
[agents.md](https://agents.md/) ecosystem convention.

New `[agents]` config block:

```toml
[agents]
canonical     = "AGENTS.md"     # default; almost no one should override
provider_mode = "pointer"       # "pointer" (default) | "full"
```

- **`pointer` mode (default)**: `CLAUDE.md` is ~10 lines saying "see
  `AGENTS.md`". Every harness reads the same canonical document.
- **`full` mode**: `CLAUDE.md` keeps the existing rich Claude-flavoured
  content. Use only when a project genuinely needs different instructions
  per harness.

Existing files are **never overwritten** (write-if-missing). Migration
from a hand-written rich `CLAUDE.md` to the new `AGENTS.md` + thin pointer
split is manual.

### 4. Provider-neutral install layout (carries forward from v0.14.4)

The full-templates rework that landed in v0.14.4 (commit `4c8bde3`)
becomes the headline architectural change in v0.15.0:

- Skills install to `context/skills/<name>/...` (was `.claude/skills/`)
- Schemas → `context/schemas/`
- State machines → `context/state-machines/`
- Processes → `context/processes/`
- Lib → `context/skills/_lib/processkit/`
- Reference templates verbatim at `context/templates/processkit/<version>/`
- Lock at top-level `aibox.lock` (Cargo-style)

`aibox sync`'s 3-way diff reads SHAs on the fly from the templates dir —
no more SHA manifest. Users can browse "what shipped" with plain `ls`
and `diff`.

## Smaller improvements

### Archive previews in yazi

Selecting an archive in yazi previously prompted "asks for 7zip installed".
Yazi 26.x already has a built-in archive previewer that calls `7z` to list
contents — the binary was just missing from the base image. Now in:

- **`p7zip-full`** apt package — feeds yazi's built-in archive previewer
  so zip / 7z / tar / tar.gz / tar.bz2 / tar.xz / iso files show their
  file tree in the preview pane (no plugin required).
- **`ouch`** static binary in `/usr/local/bin/ouch` — uniform shell-side
  archive extraction for all common formats.

### Emoji rendering in LuaLaTeX (closes #32)

The latex addon now installs `fonts-noto-color-emoji`. Wire it up in
your preamble:

```latex
\directlua{luaotfload.add_fallback("emojifallback",{"NotoColorEmoji:mode=harf;"})}
\setmainfont{FreeSans}[Scale=0.95,RawFeature={fallback=emojifallback}]
```

### `ai` layout proportions

The horizontal split changed from yazi 60% / AI 40% to **yazi 53% / AI 47%**
based on real-world feedback from a derived project.

## Upgrade path

For existing aibox-managed projects:

1. Update the CLI (download the binary or run `aibox update` if you
   already have a recent version).
2. Run `aibox sync` — the new perimeter tripwire is active immediately
   and verifies nothing outside the documented set is touched.
3. To pick up the new `AGENTS.md` scaffolding: re-run `aibox init` in a
   throwaway worktree to see the new layout, then copy `AGENTS.md` and
   the thin `CLAUDE.md` pointer over manually if you want the new shape.
   `aibox init` against an existing project will not overwrite anything.
4. To consume the release-asset path: bump `[processkit] version` to a
   release that ships an asset (processkit v0.5.1 is the first one).
   The fetcher tries the release asset first and falls back to the git
   path automatically — no config change needed for github.com sources.

## Issues closed

- #32 — latex addon: missing emoji font
- #33 — aibox init: scaffold AGENTS.md as canonical, providers as thin pointers
- #34 — Document and enforce the aibox sync perimeter

## Test surface

426 unit + 41 e2e + 16 integration tests; clippy clean.
