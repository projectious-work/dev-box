#!/usr/bin/env bash
# =============================================================================
# Mock docker binary for E2E testing
# =============================================================================
# Logs all invocations to $MOCK_LOG_FILE and returns canned responses.
# Place on PATH before the real docker binary to intercept calls.
#
# Environment variables:
#   MOCK_LOG_FILE        — Path to log file (required)
#   MOCK_FAIL_COMMAND    — If set, fail when this subcommand is invoked
#   MOCK_CONTAINER_STATE — State to return for inspect (default: "running")
# =============================================================================

set -euo pipefail

LOG="${MOCK_LOG_FILE:?MOCK_LOG_FILE must be set}"
FAIL_CMD="${MOCK_FAIL_COMMAND:-}"
STATE="${MOCK_CONTAINER_STATE:-running}"

# Log the full invocation
echo "docker $*" >> "$LOG"

# Check if this command should fail
if [[ -n "$FAIL_CMD" ]]; then
    # Match the subcommand(s) — e.g. "compose build" or "info"
    JOINED="$*"
    if [[ "$JOINED" == *"$FAIL_CMD"* ]]; then
        echo "mock-docker: simulated failure for '$FAIL_CMD'" >&2
        exit 1
    fi
fi

# Route by subcommand
case "${1:-}" in
    info)
        echo '{"ServerVersion":"mock-27.0.0","Driver":"overlay2"}'
        exit 0
        ;;
    inspect)
        # Return container state
        shift
        # Skip flags
        while [[ "${1:-}" == --* ]]; do shift; done
        CONTAINER="${1:-unknown}"
        if [[ "$STATE" == "missing" ]]; then
            echo "Error: No such object: $CONTAINER" >&2
            exit 1
        fi
        echo "$STATE"
        exit 0
        ;;
    compose)
        shift
        case "${1:-}" in
            build)
                echo "mock-docker: compose build completed"
                exit 0
                ;;
            up)
                echo "mock-docker: compose up completed"
                exit 0
                ;;
            stop)
                echo "mock-docker: compose stop completed"
                exit 0
                ;;
            down)
                echo "mock-docker: compose down completed"
                exit 0
                ;;
            *)
                echo "mock-docker: compose $* (unhandled)" >&2
                exit 0
                ;;
        esac
        ;;
    exec)
        echo "mock-docker: exec (noop)"
        exit 0
        ;;
    *)
        echo "mock-docker: $* (unhandled)" >&2
        exit 0
        ;;
esac
