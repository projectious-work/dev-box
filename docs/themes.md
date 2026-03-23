# Themes

dev-box supports consistent color theming across all terminal tools. Set a theme in `dev-box.toml`:

```toml
[appearance]
theme = "gruvbox-dark"
```

Or during project initialization:

```bash
dev-box init --theme catppuccin-mocha
```

The selected theme is applied to **Zellij**, **Vim**, **Yazi**, and **lazygit** simultaneously.

## Available Themes

### gruvbox-dark (default)

Retro groove color scheme with warm, earthy tones. High contrast and easy on the eyes.

- **Background:** `#282828` (dark brown-gray)
- **Accent:** `#D79921` (warm yellow)
- **Style:** Dark, warm, retro

<div class="asciinema" data-cast="assets/screencasts/theme-gruvbox-dark.cast" data-poster="npt:2" data-loop="true" data-theme="gruvbox-dark" data-fit="width"></div>

### catppuccin-mocha

Soothing pastel theme with a dark background. The most popular modern terminal theme.

- **Background:** `#1E1E2E` (deep purple-black)
- **Accent:** `#89B4FA` (soft blue)
- **Style:** Dark, pastel, modern

<div class="asciinema" data-cast="assets/screencasts/theme-catppuccin-mocha.cast" data-poster="npt:2" data-loop="true" data-theme="catppuccin-mocha" data-fit="width"></div>

### catppuccin-latte

Light variant of Catppuccin. Clean and readable in bright environments.

- **Background:** `#EFF1F5` (warm white)
- **Accent:** `#1E66F5` (vivid blue)
- **Style:** Light, pastel, modern

<div class="asciinema" data-cast="assets/screencasts/theme-catppuccin-latte.cast" data-poster="npt:2" data-loop="true" data-theme="catppuccin-latte" data-fit="width"></div>

### dracula

Dark theme with vibrant colors. A classic among developers.

- **Background:** `#282A36` (dark gray-blue)
- **Accent:** `#BD93F9` (purple)
- **Style:** Dark, vibrant, bold

<div class="asciinema" data-cast="assets/screencasts/theme-dracula.cast" data-poster="npt:2" data-loop="true" data-theme="dracula" data-fit="width"></div>

### tokyo-night

Inspired by Tokyo's night lights. Clean and modern with blue tones.

- **Background:** `#1A1B26` (deep blue-black)
- **Accent:** `#7AA2F7` (bright blue)
- **Style:** Dark, cool, modern

<div class="asciinema" data-cast="assets/screencasts/theme-tokyo-night.cast" data-poster="npt:2" data-loop="true" data-theme="tokyo-night" data-fit="width"></div>

### nord

Arctic, north-bluish color palette. Minimalist and calm.

- **Background:** `#2E3440` (dark blue-gray)
- **Accent:** `#88C0D0` (frost blue)
- **Style:** Dark, cool, minimalist

<div class="asciinema" data-cast="assets/screencasts/theme-nord.cast" data-poster="npt:2" data-loop="true" data-theme="nord" data-fit="width"></div>

## How It Works

Each theme is a coordinated set of config files applied to all tools:

| Tool | Config file | What's themed |
|------|------------|---------------|
| **Zellij** | `.config/zellij/themes/<name>.kdl` | Pane borders, status bar, tab colors |
| **Vim** | `.vim/colors/<name>.vim` | Syntax highlighting, UI elements |
| **Yazi** | `.config/yazi/theme.toml` | File colors, status bar, selection |
| **lazygit** | `.config/lazygit/config.yml` | Borders, selection, diff colors |

Claude Code inherits terminal colors automatically — no separate theme needed.

## Changing Themes

To switch themes in an existing project:

1. Edit `dev-box.toml`:
   ```toml
   [appearance]
   theme = "tokyo-night"
   ```

2. Run sync to apply the change:
   ```bash
   dev-box sync
   ```

3. Rebuild and restart:
   ```bash
   dev-box build --no-cache
   dev-box start
   ```

!!! note "Theme files are force-updated by sync"
    `dev-box sync` automatically overwrites theme-dependent config files (vimrc, zellij config, zellij themes, lazygit config, yazi theme) to match the selected theme. You do not need to manually delete them before switching.
