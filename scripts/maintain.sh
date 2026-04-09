#!/usr/bin/env bash
# =============================================================================
# maintain.sh — maintenance script for the aibox project itself
#
# This manages the dev-container we develop IN (not the containers we publish).
# For downstream project container management, use the aibox CLI.
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
HOST_ROOT="${HOST_ROOT:-${PROJECT_ROOT}/.aibox-home}"
WORKSPACE_DIR="${WORKSPACE_DIR:-${PROJECT_ROOT}}"
CLI_DIR="${PROJECT_ROOT}/cli"
DIST_DIR="${PROJECT_ROOT}/dist"
IMAGE_REGISTRY="ghcr.io/projectious-work/aibox"

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
${bold}maintain.sh${reset} — aibox project maintenance

${bold}Usage:${reset}
  ./scripts/maintain.sh <command> [options]

${bold}Development:${reset}
  test                     Run cargo fmt check, clippy, and tests
  build-images [--no-cache] Build published container images locally
  push-images <version>    Push images to GHCR (requires ghcr.io login)
  docs-serve               Serve MkDocs locally (http://localhost:8000)
  docs-deploy [--dry-run]  Build MkDocs and push to gh-pages branch
  test-visual              Run screencast smoke tests (~40s)
  record-docs              Regenerate all docs screencasts + README GIF

${bold}Release:${reset}
  sync-processkit          Check for new processkit release; patch constants + show diff
                           (runs automatically inside 'release'; also available standalone)
  release <version>        Sync processkit, test, tag, build CLI, generate release prompt
  release-host <version>   Build macOS binaries, upload to GH release,
                           build + push images to GHCR (run on macOS host)

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

  local flavors=("base-debian")

  for flavor in "${flavors[@]}"; do
    info "Building ${flavor} image..."
    ${RUNTIME_BIN} build ${no_cache} \
      -t "${IMAGE_REGISTRY}:${flavor}-latest" \
      -f "${PROJECT_ROOT}/images/${flavor}/Dockerfile" \
      "${PROJECT_ROOT}/images/${flavor}/"
    ok "Built ${IMAGE_REGISTRY}:${flavor}-latest"
  done

  ok "All images built"
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

  local flavors=("base-debian")

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
  ok "All ${#flavors[@]} image(s) pushed to ${IMAGE_REGISTRY}"
  info "Verify at: https://github.com/orgs/projectious-work/packages"
}

cmd_docs_serve() {
  cd "${PROJECT_ROOT}/docs-site"
  info "Serving docs with Docusaurus at http://localhost:3000 ..."
  npx docusaurus start --host 0.0.0.0
}

cmd_docs_deploy() {
  local dry_run=false
  [[ "${1:-}" == "--dry-run" ]] && dry_run=true

  command -v npx &>/dev/null    || die "npx not found. Install Node.js."
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

  cd "${PROJECT_ROOT}/docs-site"
  info "Building docs with Docusaurus..."
  npx docusaurus build
  ok "Site built in docs-site/build/"

  if [[ "${dry_run}" == "true" ]]; then
    warn "Dry run — site is in site/"
    return 0
  fi

  local tmpdir
  tmpdir=$(mktemp -d)
  trap 'rm -rf "${tmpdir}"' EXIT

  cp -r build/* "${tmpdir}/"
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

# =============================================================================
# cmd_sync_processkit — check for a new processkit release and pull it in
#
# Queries GitHub for the latest processkit tag, compares it with the
# PROCESSKIT_DEFAULT_VERSION constant in cli/src/processkit_vocab.rs, and if
# a newer version exists:
#
#   1. Patches PROCESSKIT_DEFAULT_VERSION in processkit_vocab.rs
#   2. Fetches the new FORMAT.md from the processkit repo and displays the
#      diff against the previous version so the maintainer can spot vocabulary
#      changes (new categories, renamed filenames, new directory segments, etc.)
#   3. Re-runs the vocabulary unit tests (they enforce count/no-duplicates on
#      CATEGORY_ORDER and will catch obvious drift immediately)
#
# Called automatically by cmd_release. Can also be run standalone:
#   ./scripts/maintain.sh sync-processkit
#
# After this runs, review the FORMAT.md diff and update processkit_vocab.rs
# manually if any vocabulary changed (CATEGORY_ORDER, src:: segments, filename
# constants). Then commit before running `release`.
# =============================================================================
cmd_sync_processkit() {
  command -v gh &>/dev/null || die "gh CLI required for processkit version check"

  info "Checking for processkit updates..."

  # ── Resolve latest upstream tag ───────────────────────────────────────────
  local latest_tag
  latest_tag=$(gh api repos/projectious-work/processkit/releases/latest --jq '.tag_name' 2>/dev/null) \
    || { warn "Could not reach GitHub API — skipping processkit update check"; return 0; }

  if [[ -z "${latest_tag}" ]]; then
    warn "No processkit releases found — skipping update check"
    return 0
  fi

  # ── Read the currently pinned version from processkit_vocab.rs ────────────
  local vocab_file="${CLI_DIR}/src/processkit_vocab.rs"
  local current_tag
  current_tag=$(grep 'pub const PROCESSKIT_DEFAULT_VERSION' "${vocab_file}" \
    | grep -oP '"v[^"]+"' | tr -d '"')

  info "processkit: current=${current_tag}  latest=${latest_tag}"

  if [[ "${current_tag}" == "${latest_tag}" ]]; then
    ok "processkit is already up to date (${current_tag})"
    return 0
  fi

  warn "New processkit version available: ${current_tag} → ${latest_tag}"

  # ── Fetch FORMAT.md for both versions for a vocabulary diff ───────────────
  local fmt_path="src/.processkit/FORMAT.md"
  local tmp_old tmp_new
  tmp_old=$(mktemp)
  tmp_new=$(mktemp)
  trap 'rm -f "${tmp_old}" "${tmp_new}"' RETURN

  _fetch_processkit_file() {
    local ref="$1" dest="$2"
    gh api "repos/projectious-work/processkit/contents/${fmt_path}?ref=${ref}" \
      --jq '.content' 2>/dev/null \
      | base64 -d > "${dest}" 2>/dev/null \
      || { warn "Could not fetch FORMAT.md for ${ref}"; touch "${dest}"; }
  }

  info "Fetching FORMAT.md for ${current_tag} and ${latest_tag}..."
  _fetch_processkit_file "${current_tag}" "${tmp_old}"
  _fetch_processkit_file "${latest_tag}"  "${tmp_new}"

  local diff_output
  diff_output=$(diff --unified=3 "${tmp_old}" "${tmp_new}" || true)

  if [[ -z "${diff_output}" ]]; then
    info "FORMAT.md is unchanged between ${current_tag} and ${latest_tag}"
    info "(Vocabulary constants in processkit_vocab.rs need no update)"
  else
    echo ""
    echo "${bold}FORMAT.md diff (${current_tag} → ${latest_tag}):${reset}"
    echo "${diff_output}"
    echo ""
    warn "Review the diff above. If any of these changed, update processkit_vocab.rs manually:"
    echo "  · CATEGORY_ORDER        (new/removed/reordered categories)"
    echo "  · processkit_vocab::src (new/renamed source-tree directory segments)"
    echo "  · *_FILENAME constants  (SKILL_FILENAME, PROVENANCE_FILENAME, INDEX_FILENAME, …)"
    echo ""
    warn "Press Enter to continue after reviewing, or Ctrl-C to abort and update first."
    read -r
  fi

  # ── Patch PROCESSKIT_DEFAULT_VERSION in processkit_vocab.rs ───────────────
  info "Patching PROCESSKIT_DEFAULT_VERSION: ${current_tag} → ${latest_tag}"
  sed -i "s|pub const PROCESSKIT_DEFAULT_VERSION: &str = \"${current_tag}\";|pub const PROCESSKIT_DEFAULT_VERSION: \&str = \"${latest_tag}\";|" \
    "${vocab_file}"
  ok "Patched ${vocab_file}"

  # ── Re-run vocabulary tests to catch obvious drift ────────────────────────
  info "Running processkit vocabulary tests..."
  (cd "${CLI_DIR}" && cargo test processkit_vocab 2>&1) \
    || die "Vocabulary tests failed after update — fix processkit_vocab.rs before releasing"
  ok "Vocabulary tests pass for ${latest_tag}"

  # ── Remind maintainer to commit ────────────────────────────────────────────
  echo ""
  warn "processkit_vocab.rs patched but not yet committed."
  warn "Review the diff above, make any additional vocabulary changes, then commit:"
  echo ""
  echo "  git add cli/src/processkit_vocab.rs"
  echo "  git commit -m \"chore: bump processkit default version to ${latest_tag}\""
  echo ""
  echo "Then re-run: ./scripts/maintain.sh release <version>"
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

  # ── Step 2: Sync processkit ───────────────────────────────────────────────
  # Check for a newer processkit release. Patches PROCESSKIT_DEFAULT_VERSION,
  # shows the FORMAT.md diff, and aborts if the tree is dirty — forcing any
  # required CLI changes to be made and committed before the build proceeds.
  cmd_sync_processkit
  if [[ -n "$(git status --porcelain)" ]]; then
    echo ""
    die "processkit_vocab.rs was updated. Review the diff, make any required CLI changes, commit, then re-run release."
  fi

  # ── Step 3: Run tests ──────────────────────────────────────────────────────
  info "Running tests..."
  cmd_test

  # ── Step 3: Audit dependencies ───────────────────────────────────────────
  info "Running cargo audit..."
  command -v cargo-audit &>/dev/null \
    || (cd "${CLI_DIR}" && cargo install cargo-audit --quiet)
  (cd "${CLI_DIR}" && cargo audit) \
    || die "cargo audit found advisories — resolve before releasing"
  ok "Audit clean"

  # ── Step 4: Build both Linux CLI targets ─────────────────────────────────
  info "Building CLI (release mode) for all Linux targets..."
  mkdir -p "${DIST_DIR}"

  local linux_targets=("aarch64-unknown-linux-gnu" "x86_64-unknown-linux-gnu")
  local built_archives=()

  for target in "${linux_targets[@]}"; do
    info "  → ${target}"
    (cd "${CLI_DIR}" && \
      CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
      cargo build --release --target "${target}") \
      || die "cargo build failed for ${target}"
    local binary_name="aibox-v${version}-${target}"
    cp "${CLI_DIR}/target/${target}/release/aibox" "${DIST_DIR}/${binary_name}"
    tar -czf "${DIST_DIR}/${binary_name}.tar.gz" -C "${DIST_DIR}" "${binary_name}"
    rm "${DIST_DIR}/${binary_name}"
    built_archives+=("${DIST_DIR}/${binary_name}.tar.gz")
    ok "Built ${binary_name}.tar.gz"
  done

  # ── Step 5: Create and push git tag ──────────────────────────────────────
  info "Tagging and pushing ${tag}..."
  git tag -a "${tag}" -m "Release ${tag}"
  git push origin "${tag}"
  ok "Tag ${tag} pushed"

  # ── Step 6: Create GitHub release with Linux binaries ────────────────────
  info "Creating GitHub release ${tag}..."
  local notes_file="${DIST_DIR}/RELEASE-NOTES.md"
  # Use hand-written RELEASE-NOTES.md if it exists and covers this version,
  # otherwise fall back to an auto-generated commit log.
  if [[ ! -f "${notes_file}" ]] || ! grep -q "${tag}" "${notes_file}" 2>/dev/null; then
    info "Writing auto-generated release notes..."
    local prev_tag
    prev_tag=$(git describe --tags --abbrev=0 "${tag}^" 2>/dev/null || echo "")
    {
      echo "# aibox ${tag}"
      echo ""
      if [[ -n "${prev_tag}" ]]; then
        echo "## Changes since ${prev_tag}"
        echo ""
        git log --oneline "${prev_tag}..${tag}" | sed 's/^/- /'
      else
        git log --oneline "${tag}" | head -20 | sed 's/^/- /'
      fi
    } > "${notes_file}"
  fi

  gh release create "${tag}" \
    --title "aibox ${tag}" \
    --notes-file "${notes_file}" \
    "${built_archives[@]}"
  ok "GitHub release ${tag} created with Linux binaries"

  # ── Step 7: Deploy documentation ─────────────────────────────────────────
  info "Deploying documentation..."
  cmd_docs_deploy
  ok "Documentation deployed"

  # ── Step 8: Generate host-side prompt ────────────────────────────────────
  # The macOS binaries and container image push must be done by the maintainer
  # on the macOS host (cross-compilation to Darwin is not possible from Linux;
  # container runtime is not available inside the devcontainer).
  local prompt_file="${DIST_DIR}/RELEASE-PROMPT.md"
  {
    echo "# Host-side steps for aibox ${tag}"
    echo ""
    echo "Linux binaries are already uploaded to the GitHub release."
    echo "Run the following on the macOS host to complete the release:"
    echo ""
    echo "\`\`\`bash"
    echo "./scripts/maintain.sh release-host ${version}"
    echo "\`\`\`"
    echo ""
    echo "This will:"
    echo "- Build macOS binaries (aarch64-apple-darwin, x86_64-apple-darwin)"
    echo "- Upload them to the existing GitHub release ${tag}"
    echo "- Build and push container images to GHCR"
  } > "${prompt_file}"

  ok "Host-side prompt written to dist/RELEASE-PROMPT.md"

  # ── Summary ──────────────────────────────────────────────────────────────
  echo ""
  echo "${bold}Release ${tag} complete (Linux side).${reset}"
  echo ""
  echo "  GitHub release: https://github.com/projectious-work/aibox/releases/tag/${tag}"
  echo "  Linux binaries uploaded:"
  for a in "${built_archives[@]}"; do
    echo "    $(basename "${a}")"
  done
  echo "  Documentation: deployed to gh-pages"
  echo ""
  echo "  ${bold}Remaining (macOS host):${reset} ./scripts/maintain.sh release-host ${version}"
}

# ── Host-side release (run on macOS after container-side `release`) ──────────

cmd_release_host() {
  local version="${1:-}"
  [[ -z "${version}" ]] && die "Usage: ./scripts/maintain.sh release-host <version>  (e.g. 0.10.2)"

  if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "Version must be semver: X.Y.Z (got: ${version})"
  fi

  local tag="v${version}"

  # ── Step 1: Build macOS binaries ──────────────────────────────────────────
  info "Building macOS binaries..."
  "${SCRIPT_DIR}/build-macos.sh" "${version}"

  # ── Step 2: Upload macOS binaries to existing GitHub release ──────────────
  info "Uploading macOS binaries to GitHub release ${tag}..."
  if ! gh release view "${tag}" &>/dev/null; then
    die "GitHub release ${tag} not found. Run 'release' in the container first."
  fi
  gh release upload "${tag}" "${DIST_DIR}"/aibox-v${version}-*-apple-darwin.tar.gz \
    || warn "Upload failed — binaries may already be attached"
  ok "macOS binaries uploaded to ${tag}"

  # ── Step 3: Build and push container images ───────────────────────────────
  info "Building container images..."
  cmd_build_images
  info "Pushing container images..."
  cmd_push_images "${version}"

  # ── Done ──────────────────────────────────────────────────────────────────
  echo ""
  ok "Release ${tag} host-side steps complete."
  echo ""
  echo "  macOS binaries: uploaded to GitHub release"
  echo "  Container images: pushed to GHCR"
  echo ""
  echo "  Note: docs deployment runs inside the dev-container"
  echo "  (requires Node.js/Docusaurus): ./scripts/maintain.sh docs-deploy"
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
  sync-processkit) cmd_sync_processkit ;;
  release)      cmd_release "$@" ;;
  release-host) cmd_release_host "$@" ;;
  start)        cmd_start ;;
  stop)         cmd_stop ;;
  attach)       cmd_attach ;;
  status)       cmd_status ;;
  help|--help|-h) usage ;;
  *) die "Unknown command: '${COMMAND}'. Run './scripts/maintain.sh help' for usage." ;;
esac
