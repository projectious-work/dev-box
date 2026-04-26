"""release_summary — pull the latest release marker and DORA-like stats.

Queries:
  - mcp__processkit-event-log__query_events for release.published /
    session.release events to anchor the window.
  - git log between prior and current release markers for commit count
    and contributor list.
  - mcp__processkit-index-management__query_entities for WorkItems
    closed in window to derive top deliverables.

Returns::

    {
        "version": str,
        "release_date": str,          # ISO date of release marker
        "commit_count": int,
        "contributors": list[str],    # sorted unique author names
        "top_deliverables": list[str],# up to 3 WorkItem titles (done)
        "prior_release": str | None,  # prior version or None
        "prior_release_date": str | None,
        "raw": dict,                  # full signal data for --verbose
    }
"""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any


def collect(ctx: dict) -> dict:
    """Collect release summary signal."""
    repo_root: Path = ctx["repo_root"]
    release: str = ctx["release"]
    since: str | None = ctx.get("since")
    until: str | None = ctx.get("until")
    mcp: dict = ctx.get("mcp", {})

    # ── 1. Find release LogEntry ───────────────────────────────────────────
    release_event = _find_release_event(release, mcp)
    release_date = (release_event or {}).get("timestamp", until or "")

    # ── 2. Find prior release ─────────────────────────────────────────────
    prior_event = _find_prior_release_event(release, mcp)
    prior_version = (prior_event or {}).get("details", {}).get("version")
    prior_date = (prior_event or {}).get("timestamp")
    window_start = since or prior_date

    # ── 3. Git stats in window ────────────────────────────────────────────
    commit_count, contributors = _git_stats(
        repo_root, window_start, until or "HEAD"
    )

    # ── 4. Top deliverables: WorkItems closed in window ───────────────────
    top_deliverables = _top_deliverables(mcp, window_start, until)

    raw: dict[str, Any] = {
        "release_event": release_event,
        "prior_event": prior_event,
        "commit_count": commit_count,
        "contributors": contributors,
        "top_deliverables_raw": top_deliverables,
    }

    return {
        "version": release,
        "release_date": _as_date(release_date),
        "commit_count": commit_count,
        "contributors": sorted(set(contributors)),
        "top_deliverables": top_deliverables[:3],
        "prior_release": prior_version,
        "prior_release_date": _as_date(prior_date) if prior_date else None,
        "raw": raw,
    }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _find_release_event(release: str, mcp: dict) -> dict | None:
    """Query event log for the release marker matching ``release``."""
    query_fn = mcp.get("query_events")
    if not query_fn:
        return None
    try:
        events = query_fn(
            event_type="release.published",
            limit=50,
            order="desc",
        ) or []
        for ev in events:
            details = ev.get("details", {})
            v = details.get("version") or details.get("release")
            if v and (v == release or v.lstrip("v") == release.lstrip("v")):
                return ev
        # Fallback: session.release
        events2 = query_fn(
            event_type="session.release",
            limit=50,
            order="desc",
        ) or []
        for ev in events2:
            details = ev.get("details", {})
            v = details.get("version") or details.get("release")
            if v and (v == release or v.lstrip("v") == release.lstrip("v")):
                return ev
    except Exception:
        pass
    return None


def _find_prior_release_event(release: str, mcp: dict) -> dict | None:
    """Return the release event just before ``release``."""
    query_fn = mcp.get("query_events")
    if not query_fn:
        return None
    try:
        for etype in ("release.published", "session.release"):
            events = query_fn(
                event_type=etype, limit=50, order="desc"
            ) or []
            found_current = False
            for ev in events:
                details = ev.get("details", {})
                v = details.get("version") or details.get("release") or ""
                if v.lstrip("v") == release.lstrip("v"):
                    found_current = True
                    continue
                if found_current:
                    return ev
    except Exception:
        pass
    return None


def _git_stats(
    repo_root: Path,
    since: str | None,
    until: str,
) -> tuple[int, list[str]]:
    """Return (commit_count, [author_names]) between since..until."""
    try:
        cmd = ["git", "-C", str(repo_root), "log", "--pretty=%aN"]
        if since:
            cmd += [f"--since={since}"]
        if until and until != "HEAD":
            cmd += [f"--until={until}"]
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=15
        )
        if result.returncode != 0:
            return 0, []
        names = [n.strip() for n in result.stdout.splitlines() if n.strip()]
        return len(names), list(set(names))
    except Exception:
        return 0, []


def _top_deliverables(
    mcp: dict,
    since: str | None,
    until: str | None,
) -> list[str]:
    """Return up to 3 titles of WorkItems closed in window."""
    query_fn = mcp.get("query_entities")
    if not query_fn:
        return []
    try:
        items = query_fn(kind="WorkItem", state="done") or []
        results: list[str] = []
        for item in items:
            completed = item.get("spec", {}).get("completed_at", "") or ""
            if since and completed and completed < since:
                continue
            if until and completed and completed > until:
                continue
            title = item.get("spec", {}).get("title", "")
            if title:
                results.append(title)
            if len(results) >= 3:
                break
        return results
    except Exception:
        return []


def _as_date(ts: str | None) -> str:
    """Return YYYY-MM-DD from an ISO timestamp, or '' if None/invalid."""
    if not ts:
        return ""
    return str(ts)[:10]
