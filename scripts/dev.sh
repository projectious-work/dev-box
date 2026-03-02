#!/usr/bin/env bash
# =============================================================================
# dev.sh — dev-container manager
#
# Expected layout — the project root itself is mounted as /workspace:
#
#   <project>/
#   ├── scripts/
#   │   └── dev.sh          ← this script
#   ├── .devcontainer/
#   │   ├── Dockerfile
#   │   ├── docker-compose.yml
#   │   └── config/
#   │       ├── vimrc
#   │       ├── gitconfig
#   │       └── zellij/
#   ├── .root/              ← persisted host config (created on first run)
#
# Usage:
#   ./scripts/dev.sh <command> [options]
#
# Commands:
#   build     Build (or rebuild) the container image
#   start     Ensure the container is running, then attach via zellij
#   stop      Stop the running container
#   attach    Attach to an already-running container
#   status    Show container status
#   help      Show this help
# =============================================================================
set -euo pipefail

# ── Resolve project root (parent directory of this script) ───────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Paths ─────────────────────────────────────────────────────────────────────
DEVCONTAINER_DIR="${PROJECT_ROOT}/.devcontainer"
COMPOSE_FILE="${DEVCONTAINER_DIR}/docker-compose.yml"
HOST_ROOT="${HOST_ROOT:-${PROJECT_ROOT}/.root}"
WORKSPACE_DIR="${WORKSPACE_DIR:-${PROJECT_ROOT}}"

# ── Colours ───────────────────────────────────────────────────────────────────
bold=$'\e[1m'
cyan=$'\e[36m'
yellow=$'\e[33m'
red=$'\e[31m'
green=$'\e[32m'
reset=$'\e[0m'

info()  { echo "${cyan}${bold}==>${reset} $*"; }
ok()    { echo "${green}${bold} ✓${reset} $*"; }
warn()  { echo "${yellow}${bold}  !${reset} $*"; }
die()   { echo "${red}${bold}ERR${reset} $*" >&2; exit 1; }

# ── Help ──────────────────────────────────────────────────────────────────────
usage() {
  cat <<HELP
${bold}dev.sh${reset} — dev-container manager

${bold}Usage:${reset}
  ./scripts/dev.sh <command> [options]

${bold}Commands:${reset}
  build     Build (or rebuild) the container image
  start     Ensure the container is running, then attach via zellij
  stop      Stop the running container (data in .root/ and workspace/ is preserved)
  attach    Attach to an already-running container via zellij
  status    Show current container status
  help      Show this help

${bold}Options (start / build):${reset}
  --workspace <path>    Host path to mount as /workspace  (default: ./workspace)
  --root <path>         Host path for persisted config    (default: ./.root)

${bold}Environment variables:${reset}
  WORKSPACE_DIR         Same as --workspace
  HOST_ROOT             Same as --root
HELP
}

# ── Resolve compose binary ────────────────────────────────────────────────────
if command -v podman &>/dev/null; then
  COMPOSE_BIN="podman compose"
elif command -v docker &>/dev/null; then
  COMPOSE_BIN="docker compose"
else
  die "Neither podman nor docker found in PATH."
fi

# ── Helpers ───────────────────────────────────────────────────────────────────
make_dir() {
  if [[ ! -d "$1" ]]; then
    warn "Creating missing directory: $1"
    mkdir -p "$1"
  fi
}

seed_file() {
  local src="$1" dest="$2"
  if [[ ! -f "${dest}" && -f "${src}" ]]; then
    warn "Seeding $(realpath --relative-to="${PROJECT_ROOT}" "${dest}")"
    mkdir -p "$(dirname "${dest}")"
    cp "${src}" "${dest}"
  fi
}

ensure_host_dirs() {
  info "Checking host directories…"
  make_dir "${HOST_ROOT}/.ssh"
  make_dir "${HOST_ROOT}/.vim/undo"
  make_dir "${HOST_ROOT}/.config/zellij/themes"
  make_dir "${HOST_ROOT}/.config/zellij/layouts"
  make_dir "${HOST_ROOT}/.config/git"

  # Seed configs from .devcontainer/config/ on first run
  seed_file "${DEVCONTAINER_DIR}/config/vimrc"               "${HOST_ROOT}/.vim/vimrc"
  seed_file "${DEVCONTAINER_DIR}/config/gitconfig"            "${HOST_ROOT}/.config/git/config"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/config.kdl"   "${HOST_ROOT}/.config/zellij/config.kdl"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/themes/gruvbox.kdl" \
                                                              "${HOST_ROOT}/.config/zellij/themes/gruvbox.kdl"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/layouts/dev.kdl" \
                                                              "${HOST_ROOT}/.config/zellij/layouts/dev.kdl"

  if [[ -z "$(ls -A "${HOST_ROOT}/.ssh" 2>/dev/null)" ]]; then
    warn "No SSH keys found in ${HOST_ROOT}/.ssh — git over SSH will not work."
  fi
}

container_status() {
  # Returns: running | exited | missing
  local state
  state=$(${COMPOSE_BIN} -f "${COMPOSE_FILE}" ps --format json devcontainer 2>/dev/null \
    | grep -o '"State":"[^"]*"' | head -1 | cut -d'"' -f4 || true)
  if [[ -z "${state}" ]]; then
    echo "missing"
  else
    echo "${state}"
  fi
}

export_env() {
  export HOST_ROOT WORKSPACE_DIR
}

# ── Commands ──────────────────────────────────────────────────────────────────
cmd_build() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --workspace) WORKSPACE_DIR="$2"; shift 2 ;;
      --root)      HOST_ROOT="$2";      shift 2 ;;
      *)           die "Unknown option: $1" ;;
    esac
  done
  export_env
  ensure_host_dirs
  info "Building image…"
  ${COMPOSE_BIN} -f "${COMPOSE_FILE}" build
  ok "Build complete."
}

cmd_start() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --workspace) WORKSPACE_DIR="$2"; shift 2 ;;
      --root)      HOST_ROOT="$2";      shift 2 ;;
      *)           die "Unknown option: $1" ;;
    esac
  done
  export_env
  ensure_host_dirs

  local status
  status=$(container_status)

  case "${status}" in
    running)
      info "Container already running — attaching."
      ;;
    exited)
      info "Container exists but is stopped — starting…"
      ${COMPOSE_BIN} -f "${COMPOSE_FILE}" start devcontainer
      ;;
    missing)
      # Check if image exists; build if not
      local image_exists
      image_exists=$(${COMPOSE_BIN} -f "${COMPOSE_FILE}" images -q devcontainer 2>/dev/null || true)
      if [[ -z "${image_exists}" ]]; then
        warn "Image not found — building first…"
        ${COMPOSE_BIN} -f "${COMPOSE_FILE}" build
      fi
      info "Starting container…"
      ${COMPOSE_BIN} -f "${COMPOSE_FILE}" up -d devcontainer
      ;;
  esac

  cmd_attach
}

cmd_stop() {
  export_env
  local status
  status=$(container_status)
  if [[ "${status}" == "missing" ]]; then
    warn "Container is not running."
    exit 0
  fi
  info "Stopping container…"
  ${COMPOSE_BIN} -f "${COMPOSE_FILE}" stop devcontainer
  ok "Container stopped. Your work in .root/ and workspace/ is preserved."
}

cmd_attach() {
  export_env
  local status
  status=$(container_status)
  if [[ "${status}" != "running" ]]; then
    die "Container is not running. Run './scripts/dev.sh start' first."
  fi
  info "Attaching — launching zellij…"
  echo ""
  ${COMPOSE_BIN} -f "${COMPOSE_FILE}" exec devcontainer \
    zellij --layout dev
}

cmd_status() {
  export_env
  local status
  status=$(container_status)
  case "${status}" in
    running) ok  "Container is ${bold}running${reset}." ;;
    exited)  warn "Container is ${bold}stopped${reset} (run 'start' to resume)." ;;
    missing) warn "Container does not exist (run 'start' to create it)." ;;
  esac
}

# ── Entrypoint ────────────────────────────────────────────────────────────────
COMMAND="${1:-help}"
shift || true

case "${COMMAND}" in
  build)  cmd_build  "$@" ;;
  start)  cmd_start  "$@" ;;
  stop)   cmd_stop        ;;
  attach) cmd_attach      ;;
  status) cmd_status      ;;
  help|--help|-h) usage   ;;
  *) die "Unknown command: '${COMMAND}'. Run './scripts/dev.sh help' for usage." ;;
esac
