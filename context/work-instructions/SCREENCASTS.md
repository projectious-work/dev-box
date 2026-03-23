# Screencast Recording & Visual Testing

## Overview

Terminal screencasts serve two purposes: documentation visuals and visual smoke tests.
All recordings use **asciinema** (headless PTY capture) — no Docker socket, no sibling
containers, no Chromium/browser dependencies.

## Architecture

```
asciinema rec --cols 160 --rows 45 -c driver.sh output.cast
                │                      │            │
                │                      │            └─ asciicast v2 (JSON lines)
                │                      └─ shell script that runs zellij/tools
                └─ creates a virtual PTY with fixed dimensions
```

**Key constraint:** Zellij must be the foreground process to render to the PTY.
Tab switching is achieved via a hidden 1-row pane inside the Zellij layout that
runs `zellij action go-to-tab N` commands — this works because the pane runs
inside the Zellij session where the action API is available.

Parallel recording is not possible: `pkill -x zellij` is global, and named
sessions / `kill-session` do not work headlessly. Sequential with short durations
(2s for tests, 5s for docs) is fast enough.

## Tools

| Tool | Purpose | Install |
|------|---------|---------|
| asciinema | Record terminal sessions to .cast files | `uv tool install asciinema` |
| agg | Convert .cast → animated GIF | `cargo install --git https://github.com/asciinema/agg` |
| asciinema-player | Browser playback of .cast files in docs | Vendored in `docs/assets/player/` |

## File Layout

```
scripts/
  record-asciinema.sh         # Generate docs recordings
  test-screencasts.sh          # Visual smoke test suite
docs/assets/
  player/
    asciinema-player.min.js    # Player JS (v3.15.1, vendored)
    asciinema-player.css       # Player CSS
    asciinema-init.js          # Auto-initializer for <div class="asciinema">
    asciinema-fonts.css        # @font-face for Nerd Font symbols
    NerdFontsSymbols.ttf       # Nerd Fonts Symbols Only (icons for Yazi/Zellij)
  screencasts/
    layout-{dev,focus,cowork}.cast    # Layout recordings
    theme-{name}.cast                 # Theme tour recordings (6 themes)
    init-demo.cast                    # CLI init demo
    *.gif                             # GIF exports (via agg)
  readme-dev-layout.gif              # Animated GIF for README.md
```

## Recording Scripts

### record-asciinema.sh

```bash
./scripts/record-asciinema.sh              # all (layouts + themes + demos + GIFs)
./scripts/record-asciinema.sh layouts      # 3 layout recordings (~17s)
./scripts/record-asciinema.sh themes       # 6 theme tour recordings (~65s)
./scripts/record-asciinema.sh demos        # CLI demo recordings (~10s)
./scripts/record-asciinema.sh gif          # generate GIFs from existing casts
./scripts/record-asciinema.sh readme       # generate README animated GIF
```

**Layout recordings:** Start Zellij with a layout, wait 5s for full render, kill.
Simple — just captures the static layout.

**Theme tour recordings:** More complex. Creates a custom layout with an embedded
1-row pane that runs a switcher script:

1. Zellij starts with `dev` layout + theme config override (3s to render)
2. Switcher pane runs `zellij action go-to-tab 3` → lazygit tab (2s)
3. Switcher runs `go-to-tab 4` → shell tab with starship (2s)
4. Switcher runs `go-to-tab 1` → back to dev tab (1s)
5. Background process kills Zellij after 10s total

**CLI demos:** Scripted typing simulation (printf + sleep per character) running
the actual `dev-box` binary. PATH is set to find the binary in `cli/target/`.

### test-screencasts.sh

```bash
./scripts/test-screencasts.sh              # all tests (~40s)
./scripts/test-screencasts.sh layouts      # layout tests only
./scripts/test-screencasts.sh themes       # theme tests only
./scripts/test-screencasts.sh tools        # tool availability tests
./scripts/test-screencasts.sh cli          # CLI functionality tests
```

Records to `/tmp/` (never overwrites docs recordings). Validates:
- Cast file exists and header is valid JSON (asciicast v2)
- Event count exceeds minimum threshold
- File size exceeds minimum (ensures real rendering, not empty)

Tests are fast (2s per layout/theme recording vs 5s for docs).

### maintain.sh integration

```bash
./scripts/maintain.sh test-visual          # run visual smoke tests
./scripts/maintain.sh record-docs          # regenerate all docs recordings
```

Visual tests are part of the release checklist (Phase 1, before build).

## Embedding in Docs

The asciinema-player is loaded via `extra_css` / `extra_javascript` in `zensical.toml`.
Any markdown page can embed a screencast:

```html
<!-- Interactive player (click to play) -->
<div class="asciinema" data-cast="assets/screencasts/layout-dev.cast"
     data-poster="npt:4" data-autoplay="false" data-fit="width"></div>

<!-- Still frame (no controls, poster frozen at timestamp) -->
<div class="asciinema" data-cast="assets/screencasts/theme-nord.cast"
     data-poster="npt:2" data-controls="false" data-autoplay="false" data-fit="width"></div>
```

**Path resolution:** The `data-cast` path is always relative to the docs root
(e.g., `assets/screencasts/...`). The init script (`asciinema-init.js`) derives
the site base URL from the player CSS link that MkDocs already relativizes.

**Nerd Fonts:** Yazi and Zellij use Nerd Font icons. The player's default font
stack includes `"Symbols Nerd Font"` as a fallback — `asciinema-fonts.css`
provides the `@font-face` that loads the vendored `NerdFontsSymbols.ttf`.

**GitHub README:** Does not support custom JS. Use the animated GIF instead:
```markdown
![dev-box dev layout](docs/assets/readme-dev-layout.gif)
```

## Adding a New Recording

1. Add a function to `record-asciinema.sh` (follow existing patterns)
2. Add a corresponding test to `test-screencasts.sh`
3. Run `./scripts/record-asciinema.sh <mode>` to generate the cast
4. Embed in the relevant docs page with `<div class="asciinema" ...>`
5. Run `zensical build -f zensical.toml` to verify

## Adding a New Theme

1. Add the `.kdl` file to `images/base/config/zellij/themes/`
2. Copy to `.devcontainer/config/zellij/themes/` (for local recordings)
3. Add the theme name to the `THEMES` array in `record-asciinema.sh`
4. Run `./scripts/record-asciinema.sh themes` to record it
5. Add the screencast embed to `docs/themes.md`
6. Add yazi theme if applicable to `images/base/config/yazi/themes/`

## Limitations

- **No parallel recording.** Zellij requires exclusive foreground PTY access.
  Sequential with 2s duration is ~40s for the full test suite.
- **Theme tour shows Zellij theming only.** Vim, Yazi, and lazygit inherit
  terminal colors but their full theme configs are not applied during recording
  (would require the tools to have active content loaded).
- **Nerd Font rendering depends on browser.** The vendored Symbols font covers
  most icons but exotic glyphs may still show as boxes.
