#!/usr/bin/env bash
# =============================================================================
# deploy-docs.sh — Build MkDocs site locally and push to gh-pages branch
#
# Usage:
#   ./scripts/deploy-docs.sh [--dry-run]
#
# This script:
#   1. Builds the MkDocs site into site/
#   2. Creates an orphan gh-pages branch (or updates existing)
#   3. Pushes the built HTML to the gh-pages branch on origin
#
# Prerequisites:
#   - mkdocs-material installed (pip install mkdocs-material)
#   - git configured with push access to origin
#   - gh CLI authenticated (for GitHub Pages setup)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Colors ───────────────────────────────────────────────────────────────────
bold=$'\e[1m'
cyan=$'\e[36m'
green=$'\e[32m'
yellow=$'\e[33m'
red=$'\e[31m'
reset=$'\e[0m'

info()  { echo "${cyan}${bold}==>${reset} $*"; }
ok()    { echo "${green}${bold} ✓${reset} $*"; }
warn()  { echo "${yellow}${bold}  !${reset} $*"; }
die()   { echo "${red}${bold}ERR${reset} $*" >&2; exit 1; }

# ── Parse args ───────────────────────────────────────────────────────────────
DRY_RUN=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --help|-h) echo "Usage: $0 [--dry-run]"; exit 0 ;;
        *) die "Unknown option: $1" ;;
    esac
done

cd "${PROJECT_ROOT}"

# ── Preflight checks ────────────────────────────────────────────────────────
command -v mkdocs &>/dev/null || die "mkdocs not found. Install: pip install mkdocs-material"
command -v git &>/dev/null    || die "git not found"

# Ensure we're in a git repo
git rev-parse --is-inside-work-tree &>/dev/null || die "Not inside a git repository"

# Get remote URL and current branch
REMOTE_URL=$(git remote get-url origin 2>/dev/null) || die "No 'origin' remote configured"
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
COMMIT_SHA=$(git rev-parse --short HEAD)
COMMIT_MSG="docs: deploy from ${CURRENT_BRANCH}@${COMMIT_SHA} ($(date -u +%Y-%m-%dT%H:%M:%SZ))"

# Extract owner/repo slug from remote URL (used for GitHub Pages setup and final URL)
REPO_SLUG=$(echo "${REMOTE_URL}" | sed -E 's|.*[:/]([^/]+/[^/]+?)(\.git)?$|\1|')

info "Remote: ${REMOTE_URL}"
info "Source: ${CURRENT_BRANCH}@${COMMIT_SHA}"

# ── Step 1: Build the site ──────────────────────────────────────────────────
info "Building MkDocs site..."
mkdocs build --strict --clean
ok "Site built in site/"

if [[ "${DRY_RUN}" == "true" ]]; then
    warn "Dry run — skipping git push. Site is in site/"
    exit 0
fi

# ── Step 2: Prepare gh-pages content in a temp directory ─────────────────────
TMPDIR=$(mktemp -d)
trap 'rm -rf "${TMPDIR}"' EXIT

info "Preparing gh-pages content..."

# Copy built site to temp dir
cp -r site/* "${TMPDIR}/"

# Add .nojekyll to prevent GitHub from processing with Jekyll
touch "${TMPDIR}/.nojekyll"

# ── Step 3: Push to gh-pages branch ─────────────────────────────────────────
info "Pushing to gh-pages branch..."

cd "${TMPDIR}"
git init -q
git checkout -q -b gh-pages
git add -A
git commit -q -m "${COMMIT_MSG}"

# Force push to the gh-pages branch on origin
# This is safe because gh-pages is a deployment branch with only generated content
git push --force "${REMOTE_URL}" gh-pages:gh-pages

cd "${PROJECT_ROOT}"

ok "Deployed to gh-pages branch"

# ── Step 4: Ensure GitHub Pages is configured ────────────────────────────────
if command -v gh &>/dev/null; then
    if [[ -n "${REPO_SLUG}" ]]; then
        info "Configuring GitHub Pages for ${REPO_SLUG}..."

        # Enable Pages from gh-pages branch (idempotent)
        gh api --method PUT "repos/${REPO_SLUG}/pages" \
            -f "source[branch]=gh-pages" \
            -f "source[path]=/" \
            --silent 2>/dev/null && ok "GitHub Pages configured (source: gh-pages)" \
            || warn "Could not configure Pages automatically. Enable manually: Settings → Pages → Source: gh-pages"
    fi
else
    warn "gh CLI not available — configure GitHub Pages manually: Settings → Pages → Source: gh-pages"
fi

echo ""
ok "Documentation deployed."
if [[ -n "${REPO_SLUG}" ]]; then
    info "URL: https://${REPO_SLUG/\//.github.io\/}/"
fi
