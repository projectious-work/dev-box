"""
check_route_task_called.py — processkit PreToolUse hook gate.

WorkItem: FEAT-20260414_1433-SteadyHand-provider-neutral-hook-scripts
Hooks guide: https://code.claude.com/docs/en/hooks-guide
Marker format: context/skills/processkit/skill-gate/mcp/SERVER.md §"Session marker file"

Purpose
-------
Block write-side processkit tools (and Write/Edit/MultiEdit under
context/**) until the agent has called acknowledge_contract() and a
valid marker file exists.

Validity rule (revised 2026-04-17)
----------------------------------
Earlier versions keyed the marker on a session_id resolved from hook
input (Claude Code's session UUID). The MCP server writes the marker
keyed on its own pid or PROCESSKIT_SESSION_ID env — the MCP protocol
does NOT propagate the harness session UUID to tool calls, so the two
identifiers are structurally disjoint and the gate was unsatisfiable.

Fix: drop the session_id coupling. A marker is valid if
  1. its contract_hash matches the current compliance-contract.md, AND
  2. acknowledged_at is within _ACK_LIFETIME_HOURS of now.
The hook scans _MARKER_DIR for any such marker. TTL still bounds staleness.

Wire this script as a PreToolUse hook in your harness config (aibox
handles the wiring; see scripts/README.md).

stdin
-----
Reads a JSON object from stdin (Claude Code PreToolUse shape).
Fields consumed:
  tool_name    str  — name of the tool about to be invoked
  tool_input   obj  — tool arguments (used to check the target file path)
  cwd          str  — working directory at invocation time (used to
                      resolve relative context/ path)

Write-side tool locked list
---------------------------
The following tool names are treated as write-side processkit tools and
will be blocked when no valid acknowledgement marker is present:
  create_workitem, transition_workitem, record_decision, link_entities,
  open_discussion, create_artifact, log_event, create_note.
acknowledge_contract is intentionally excluded — it IS the gate call.

Additionally, Write | Edit | MultiEdit calls that target a path starting
with context/ (resolved relative to cwd) are blocked.

Exit codes
----------
0  tool call is allowed (pass)
2  tool call is blocked (stderr shown to user; do NOT use exit 1 here)
"""

import hashlib
import json
import os
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_ASSETS_DIR = Path(__file__).parent.parent / "assets"
_CONTRACT_PATH = _ASSETS_DIR / "compliance-contract.md"
_MARKER_SUBDIR = Path("context") / ".state" / "skill-gate"
_ACK_LIFETIME_HOURS = 12

# Write-side processkit MCP tools that require an acknowledged contract.
_LOCKED_PROCESSKIT_TOOLS = frozenset(
    {
        "create_workitem",
        "transition_workitem",
        "record_decision",
        "link_entities",
        "open_discussion",
        "create_artifact",
        "log_event",
        "create_note",
    }
)

# Generic file-editing tools that are locked only when targeting context/.
_FILE_WRITE_TOOLS = frozenset({"Write", "Edit", "MultiEdit"})

_REMEDIATION_MSG = (
    "processkit compliance gate: call `acknowledge_contract(version='v1')` "
    "before any write-side processkit tool.\n"
    "  MCP call: {\"tool\": \"acknowledge_contract\", \"arguments\": {\"version\": \"v1\"}}\n"
    "  This writes a marker that the gate scans on every write.\n"
    "  If the tool is unavailable, add `processkit-skill-gate` to your\n"
    "  harness's enabled MCP servers, then retry."
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _contract_hash() -> str:
    """Return SHA-256 hex digest of the compliance contract file."""
    return hashlib.sha256(_CONTRACT_PATH.read_bytes()).hexdigest()


def _project_root(cwd: str) -> Path:
    """Walk up from cwd to find the directory containing context/."""
    candidate = Path(cwd).resolve()
    while True:
        if (candidate / "context").is_dir():
            return candidate
        parent = candidate.parent
        if parent == candidate:
            return Path(cwd).resolve()
        candidate = parent


def _marker_dir(cwd: str) -> Path:
    return _project_root(cwd) / _MARKER_SUBDIR


def _targets_context(tool_name: str, tool_input: dict, cwd: str) -> bool:
    """Return True if a generic file-write tool is targeting a path under context/."""
    if tool_name not in _FILE_WRITE_TOOLS:
        return False
    path_candidates: list[str] = []
    for field in ("file_path", "path", "new_path"):
        val = tool_input.get(field)
        if val and isinstance(val, str):
            path_candidates.append(val)

    context_dir = _project_root(cwd) / "context"
    for p_str in path_candidates:
        p = Path(p_str)
        if not p.is_absolute():
            p = Path(cwd) / p
        try:
            p.resolve().relative_to(context_dir.resolve())
            return True
        except ValueError:
            pass
    return False


def _is_locked(tool_name: str, tool_input: dict, cwd: str) -> bool:
    """Return True if this tool call requires an acknowledged contract."""
    if tool_name in _LOCKED_PROCESSKIT_TOOLS:
        return True
    if _targets_context(tool_name, tool_input, cwd):
        return True
    return False


def _any_valid_marker(cwd: str) -> bool:
    """
    Return True if any marker file has a contract_hash matching the
    current contract AND acknowledged_at within the TTL.

    Session-id coupling is intentionally absent — see module docstring.
    """
    if not _CONTRACT_PATH.exists():
        return False
    current_hash = _contract_hash()
    marker_dir = _marker_dir(cwd)
    if not marker_dir.is_dir():
        return False
    ttl = timedelta(hours=_ACK_LIFETIME_HOURS)
    now = datetime.now(timezone.utc)
    for marker in marker_dir.glob("session-*.ack"):
        try:
            data = json.loads(marker.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            continue
        if data.get("contract_hash") != current_hash:
            continue
        acked_at_str = data.get("acknowledged_at", "")
        try:
            acked_at = datetime.fromisoformat(acked_at_str)
        except (TypeError, ValueError):
            continue
        if acked_at.tzinfo is None:
            acked_at = acked_at.replace(tzinfo=timezone.utc)
        if now - acked_at <= ttl:
            return True
    return False


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main() -> int:
    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        print(f"check_route_task_called: could not parse stdin JSON: {exc}", file=sys.stderr)
        return 0

    tool_name: str = hook_input.get("tool_name", "")
    tool_input: dict = hook_input.get("tool_input", {})
    cwd: str = hook_input.get("cwd", os.getcwd())

    if not _is_locked(tool_name, tool_input, cwd):
        return 0

    if _any_valid_marker(cwd):
        return 0

    print(_REMEDIATION_MSG, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
