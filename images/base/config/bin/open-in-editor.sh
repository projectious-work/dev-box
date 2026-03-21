#!/bin/bash
# open-in-editor.sh — Open a file in the adjacent vim pane from yazi.
#
# Uses zellij actions to focus the editor pane and send :e <file>.
# Moves focus right (dev layout) or down (cowork layout) to find vim.
# After opening, returns focus to yazi.

file="$1"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || echo "$file")"

# Remember starting pane by moving to editor, sending command, moving back.
# Try right first (dev layout), fall back to down (cowork layout).
# We use a subshell trick: try right, send a test, if it works we're done.

# In dev layout: vim is RIGHT of yazi
# In cowork layout: vim is DOWN from yazi
# Strategy: move right, send the command, move back left.
# If right didn't reach vim (cowork), the :e goes to yazi — harmless (yazi ignores it).
# Then try down, send again, move back up.

zellij action move-focus right
zellij action write 27
sleep 0.03
zellij action write-chars ":e ${file}"
zellij action write 13
zellij action move-focus left

zellij action move-focus down
zellij action write 27
sleep 0.03
zellij action write-chars ":e ${file}"
zellij action write 13
zellij action move-focus up
