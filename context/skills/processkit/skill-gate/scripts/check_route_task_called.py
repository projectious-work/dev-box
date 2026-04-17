"""
check_route_task_called.py — processkit PreToolUse hook gate.

WorkItem: FEAT-20260414_1433-SteadyHand-provider-neutral-hook-scripts
Hooks guide: https://code.claude.com/docs/en/hooks-guide
Marker format: context/skills/processkit/skill-gate/mcp/SERVER.md §"Session marker file"

Purpose
-------
Block write-side processkit tools (and Write/Edit/MultiEdit under
context/**) until the agent has called acknowledge_contract() and a
valid session marker file exists.

Wire this script as a PreToolUse hook in your harness config (aibox
handles the wiring; see scripts/README.md).

stdin
-----
Reads a JSON object from stdin (Claude Code PreToolUse shape).
Fields consumed:
  session_id   str  — stable session identifier supplied by the harness
  tool_name    str  — name of the tool about to be invoked
  tool_input   obj  — tool arguments (used to check the target file path)
  cwd          str  — working directory at invocation time (used to
                      resolve relative session-marker path)

Session-ID precedence
---------------------
1. hook input JSON field `session_id`       (most specific; harness-provided)
2. env var PROCESSKIT_SESSION_ID            (test / CI injection)
3. str(os.getpid())                         (fallback: parent PID)

Write-side tool locked list
---------------------------
The following tool names are treated as write-side processkit tools and
will be blocked when no valid acknowledgement marker is present:
  create_workitem, transition_workitem, record_decision, link_entities,
  open_discussion, create_artifact, log_event, create_note,
  acknowledge_contract is intentionally excluded — it IS the gate call.

Additionally, Write | Edit | MultiEdit | Bash calls that target a path
starting with context/ (resolved relative to cwd) are blocked.

Exit codes
----------
0  tool call is allowed (pass)
2  tool call is blocked (stderr shown to user; do NOT use exit 1 here)
"""

import hashlib
import json
import os
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_ASSETS_DIR = Path(__file__).parent.parent / "assets"
_CONTRACT_PATH = _ASSETS_DIR / "compliance-contract.md"

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
_FILE_WRITE_TOOLS = frozenset({"Write", "Edit", "MultiEdit", "Bash"})

_REMEDIATION_MSG = (
    "processkit compliance gate: call `acknowledge_contract(version='v1')` "
    "before any write-side processkit tool.\n"
    "  MCP call: {\"tool\": \"acknowledge_contract\", \"arguments\": {\"version\": \"v1\"}}\n"
    "  This writes a session marker that this gate checks on every write."
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _contract_hash() -> str:
    """Return SHA-256 hex digest of the compliance contract file."""
    return hashlib.sha256(_CONTRACT_PATH.read_bytes()).hexdigest()


def _resolve_session_id(hook_input: dict) -> str:
    """
    Return the session ID using the documented precedence:
    1. hook input JSON field 'session_id'
    2. env var PROCESSKIT_SESSION_ID
    3. str(os.getpid())
    """
    sid = hook_input.get("session_id")
    if sid:
        return str(sid)
    sid = os.environ.get("PROCESSKIT_SESSION_ID")
    if sid:
        return sid
    return str(os.getpid())


def _marker_path(cwd: str, session_id: str) -> Path:
    """
    Resolve the session marker path.

    The marker lives at:
        <project_root>/context/.state/skill-gate/session-<SESSION_ID>.ack

    We walk up from cwd to find the project root (directory containing
    'context/').  If not found, fall back to cwd itself.
    """
    candidate = Path(cwd).resolve()
    while True:
        if (candidate / "context").is_dir():
            return candidate / "context" / ".state" / "skill-gate" / f"session-{session_id}.ack"
        parent = candidate.parent
        if parent == candidate:
            # Reached filesystem root without finding 'context/'.
            break
        candidate = parent
    # Fallback: treat cwd as project root.
    base = Path(cwd).resolve()
    return base / "context" / ".state" / "skill-gate" / f"session-{session_id}.ack"


def _targets_context(tool_name: str, tool_input: dict, cwd: str) -> bool:
    """
    Return True if a generic file-write tool is targeting a path under context/.
    """
    if tool_name not in _FILE_WRITE_TOOLS:
        return False
    # Extract candidate path fields depending on the tool.
    path_candidates: list[str] = []
    for field in ("file_path", "path", "new_path", "command"):
        val = tool_input.get(field)
        if val and isinstance(val, str):
            path_candidates.append(val)

    # For Bash, scan the command string for 'context/' literal.
    if tool_name == "Bash":
        cmd = tool_input.get("command", "")
        return "context/" in cmd

    project_root = Path(cwd).resolve()
    # Walk up to find actual project root.
    candidate = project_root
    while True:
        if (candidate / "context").is_dir():
            project_root = candidate
            break
        parent = candidate.parent
        if parent == candidate:
            break
        candidate = parent

    context_dir = project_root / "context"
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


def _marker_is_valid(marker: Path) -> bool:
    """
    Return True if the marker file exists, is parseable, and its
    contract_hash matches the current contract file.
    """
    if not marker.exists():
        return False
    try:
        data = json.loads(marker.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return False
    stored_hash = data.get("contract_hash", "")
    if not _CONTRACT_PATH.exists():
        # Contract missing — can't validate; let the session proceed.
        return False
    return stored_hash == _contract_hash()


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main() -> int:
    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        # Malformed input — don't block, but warn.
        print(f"check_route_task_called: could not parse stdin JSON: {exc}", file=sys.stderr)
        return 0

    tool_name: str = hook_input.get("tool_name", "")
    tool_input: dict = hook_input.get("tool_input", {})
    cwd: str = hook_input.get("cwd", os.getcwd())

    if not _is_locked(tool_name, tool_input, cwd):
        return 0  # Not a gated tool — pass immediately.

    session_id = _resolve_session_id(hook_input)
    marker = _marker_path(cwd, session_id)

    if _marker_is_valid(marker):
        return 0  # Acknowledged and hash is current — pass.

    print(_REMEDIATION_MSG, file=sys.stderr)
    return 2  # Block.


if __name__ == "__main__":
    sys.exit(main())
