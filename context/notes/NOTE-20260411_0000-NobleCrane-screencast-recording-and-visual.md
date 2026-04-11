---
id: NOTE-20260411_0000-NobleCrane-screencast-recording-and-visual
title: "Screencast Recording and Visual Testing"
type: reference
status: permanent
created: 2026-04-11T00:00:00Z
tags: [screencasts, asciinema, docs, testing, visual]
skill: documentation
---

# Screencast Recording and Visual Testing

Terminal screencasts serve two purposes: documentation visuals and visual smoke tests.
All recordings use **asciinema** (headless PTY capture).

## Architecture

```
asciinema rec --cols 160 --rows 45 -c driver.sh output.cast
```

**Key constraint:** Zellij must be the foreground process to render to the PTY.
Tab switching is achieved via a hidden 1-row pane running `zellij action go-to-tab N`.
Parallel recording is not possible (`pkill -x zellij` is global). Sequential with
short durations (2s for tests, 5s for docs) is fast enough.

## Tools

| Tool | Purpose | Install |
|------|---------|---------|
| asciinema | Record terminal sessions to .cast files | `uv tool install asciinema` |
| agg | Convert .cast → animated GIF | `cargo install --git https://github.com/asciinema/agg` |
| asciinema-player | Browser playback in docs | Vendored in `docs-site/assets/player/` |

## File Layout

```
scripts/
  record-asciinema.sh         # Generate docs recordings
  test-screencasts.sh         # Visual smoke test suite
docs-site/assets/
  player/                     # Vendored asciinema player (v3.15.1)
  screencasts/
    layout-{dev,focus,cowork}.cast
    theme-{name}.cast          # 6 themes
    init-demo.cast
    *.gif                      # GIF exports
  readme-dev-layout.gif        # Animated GIF for README.md
```

## Recording Scripts

```bash
./scripts/record-asciinema.sh              # all (layouts + themes + demos + GIFs)
./scripts/record-asciinema.sh layouts      # 3 layout recordings (~17s)
./scripts/record-asciinema.sh themes       # 6 theme tour recordings (~65s)
./scripts/record-asciinema.sh demos        # CLI demo recordings (~10s)
./scripts/record-asciinema.sh gif          # generate GIFs from existing casts
./scripts/record-asciinema.sh readme       # generate README animated GIF
```

**Theme tour flow:** Zellij starts with dev layout + theme config override (3s) →
switcher pane runs go-to-tab 3 (lazygit, 2s) → go-to-tab 4 (shell+starship, 2s) →
go-to-tab 1 (back, 1s) → background process kills Zellij after 10s total.

## Visual Smoke Tests

```bash
./scripts/test-screencasts.sh              # all tests (~40s)
./scripts/test-screencasts.sh layouts      # layout tests only
./scripts/test-screencasts.sh themes       # theme tests only
```

Records to `/tmp/` (never overwrites docs recordings). Validates: cast file exists and
header is valid JSON, event count exceeds minimum, file size exceeds minimum.
Visual tests are part of the release checklist (Phase 1, before build).

## Embedding in Docs

```html
<!-- Interactive player -->
<div class="asciinema" data-cast="assets/screencasts/layout-dev.cast"
     data-poster="npt:4" data-autoplay="false" data-fit="width"></div>

<!-- Still frame -->
<div class="asciinema" data-cast="assets/screencasts/theme-nord.cast"
     data-poster="npt:2" data-controls="false" data-autoplay="false" data-fit="width"></div>
```

`data-cast` paths are relative to the docs root.
Nerd Fonts: `asciinema-fonts.css` provides `@font-face` loading `NerdFontsSymbols.ttf`.

## Adding a New Recording

1. Add a function to `record-asciinema.sh`
2. Add a corresponding test to `test-screencasts.sh`
3. Run `./scripts/record-asciinema.sh <mode>` to generate the cast
4. Embed in the relevant docs page with `<div class="asciinema" ...>`

## Adding a New Theme

1. Add `.kdl` file to `images/base-debian/config/zellij/themes/`
2. Copy to `.devcontainer/config/zellij/themes/`
3. Add theme name to `THEMES` array in `record-asciinema.sh`
4. Run `./scripts/record-asciinema.sh themes`
5. Add screencast embed to `docs-site/docs/themes.md` (or equivalent)
6. Add yazi theme if applicable

## Limitations

- No parallel recording — Zellij requires exclusive foreground PTY access.
- Theme tour shows Zellij theming only; Vim/Yazi/lazygit full themes not applied during recording.
- Nerd Font rendering depends on browser; exotic glyphs may render as boxes.
