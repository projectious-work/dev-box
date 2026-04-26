"""workitems — closed/deferred/superseded WorkItems in the release window.

Uses index-management query_entities to find WorkItems whose
``completed_at`` or ``deferred_at`` falls within the time window.
Partitions them into "held" (done) and "slipped" (deferred / superseded).

Classification rules
--------------------
* **held**    — final state is ``done`` (or legacy aliases ``closed`` /
                ``completed``) and ``completed_at`` falls in the window.
* **slipped** — final state is ``deferred`` or ``superseded`` and
                ``deferred_at`` (or ``completed_at``) falls in the window.
* **cancelled** — excluded from both buckets.  ``cancelled`` means "we're
                not doing this; no longer a concern" — it is success-neutral
                noise from the retrospective's perspective and must not be
                counted as slippage.

Returns::

    {
        "held": [
            {
                "id": str,
                "title": str,
                "completed_at": str,
                "state": str,
            },
            ...
        ],
        "slipped": [
            {
                "id": str,
                "title": str,
                "deferred_at": str | None,
                "state": str,
                "reason": str,  # from notes or empty string
            },
            ...
        ],
        "held_count": int,
        "slipped_count": int,
        "total_closed": int,
        "bug_closed": int,      # for change-failure-rate calculation
        "raw": dict,
    }
"""

from __future__ import annotations

from pathlib import Path
from typing import Any


_HELD_STATES = frozenset({"done", "closed", "completed"})
# "cancelled" is intentionally excluded: it means "no longer a concern",
# not "we failed to deliver it".  Only genuine deferrals count as slippage.
_SLIPPED_STATES = frozenset({"deferred", "superseded"})


def collect(ctx: dict) -> dict:
    """Collect WorkItem signal."""
    since: str | None = ctx.get("since")
    until: str | None = ctx.get("until")
    mcp: dict = ctx.get("mcp", {})
    repo_root: Path = ctx["repo_root"]

    held, slipped, bug_closed = _fetch_workitems(
        mcp, repo_root, since, until
    )

    total_closed = len(held) + len(slipped)
    raw: dict[str, Any] = {
        "held_ids": [w["id"] for w in held],
        "slipped_ids": [w["id"] for w in slipped],
        "total_closed": total_closed,
        "bug_closed": bug_closed,
    }

    return {
        "held": held,
        "slipped": slipped,
        "held_count": len(held),
        "slipped_count": len(slipped),
        "total_closed": total_closed,
        "bug_closed": bug_closed,
        "raw": raw,
    }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _fetch_workitems(
    mcp: dict,
    repo_root: Path,
    since: str | None,
    until: str | None,
) -> tuple[list[dict], list[dict], int]:
    """Return (held, slipped, bug_closed_count)."""
    query_fn = mcp.get("query_entities")
    if not query_fn:
        # Disk fallback
        return _scan_workitem_files(repo_root, since, until)

    held: list[dict] = []
    slipped: list[dict] = []
    bug_closed = 0

    try:
        for state in list(_HELD_STATES) + list(_SLIPPED_STATES):
            items = query_fn(kind="WorkItem", state=state) or []
            for item in items:
                spec = item.get("spec", {}) or {}
                mid = item.get("metadata", {}).get("id", "")
                title = spec.get("title", "")
                wi_type = spec.get("type", "")
                completed = spec.get("completed_at", "")
                deferred = spec.get("deferred_at", "")
                ts = completed or deferred or ""

                if not _in_window(ts, since, until):
                    continue

                record = {
                    "id": mid,
                    "title": title,
                    "state": state,
                    "completed_at": completed,
                    "deferred_at": deferred,
                    "reason": spec.get("notes", "") or "",
                }

                if state in _HELD_STATES:
                    held.append(record)
                    if wi_type in ("bug", "defect", "fix"):
                        bug_closed += 1
                else:
                    slipped.append(record)
    except Exception:
        pass

    return held, slipped, bug_closed


def _in_window(ts: str, since: str | None, until: str | None) -> bool:
    """Return True if ts falls within [since, until]."""
    if not ts:
        return True  # missing timestamp: include conservatively
    if since and ts < since:
        return False
    if until and ts > until:
        return False
    return True


def _scan_workitem_files(
    repo_root: Path,
    since: str | None,
    until: str | None,
) -> tuple[list[dict], list[dict], int]:
    """Disk fallback: parse WorkItem YAML files directly."""
    wi_root = repo_root / "context" / "workitems"
    if not wi_root.exists():
        return [], [], 0

    held: list[dict] = []
    slipped: list[dict] = []
    bug_closed = 0

    for md_file in sorted(wi_root.glob("**/*.md")):
        try:
            text = md_file.read_text(encoding="utf-8")
            doc = _parse_yaml_front(text)
            if not doc:
                continue
            spec = doc.get("spec", {}) or {}
            state = spec.get("state", "")
            completed = spec.get("completed_at", "")
            deferred = spec.get("deferred_at", "")
            ts = completed or deferred or ""

            if not _in_window(ts, since, until):
                continue

            mid = doc.get("metadata", {}).get("id", "")
            title = spec.get("title", "")
            wi_type = spec.get("type", "")

            record = {
                "id": mid,
                "title": title,
                "state": state,
                "completed_at": completed,
                "deferred_at": deferred,
                "reason": spec.get("notes", "") or "",
            }

            if state in _HELD_STATES:
                held.append(record)
                if wi_type in ("bug", "defect", "fix"):
                    bug_closed += 1
            elif state in _SLIPPED_STATES:
                slipped.append(record)
        except Exception:
            continue

    return held, slipped, bug_closed


def _parse_yaml_front(text: str) -> dict | None:
    """Extract YAML frontmatter."""
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
