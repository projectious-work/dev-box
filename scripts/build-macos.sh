#!/usr/bin/env bash
# =============================================================================
# build-macos.sh — build dev-box CLI binaries for macOS
#
# Run this on a macOS host (not inside the dev-container).
# Produces release binaries for both Apple Silicon and Intel Macs.
#
# Usage:
#   ./scripts/build-macos.sh [version]
#
# Examples:
#   ./scripts/build-macos.sh           # build without version tag
#   ./scripts/build-macos.sh 0.2.0     # build with version in artifact names
#
# Output:
#   dist/dev-box-[vVERSION-]aarch64-apple-darwin.tar.gz
#   dist/dev-box-[vVERSION-]x86_64-apple-darwin.tar.gz
#
# Prerequisites:
#   - macOS (any version with Xcode command line tools)
#   - Rust toolchain (script will prompt to install if missing)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"

# ── Colours ──────────────────────────────────────────────────────────────────
bold=$'\e[1m'
cyan=$'\e[36m'
red=$'\e[31m'
green=$'\e[32m'
yellow=$'\e[33m'
reset=$'\e[0m'

info()  { echo "${cyan}${bold}==>${reset} $*"; }
ok()    { echo "${green}${bold} ✓${reset} $*"; }
warn()  { echo "${yellow}${bold}  !${reset} $*"; }
die()   { echo "${red}${bold}ERR${reset} $*" >&2; exit 1; }

# ── Preflight checks ────────────────────────────────────────────────────────

# Must be macOS
[[ "$(uname -s)" == "Darwin" ]] || die "This script must be run on macOS."

# Check for Rust toolchain
if ! command -v cargo &>/dev/null; then
  echo ""
  warn "Rust toolchain not found."
  echo "  Install it with:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo ""
  die "Install Rust and re-run this script."
fi

# ── Parse arguments ──────────────────────────────────────────────────────────
VERSION="${1:-}"
if [[ -n "${VERSION}" ]]; then
  if ! [[ "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${VERSION})"
  fi
  VERSION_TAG="v${VERSION}-"
else
  VERSION_TAG=""
fi

# ── Targets ──────────────────────────────────────────────────────────────────
TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")

info "Ensuring Rust targets are installed..."
for target in "${TARGETS[@]}"; do
  rustup target add "${target}" 2>/dev/null || true
done
ok "Targets ready: ${TARGETS[*]}"

# ── Build ────────────────────────────────────────────────────────────────────
mkdir -p "${DIST_DIR}"

for target in "${TARGETS[@]}"; do
  info "Building for ${target}..."
  (cd "${CLI_DIR}" && cargo build --release --target "${target}")

  local_name="dev-box-${VERSION_TAG}${target}"
  cp "${CLI_DIR}/target/${target}/release/dev-box" "${DIST_DIR}/${local_name}"
  tar -czf "${DIST_DIR}/${local_name}.tar.gz" -C "${DIST_DIR}" "${local_name}"
  rm "${DIST_DIR}/${local_name}"
  ok "Built ${local_name}.tar.gz"
done

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo "${bold}macOS binaries built:${reset}"
echo ""
for target in "${TARGETS[@]}"; do
  local_name="dev-box-${VERSION_TAG}${target}"
  echo "  ${DIST_DIR}/${local_name}.tar.gz"
done
echo ""

if [[ -n "${VERSION}" ]]; then
  echo "To attach to an existing GitHub release:"
  echo ""
  echo "  gh release upload v${VERSION} dist/dev-box-v${VERSION}-*-apple-darwin.tar.gz"
  echo ""
fi
