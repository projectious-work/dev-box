# Plan: Screencast Recording Infrastructure & Visual Test Suite

**Date:** 2026-03-23
**Status:** Approved — ready for implementation

---

## Context

We replaced VHS (Chromium + ffmpeg + Docker sibling containers) with asciinema for headless
terminal recordings. The new approach is lightweight (~10MB vs ~400MB), needs no Docker socket,
and produces `.cast` files playable directly in docs via asciinema-player.

This plan covers three goals:

1. **Dogfooding alignment** — sync this project's config with what we ship to derived projects
2. **Visual test suite** — automated smoke tests using screencast recordings
3. **Complete docs recordings** — all layouts × all themes, plus CLI demos and tool showcases

### Decisions (from review)

- **Theme recordings show all apps.** Use the `dev` layout (shows 3 apps on first screen:
  Yazi, Vim, status bar), then cycle through tabs to show lazygit and shell with starship.
  Each recording is a mini "theme tour" — not just Zellij chrome.
- **README uses animated GIF.** Eye-catching, proven pipeline via `agg`.
- **Tests run on every release** and when implementation changes warrant it. Not on every
  `maintain.sh test` by default.
- **Sequential recording, not parallel.** Zellij must be the foreground PTY process for
  rendering. `pkill -x zellij` is global — no way to kill one session without affecting
  others. Tested: named sessions, backgrounded Zellij, kill-session API — none work
  headlessly. Sequential with 2s duration is fast enough: 6 themes in ~14s, 3 layouts
  in ~7s.

---

## Part 1: Dogfooding — Config Alignment

### Current state

This project has `dev-box.toml` with `process = "product"` but `.devcontainer/` is hand-maintained
(DEC comment in dev-box.toml: "avoiding circular deps"). The config files in `.devcontainer/config/`
are ~85% identical to `images/base/config/`, with specific gaps:

| Component | .devcontainer/ | images/base/ | Gap |
|-----------|---------------|-------------|-----|
| vimrc, gitconfig, asoundrc | identical | identical | none |
| zellij layouts (3) | identical | identical | none |
| zellij themes | gruvbox only | 6 themes | **5 missing** |
| yazi config (3 files) | identical | identical | none |
| yazi themes | missing | 5 theme files | **all missing** |
| vim colorschemes | none baked | 6 downloaded | **by design (DEC-013)** |
| open-in-editor.sh | v1 (basic) | v2 (layout-aware) | **outdated** |
| vim-loop.sh | missing | present | **missing** |
| bashrc | missing | present | **missing** |
| cheatsheet.txt | present | present | minor wording diff |

### Proposed sync

**Do now (prerequisite for theme recordings):**

1. Copy all 6 zellij themes from `images/base/config/zellij/themes/` into
   `.devcontainer/config/zellij/themes/`
2. Copy yazi theme files from `images/base/config/yazi/themes/`
3. Update `open-in-editor.sh` to the v2 layout-aware version

**Do later (tracked in backlog):**

4. Copy `bashrc` and `vim-loop.sh` from base image config
5. Evaluate whether `dev-box sync` should have a `--self` mode for this project

### What stays hand-maintained

The Dockerfile and docker-compose.yml remain hand-maintained. This project needs
Rust cross-compilation tooling, zensical, and asciinema — none of which belong in
the published base images. The circular dependency is real and intentional.

---

## Part 2: Visual Test Suite

### Concept

Use asciinema recordings as lightweight smoke tests. Instead of full end-to-end
container builds, we verify that tools start, render correctly, and respond to input —
all inside the current dev-container.

### Test categories

#### A. Layout tests (3 tests, ~2s each)

Record each layout for 2s, validate the cast file:
- Has >10 events (Zellij rendered something)
- File size >10KB (real rendering, not empty)
- Header has correct dimensions (160×45)

#### B. Theme tests (6 tests, ~2s each)

For each of the 6 themes, record the `dev` layout with theme override:
- Validate cast file is non-empty and >10KB
- Verify theme-specific ANSI color codes appear (each theme has a distinct bg color)

#### C. Tool smoke tests (8 tests, ~1s each)

Record a quick invocation of each tool, verify it starts:

| Tool | Command | Validate |
|------|---------|----------|
| zellij | `zellij --version` | output contains version |
| yazi | `yazi --version` | output contains version |
| vim | `vim --version` | output contains "VIM" |
| lazygit | `lazygit --version` | output contains version |
| starship | `starship --version` | output present |
| bat | `bat --version` | output present |
| eza | `eza --version` | output present |
| fzf | `fzf --version` | output present |

Note: tools not present in the dev-container are skipped (tested in base image builds).

#### D. CLI demo tests (2 tests, ~3s each)

- `dev-box init` with all flag combinations → verify exit code 0 and files created
- `dev-box doctor` → verify exit code 0

### Speed

Sequential recording with 2s duration per test. Verified timings:

| Category | Count | Duration each | Wall clock |
|----------|-------|--------------|------------|
| Layouts | 3 | 2s | ~7s |
| Themes | 6 | 2s | ~14s |
| Tool smoke | 8 | 1s | ~9s |
| CLI demos | 2 | 3s | ~7s |
| **Total** | **19** | | **~37s** |

### Parallelization — ruled out

Explored and rejected. Zellij must be the foreground process to render to the PTY.
Constraints tested:

- `pkill -x zellij` is global — kills all instances, can't target one session
- `zellij kill-session <name>` requires the server to be reachable, which fails
  when Zellij runs inside asciinema's isolated PTY
- Backgrounding Zellij (`&`) detaches it from the PTY — produces empty recordings
- Named sessions aren't created when Zellij runs headlessly (no session picker)

Sequential with 2s duration is ~37s total — fast enough for release-time testing.

### Validation script

`scripts/test-screencasts.sh`:
1. Records all casts in test mode (2s, no typing simulation) to `/tmp/devbox-test-casts/`
2. Validates each cast: header JSON, dimensions, event count >10, file size >10KB
3. Exits 0/1 for CI compatibility
4. Does NOT overwrite docs recordings

### Integration

```bash
./scripts/maintain.sh test-visual    # run screencast smoke tests (~37s)
./scripts/maintain.sh record-docs    # regenerate all docs recordings (~60s)
```

Run `test-visual` on every release (via release checklist) and after changes to
Dockerfile, layouts, themes, or installed tools.

---

## Part 3: Docs Recordings — Complete Set

### Theme tour recordings (6 recordings)

Each theme recording shows the `dev` layout with the theme applied and cycles
through the visible tools. The `dev` layout shows 3 apps on the first screen
(Yazi file manager, Vim editor, Zellij status bar). The recording script then
navigates to the lazygit tab and the shell tab (showing starship prompt) to
give a complete picture of the theme across all tools.

**Recording approach per theme:**

```
1. Start Zellij with dev layout + theme config override
2. Wait 2s for initial render (Yazi + Vim + status bar visible)
3. Switch to lazygit tab (Ctrl+b, 3) — wait 1s
4. Switch to shell tab (Ctrl+b, 4) — wait 1s
5. Switch back to dev tab (Ctrl+b, 1) — wait 1s
6. Kill Zellij
```

Total per theme: ~6s. Total for 6 themes: ~38s.

| Cast file | Theme | Embedded on |
|-----------|-------|------------|
| `theme-gruvbox-dark.cast` | Gruvbox Dark (default) | Themes page |
| `theme-catppuccin-mocha.cast` | Catppuccin Mocha | Themes page |
| `theme-catppuccin-latte.cast` | Catppuccin Latte | Themes page |
| `theme-dracula.cast` | Dracula | Themes page |
| `theme-tokyo-night.cast` | Tokyo Night | Themes page |
| `theme-nord.cast` | Nord | Themes page |

Each embedded as interactive player with `poster="npt:1"` (first screen with all 3 apps),
autoplay on hover or click.

### Layout recordings (3 recordings, already done)

| Cast file | Embedded on | Mode |
|-----------|------------|------|
| `layout-dev.cast` | Homepage, Base Image > Layouts | poster npt:4, autoplay false |
| `layout-focus.cast` | Base Image > Layouts | poster npt:4, controls false |
| `layout-cowork.cast` | Base Image > Layouts | poster npt:4, controls false |

### CLI demos (2 recordings)

| Cast file | Embedded on | Notes |
|-----------|------------|-------|
| `init-demo.cast` | Homepage, New Project | already done |
| `sync-demo.cast` | Configuration page | future — show theme change + sync |

### README

GitHub does not support custom JavaScript — asciinema-player won't work.

**Approach:** Animated GIF of the `dev` layout (gruvbox-dark theme) generated via `agg`,
embedded directly in README.md. Links to the full docs site for interactive versions.

Pipeline:
```bash
agg docs/assets/screencasts/layout-dev.cast docs/assets/readme-dev-layout.gif
```

### Recording script additions

New `record_theme()` function that:
1. Creates a temp Zellij config with `theme "<name>"`
2. Creates a driver script that starts Zellij, waits, cycles tabs, then gets killed
3. Records via asciinema with 160×45 dimensions

New modes for `record-asciinema.sh`:
- `themes` — record all 6 themes
- `readme` — generate animated GIF for README
- `all` — layouts + themes + demos + readme

---

## Execution Order

| Step | Task | Depends on | Effort |
|------|------|-----------|--------|
| 1 | Copy 6 zellij themes + yazi themes to .devcontainer/config/ | — | 5 min |
| 2 | Add `record_theme()` with tab cycling to record-asciinema.sh | Step 1 | 20 min |
| 3 | Record all 6 themes | Step 2 | 1 min (automated) |
| 4 | Embed theme screencasts on Themes docs page | Step 3 | 10 min |
| 5 | Add `readme` mode + generate GIF, embed in README.md | Step 3 | 10 min |
| 6 | Add `scripts/test-screencasts.sh` validation script | Step 2 | 20 min |
| 7 | Wire into maintain.sh + release checklist | Step 6 | 5 min |
| 8 | Copy bashrc + updated scripts from base image (backlog) | — | 15 min |

**Total immediate work (steps 1–7):** ~70 min
**Deferred (step 8):** backlog item
