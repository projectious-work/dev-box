"""
check_decision_captured.py — Rail-5 Lever-1 PreToolUse decision-capture gate.

WorkItem: FEAT-20260415_1700-QuietLedger-rail5-auto-decision-capture-implementation
Hooks guide: https://code.claude.com/docs/en/hooks

Purpose
-------
Before any write-side processkit tool executes, check whether the recent
user messages contain Tier-A decision language. If they do AND no
`record_decision` call or `skip_decision_record` acknowledgement has been
observed in this session, emit a warning (shadow mode, default) or block
the tool call (--mode=block).

Ships in SHADOW MODE by default.  Pass --mode=block to enable blocking.

stdin
-----
Reads a Claude Code PreToolUse JSON object:
  session_id       str  — harness-provided stable session ID
  tool_name        str  — tool about to be invoked
  tool_input       obj  — tool arguments
  transcript_path  str  — path to session transcript JSONL (Claude Code 2.1+)
  cwd              str  — working directory

Non-Claude-Code harnesses
--------------------------
If transcript_path is absent or unreadable, log to stderr and exit 0.
This ensures graceful degradation on Cursor, Codex CLI, etc.

Exit codes
----------
0   allow (always in shadow mode; in block mode: no Tier-A markers, or
    a valid skip/record_decision acknowledgement is present)
2   block (--mode=block only: Tier-A marker detected, no ack)

Session marker files
--------------------
  context/.state/skill-gate/session-<SESSION_ID>.decision-skip
      Written by MCP tool skip_decision_record(). Treated as valid
      acknowledgement if not expired (24 h).

  context/.state/skill-gate/session-<SESSION_ID>.decision-observed
      Written by record_decision_observer.py PostToolUse hook.
      Presence means record_decision was called this session.
"""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

# Gated tool set: same family as check_route_task_called.py
_GATED_TOOLS = frozenset(
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

# Also gate the qualified processkit tool name variants (with server prefix).
_GATED_TOOL_PREFIXES = ("create_", "record_", "transition_", "link_")

# Number of user messages to scan (last N).
_SCAN_WINDOW = 5

# Skip-marker TTL in seconds (24 hours).
_SKIP_MARKER_TTL = 86400

_SHADOW_WARNING = (
    "[rail-5 shadow] Tier-A decision marker detected in last 5 user messages;"
    " no record_decision or skip_decision_record observed."
    " Set --mode=block to enforce."
)

_BLOCK_REMEDIATION = (
    "[rail-5] Call record_decision() or skip_decision_record(reason='...')"
    " before continuing."
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


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
    # Fallback: treat cwd as project root.
    return Path(cwd).resolve() / "context" / ".state" / "skill-gate"


def _is_gated(tool_name: str) -> bool:
    """Return True if this tool should be checked for decision capture."""
    if tool_name in _GATED_TOOLS:
        return True
    # Check qualified names like "processkit-decision-record__record_decision"
    bare = tool_name.split("__")[-1] if "__" in tool_name else tool_name
    if bare in _GATED_TOOLS:
        return True
    for prefix in _GATED_TOOL_PREFIXES:
        if bare.startswith(prefix):
            return True
    return False


def _content_has_tool_blocks(content: object) -> bool:
    """Return True if *content* (a list of blocks) contains any tool_use or tool_result blocks."""
    if not isinstance(content, list):
        return False
    return any(
        isinstance(block, dict) and block.get("type") in ("tool_use", "tool_result")
        for block in content
    )


def _content_text_only(content: object) -> str:
    """
    Extract plain text from a content value, returning only text-type blocks.

    Returns "" when content contains non-text blocks (tool_use / tool_result),
    which causes the caller to skip the entry.
    """
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        if _content_has_tool_blocks(content):
            return ""
        parts = [
            p.get("text", "") for p in content if isinstance(p, dict) and p.get("type") == "text"
        ]
        return " ".join(parts)
    return ""


def _read_last_n_user_messages(transcript_path: str, n: int) -> list[str]:
    """
    Read the JSONL transcript and return the text of the last *n* genuine user messages.

    Genuine user messages are entries where:
    - The entry does NOT have isCompactSummary: true
    - The entry does NOT have isSidechain: true
    - The role is "user" (not "assistant", "tool", or anything else)
    - The content does NOT contain tool_use or tool_result blocks
    - The content text does NOT start with "<local-command-"

    Transcript JSONL: each line is a JSON object with at least:
      {"role": "user", "content": "<text>"}  or
      {"type": "user", "message": {"role": "user", "content": [...]}}

    Returns empty list on any read / parse error.
    """
    try:
        path = Path(transcript_path)
        if not path.is_file():
            return []
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    user_messages: list[str] = []
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue

        # Skip context-compression artifacts and subagent metadata.
        if obj.get("isCompactSummary") or obj.get("isSidechain"):
            continue

        # Claude Code transcript shape: top-level "type" field.
        msg_type = obj.get("type", "")
        role = obj.get("role", "")

        # Shape 1: {"type": "user", "message": {"role": "user", "content": ...}}
        if msg_type == "user":
            inner = obj.get("message", {})
            # Only process genuine user-role messages; skip tool/system roles.
            if inner.get("role", "user") != "user":
                continue
            text = _content_text_only(inner.get("content", ""))
            if not text or text.lstrip().startswith("<local-command-"):
                continue
            user_messages.append(text)

        # Shape 2: flat {"role": "user", "content": "..."}
        elif role == "user":
            text = _content_text_only(obj.get("content", ""))
            if not text or text.lstrip().startswith("<local-command-"):
                continue
            user_messages.append(text)

        # Explicitly skip assistant, tool, and all other roles — Lever 1
        # must only scan genuine human-typed user messages.

    return user_messages[-n:]


def _skip_marker_valid(state_dir: Path, session_id: str) -> bool:
    """Return True if a non-expired skip marker exists for this session."""
    marker = state_dir / f"session-{session_id}.decision-skip"
    if not marker.is_file():
        return False
    try:
        data = json.loads(marker.read_text(encoding="utf-8"))
        expires_at_str = data.get("expires_at", "")
        if not expires_at_str:
            # No expiry field — treat as valid (legacy).
            return True
        expires_at = datetime.fromisoformat(expires_at_str)
        return datetime.now(timezone.utc) < expires_at
    except Exception:
        return False


def _decision_observed(state_dir: Path, session_id: str) -> bool:
    """Return True if record_decision was observed this session."""
    marker = state_dir / f"session-{session_id}.decision-observed"
    return marker.is_file()


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main(argv: list[str] | None = None) -> int:
    if argv is None:
        argv = sys.argv[1:]

    block_mode = "--mode=block" in argv

    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        print(f"check_decision_captured: could not parse stdin JSON: {exc}", file=sys.stderr)
        return 0

    tool_name: str = hook_input.get("tool_name", "")

    # Skip non-gated tools immediately.
    if not _is_gated(tool_name):
        return 0

    transcript_path: str = hook_input.get("transcript_path", "")
    if not transcript_path:
        print(
            "check_decision_captured: transcript_path absent (non-Claude-Code harness?); skipping decision scan.",
            file=sys.stderr,
        )
        return 0

    try:
        user_messages = _read_last_n_user_messages(transcript_path, _SCAN_WINDOW)
    except Exception as exc:
        print(f"check_decision_captured: could not read transcript: {exc}", file=sys.stderr)
        return 0

    if not user_messages:
        return 0

    # Import the shared marker library.
    try:
        from decision_markers import scan as _scan  # type: ignore[import]
    except ImportError:
        # Attempt path-based import when not on sys.path.
        import importlib.util

        lib_path = Path(__file__).parent / "decision_markers.py"
        spec = importlib.util.spec_from_file_location("decision_markers", lib_path)
        if spec is None or spec.loader is None:
            print("check_decision_captured: could not import decision_markers; skipping.", file=sys.stderr)
            return 0
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)  # type: ignore[union-attr]
        _scan = mod.scan  # type: ignore[assignment]

    combined_text = "\n".join(user_messages)
    hits = _scan(combined_text, tier="A")
    if not hits:
        return 0  # No Tier-A markers — nothing to check.

    # Check for acknowledgement.
    session_id = _resolve_session_id(hook_input)
    cwd: str = hook_input.get("cwd", os.getcwd())
    state_dir = _find_state_dir(cwd)

    if _skip_marker_valid(state_dir, session_id):
        return 0  # Valid skip acknowledgement.

    if _decision_observed(state_dir, session_id):
        return 0  # record_decision was called this session.

    # Marker detected, no acknowledgement.
    if block_mode:
        print(_BLOCK_REMEDIATION, file=sys.stderr)
        return 2
    else:
        # Shadow mode (default): warn but don't block.
        print(_SHADOW_WARNING, file=sys.stderr)
        return 0


if __name__ == "__main__":
    sys.exit(main())
