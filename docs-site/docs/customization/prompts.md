---
sidebar_position: 2
title: "Prompt Presets"
---

# Starship Prompt Presets

aibox includes 7 [Starship](https://starship.rs) prompt presets that work with any theme. Set a preset in `aibox.toml`:

```toml
[appearance]
prompt = "default"
```

## Available Presets

### default

Full-featured two-line prompt with directory, git branch/status, language versions, and command duration. Uses Nerd Font symbols. Good all-around choice.

<!-- recording pending -->

### plain

Same information as `default` but uses ASCII characters only — no Nerd Font or special font needed. Works in any terminal. Good for remote SSH sessions or environments without font customization.

<!-- recording pending -->

### minimal

Directory and git branch only, with a minimal `❯` character indicator. Two-line. For distraction-free, low-noise work.

<!-- recording pending -->

### nerd-font

Rich prompt with Nerd Font icons for OS, languages, git status, Docker context, and system info. Requires a [Nerd Font](https://www.nerdfonts.com/) installed on the host terminal.

<!-- recording pending -->

### pastel

Soft powerline-style prompt with filled rounded segment separators (``) and gentle colors. Directory and git branch appear in colored blocks. Nerd Font recommended.

<!-- recording pending -->

### bracketed

Each segment wrapped in square brackets — `[dir] [branch] [status]`. Clean, structured appearance without special fonts. A good alternative to `plain` with more visual structure.

<!-- recording pending -->

### arrow

Airline/powerline-style prompt with hard chevron separators (`►`). Segments for directory, git branch, and git status appear as connected colored blocks, with command duration shown inline. Requires a Nerd Font or Powerline-patched font.

```
 ~/workspace/myproject  main +1 !2  took 3s
❯
```

<!-- recording pending -->

## Changing Presets

1. Edit `aibox.toml`:
   ```toml
   [appearance]
   prompt = "arrow"
   ```

2. Run sync:
   ```bash
   aibox sync
   ```

The Starship config is regenerated at `.aibox-home/.config/starship.toml`. Colors are derived from the active theme.

## Font Requirements

| Preset | Font requirement |
|--------|-----------------|
| `default` | Nerd Font recommended (for `❯` symbol) |
| `plain` | Any font — ASCII only |
| `minimal` | Nerd Font recommended (for `❯` symbol) |
| `nerd-font` | Nerd Font required |
| `pastel` | Nerd Font or Powerline font required |
| `bracketed` | Any font — no special glyphs |
| `arrow` | Nerd Font or Powerline font required |

Install a Nerd Font from [nerdfonts.com](https://www.nerdfonts.com/) and configure it in your terminal emulator to use icon-based presets.
