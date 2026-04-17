#!/usr/bin/env bash
# scripts/aibox-upgrade-test.sh — host-side driver that exercises aibox
# cross-version upgrade paths and captures the outputs an AI reviewer
# needs to verify the fix for multi-version migration gaps.
#
# Runs against whatever `aibox` is on your $PATH. Writes everything under
# tmp/aibox-upgrade-test/ (gitignored). Safe to re-run; `clean` wipes state.
#
# Usage:
#   scripts/aibox-upgrade-test.sh run              # one-shot into run-<ts>/
#   scripts/aibox-upgrade-test.sh watch [SECONDS]  # loop: run + prune old runs (default 900s)
#   scripts/aibox-upgrade-test.sh clean            # rm -rf tmp/aibox-upgrade-test/
#   scripts/aibox-upgrade-test.sh status           # list recent runs
#
# Environment (pins the baseline; defaults target cross-minor jumps):
#   FROM_VERSION=v0.17.10   FROM_PK_VERSION=v0.13.0
#   RETENTION_MINUTES=60    (watch mode: delete runs older than this)

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_BASE="${ROOT_DIR}/tmp/aibox-upgrade-test"

FROM_VERSION="${FROM_VERSION:-v0.17.10}"
FROM_PK_VERSION="${FROM_PK_VERSION:-v0.13.0}"
RETENTION_MINUTES="${RETENTION_MINUTES:-60}"

log()  { printf '[%s] %s\n' "$(date +'%Y-%m-%d %H:%M:%S')" "$*"; }
step() { printf '\n=== %s ===\n' "$*"; }

usage() {
  sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
}

require_aibox() {
  if ! command -v aibox >/dev/null 2>&1; then
    echo "ERROR: aibox not on PATH — install it first" >&2
    exit 1
  fi
}

rewrite_version_pin() {
  local toml="$1" new_version="$2" new_pk_version="${3:-}"
  python3 - "$toml" "$new_version" "$new_pk_version" <<'PYINNER'
import re, sys, pathlib
toml, new_v, new_pk = sys.argv[1], sys.argv[2], sys.argv[3]
s = pathlib.Path(toml).read_text()
s = re.sub(r'(?m)^(version\s*=\s*)"[^"]*"', lambda m: f'{m.group(1)}"{new_v}"', s, count=1)
if new_pk:
    s = re.sub(
        r'(\[processkit\][^\[]*?\n\s*version\s*=\s*)"[^"]*"',
        lambda m: f'{m.group(1)}"{new_pk}"',
        s, count=1, flags=re.MULTILINE,
    )
pathlib.Path(toml).write_text(s)
PYINNER
}

capture_state() {
  local project_dir="$1" out_dir="$2" label="$3"
  mkdir -p "$out_dir"
  cp "$project_dir/aibox.toml" "$out_dir/aibox.toml.${label}" 2>/dev/null || true
  cp "$project_dir/aibox.lock" "$out_dir/aibox.lock.${label}" 2>/dev/null || true
  if [[ -d "$project_dir/context/migrations" ]]; then
    cp -r "$project_dir/context/migrations" "$out_dir/migrations.${label}" 2>/dev/null || true
  fi
  if [[ -d "$project_dir/context/templates/aibox-home" ]]; then
    ls -1 "$project_dir/context/templates/aibox-home" > "$out_dir/aibox-home-snapshots.${label}.txt" 2>/dev/null || true
  fi
  if [[ -f "$project_dir/.mcp.json" ]]; then
    cp "$project_dir/.mcp.json" "$out_dir/mcp.json.${label}"
  else
    echo "MISSING (tracks aibox#53)" > "$out_dir/mcp.json.${label}.absent"
  fi
  if [[ -d "$project_dir/context/skills" ]]; then
    find "$project_dir/context/skills" -name mcp-config.json \
      > "$out_dir/per-skill-mcp-configs.${label}.txt" 2>/dev/null || true
  fi
}

do_run() {
  require_aibox
  local ts run_dir project_dir log_file
  ts="$(date +%Y%m%d_%H%M%S)"
  run_dir="${TMP_BASE}/run-${ts}"
  project_dir="${run_dir}/project"
  log_file="${run_dir}/log.txt"
  mkdir -p "$project_dir" "$run_dir/captured"

  exec > >(tee -a "$log_file") 2>&1

  log "run started: $run_dir"
  step "environment"
  aibox --version || true
  uname -a
  which aibox

  step "step 1 — aibox init (fresh project at $project_dir)"
  ( cd "$project_dir" && aibox init --name canary ) || log "WARN: aibox init failed"
  ls -la "$project_dir"

  step "step 2 — pin baseline: aibox=${FROM_VERSION}, processkit=${FROM_PK_VERSION}"
  if [[ -f "$project_dir/aibox.toml" ]]; then
    rewrite_version_pin "$project_dir/aibox.toml" "$FROM_VERSION" "$FROM_PK_VERSION"
    grep -E '^(version|\[)' "$project_dir/aibox.toml" | head -20
  else
    log "ERROR: aibox.toml missing after init — aborting run"
    return
  fi

  step "step 3 — aibox sync at baseline ${FROM_VERSION}"
  ( cd "$project_dir" && aibox sync ) || log "WARN: baseline sync returned nonzero"
  capture_state "$project_dir" "$run_dir/captured" "after-baseline"

  step "step 4 — flip [aibox].version to \"latest\" and re-sync (cross-version upgrade)"
  rewrite_version_pin "$project_dir/aibox.toml" "latest"
  ( cd "$project_dir" && aibox sync ) || log "WARN: upgrade sync returned nonzero"
  capture_state "$project_dir" "$run_dir/captured" "after-upgrade"

  step "step 5 — inspect generated migration docs"
  if [[ -d "$project_dir/context/migrations" ]]; then
    ls -la "$project_dir/context/migrations"
    for f in "$project_dir/context/migrations"/*-to-*.md; do
      [[ -e "$f" ]] || continue
      echo "--- $(basename "$f") ---"
      sed -n '1,80p' "$f"
    done
  fi

  step "step 6 — intermediate template snapshots present"
  if [[ -d "$project_dir/context/templates/aibox-home" ]]; then
    ls -1 "$project_dir/context/templates/aibox-home" || true
  fi

  step "step 7 — MCP wiring check (aibox#53)"
  if [[ -f "$project_dir/.mcp.json" ]]; then
    log "OK: .mcp.json exists"
    wc -l "$project_dir/.mcp.json"
  else
    log "MISS: .mcp.json absent — aibox#53 reproduces"
  fi
  local n
  n=$(find "$project_dir/context/skills" -name mcp-config.json 2>/dev/null | wc -l | tr -d ' ')
  echo "per-skill mcp-config.json files present: $n"

  log "run complete: $run_dir"
}

do_watch() {
  local interval="${1:-900}"
  log "watch: interval=${interval}s, retention=${RETENTION_MINUTES}m"
  trap 'log "watch: interrupted"; exit 0' INT TERM
  while :; do
    do_run || log "iteration failed, continuing"
    if [[ -d "$TMP_BASE" ]]; then
      find "$TMP_BASE" -maxdepth 1 -type d -name 'run-*' \
        -mmin +"$RETENTION_MINUTES" -exec rm -rf {} + 2>/dev/null || true
    fi
    log "watch: sleeping ${interval}s"
    sleep "$interval"
  done
}

do_clean() {
  if [[ -d "$TMP_BASE" ]]; then
    rm -rf "$TMP_BASE"
    echo "removed $TMP_BASE"
  else
    echo "nothing to clean at $TMP_BASE"
  fi
}

do_status() {
  if [[ ! -d "$TMP_BASE" ]]; then
    echo "no runs — base dir absent: $TMP_BASE"
    return
  fi
  local count
  count=$(find "$TMP_BASE" -maxdepth 1 -type d -name 'run-*' | wc -l | tr -d ' ')
  echo "runs at $TMP_BASE: $count"
  find "$TMP_BASE" -maxdepth 1 -type d -name 'run-*' 2>/dev/null \
    | sort -r | head -20
}

case "${1:-}" in
  run)    do_run ;;
  watch)  shift; do_watch "${1:-900}" ;;
  clean)  do_clean ;;
  status) do_status ;;
  -h|--help|help|"") usage ;;
  *) usage; exit 1 ;;
esac
