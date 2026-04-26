---
name: shell-scripting
description: |
  Bash scripting — strict mode, quoting, arg parsing, traps, shellcheck compliance. Use when writing or reviewing bash scripts, adding error handling and argument parsing, or fixing shellcheck warnings.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-shell-scripting
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Shell Scripting

## Intro

Every bash script starts with `#!/usr/bin/env bash` and
`set -euo pipefail`, quotes every variable expansion, traps for
cleanup, and passes `shellcheck -o all` cleanly. Anything less is
a bug in waiting.

## Overview

### Script header

```bash
#!/usr/bin/env bash
set -euo pipefail
```

- `set -e` — exit on any command failure
- `set -u` — error on unset variables
- `set -o pipefail` — propagate failures through pipes

Use `#!/usr/bin/env bash` rather than `#!/bin/bash` for
portability across macOS and Linux.

### Variable quoting

Always double-quote variable expansions to prevent word splitting
and glob expansion:

```bash
# Correct
cp "$source" "$dest"
for file in "$dir"/*; do
  echo "Processing: $file"
done

# Wrong — breaks on paths with spaces
cp $source $dest
```

Use `${var:-default}` for defaults and `${var:?error message}`
for required variables:

```bash
config_file="${1:?Usage: $0 <config-file>}"
log_dir="${LOG_DIR:-/var/log/myapp}"
```

### Argument parsing

For simple positional arguments use `${1:?...}`. For options use
`getopts`:

```bash
verbose=false
output=""
while getopts ":vo:h" opt; do
  case $opt in
    v) verbose=true ;;
    o) output="$OPTARG" ;;
    h) usage; exit 0 ;;
    :) echo "Option -$OPTARG requires an argument" >&2; exit 1 ;;
    \?) echo "Unknown option: -$OPTARG" >&2; exit 1 ;;
  esac
done
shift $((OPTIND - 1))
```

### Functions

Define functions with `name()`. Use `local` for every function
variable. Return data via stdout (captured with `$()`), not via
return codes — codes are limited to 0–255 and conflate errors
with values.

```bash
log() {
  local level="$1"
  shift
  printf '[%s] [%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$level" "$*" >&2
}

die() { log "ERROR" "$@"; exit 1; }
```

### Cleanup with trap

```bash
cleanup() { rm -rf "${tmpdir:-}"; }
trap cleanup EXIT

tmpdir="$(mktemp -d)"
```

`EXIT` fires on normal exit, errors (via `set -e`), and signals.
Add `trap 'echo Interrupted >&2; exit 130' INT TERM` when you
need distinct handling for user interrupts.

### Arrays

Use arrays for lists of items — never a space-separated string:

```bash
files=()
while IFS= read -r -d '' file; do
  files+=("$file")
done < <(find /path -name "*.txt" -print0)

for file in "${files[@]}"; do
  process "$file"
done
echo "Found ${#files[@]} files"
```

### Shellcheck

Run `shellcheck -o all script.sh` on every script before
committing. When a warning is a genuine false positive, disable
it locally with a comment, never globally:

```bash
# shellcheck disable=SC2034  # variable used in sourced file
unused_var="value"
```

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Missing `set -euo pipefail` at the top of every script.** Without `set -e`, a command that fails silently allows the script to continue and produce incorrect results or corrupt state. Without `set -u`, an unset variable expands to an empty string, which can turn `rm -rf "${dir}/"` into `rm -rf "/"`. Without `set -o pipefail`, a failed command in a pipe is masked by the exit code of the last command. These three flags are non-optional.
- **Unquoted variable expansions.** `cp $source $dest` splits on whitespace and expands globs — a path with spaces becomes multiple arguments, and a path containing `*` expands to matching files. Every variable expansion must be double-quoted: `cp "$source" "$dest"`. The only exception is when word-splitting is explicitly desired, which is rare.
- **Parsing `ls` output.** `ls` output is formatted for humans — file names can contain spaces, newlines, and control characters that break any parsing approach. Use `for f in /path/*` for simple iteration, `find -print0` piped to `read -d ''` for robust handling of arbitrary filenames, or `find -exec` to run a command on each result.
- **Using `[ ]` instead of `[[ ]]` for conditionals.** Single brackets are POSIX `test` — they require quoting of every variable, don't support `&&` and `||` inside the brackets, and treat `<` as file redirection. Double brackets `[[ ]]` are a bash built-in that handles unquoted variables safely, supports pattern matching with `=~`, and behaves predictably. Use `[[ ]]` in bash scripts.
- **Leaving temporary files without a cleanup trap.** A script that creates a temporary directory with `mktemp -d` and exits early due to an error will leave that directory on disk forever. Set up `trap cleanup EXIT` immediately after creating any temporary resource so cleanup runs on both normal exit and error exit.
- **Returning data via exit codes.** Bash exit codes are 0–255. Using `return 42` to pass a numeric result and `$?` to read it is limited, conflates success/failure with data values, and does not work for strings. Return data from functions by printing to stdout and capturing with `$(function_call)`. Use the return code only for success (0) or failure (non-zero).
- **Not running shellcheck before committing.** Shell scripts contain an enormous variety of subtle bugs — quoting issues, portability problems, unused variables, and incorrect conditionals — that are invisible to casual reading but immediately caught by `shellcheck`. Run `shellcheck -o all script.sh` on every script as part of the commit process or CI pipeline.

## Full reference

### Common pitfalls

- Never parse `ls` output — use globs (`for f in *.txt`) or
  `find -print0` with `read -d ''`.
- Use `[[ ]]` instead of `[ ]` — safer, supports regex and
  pattern matching.
- Use `$(command)` instead of backticks.
- Always use `"$@"` (quoted) to forward arguments, never `$*`.
- Check for commands with `command -v cmd`, not `which cmd`.
- Use `printf` instead of `echo` for portable output (`echo`
  behavior varies across shells).

### Structured logging with levels

```bash
readonly LOG_LEVEL="${LOG_LEVEL:-INFO}"

log() {
  local level="$1"; shift
  local -A levels=([DEBUG]=0 [INFO]=1 [WARN]=2 [ERROR]=3)
  local threshold="${levels[$LOG_LEVEL]:-1}"
  local current="${levels[$level]:-1}"
  (( current >= threshold )) || return 0
  printf '[%s] [%-5s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$level" "$*" >&2
}
```

### Lock file (prevent concurrent runs)

```bash
readonly LOCKFILE="/var/lock/$(basename "$0").lock"

acquire_lock() {
  if ! mkdir "$LOCKFILE" 2>/dev/null; then
    local pid
    pid="$(cat "$LOCKFILE/pid" 2>/dev/null || echo unknown)"
    echo "Already running (PID: $pid)" >&2
    exit 1
  fi
  echo $$ > "$LOCKFILE/pid"
  trap 'rm -rf "$LOCKFILE"' EXIT
}
```

`mkdir` is atomic, which works reliably on NFS where `flock`
sometimes does not.

### Retry with exponential backoff

```bash
retry() {
  local max_attempts="$1" delay="$2"
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

retry 5 2 curl -sSf "https://api.example.com/health"
```

### Safe file writing (atomic)

```bash
safe_write() {
  local target="$1"
  local tmpfile
  tmpfile="$(mktemp "${target}.tmp.XXXXXXXXXX")"
  cat > "$tmpfile"
  mv "$tmpfile" "$target"
}

generate_config | safe_write /etc/myapp/config.toml
```

### Command existence check

```bash
require_commands() {
  local missing=()
  for cmd in "$@"; do
    command -v "$cmd" &>/dev/null || missing+=("$cmd")
  done
  if (( ${#missing[@]} > 0 )); then
    echo "Missing required commands: ${missing[*]}" >&2
    return 1
  fi
}

require_commands curl jq git docker
```

### Script self-identification

```bash
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"

source "$SCRIPT_DIR/lib/utils.sh"
config="$SCRIPT_DIR/../config/defaults.toml"
```

### Color output with NO_COLOR support

```bash
if [[ -t 2 ]] && [[ -z "${NO_COLOR:-}" ]]; then
  readonly RED=$'\033[0;31m' GREEN=$'\033[0;32m' RESET=$'\033[0m'
else
  readonly RED="" GREEN="" RESET=""
fi
info()  { echo "${GREEN}[INFO]${RESET} $*" >&2; }
error() { echo "${RED}[ERROR]${RESET} $*" >&2; }
```

Respects the `NO_COLOR` convention (<https://no-color.org/>) and
detects non-TTY output.

### Here-documents

```bash
# With variable expansion
cat << EOF
Hello, $USER. Today is $(date).
EOF

# Without expansion (quote the delimiter)
cat << 'EOF'
This $variable is not expanded.
EOF
```

### Further reading

- `references/bash-patterns.md` — full reusable pattern library.
