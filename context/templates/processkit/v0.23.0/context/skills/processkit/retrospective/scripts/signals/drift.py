"""drift — skip-marker deltas + doctor.report ERROR/WARN deltas.

Queries doctor.report LogEntries in the release window; computes
ERROR/WARN count deltas between the first and last report in the window.
Also counts skip-marker LogEntries (event_type: skip.*) as a proxy for
known-bad-behaviour debt accumulating over the cycle.

Returns::

    {
        "doctor_error_delta": int,    # last - first ERROR count in window
        "doctor_warn_delta": int,     # last - first WARN count in window
        "skip_marker_count": int,     # skip.* events in window
        "skip_marker_ids": list[str], # entity IDs of skip events
        "first_doctor": dict | None,  # first doctor.report in window
        "last_doctor": dict | None,   # last doctor.report in window
        "slipped_signals": list[dict],# actionable degradation items
        "raw": dict,
    }

``slipped_signals`` is a list of dicts with keys:
    {"description": str, "entity": str | None, "certain": bool}

Each item maps to one "What Slipped" bullet. ``certain=False`` triggers
the ``[uncertain]`` tag in the aggregator.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any


def collect(ctx: dict) -> dict:
    """Collect drift signal."""
    since: str | None = ctx.get("since")
    until: str | None = ctx.get("until")
    mcp: dict = ctx.get("mcp", {})
    repo_root: Path = ctx["repo_root"]

    doctor_reports = _fetch_doctor_reports(mcp, repo_root, since, until)
    skip_events = _fetch_skip_events(mcp, repo_root, since, until)

    first_doc = doctor_reports[0] if doctor_reports else None
    last_doc = doctor_reports[-1] if doctor_reports else None

    error_delta, warn_delta = _compute_deltas(first_doc, last_doc)

    slipped: list[dict] = []

    # Doctor degradation signal
    if error_delta > 0:
        entity_id = (last_doc or {}).get("id", "")
        slipped.append({
            "description": (
                f"ERROR count increased by {error_delta} "
                f"across the release window"
            ),
            "entity": entity_id or None,
            "certain": bool(entity_id),
        })

    if warn_delta > 0:
        entity_id = (last_doc or {}).get("id", "")
        slipped.append({
            "description": (
                f"WARN count increased by {warn_delta} "
                f"across the release window"
            ),
            "entity": entity_id or None,
            "certain": bool(entity_id),
        })

    # Skip marker accumulation signal
    skip_ids = [ev.get("id", "") for ev in skip_events if ev.get("id")]
    if skip_ids:
        slipped.append({
            "description": (
                f"{len(skip_ids)} skip-marker event(s) recorded "
                f"in the release window"
            ),
            "entity": skip_ids[0] if skip_ids else None,
            "certain": bool(skip_ids),
        })

    raw: dict[str, Any] = {
        "doctor_report_count": len(doctor_reports),
        "skip_event_count": len(skip_events),
        "error_delta": error_delta,
        "warn_delta": warn_delta,
    }

    return {
        "doctor_error_delta": error_delta,
        "doctor_warn_delta": warn_delta,
        "skip_marker_count": len(skip_events),
        "skip_marker_ids": skip_ids,
        "first_doctor": first_doc,
        "last_doctor": last_doc,
        "slipped_signals": slipped,
        "raw": raw,
    }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _fetch_doctor_reports(
    mcp: dict,
    repo_root: Path,
    since: str | None,
    until: str | None,
) -> list[dict]:
    """Fetch doctor.report LogEntries in window, ordered ascending."""
    query_fn = mcp.get("query_events")
    if query_fn:
        try:
            evs = query_fn(
                event_type="doctor.report",
                since=since,
                until=until,
                limit=100,
                order="asc",
            ) or []
            return list(evs)
        except Exception:
            pass
    return _scan_log_files_for_type(
        repo_root, "doctor.report", since, until
    )


def _fetch_skip_events(
    mcp: dict,
    repo_root: Path,
    since: str | None,
    until: str | None,
) -> list[dict]:
    """Fetch skip.* LogEntries in window."""
    query_fn = mcp.get("query_events")
    if query_fn:
        results: list[dict] = []
        for etype in ("skip.decision_record", "skip.contract"):
            try:
                evs = query_fn(
                    event_type=etype,
                    since=since,
                    until=until,
                    limit=100,
                    order="asc",
                ) or []
                results.extend(evs)
            except Exception:
                pass
        return results
    return _scan_log_files_for_prefix(
        repo_root, "skip.", since, until
    )


def _compute_deltas(
    first: dict | None,
    last: dict | None,
) -> tuple[int, int]:
    """Return (error_delta, warn_delta) between first and last doctor report."""
    if not first or not last or first is last:
        return 0, 0

    def _total_sev(doc: dict, sev: str) -> int:
        cats = (
            doc.get("details", {})
            .get("categories", {})
        )
        if isinstance(cats, dict):
            return sum(
                (v.get(sev, 0) if isinstance(v, dict) else 0)
                for v in cats.values()
            )
        return 0

    e_first = _total_sev(first, "ERROR")
    e_last = _total_sev(last, "ERROR")
    w_first = _total_sev(first, "WARN")
    w_last = _total_sev(last, "WARN")
    return e_last - e_first, w_last - w_first


def _scan_log_files_for_type(
    repo_root: Path,
    event_type: str,
    since: str | None,
    until: str | None,
) -> list[dict]:
    """Disk fallback: return parsed LogEntry specs for a given event_type."""
    return _scan_log_files_for_prefix(
        repo_root, event_type, since, until, exact=True
    )


def _scan_log_files_for_prefix(
    repo_root: Path,
    prefix: str,
    since: str | None,
    until: str | None,
    exact: bool = False,
) -> list[dict]:
    logs_root = repo_root / "context" / "logs"
    if not logs_root.exists():
        return []
    results: list[dict] = []
    for md in sorted(logs_root.glob("**/*.md")):
        try:
            import yaml
            text = md.read_text(encoding="utf-8")
            if not text.startswith("---"):
                continue
            end = text.find("---", 3)
            if end < 0:
                continue
            doc = yaml.safe_load(text[3:end])
            if not isinstance(doc, dict):
                continue
            spec = doc.get("spec", {}) or {}
            etype = spec.get("event_type", "")
            if exact and etype != prefix:
                continue
            if not exact and not etype.startswith(prefix):
                continue
            ts = spec.get("timestamp", "")
            if since and ts < since:
                continue
            if until and ts > until:
                continue
            entry = dict(spec)
            entry.setdefault("id", doc.get("metadata", {}).get("id", ""))
            results.append(entry)
        except Exception:
            continue
    results.sort(key=lambda e: e.get("timestamp", ""))
    return results
