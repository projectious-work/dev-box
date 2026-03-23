#!/usr/bin/env bash
# test-screencasts.sh — visual smoke tests using asciinema recordings
#
# Records fast (2s) casts to a temp directory, validates they contain real
# terminal output. Does NOT overwrite docs recordings.
#
# Usage:
#   ./scripts/test-screencasts.sh              # run all tests
#   ./scripts/test-screencasts.sh layouts      # only layout tests
#   ./scripts/test-screencasts.sh themes       # only theme tests
#   ./scripts/test-screencasts.sh tools        # only tool smoke tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEST_DIR=$(mktemp -d /tmp/devbox-test-casts-XXXX)
FAILURES=0
PASSES=0
SKIPS=0

THEMES=(gruvbox-dark catppuccin-mocha catppuccin-latte dracula tokyo-night nord)
LAYOUTS=(dev focus cowork)

info()  { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
pass()  { printf '\033[1;32m ✓\033[0m  %s\n' "$1"; PASSES=$((PASSES + 1)); }
fail()  { printf '\033[1;31m ✗\033[0m  %s\n' "$1"; FAILURES=$((FAILURES + 1)); }
skip()  { printf '\033[1;33m ○\033[0m  %s (skipped)\n' "$1"; SKIPS=$((SKIPS + 1)); }

cleanup_zellij() {
  zellij delete-all-sessions --yes 2>/dev/null || true
  pkill -9 -x zellij 2>/dev/null || true
  rm -rf /tmp/zellij-* 2>/dev/null || true
  sleep 0.5
}

# ─── Validation helpers ───────────────────────────────────────────────────────

validate_cast() {
  local cast="$1"
  local label="$2"
  local min_events="${3:-10}"
  local min_size="${4:-5000}"

  if [ ! -f "${cast}" ]; then
    fail "${label}: cast file not created"
    return 1
  fi

  local size
  size=$(stat -c%s "${cast}" 2>/dev/null || echo 0)
  if [ "${size}" -lt "${min_size}" ]; then
    fail "${label}: too small (${size} bytes, need >${min_size})"
    return 1
  fi

  local events
  events=$(wc -l < "${cast}")
  if [ "${events}" -lt "${min_events}" ]; then
    fail "${label}: too few events (${events}, need >${min_events})"
    return 1
  fi

  # Verify header is valid JSON with correct version
  local version
  version=$(head -1 "${cast}" | python3 -c "import sys,json; print(json.load(sys.stdin)['version'])" 2>/dev/null || echo "")
  if [ "${version}" != "2" ]; then
    fail "${label}: invalid header (version=${version})"
    return 1
  fi

  pass "${label} (${events} events, $(numfmt --to=iec ${size}))"
  return 0
}

# ─── Layout tests ─────────────────────────────────────────────────────────────

test_layouts() {
  info "Testing layouts..."

  for layout in "${LAYOUTS[@]}"; do
    cleanup_zellij
    local cast="${TEST_DIR}/layout-${layout}.cast"
    local driver
    driver=$(mktemp /tmp/test-XXXX.sh)
    cat > "${driver}" << EOF
#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
(sleep 2 && pkill -x zellij 2>/dev/null) &
zellij --layout "${layout}" 2>/dev/null
true
EOF
    chmod +x "${driver}"
    asciinema rec --cols 160 --rows 45 --overwrite -c "${driver}" "${cast}" 2>/dev/null || true
    rm -f "${driver}"
    validate_cast "${cast}" "layout:${layout}"
  done
}

# ─── Theme tests ──────────────────────────────────────────────────────────────

test_themes() {
  info "Testing themes..."

  for theme in "${THEMES[@]}"; do
    cleanup_zellij
    local cast="${TEST_DIR}/theme-${theme}.cast"
    local config
    config=$(mktemp /tmp/theme-XXXX.kdl)
    echo "theme \"${theme}\"" > "${config}"

    local driver
    driver=$(mktemp /tmp/test-XXXX.sh)
    cat > "${driver}" << EOF
#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
(sleep 2 && pkill -x zellij 2>/dev/null) &
zellij --layout dev --config "${config}" 2>/dev/null
true
EOF
    chmod +x "${driver}"
    asciinema rec --cols 160 --rows 45 --overwrite -c "${driver}" "${cast}" 2>/dev/null || true
    rm -f "${driver}" "${config}"
    validate_cast "${cast}" "theme:${theme}"
  done
}

# ─── Tool smoke tests ────────────────────────────────────────────────────────

test_tools() {
  info "Testing tools..."

  declare -A tools=(
    [zellij]="zellij --version"
    [yazi]="yazi --version"
    [vim]="vim --version"
    [lazygit]="lazygit --version"
    [git]="git --version"
    [gh]="gh --version"
  )

  for tool in "${!tools[@]}"; do
    if ! command -v "${tool}" &>/dev/null; then
      skip "tool:${tool} (not installed)"
      continue
    fi

    local cast="${TEST_DIR}/tool-${tool}.cast"
    local cmd="${tools[${tool}]}"
    asciinema rec --cols 80 --rows 10 --overwrite \
      -c "${cmd}" "${cast}" 2>/dev/null || true
    validate_cast "${cast}" "tool:${tool}" 2 100
  done
}

# ─── CLI tests ────────────────────────────────────────────────────────────────

test_cli() {
  info "Testing CLI..."

  local devbox=""
  if [ -x "${PROJECT_ROOT}/cli/target/release/dev-box" ]; then
    devbox="${PROJECT_ROOT}/cli/target/release/dev-box"
  elif [ -x "${PROJECT_ROOT}/cli/target/debug/dev-box" ]; then
    devbox="${PROJECT_ROOT}/cli/target/debug/dev-box"
  else
    skip "cli:init (dev-box binary not found)"
    skip "cli:doctor (dev-box binary not found)"
    return
  fi

  # Test init
  local workdir
  workdir=$(mktemp -d /tmp/test-init-XXXX)
  local cast="${TEST_DIR}/cli-init.cast"
  asciinema rec --cols 100 --rows 20 --overwrite \
    -c "cd ${workdir} && ${devbox} init --name test --image base --process minimal 2>&1" \
    "${cast}" 2>/dev/null || true
  if [ -f "${workdir}/dev-box.toml" ]; then
    pass "cli:init (dev-box.toml created)"
  else
    fail "cli:init (dev-box.toml not found)"
  fi
  rm -rf "${workdir}"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

info "Visual smoke tests (output: ${TEST_DIR})"
echo ""

MODE="${1:-all}"

case "${MODE}" in
  layouts) test_layouts ;;
  themes)  test_themes ;;
  tools)   test_tools ;;
  cli)     test_cli ;;
  all)
    test_layouts
    echo ""
    test_themes
    echo ""
    test_tools
    echo ""
    test_cli
    ;;
  *)
    echo "Usage: $0 [all|layouts|themes|tools|cli]" >&2
    exit 1
    ;;
esac

# ─── Summary ──────────────────────────────────────────────────────────────────

echo ""
info "Results: ${PASSES} passed, ${FAILURES} failed, ${SKIPS} skipped"

# Cleanup
rm -rf "${TEST_DIR}"
cleanup_zellij

if [ "${FAILURES}" -gt 0 ]; then
  exit 1
fi
