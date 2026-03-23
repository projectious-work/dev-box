#!/bin/bash
# vim-loop.sh — Run vim in a loop so the editor pane never dies.
# When vim exits (:q), it restarts with an empty buffer.
# Exit the loop with :cq (exit with error code) or Ctrl+C.

while true; do
    vim "$@"
    exit_code=$?
    # :cq exits with code 1 — use this to truly quit
    if [ "$exit_code" -ne 0 ]; then
        break
    fi
    # Normal :q — return focus to yazi pane, then restart vim
    dir="${DEVBOX_EDITOR_DIR:-right}"
    case "$dir" in
        down) zellij action move-focus up 2>/dev/null ;;
        tab)  zellij action go-to-tab-name "files" 2>/dev/null ;;
        *)    zellij action move-focus left 2>/dev/null ;;
    esac
done
