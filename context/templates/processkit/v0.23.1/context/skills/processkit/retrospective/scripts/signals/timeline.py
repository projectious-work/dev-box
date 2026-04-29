"""timeline — session boundaries + major milestones in the release window.

Reads LogEntries from ``context/logs/<year>/<month>/`` filtered by
event_type in {session.started, session.ended, session.release,
release.published} within the time window.

Falls back to scanning log files on disk when the MCP query_events
function is not available (e.g. running standalone without MCP).

Returns::

    {
        "sessions": [
            {
                "start": str,         # ISO timestamp or date
                "end": str | None,
                "summary": str,
            },
            ...
        ],
        "milestones": [
            {"timestamp": str, "description": str, "entity": str},
            ...
        ],
        "session_count": int,
        "raw": dict,
    }
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any


_SESSION_TYPES = frozenset({
    "session.started",
    "session.ended",
    "session.handover",
    "session.release",
    "release.published",
})


def collect(ctx: dict) -> dict:
    """Collect timeline signal."""
    repo_root: Path = ctx["repo_root"]
    since: str | None = ctx.get("since")
    until: str | None = ctx.get("until")
    mcp: dict = ctx.get("mcp", {})

    events = _fetch_events(mcp, repo_root, since, until)
    sessions = _build_sessions(events)
    milestones = _build_milestones(events)

    raw: dict[str, Any] = {
        "event_count": len(events),
        "events_sample": events[:5],
    }

    return {
        "sessions": sessions,
        "milestones": milestones,
        "session_count": len(sessions),
        "raw": raw,
    }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _fetch_events(
    mcp: dict,
    repo_root: Path,
    since: str | None,
    until: str | None,
) -> list[dict]:
    """Fetch events via MCP or disk fallback."""
    query_fn = mcp.get("query_events")
    if query_fn:
        try:
            results: list[dict] = []
            for etype in _SESSION_TYPES:
                evs = query_fn(
                    event_type=etype, since=since, until=until,
                    limit=200, order="asc",
                ) or []
                results.extend(evs)
            results.sort(key=lambda e: e.get("timestamp", ""))
            return results
        except Exception:
            pass
    # Disk fallback: scan context/logs/<year>/<month>/*.md
    return _scan_log_files(repo_root, since, until)


def _scan_log_files(
    repo_root: Path,
    since: str | None,
    until: str | None,
) -> list[dict]:
    """Parse log files on disk for relevant events."""
    logs_root = repo_root / "context" / "logs"
    if not logs_root.exists():
        return []
    events: list[dict] = []
    for md in sorted(logs_root.glob("**/*.md")):
        try:
            text = md.read_text(encoding="utf-8")
            ev = _parse_logentry(text)
            if ev is None:
                continue
            etype = ev.get("spec", {}).get("event_type", "")
            if etype not in _SESSION_TYPES:
                continue
            ts = ev.get("spec", {}).get("timestamp", "")
            if since and ts < since:
                continue
            if until and ts > until:
                continue
            events.append(ev.get("spec", {}))
        except Exception:
            continue
    events.sort(key=lambda e: e.get("timestamp", ""))
    return events


def _parse_logentry(text: str) -> dict | None:
    """Minimal YAML frontmatter parser for LogEntry files."""
    if not text.startswith("---"):
        return None
    end = text.find("---", 3)
    if end < 0:
        return None
    try:
        import yaml
        doc = yaml.safe_load(text[3:end])
        if isinstance(doc, dict):
            return doc
    except Exception:
        pass
    return None


def _build_sessions(events: list[dict]) -> list[dict]:
    """Pair session start/end events into session records."""
    sessions: list[dict] = []
    current: dict | None = None
    for ev in events:
        etype = ev.get("event_type", "")
        ts = ev.get("timestamp", "")
        summary_field = ev.get("summary", "")
        if etype == "session.started":
            current = {"start": ts, "end": None, "summary": summary_field}
        elif etype in ("session.ended", "session.handover"):
            if current:
                current["end"] = ts
                if not current["summary"] and summary_field:
                    current["summary"] = summary_field
                sessions.append(current)
                current = None
            else:
                sessions.append({
                    "start": "",
                    "end": ts,
                    "summary": summary_field,
                })
    if current:
        sessions.append(current)
    return sessions


def _build_milestones(events: list[dict]) -> list[dict]:
    """Extract milestone events (release markers)."""
    milestones: list[dict] = []
    for ev in events:
        etype = ev.get("event_type", "")
        if etype in ("session.release", "release.published"):
            details = ev.get("details", {}) or {}
            version = details.get("version") or details.get("release", "")
            entity_id = ev.get("id", "")
            desc = f"Release {version}" if version else ev.get("summary", "")
            milestones.append({
                "timestamp": ev.get("timestamp", ""),
                "description": desc,
                "entity": entity_id,
            })
    return milestones
