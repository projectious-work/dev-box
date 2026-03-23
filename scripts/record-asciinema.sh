#!/usr/bin/env bash
# record-asciinema.sh — generate documentation screencasts using asciinema
#
# Records terminal sessions as .cast files (asciicast v2 format) that can be
# played back with the asciinema-player in docs, or converted to GIF via agg.
#
# No sibling containers, no Docker socket, no Chromium — just a PTY.
#
# Prerequisites:
#   - asciinema (pip/uv: asciinema)
#   - agg (cargo install --git https://github.com/asciinema/agg) — optional, for GIF export
#   - zellij (for layout recordings)
#
# Usage:
#   ./scripts/record-asciinema.sh              # record all (layouts + themes + demos)
#   ./scripts/record-asciinema.sh layouts      # only layout recordings
#   ./scripts/record-asciinema.sh themes       # only theme tour recordings
#   ./scripts/record-asciinema.sh demos        # only CLI demo recordings
#   ./scripts/record-asciinema.sh gif          # generate GIFs via agg
#   ./scripts/record-asciinema.sh readme       # generate README animated GIF

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${PROJECT_ROOT}/docs/assets/screencasts"

# Terminal dimensions for recordings
LAYOUT_COLS=160
LAYOUT_ROWS=45
DEMO_COLS=100
DEMO_ROWS=30

# Available themes (must match files in .devcontainer/config/zellij/themes/)
THEMES=(gruvbox-dark catppuccin-mocha catppuccin-latte dracula tokyo-night nord)

# Theme bg/fg colors (must match images/base/config/zellij/themes/*.kdl)
declare -A THEME_BG=(
  [gruvbox-dark]="#282828"
  [catppuccin-mocha]="#1E1E2E"
  [catppuccin-latte]="#EFF1F5"
  [dracula]="#282A36"
  [tokyo-night]="#1A1B26"
  [nord]="#2E3440"
)
declare -A THEME_FG=(
  [gruvbox-dark]="#D5C4A1"
  [catppuccin-mocha]="#CDD6F4"
  [catppuccin-latte]="#4C4F69"
  [dracula]="#F8F8F2"
  [tokyo-night]="#C0CAF5"
  [nord]="#D8DEE9"
)

info()  { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
ok()    { printf '\033[1;32m ✓\033[0m  %s\n' "$1"; }
warn()  { printf '\033[1;33m !\033[0m  %s\n' "$1"; }
die()   { printf '\033[1;31mError:\033[0m %s\n' "$1" >&2; exit 1; }

# Clean up any leftover zellij sessions before recording
cleanup_zellij() {
  zellij delete-all-sessions --yes 2>/dev/null || true
  pkill -9 -x zellij 2>/dev/null || true
  rm -rf /tmp/zellij-* 2>/dev/null || true
  sleep 0.5
}

# Trim a cast file: keep only events between first large render and last large render.
# Removes shell startup noise before Zellij and exit noise after.
# Preserves any OSC escape sequences (terminal color settings) from early events.
trim_cast() {
  local cast="$1"
  python3 - "${cast}" << 'PYEOF'
import json, sys, re

cast_path = sys.argv[1]
lines = open(cast_path).readlines()
header = lines[0]
events = []
for line in lines[1:]:
    try:
        events.append(json.loads(line))
    except:
        pass

if not events:
    sys.exit(0)

# Collect OSC sequences (terminal color settings) from all events
osc_data = ""
for ev in events:
    # Match OSC 10 (fg) and OSC 11 (bg) sequences
    oscs = re.findall(r'\x1b\]1[01];#[A-Fa-f0-9]+\x1b\\\\?', ev[2])
    for osc in oscs:
        osc_data += osc

# Find first event with >500 bytes (Zellij first render)
first = 0
for i, ev in enumerate(events):
    if len(ev[2]) > 500:
        first = i
        break

# Find last event with >500 bytes (last real render before exit)
last = len(events) - 1
for i in range(len(events) - 1, -1, -1):
    if len(events[i][2]) > 100:
        last = i
        break

# Rebase timestamps to start at 0
trimmed = events[first:last+1]
if trimmed:
    t0 = trimmed[0][0]
    for ev in trimmed:
        ev[0] = round(ev[0] - t0, 6)

# Prepend OSC sequences as the first event (at t=0) so player picks up colors
if osc_data and trimmed:
    trimmed.insert(0, [0.0, "o", osc_data])

with open(cast_path, 'w') as f:
    f.write(header)
    for ev in trimmed:
        f.write(json.dumps(ev) + '\n')
PYEOF
}

# ─── Layout recording ────────────────────────────────────────────────────────
# Records a Zellij layout session headlessly via asciinema.
# Zellij runs in foreground inside asciinema's PTY; a background process
# kills it after DURATION seconds.

record_layout() {
  local layout="$1"
  local duration="${2:-5}"
  local output="${OUTPUT_DIR}/layout-${layout}.cast"

  info "Recording layout: ${layout} (${duration}s)..."
  cleanup_zellij

  local driver
  driver=$(mktemp /tmp/record-XXXX.sh)
  cat > "${driver}" << DRIVER
#!/usr/bin/env bash
export TERM=xterm-256color
export COLORTERM=truecolor
(sleep ${duration} && pkill -x zellij 2>/dev/null) &
zellij --layout "${layout}" 2>/dev/null
true
DRIVER
  chmod +x "${driver}"

  asciinema rec \
    --cols "${LAYOUT_COLS}" \
    --rows "${LAYOUT_ROWS}" \
    --overwrite \
    -c "${driver}" \
    "${output}" 2>/dev/null

  rm -f "${driver}"
  trim_cast "${output}"
  ok "layout-${layout}.cast ($(wc -l < "${output}") events)"
}

# ─── Theme tour recording ────────────────────────────────────────────────────
# Records the dev layout with a specific theme, cycling through tabs to show
# all themed tools: Yazi + Vim (tab 1), lazygit (tab 3), shell/starship (tab 4).
#
# Uses a hidden 1-row pane inside the layout that runs zellij action commands
# to switch tabs — this works because the pane runs inside the Zellij session.

record_theme() {
  local theme="$1"
  local output="${OUTPUT_DIR}/theme-${theme}.cast"

  info "Recording theme: ${theme}..."
  cleanup_zellij

  # Create the tab-switching script (runs inside a zellij pane)
  local switcher
  switcher=$(mktemp /tmp/switcher-XXXX.sh)
  cat > "${switcher}" << 'SWITCHER'
#!/usr/bin/env bash
sleep 3
zellij action go-to-tab 3 2>/dev/null   # git (lazygit)
sleep 2
zellij action go-to-tab 4 2>/dev/null   # shell (starship)
sleep 2
zellij action go-to-tab 1 2>/dev/null   # back to dev (yazi + vim)
sleep 1
sleep infinity
SWITCHER
  chmod +x "${switcher}"

  # Create a layout with the switcher embedded as a hidden 1-row pane
  local layout_file
  layout_file=$(mktemp /tmp/tour-layout-XXXX.kdl)
  cat > "${layout_file}" << LAYOUT
layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }
    tab name="dev" focus=true {
        pane split_direction="vertical" {
            pane size="40%" name="files" focus=true {
                command "yazi"
                cwd "/workspace"
            }
            pane size="60%" name="editor" {
                command "vim"
                cwd "/workspace"
            }
        }
        // Hidden switcher pane — cycles tabs during recording
        pane size=1 borderless=true {
            command "bash"
            args "-c" "${switcher}"
        }
    }
    tab name="claude" {
        pane name="claude" {
            command "bash"
            args "-c" "echo 'Claude Code'; sleep infinity"
            cwd "/workspace"
        }
    }
    tab name="git" {
        pane name="lazygit" {
            command "lazygit"
            cwd "/workspace"
        }
    }
    tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
        }
    }
}
LAYOUT

  # Create the theme config override
  local config_file
  config_file=$(mktemp /tmp/zellij-theme-XXXX.kdl)
  cat > "${config_file}" << CONF
theme "${theme}"
CONF

  # Look up theme colors for OSC injection
  local bg="${THEME_BG[${theme}]:-#000000}"
  local fg="${THEME_FG[${theme}]:-#FFFFFF}"

  # Driver script: set terminal bg/fg via OSC, then start zellij with theme
  local driver
  driver=$(mktemp /tmp/record-XXXX.sh)
  cat > "${driver}" << DRIVER
#!/usr/bin/env bash
export TERM=xterm-256color
export COLORTERM=truecolor
# Set terminal background/foreground to match theme (OSC 11/10)
printf '\033]11;${bg}\033\\\\'
printf '\033]10;${fg}\033\\\\'
(sleep 10 && pkill -x zellij 2>/dev/null) &
zellij --layout "${layout_file}" --config "${config_file}" 2>/dev/null
true
DRIVER
  chmod +x "${driver}"

  asciinema rec \
    --cols "${LAYOUT_COLS}" \
    --rows "${LAYOUT_ROWS}" \
    --overwrite \
    -c "${driver}" \
    "${output}" 2>/dev/null

  rm -f "${driver}" "${switcher}" "${layout_file}" "${config_file}"
  trim_cast "${output}"
  ok "theme-${theme}.cast ($(wc -l < "${output}") events)"
}

# ─── CLI demo recording ──────────────────────────────────────────────────────
# Records a scripted CLI demo (e.g., dev-box init) with simulated typing.

record_init_demo() {
  local output="${OUTPUT_DIR}/init-demo.cast"

  info "Recording demo: init..."

  local driver
  driver=$(mktemp /tmp/record-XXXX.sh)
  local workdir
  workdir=$(mktemp -d /tmp/demo-project-XXXX)

  cat > "${driver}" << DRIVER
#!/usr/bin/env bash
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="${PROJECT_ROOT}/cli/target/release:${PROJECT_ROOT}/cli/target/debug:\$PATH"
cd "${workdir}"

# Simulate typing: mkdir + cd
sleep 0.5
echo -ne '\033[32m❯\033[0m '
sleep 0.3
for c in m k d i r ' ' m y - p r o j e c t ' ' '&' '&' ' ' c d ' ' m y - p r o j e c t; do
  printf '%s' "\$c"
  sleep 0.06
done
echo
mkdir -p my-project && cd my-project

sleep 0.3
echo -ne '\033[32m❯\033[0m '
sleep 0.3
for c in d e v - b o x ' ' i n i t ' ' - - n a m e ' ' m y - p r o j e c t ' ' - - i m a g e ' ' p y t h o n ' ' - - p r o c e s s ' ' m a n a g e d; do
  printf '%s' "\$c"
  sleep 0.06
done
echo
dev-box init --name my-project --image python --process managed 2>&1 || true

sleep 1
echo -ne '\033[32m❯\033[0m '
sleep 0.3
for c in c a t ' ' d e v - b o x . t o m l; do
  printf '%s' "\$c"
  sleep 0.06
done
echo
cat dev-box.toml 2>/dev/null || echo "(dev-box.toml would appear here)"

sleep 2
DRIVER
  chmod +x "${driver}"

  asciinema rec \
    --cols "${DEMO_COLS}" \
    --rows "${DEMO_ROWS}" \
    --overwrite \
    -c "${driver}" \
    "${output}" 2>/dev/null

  rm -f "${driver}"
  rm -rf "${workdir}"
  ok "init-demo.cast"
}

# ─── GIF export via agg ──────────────────────────────────────────────────────

generate_gifs() {
  local pattern="${1:-*.cast}"

  if ! command -v agg &>/dev/null; then
    warn "agg not found — skipping GIF generation (cargo install --git https://github.com/asciinema/agg)"
    return
  fi

  info "Generating GIFs..."
  for cast in "${OUTPUT_DIR}"/${pattern}; do
    [ -f "${cast}" ] || continue
    local name
    name=$(basename "${cast}" .cast)
    local gif="${OUTPUT_DIR}/${name}.gif"
    agg "${cast}" "${gif}" 2>/dev/null
    ok "${name}.gif ($(du -h "${gif}" | cut -f1))"
  done
}

generate_readme_gif() {
  local cast="${OUTPUT_DIR}/layout-dev.cast"
  local gif="${PROJECT_ROOT}/docs/assets/readme-dev-layout.gif"

  if ! command -v agg &>/dev/null; then
    warn "agg not found — skipping README GIF"
    return
  fi

  [ -f "${cast}" ] || die "layout-dev.cast not found — run 'layouts' first"

  info "Generating README GIF..."
  agg "${cast}" "${gif}" 2>/dev/null
  ok "readme-dev-layout.gif ($(du -h "${gif}" | cut -f1))"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

mkdir -p "${OUTPUT_DIR}"

MODE="${1:-all}"

case "${MODE}" in
  layouts)
    record_layout dev 5
    record_layout focus 5
    record_layout cowork 5
    ;;
  themes)
    for theme in "${THEMES[@]}"; do
      record_theme "${theme}"
    done
    ;;
  demos)
    record_init_demo
    ;;
  gif)
    generate_gifs
    ;;
  readme)
    generate_readme_gif
    ;;
  all)
    record_layout dev 5
    record_layout focus 5
    record_layout cowork 5
    for theme in "${THEMES[@]}"; do
      record_theme "${theme}"
    done
    record_init_demo
    info "All recordings complete."
    echo ""
    generate_gifs
    generate_readme_gif
    ;;
  *)
    die "Unknown mode: ${MODE} (use: all, layouts, themes, demos, gif, readme)"
    ;;
esac

echo ""
info "Cast files:"
ls -1 "${OUTPUT_DIR}"/*.cast 2>/dev/null || echo "  (none)"
echo ""
if ls "${OUTPUT_DIR}"/*.gif &>/dev/null 2>&1; then
  info "GIF files:"
  ls -1 "${OUTPUT_DIR}"/*.gif
fi
