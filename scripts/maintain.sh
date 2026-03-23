#!/usr/bin/env bash
# =============================================================================
# maintain.sh — maintenance script for the dev-box project itself
#
# This manages the dev-container we develop IN (not the containers we publish).
# For downstream project container management, use the dev-box CLI.
#
# Usage:
#   ./scripts/maintain.sh <command> [options]
#
# Commands:
#   test              Run cargo fmt, clippy, and tests
#   build-images      Build all 10 published images locally
#   docs-serve        Serve MkDocs locally for preview
#   docs-deploy       Build MkDocs and push HTML to gh-pages
#   release <version> Tag, build, compile CLI, generate release prompt
#   start             Start this project's dev-container
#   stop              Stop this project's dev-container
#   attach            Attach to running dev-container via zellij
#   status            Show dev-container status
#   help              Show this help
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Paths ────────────────────────────────────────────────────────────────────
DEVCONTAINER_DIR="${PROJECT_ROOT}/.devcontainer"
COMPOSE_FILE="${DEVCONTAINER_DIR}/docker-compose.yml"
HOST_ROOT="${HOST_ROOT:-${PROJECT_ROOT}/.root}"
WORKSPACE_DIR="${WORKSPACE_DIR:-${PROJECT_ROOT}}"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"
IMAGE_REGISTRY="ghcr.io/projectious-work/dev-box"

# ── Read container name from docker-compose.yml ─────────────────────────────
_init_names() {
  local svc cn
  svc=$(grep -E '^\s{2}[a-zA-Z0-9_-]+:' "${COMPOSE_FILE}" | head -1 | tr -d ' :')
  cn=$(grep 'container_name:' "${COMPOSE_FILE}" | head -1 | awk '{print $2}')
  SERVICE_NAME="${svc}"
  CONTAINER_NAME="${cn:-${svc}}"
}
_init_names

# ── Colours ──────────────────────────────────────────────────────────────────
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

# ── Resolve container runtime ────────────────────────────────────────────────
# Check which runtime is actually functional, not just installed on PATH.
# Podman may exist as a compatibility shim (e.g., OrbStack) but not be running.
if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
  COMPOSE_BIN="docker compose"
  RUNTIME_BIN="docker"
elif command -v podman &>/dev/null && podman info &>/dev/null 2>&1; then
  COMPOSE_BIN="podman compose"
  RUNTIME_BIN="podman"
else
  # Not fatal — some commands (test, docs) don't need a runtime
  COMPOSE_BIN=""
  RUNTIME_BIN=""
fi

# ── Help ─────────────────────────────────────────────────────────────────────
usage() {
  cat <<HELP
${bold}maintain.sh${reset} — dev-box project maintenance

${bold}Usage:${reset}
  ./scripts/maintain.sh <command> [options]

${bold}Development:${reset}
  test                     Run cargo fmt check, clippy, and tests
  build-images [--no-cache] Build all 10 published images locally
  push-images <version>    Push images to GHCR (requires ghcr.io login)
  docs-serve               Serve MkDocs locally (http://localhost:8000)
  docs-deploy [--dry-run]  Build MkDocs and push to gh-pages branch
  test-visual              Run screencast smoke tests (~40s)
  record-docs              Regenerate all docs screencasts + README GIF

${bold}Release:${reset}
  release <version>        Tag, build CLI, generate release prompt
                           Version must be semver (e.g. 0.2.0)

${bold}Container (this project's dev-container):${reset}
  start                    Ensure running, then attach via zellij
  stop                     Stop the dev-container
  attach                   Attach to running dev-container
  status                   Show dev-container status
  help                     Show this help
HELP
}

# =============================================================================
# Container helpers (from the original dev.sh)
# =============================================================================

_require_runtime() {
  [[ -n "${RUNTIME_BIN}" ]] || die "Neither podman nor docker found."
}

container_status() {
  _require_runtime
  local state
  state=$(${RUNTIME_BIN} inspect --format '{{.State.Status}}' "${CONTAINER_NAME}" 2>/dev/null || true)
  case "${state}" in
    running)        echo "running" ;;
    exited|stopped) echo "exited"  ;;
    *)              echo "missing" ;;
  esac
}

wait_for_running() {
  local attempts=0 max=15
  while [[ $attempts -lt $max ]]; do
    if [[ "$(container_status)" == "running" ]]; then
      return 0
    fi
    sleep 0.5
    (( attempts++ ))
  done
  die "Container did not reach running state."
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
  info "Checking host directories..."
  mkdir -p "${HOST_ROOT}"/{.ssh,.vim/undo,.config/zellij/{themes,layouts},.config/yazi,.config/git,.claude}

  seed_file "${DEVCONTAINER_DIR}/config/vimrc"                        "${HOST_ROOT}/.vim/vimrc"
  seed_file "${DEVCONTAINER_DIR}/config/gitconfig"                     "${HOST_ROOT}/.config/git/config"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/config.kdl"             "${HOST_ROOT}/.config/zellij/config.kdl"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/themes/gruvbox.kdl"     "${HOST_ROOT}/.config/zellij/themes/gruvbox.kdl"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/layouts/dev.kdl"        "${HOST_ROOT}/.config/zellij/layouts/dev.kdl"
  seed_file "${DEVCONTAINER_DIR}/config/zellij/layouts/focus.kdl"      "${HOST_ROOT}/.config/zellij/layouts/focus.kdl"
  seed_file "${DEVCONTAINER_DIR}/config/yazi/yazi.toml"                "${HOST_ROOT}/.config/yazi/yazi.toml"
  seed_file "${DEVCONTAINER_DIR}/config/yazi/keymap.toml"              "${HOST_ROOT}/.config/yazi/keymap.toml"
  seed_file "${DEVCONTAINER_DIR}/config/yazi/theme.toml"               "${HOST_ROOT}/.config/yazi/theme.toml"

  if [[ -z "$(ls -A "${HOST_ROOT}/.ssh" 2>/dev/null)" ]]; then
    warn "No SSH keys in ${HOST_ROOT}/.ssh"
  fi
}

# =============================================================================
# Commands
# =============================================================================

cmd_test() {
  info "Running cargo fmt check..."
  (cd "${CLI_DIR}" && cargo fmt --check) || die "Format check failed. Run: cd cli && cargo fmt"
  ok "Format OK"

  info "Running clippy..."
  (cd "${CLI_DIR}" && cargo clippy -- -D warnings) || die "Clippy failed"
  ok "Clippy OK"

  info "Running tests..."
  (cd "${CLI_DIR}" && cargo test) || die "Tests failed"
  ok "All tests passed"
}

cmd_build_images() {
  _require_runtime
  local no_cache=""
  [[ "${1:-}" == "--no-cache" ]] && no_cache="--no-cache"

  local flavors=("base" "python" "rust" "latex" "typst" "node" "go" "python-latex" "python-typst" "rust-latex")

  for flavor in "${flavors[@]}"; do
    info "Building ${flavor} image..."
    ${RUNTIME_BIN} build ${no_cache} \
      -t "${IMAGE_REGISTRY}:${flavor}-latest" \
      -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
      "${PROJECT_ROOT}/images/${flavor}/"
    ok "Built ${IMAGE_REGISTRY}:${flavor}-latest"
  done

  ok "All 10 images built"
}

cmd_push_images() {
  _require_runtime
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh push-images <version>  (e.g. 0.2.0)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  # Verify GHCR login — try gh auth first, then fall back to manual instructions
  if ! ${RUNTIME_BIN} login ghcr.io --get-login &>/dev/null 2>&1; then
    if command -v gh &>/dev/null && gh auth status &>/dev/null; then
      info "Logging into ghcr.io via gh auth..."
      gh auth token | ${RUNTIME_BIN} login ghcr.io -u "$(gh api user --jq .login)" --password-stdin \
        || die "Failed to log in to ghcr.io via gh. Ensure your gh token has write:packages scope."
      ok "Logged in to ghcr.io"
    else
      echo ""
      info "Not logged in to ghcr.io. Either:"
      echo ""
      echo "  1. Install and authenticate gh CLI: gh auth login"
      echo "  2. Or log in manually:"
      echo "     echo \$GITHUB_TOKEN | ${RUNTIME_BIN} login ghcr.io -u <username> --password-stdin"
      echo ""
      echo "  Your token needs the write:packages scope."
      echo ""
      die "GHCR authentication required."
    fi
  fi

  local flavors=("base" "python" "rust" "latex" "typst" "node" "go" "python-latex" "python-typst" "rust-latex")

  # Verify all latest images exist and create versioned tags
  for flavor in "${flavors[@]}"; do
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"
    local versioned="${IMAGE_REGISTRY}:${flavor}-v${version}"
    if ! ${RUNTIME_BIN} image exists "${latest}" 2>/dev/null && \
       ! ${RUNTIME_BIN} inspect "${latest}" &>/dev/null; then
      die "Image ${latest} not found locally. Run 'build-images' first."
    fi
    ${RUNTIME_BIN} tag "${latest}" "${versioned}"
  done

  ok "All images found and tagged for v${version}"

  # Push versioned and latest tags
  for flavor in "${flavors[@]}"; do
    local versioned="${IMAGE_REGISTRY}:${flavor}-v${version}"
    local latest="${IMAGE_REGISTRY}:${flavor}-latest"

    info "Pushing ${flavor}..."
    ${RUNTIME_BIN} push "${versioned}" || die "Failed to push ${versioned}"
    ${RUNTIME_BIN} push "${latest}" || die "Failed to push ${latest}"
    ok "Pushed ${flavor}-v${version} + ${flavor}-latest"
  done

  echo ""
  ok "All 10 images pushed to ${IMAGE_REGISTRY}"
  info "Verify at: https://github.com/orgs/projectious-work/packages"
}

cmd_docs_serve() {
  cd "${PROJECT_ROOT}"
  if command -v zensical &>/dev/null; then
    info "Serving docs with Zensical at http://localhost:8000 ..."
    zensical serve -f zensical.toml -a 0.0.0.0:8000
  elif command -v mkdocs &>/dev/null; then
    info "Serving docs with MkDocs at http://localhost:8000 ..."
    mkdocs serve -a 0.0.0.0:8000
  else
    die "Neither zensical nor mkdocs found. Install: pip install zensical"
  fi
}

cmd_docs_deploy() {
  local dry_run=false
  [[ "${1:-}" == "--dry-run" ]] && dry_run=true

  local docs_cmd=""
  if command -v zensical &>/dev/null; then
    docs_cmd="zensical"
  elif command -v mkdocs &>/dev/null; then
    docs_cmd="mkdocs"
  else
    die "Neither zensical nor mkdocs found. Install: pip install zensical"
  fi
  command -v git &>/dev/null    || die "git not found"
  git rev-parse --is-inside-work-tree &>/dev/null || die "Not inside a git repository"

  local remote_url current_branch commit_sha commit_msg repo_slug
  remote_url=$(git remote get-url origin 2>/dev/null) || die "No 'origin' remote"
  current_branch=$(git rev-parse --abbrev-ref HEAD)
  commit_sha=$(git rev-parse --short HEAD)
  commit_msg="docs: deploy from ${current_branch}@${commit_sha} ($(date -u +%Y-%m-%dT%H:%M:%SZ))"
  repo_slug=$(echo "${remote_url}" | sed -E 's|.*[:/]([^/]+/[^/]+)(\.git)?$|\1|' | sed 's/\.git$//')

  info "Remote: ${remote_url}"
  info "Source: ${current_branch}@${commit_sha}"

  cd "${PROJECT_ROOT}"
  info "Building docs with ${docs_cmd}..."
  if [[ "${docs_cmd}" == "zensical" ]]; then
    ${docs_cmd} build -f zensical.toml -c
  else
    ${docs_cmd} build --strict --clean
  fi
  ok "Site built in site/"

  if [[ "${dry_run}" == "true" ]]; then
    warn "Dry run — site is in site/"
    return 0
  fi

  local tmpdir
  tmpdir=$(mktemp -d)
  trap 'rm -rf "${tmpdir}"' EXIT

  cp -r site/* "${tmpdir}/"
  touch "${tmpdir}/.nojekyll"

  info "Pushing to gh-pages branch..."
  cd "${tmpdir}"
  git init -q
  git checkout -q -b gh-pages
  git add -A
  git commit -q -m "${commit_msg}"
  git push --force "${remote_url}" gh-pages:gh-pages
  cd "${PROJECT_ROOT}"
  ok "Deployed to gh-pages branch"

  # Configure GitHub Pages if gh is available
  if command -v gh &>/dev/null && [[ -n "${repo_slug}" ]]; then
    info "Configuring GitHub Pages for ${repo_slug}..."
    gh api --method PUT "repos/${repo_slug}/pages" \
      -f "source[branch]=gh-pages" -f "source[path]=/" \
      --silent 2>/dev/null && ok "GitHub Pages configured" \
      || warn "Could not configure Pages automatically"
  fi

  echo ""
  ok "Documentation deployed."
  [[ -n "${repo_slug}" ]] && info "URL: https://${repo_slug/\//.github.io\/}/"
  trap - EXIT
  rm -rf "${tmpdir}"
}

cmd_release() {
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release <version>  (e.g. 0.2.0)"

  # Validate semver (simple check)
  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  local tag="v${version}"

  # ── Step 1: Preflight ──────────────────────────────────────────────────────
  info "Preparing release ${tag}..."

  # Check for uncommitted changes
  if [[ -n "$(git status --porcelain)" ]]; then
    die "Working tree is dirty. Commit or stash changes first."
  fi

  # Check tag doesn't exist
  if git rev-parse "${tag}" &>/dev/null; then
    die "Tag ${tag} already exists."
  fi

  # ── Step 2: Run tests ──────────────────────────────────────────────────────
  info "Running tests..."
  cmd_test

  # ── Step 3: Build CLI for current architecture ─────────────────────────────
  info "Building CLI (release mode)..."
  mkdir -p "${DIST_DIR}"

  local arch target binary_name
  arch=$(uname -m)
  case "${arch}" in
    x86_64)  target="x86_64-unknown-linux-gnu" ;;
    aarch64) target="aarch64-unknown-linux-gnu" ;;
    *)       target="${arch}-unknown-linux-gnu" ;;
  esac

  (cd "${CLI_DIR}" && cargo build --release)
  binary_name="dev-box-v${version}-${target}"
  cp "${CLI_DIR}/target/release/dev-box" "${DIST_DIR}/${binary_name}"
  tar -czf "${DIST_DIR}/${binary_name}.tar.gz" -C "${DIST_DIR}" "${binary_name}"
  rm "${DIST_DIR}/${binary_name}"
  ok "Built ${binary_name}.tar.gz"

  # ── Step 4: Build images (if runtime available) ────────────────────────────
  if [[ -n "${RUNTIME_BIN}" ]]; then
    info "Building container images..."
    local flavors=("base" "python" "rust" "latex" "typst" "node" "go" "python-latex" "python-typst" "rust-latex")
    for flavor in "${flavors[@]}"; do
      ${RUNTIME_BIN} build \
        -t "${IMAGE_REGISTRY}:${flavor}-v${version}" \
        -t "${IMAGE_REGISTRY}:${flavor}-latest" \
        -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
        "${PROJECT_ROOT}/images/${flavor}/" &>/dev/null
      ok "Built ${flavor}-v${version}"
    done
  else
    warn "No container runtime — skipping image builds"
  fi

  # ── Step 5: Create git tag ────────────────────────────────────────────────
  info "Creating tag ${tag}..."
  git tag -a "${tag}" -m "Release ${tag}"
  ok "Tag ${tag} created (not pushed yet)"

  # ── Step 6: Generate release notes ─────────────────────────────────────────
  info "Generating release notes..."
  local prev_tag notes_file
  prev_tag=$(git describe --tags --abbrev=0 "${tag}^" 2>/dev/null || echo "")
  notes_file="${DIST_DIR}/RELEASE-NOTES.md"

  {
    echo "# dev-box ${tag}"
    echo ""
    if [[ -n "${prev_tag}" ]]; then
      echo "## Changes since ${prev_tag}"
      echo ""
      git log --oneline "${prev_tag}..${tag}" | sed 's/^/- /'
    else
      echo "## Initial release"
      echo ""
      git log --oneline "${tag}" | head -20 | sed 's/^/- /'
    fi
    echo ""
    echo "## Container Images"
    echo ""
    echo "| Image | Tag |"
    echo "|-------|-----|"
    for flavor in base python latex typst rust node go python-latex python-typst rust-latex; do
      echo "| ${flavor} | \`${IMAGE_REGISTRY}:${flavor}-v${version}\` |"
    done
    echo ""
    echo "## CLI Binaries"
    echo ""
    for f in "${DIST_DIR}"/dev-box-v${version}-*.tar.gz; do
      [[ -f "$f" ]] && echo "- $(basename "$f")"
    done
  } > "${notes_file}"

  ok "Release notes written to dist/RELEASE-NOTES.md"

  # ── Step 7: Generate release prompt ────────────────────────────────────────
  local prompt_file="${DIST_DIR}/RELEASE-PROMPT.md"
  local remote_url
  remote_url=$(git remote get-url origin 2>/dev/null || echo "origin")
  local repo_slug
  repo_slug=$(echo "${remote_url}" | sed -E 's|.*[:/]([^/]+/[^/]+)(\.git)?$|\1|' | sed 's/\.git$//')

  {
    echo "# Release Prompt for dev-box ${tag}"
    echo ""
    echo "Give this prompt to an AI agent or execute the commands manually."
    echo ""
    echo "---"
    echo ""
    echo "## Task: Create GitHub Release for dev-box ${tag}"
    echo ""
    echo "### Step 1: Push the tag"
    echo ""
    echo "\`\`\`bash"
    echo "git push origin ${tag}"
    echo "\`\`\`"
    echo ""
    echo "### Step 2: Push container images to GHCR"
    echo ""
    echo "\`\`\`bash"
    echo "./scripts/maintain.sh push-images ${version}"
    echo "\`\`\`"
    echo ""
    echo "### Step 3: Create the GitHub release with artifacts"
    echo ""
    echo "Use the following command to create the release and attach all CLI binaries:"
    echo ""
    echo "\`\`\`bash"
    echo "gh release create ${tag} \\"
    echo "  --repo ${repo_slug} \\"
    echo "  --title \"dev-box ${tag}\" \\"
    echo "  --notes-file dist/RELEASE-NOTES.md \\"
    # List all artifacts
    for f in "${DIST_DIR}"/dev-box-v${version}-*.tar.gz; do
      [[ -f "$f" ]] && echo "  \"$f\" \\"
    done
    echo ""
    echo "\`\`\`"
    echo ""
    echo "### Step 4: Deploy documentation"
    echo ""
    echo "\`\`\`bash"
    echo "./scripts/maintain.sh docs-deploy"
    echo "\`\`\`"
    echo ""
    echo "### Missing artifacts"
    echo ""
    echo "The following targets could not be built from this environment"
    echo "(build them on the respective platforms and attach to the release):"
    echo ""
    # List targets we didn't build
    local built_target="${target}"
    for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin; do
      if [[ "${t}" != "${built_target}" ]]; then
        echo "- \`dev-box-v${version}-${t}.tar.gz\` — build with:"
        echo "  \`\`\`"
        echo "  rustup target add ${t}"
        echo "  cargo build --release --target ${t}"
        echo "  cp target/${t}/release/dev-box dist/dev-box-v${version}-${t}"
        echo "  tar -czf dist/dev-box-v${version}-${t}.tar.gz -C dist dev-box-v${version}-${t}"
        echo "  gh release upload ${tag} dist/dev-box-v${version}-${t}.tar.gz"
        echo "  \`\`\`"
      fi
    done
  } > "${prompt_file}"

  ok "Release prompt written to dist/RELEASE-PROMPT.md"

  # ── Summary ────────────────────────────────────────────────────────────────
  echo ""
  echo "${bold}Release ${tag} prepared.${reset}"
  echo ""
  echo "  Artifacts:     ${DIST_DIR}/"
  for f in "${DIST_DIR}"/dev-box-v${version}-*.tar.gz; do
    [[ -f "$f" ]] && echo "                 $(basename "$f")"
  done
  echo "  Release notes: dist/RELEASE-NOTES.md"
  echo "  Release prompt: dist/RELEASE-PROMPT.md"
  echo ""
  echo "  ${bold}Next:${reset} Review dist/RELEASE-PROMPT.md, then give it to"
  echo "  an AI agent or run the commands yourself."
}

# ── Container commands ───────────────────────────────────────────────────────

cmd_start() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  ensure_host_dirs

  local status
  status=$(container_status)
  case "${status}" in
    running)
      info "Container already running — attaching."
      ;;
    exited)
      info "Starting stopped container..."
      if ! ${COMPOSE_BIN} -f "${COMPOSE_FILE}" start "${SERVICE_NAME}" 2>/dev/null; then
        ${RUNTIME_BIN} start "${CONTAINER_NAME}"
      fi
      wait_for_running
      ;;
    missing)
      local image_exists
      image_exists=$(${COMPOSE_BIN} -f "${COMPOSE_FILE}" images -q "${SERVICE_NAME}" 2>/dev/null || true)
      if [[ -z "${image_exists}" ]]; then
        warn "Image not found — building first..."
        ${COMPOSE_BIN} -f "${COMPOSE_FILE}" build
      fi
      info "Starting container..."
      ${COMPOSE_BIN} -f "${COMPOSE_FILE}" up -d "${SERVICE_NAME}"
      wait_for_running
      ;;
  esac
  cmd_attach
}

cmd_stop() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  local status
  status=$(container_status)
  if [[ "${status}" == "missing" ]]; then
    warn "Container is not running."
    exit 0
  fi
  info "Stopping container..."
  if ! ${COMPOSE_BIN} -f "${COMPOSE_FILE}" stop "${SERVICE_NAME}" 2>/dev/null; then
    ${RUNTIME_BIN} stop "${CONTAINER_NAME}"
  fi
  ok "Container stopped."
}

cmd_attach() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  local status
  status=$(container_status)
  if [[ "${status}" != "running" ]]; then
    die "Container is not running. Run './scripts/maintain.sh start' first."
  fi
  info "Attaching — launching zellij..."
  echo ""
  ${RUNTIME_BIN} exec -it "${CONTAINER_NAME}" zellij --layout dev
}

cmd_status() {
  _require_runtime
  export HOST_ROOT WORKSPACE_DIR
  local status
  status=$(container_status)
  case "${status}" in
    running) ok  "Container is ${bold}running${reset}." ;;
    exited)  warn "Container is ${bold}stopped${reset} (run 'start' to resume)." ;;
    missing) warn "Container does not exist (run 'start' to create it)." ;;
  esac
}

cmd_test_visual() {
  info "Running visual smoke tests..."
  "${SCRIPT_DIR}/test-screencasts.sh" all
}

cmd_record_docs() {
  info "Recording all docs screencasts..."
  "${SCRIPT_DIR}/record-asciinema.sh" all
}

# =============================================================================
# Entrypoint
# =============================================================================
COMMAND="${1:-help}"
shift || true

case "${COMMAND}" in
  test)         cmd_test ;;
  build-images) cmd_build_images "$@" ;;
  push-images)  cmd_push_images "$@" ;;
  docs-serve)   cmd_docs_serve ;;
  docs-deploy)  cmd_docs_deploy "$@" ;;
  test-visual)  cmd_test_visual ;;
  record-docs)  cmd_record_docs ;;
  release)      cmd_release "$@" ;;
  start)        cmd_start ;;
  stop)         cmd_stop ;;
  attach)       cmd_attach ;;
  status)       cmd_status ;;
  help|--help|-h) usage ;;
  *) die "Unknown command: '${COMMAND}'. Run './scripts/maintain.sh help' for usage." ;;
esac
