#!/usr/bin/env bash
# =============================================================================
# Mock podman binary for E2E testing
# =============================================================================
# Logs all invocations to $MOCK_LOG_FILE and returns canned responses.
# Place on PATH before the real podman binary to intercept calls.
#
# Environment variables:
#   MOCK_LOG_FILE          — Path to log file (required)
#   MOCK_FAIL_COMMAND      — If set, fail when this subcommand is invoked
#   MOCK_CONTAINER_STATE   — State to return for status inspect (default: "running")
#   MOCK_CONTAINER_VERSION — Value to return for aibox.version label inspect (default: "")
# =============================================================================

set -euo pipefail

LOG="${MOCK_LOG_FILE:?MOCK_LOG_FILE must be set}"
FAIL_CMD="${MOCK_FAIL_COMMAND:-}"
STATE="${MOCK_CONTAINER_STATE:-running}"

# Log the full invocation
echo "podman $*" >> "$LOG"

# Check if this command should fail
if [[ -n "$FAIL_CMD" ]]; then
    JOINED="$*"
    if [[ "$JOINED" == *"$FAIL_CMD"* ]]; then
        echo "mock-podman: simulated failure for '$FAIL_CMD'" >&2
        exit 1
    fi
fi

# Route by subcommand
case "${1:-}" in
    info)
        echo '{"version":{"Version":"mock-5.0.0"}}'
        exit 0
        ;;
    inspect)
        shift
        # Parse --format <value> properly so we can route by format string.
        FORMAT=""
        while [[ $# -gt 0 ]]; do
            case "${1:-}" in
                --format)
                    FORMAT="${2:-}"
                    shift 2
                    ;;
                --*)
                    shift
                    ;;
                *)
                    # Container name — mock ignores it (responds to all names)
                    shift
                    ;;
            esac
        done

        if [[ "$STATE" == "missing" ]]; then
            echo "Error: no such container unknown" >&2
            exit 1
        fi

        # Route by format: label queries return MOCK_CONTAINER_VERSION;
        # state queries return MOCK_CONTAINER_STATE.
        if [[ "$FORMAT" == *"Labels"* ]]; then
            echo "${MOCK_CONTAINER_VERSION:-}"
        else
            echo "$STATE"
        fi
        exit 0
        ;;
    compose)
        shift
        case "${1:-}" in
            build)
                echo "mock-podman: compose build completed"
                exit 0
                ;;
            up)
                echo "mock-podman: compose up completed"
                exit 0
                ;;
            stop)
                echo "mock-podman: compose stop completed"
                exit 0
                ;;
            down)
                echo "mock-podman: compose down completed"
                exit 0
                ;;
            *)
                echo "mock-podman: compose $* (unhandled)" >&2
                exit 0
                ;;
        esac
        ;;
    exec)
        echo "mock-podman: exec (noop)"
        exit 0
        ;;
    *)
        echo "mock-podman: $* (unhandled)" >&2
        exit 0
        ;;
esac
