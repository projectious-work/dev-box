"""
decision_sweeper.py — Rail-5 Lever-2 SessionEnd sweeper.

WorkItem: FEAT-20260415_1700-QuietLedger-rail5-auto-decision-capture-implementation
Hooks guide: https://code.claude.com/docs/en/hooks

Purpose
-------
At session end, scan the FULL session transcript for decision-language
markers (Tier A + Tier B, higher-recall pass) and write a Note artifact
listing candidates the owner can review asynchronously.

This script NEVER blocks.  It exits 0 in all cases.

stdin
-----
Reads a Claude Code SessionEnd JSON object:
  session_id       str  — harness-provided stable session ID
  transcript_path  str  — path to session transcript JSONL
  cwd              str  — working directory
  reason           str  — reason for session end (optional)

Non-Claude-Code harnesses
--------------------------
SessionEnd is a Claude-Code-specific hook event.  On other harnesses
this script is simply not wired.  If invoked without transcript_path,
it logs a note to stderr and exits 0.

Output
------
Writes a markdown Note artifact at:
    context/notes/NOTE-<SESSION_ID>-decision-candidates.md

Frontmatter complies with the processkit Note schema.
The body is a table of detected decision-language cues with:
  - timestamp (from transcript entry if available, else "unknown")
  - matched text snippet (up to 120 chars)
  - the Tier-A or Tier-B marker that triggered the match
  - whether a record_decision was observed this session ("yes" or "no record")
"""

from __future__ import annotations

import importlib.util
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


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


def _find_project_root(cwd: str) -> Path:
    candidate = Path(cwd).resolve()
    while True:
        if (candidate / "context").is_dir():
            return candidate
        parent = candidate.parent
        if parent == candidate:
            break
        candidate = parent
    return Path(cwd).resolve()


def _load_decision_markers():
    """Import decision_markers from the same directory as this script."""
    try:
        from decision_markers import scan  # type: ignore[import]
        return scan
    except ImportError:
        pass

    lib_path = Path(__file__).parent / "decision_markers.py"
    spec = importlib.util.spec_from_file_location("decision_markers", lib_path)
    if spec is None or spec.loader is None:
        return None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod.scan  # type: ignore[return-value]


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

    Returns "" when content contains tool_use / tool_result blocks so the
    caller can skip such entries.
    """
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        if _content_has_tool_blocks(content):
            return ""
        parts = [
            p.get("text", "")
            for p in content
            if isinstance(p, dict) and p.get("type") == "text"
        ]
        return " ".join(parts)
    return ""


def _read_all_user_messages(transcript_path: str) -> list[dict]:
    """
    Read genuine user and text-only assistant messages from the JSONL transcript.

    Filtered OUT (false-positive sources per ART-20260415_2000-ShadowCount §9):
    - Entries with isCompactSummary: true  (context-compression artifacts)
    - Entries with isSidechain: true       (subagent metadata)
    - Any entry whose role is "tool" or content contains tool_use / tool_result blocks
    - User entries whose text starts with "<local-command-" (slash-command output)
    - Assistant entries with tool_use blocks (the agent's own write payloads)

    Returns list of dicts with keys:
      role      str   — "user" or "assistant"
      content   str   — message text
      timestamp str   — ISO-8601 if available, else ""
    """
    try:
        path = Path(transcript_path)
        if not path.is_file():
            return []
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    messages: list[dict] = []
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

        msg_type = obj.get("type", "")
        role = obj.get("role", "")
        timestamp = obj.get("timestamp", obj.get("created_at", ""))

        # Shape 1: {"type": "user"|"assistant", "message": {...}}
        if msg_type in ("user", "assistant"):
            inner = obj.get("message", {})
            r = inner.get("role", msg_type)
            # Skip tool roles embedded in message wrappers.
            if r not in ("user", "assistant"):
                continue
            text = _content_text_only(inner.get("content", ""))
            # Skip entries with tool blocks (tool_use / tool_result).
            if not text:
                continue
            # Skip slash-command output in user messages.
            if r == "user" and text.lstrip().startswith("<local-command-"):
                continue
            messages.append({"role": r, "content": text, "timestamp": timestamp})

        # Shape 2: flat {"role": "user"|"assistant", "content": ...}
        elif role in ("user", "assistant"):
            text = _content_text_only(obj.get("content", ""))
            if not text:
                continue
            if role == "user" and text.lstrip().startswith("<local-command-"):
                continue
            messages.append({"role": role, "content": text, "timestamp": timestamp})

    return messages


def _decision_observed(state_dir: Path, session_id: str) -> bool:
    marker = state_dir / f"session-{session_id}.decision-observed"
    return marker.is_file()


def _snippet(text: str, start: int, max_len: int = 120) -> str:
    """Return a short snippet around the match start."""
    lo = max(0, start - 30)
    hi = min(len(text), lo + max_len)
    snip = text[lo:hi].replace("\n", " ").replace("|", "\\|")
    prefix = "…" if lo > 0 else ""
    suffix = "…" if hi < len(text) else ""
    return prefix + snip + suffix


def _write_note(project_root: Path, session_id: str, rows: list[dict], decision_was_recorded: bool) -> Path:
    """Write the Note artifact and return its path."""
    notes_dir = project_root / "context" / "notes"
    notes_dir.mkdir(parents=True, exist_ok=True)

    now = datetime.now(timezone.utc)
    date_str = now.strftime("%Y%m%d_%H%M")
    filename = f"NOTE-{session_id[:40]}-decision-candidates.md"
    note_path = notes_dir / filename

    recorded_status = "yes" if decision_was_recorded else "no record"
    candidate_count = len(rows)

    lines: list[str] = [
        "---",
        "apiVersion: processkit.projectious.work/v1",
        "kind: Note",
        "metadata:",
        f"  id: NOTE-{session_id[:40]}-decision-candidates",
        f"  created: '{now.isoformat()}'",
        "  labels:",
        "    tag: decision-candidates",
        "    rail: '5'",
        f"    session_id: '{session_id}'",
        "spec:",
        f"  title: 'Decision candidates — session {session_id[:20]}'",
        "  state: fleeting",
        f"  record_decision_observed: {str(decision_was_recorded).lower()}",
        f"  candidate_count: {candidate_count}",
        "---",
        "",
        f"# Decision candidates — session `{session_id[:40]}`",
        "",
        f"Generated by Rail-5 Lever-2 SessionEnd sweeper on {now.strftime('%Y-%m-%d %H:%M UTC')}.",
        f"record_decision observed this session: **{recorded_status}**",
        "",
    ]

    if not rows:
        lines += [
            "No Tier-A or Tier-B decision-language markers detected in this session.",
            "",
        ]
    else:
        lines += [
            "| Tier | Marker | Timestamp | Snippet | record_decision |",
            "|------|--------|-----------|---------|-----------------|",
        ]
        for row in rows:
            tier = row.get("tier", "?")
            marker = row.get("marker", "").replace("|", "\\|")
            ts = row.get("timestamp", "unknown") or "unknown"
            snip = row.get("snippet", "")
            status = recorded_status
            lines.append(f"| {tier} | `{marker}` | {ts} | {snip} | {status} |")
        lines.append("")

    lines += [
        "## Review instructions",
        "",
        "For each row:",
        "- If this was a real decision → call `record_decision` with a clear title.",
        "- If it was not a decision → discard this row.",
        "",
        "> This note is auto-generated. Promote real decisions via `note-management-promote`",
        "> or by calling `record_decision` directly.",
    ]

    note_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return note_path


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main() -> int:
    raw = sys.stdin.read()
    try:
        hook_input: dict = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        print(f"decision_sweeper: could not parse stdin: {exc}", file=sys.stderr)
        return 0

    session_id = _resolve_session_id(hook_input)
    transcript_path: str = hook_input.get("transcript_path", "")
    cwd: str = hook_input.get("cwd", os.getcwd())

    if not transcript_path:
        print(
            "decision_sweeper: transcript_path absent (non-Claude-Code harness or SessionEnd not wired); no-op.",
            file=sys.stderr,
        )
        return 0

    scan = _load_decision_markers()
    if scan is None:
        print("decision_sweeper: could not load decision_markers; no-op.", file=sys.stderr)
        return 0

    messages = _read_all_user_messages(transcript_path)
    project_root = _find_project_root(cwd)
    state_dir = project_root / "context" / ".state" / "skill-gate"
    decision_was_recorded = _decision_observed(state_dir, session_id)

    rows: list[dict] = []
    for msg in messages:
        content = msg.get("content", "")
        if not content:
            continue
        hits = scan(content, tier="A+B")
        for hit in hits:
            rows.append(
                {
                    "tier": hit.tier,
                    "marker": hit.pattern,
                    "timestamp": msg.get("timestamp", ""),
                    "snippet": _snippet(content, hit.start),
                }
            )

    try:
        note_path = _write_note(project_root, session_id, rows, decision_was_recorded)
        print(
            f"decision_sweeper: wrote {len(rows)} candidate(s) to {note_path}",
            file=sys.stderr,
        )
    except OSError as exc:
        print(f"decision_sweeper: could not write note: {exc}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
