#!/bin/bash

###########################################
# Usage: start-box.sh <image[:tag]> [podman args...] -- [container command]
#
# Examples:
#
# Default: starting with zellij
#       ./start-box.sh bnaard/dev-box:0.0.1
#
# Starting with bash
#       ./start-box.sh bnaard/dev-box:0.0.1 -- bash
#
# With extra volumes:
#       ./start-box.sh bnaard/dev-box:0.0.1 -v /tmp:/tmp -- bash
#
# Custom image and helix directly:
#
#       ./start-box.sh my/helix:latest -p 8080:8080 -- helix /workspace
#
###########################################


# Determine container runtime: podman preferred, docker fallback
if command -v podman >/dev/null 2>&1; then
    RUNTIME="podman"
elif command -v docker >/dev/null 2>&1; then
    RUNTIME="docker"
else
    echo "Error: Neither podman nor docker found" >&2
    exit 1
fi

# Build the container run command
# First arg is image[:tag], rest are podman args before --, then user command after
if [ $# -eq 0 ]; then
    echo "Usage: $0 <image> [podman args...] -- [container command]" >&2
    exit 1
fi

IMAGE="$1"
shift
PRE_ARGS=()
POST_ARGS=()

# Parse args: collect until --, rest goes after
while [ $# -gt 0 ]; do
    if [ "$1" = "--" ]; then
        shift
        break
    fi
    PRE_ARGS+=("$1")
    shift
done
POST_ARGS=("$@")

# Default run args
DEFAULT_ARGS=( "run" \
               "-it" \
               "--rm" \
               "-v" "$(pwd):/workspace" \
               "-v" "$(pwd)/config/helix:/root/.config/helix" \
               "-v" "$(pwd)/config/zellij:/root/.config/zellij" \
               "-v" "$(pwd)/config/opencode:/root/.config/opencode" \
               "-v" "$(pwd)/config/lazygit:/root/.config/lazygit" \
               "-e" "TERM=$TERM" \
            )
# Combine: runtime + defaults + pre-args + image + post-args
set -- "$RUNTIME" "${DEFAULT_ARGS[@]}" "${PRE_ARGS[@]}" "$IMAGE" "${POST_ARGS[@]}"

echo "Using $RUNTIME: $@"
exec "$@"
