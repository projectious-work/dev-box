# Bash Patterns

Reusable patterns for common scripting tasks. Copy and adapt these into your scripts.

## Logging Function

Structured logging with timestamps and severity levels:

```bash
readonly LOG_LEVEL="${LOG_LEVEL:-INFO}"

log() {
  local level="$1"
  shift
  local -A levels=([DEBUG]=0 [INFO]=1 [WARN]=2 [ERROR]=3)
  local threshold="${levels[$LOG_LEVEL]:-1}"
  local current="${levels[$level]:-1}"
  (( current >= threshold )) || return 0
  printf '[%s] [%-5s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$level" "$*" >&2
}

log INFO "Starting process"
log DEBUG "Variable x=$x"
log ERROR "Something went wrong"
```

## Temp File Cleanup

Guaranteed cleanup with trap, even on errors or interrupts:

```bash
tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/myapp.XXXXXXXXXX")"
trap 'rm -rf "$tmpdir"' EXIT

# Use $tmpdir for all temporary files
download="$tmpdir/download.tar.gz"
workdir="$tmpdir/work"
mkdir -p "$workdir"
```

## Lock File (Prevent Concurrent Runs)

Ensure only one instance of a script runs at a time:

```bash
readonly LOCKFILE="/var/lock/$(basename "$0").lock"

acquire_lock() {
  if ! mkdir "$LOCKFILE" 2>/dev/null; then
    local pid
    pid="$(cat "$LOCKFILE/pid" 2>/dev/null || echo "unknown")"
    echo "Already running (PID: $pid)" >&2
    exit 1
  fi
  echo $$ > "$LOCKFILE/pid"
  trap 'rm -rf "$LOCKFILE"' EXIT
}

acquire_lock
```

Uses `mkdir` for atomic lock creation (works on NFS, unlike flock in some cases).

## Retry Loop

Retry a command with exponential backoff:

```bash
retry() {
  local max_attempts="${1}"
  local delay="${2}"
  shift 2
  local attempt=1

  until "$@"; do
    if (( attempt >= max_attempts )); then
      echo "Failed after $attempt attempts: $*" >&2
      return 1
    fi
    echo "Attempt $attempt failed, retrying in ${delay}s..." >&2
    sleep "$delay"
    (( attempt++ ))
    (( delay *= 2 ))
  done
}

# Usage: retry <max_attempts> <initial_delay_seconds> <command...>
retry 5 2 curl -sSf "https://api.example.com/health"
```

## Color Output

Colors for terminal output with automatic detection:

```bash
if [[ -t 2 ]] && [[ "${NO_COLOR:-}" == "" ]]; then
  readonly RED=$'\033[0;31m'
  readonly GREEN=$'\033[0;32m'
  readonly YELLOW=$'\033[0;33m'
  readonly BLUE=$'\033[0;34m'
  readonly BOLD=$'\033[1m'
  readonly RESET=$'\033[0m'
else
  readonly RED="" GREEN="" YELLOW="" BLUE="" BOLD="" RESET=""
fi

info()  { echo "${GREEN}[INFO]${RESET} $*" >&2; }
warn()  { echo "${YELLOW}[WARN]${RESET} $*" >&2; }
error() { echo "${RED}[ERROR]${RESET} $*" >&2; }
```

Respects the `NO_COLOR` convention (https://no-color.org/) and detects non-TTY environments (piped output).

## Progress Indicator

Simple spinner for long-running operations:

```bash
spinner() {
  local pid="$1"
  local msg="${2:-Working...}"
  local chars='|/-\'
  local i=0

  while kill -0 "$pid" 2>/dev/null; do
    printf '\r%s %s' "${chars:i%4:1}" "$msg" >&2
    (( i++ ))
    sleep 0.2
  done
  printf '\r%s\n' "$msg done." >&2
  wait "$pid"
}

# Usage
long_running_command &
spinner $! "Processing files"
```

## Confirmation Prompt

Ask for user confirmation before destructive operations:

```bash
confirm() {
  local msg="${1:-Are you sure?}"
  if [[ "${FORCE:-}" == "true" ]]; then
    return 0
  fi
  read -rp "$msg [y/N] " response
  [[ "$response" =~ ^[Yy]$ ]]
}

# Usage
confirm "Delete all logs?" || exit 0
rm -rf /var/log/myapp/*
```

Supports a `FORCE=true` environment variable to skip prompts in CI.

## Safe File Writing

Write to a temp file and atomically move to avoid partial writes:

```bash
safe_write() {
  local target="$1"
  local tmpfile
  tmpfile="$(mktemp "${target}.tmp.XXXXXXXXXX")"

  # Write to temp file (content from stdin)
  cat > "$tmpfile"

  # Atomic rename
  mv "$tmpfile" "$target"
}

# Usage
generate_config | safe_write /etc/myapp/config.toml
```

## Command Existence Check

Verify required tools are installed before proceeding:

```bash
require_commands() {
  local missing=()
  for cmd in "$@"; do
    if ! command -v "$cmd" &>/dev/null; then
      missing+=("$cmd")
    fi
  done
  if (( ${#missing[@]} > 0 )); then
    echo "Missing required commands: ${missing[*]}" >&2
    return 1
  fi
}

# Usage
require_commands curl jq git docker
```

## Script Self-Identification

Reliable path resolution for scripts that reference sibling files:

```bash
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"

# Reference sibling files reliably
source "$SCRIPT_DIR/lib/utils.sh"
config="$SCRIPT_DIR/../config/defaults.toml"
```
