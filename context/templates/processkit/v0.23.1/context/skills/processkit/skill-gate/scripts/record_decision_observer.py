"""
record_decision_observer.py — Rail-5 PostToolUse observer for record_decision.

WorkItem: FEAT-20260415_1700-QuietLedger-rail5-auto-decision-capture-implementation
Hooks guide: https://code.claude.com/docs/en/hooks

Purpose
-------
When `record_decision` (or the qualified MCP name) is invoked, write a
per-session marker file so that the Lever-1 gate
(`check_decision_captured.py`) knows a record_decision call was made in
this session.

This script NEVER blocks.  It exits 0 in all cases.  It is purely
observational — a side-channel that feeds the session-scoped dedup cache.

stdin
-----
Reads a Claude Code PostToolUse JSON object:
  session_id   str  — harness-provided stable session ID
  tool_name    str  — name of the tool that just finished
  tool_input   obj  — tool arguments (not used; present for completeness)
  cwd          str  — working directory

Marker file written
-------------------
  context/.state/skill-gate/session-<SESSION_ID>.decision-observed

  Content: JSON with timestamp.
      {"observed_at": "<ISO-8601 UTC>"}
"""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

# Tool names that count as a "record_decision" call.
_RECORD_DECISION_NAMES = frozenset(
    {
        "record_decision",
        "processkit-decision-record__record_decision",
    }
)


def _resolve_session_id(hook_input: dict) -> str:
    sid = hook_input.get("session_id")
    if sid:
        return str(sid)
    sid = os.environ.get("PROCESSKIT_SESSION_ID")
    if sid:
        return sid
    return str(os.getpid())


def _find_state_dir(cwd: str) -> Path:
    """Walk up from cwd to find context/.state/skill-gate/."""
    candidate = Path(cwd).resolve()
    while True:
        if (candidate / "context").is_dir():
            return candidate / "context" / ".state" / "skill-gate"
        parent = candidate.parent
        if parent == candidate:
            break
        candidate = parent
    return Path(cwd).resolve() / "context" / ".state" / "skill-gate"


def main() -> int:
    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        # Bad input — log and exit 0; never block PostToolUse.
        print(f"record_decision_observer: could not parse stdin: {exc}", file=sys.stderr)
        return 0

    tool_name: str = hook_input.get("tool_name", "")

    # Check if it's a record_decision call (by full name or bare suffix).
    bare = tool_name.split("__")[-1] if "__" in tool_name else tool_name
    if tool_name not in _RECORD_DECISION_NAMES and bare not in _RECORD_DECISION_NAMES:
        return 0  # Not a record_decision call — nothing to do.

    session_id = _resolve_session_id(hook_input)
    cwd: str = hook_input.get("cwd", os.getcwd())
    state_dir = _find_state_dir(cwd)

    try:
        state_dir.mkdir(parents=True, exist_ok=True)
        marker = state_dir / f"session-{session_id}.decision-observed"
        now = datetime.now(timezone.utc)
        marker.write_text(
            json.dumps({"observed_at": now.isoformat()}, indent=2) + "\n",
            encoding="utf-8",
        )
    except OSError as exc:
        # Log the error but never block.
        print(f"record_decision_observer: could not write marker: {exc}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
