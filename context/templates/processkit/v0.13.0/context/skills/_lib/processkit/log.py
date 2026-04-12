"""Side-effect logging for entity-creating and mutating MCP servers.

Every mechanical operation that changes an entity (create, transition,
supersede, …) calls ``log_side_effect`` after the primary write succeeds.
This produces an append-only audit trail without requiring agent discipline.

Distinct from the ``event-log`` MCP server, which is the agent-facing wrapper
for explicit judgment-call entries (session handovers, behavioral notes, …).
Both write the same ``LogEntry`` entity format; they differ in who calls them.

Errors are swallowed and returned as ``None`` so that a logging failure never
breaks the primary operation.
"""
from __future__ import annotations

import datetime as _dt
from pathlib import Path
from typing import Any


def log_side_effect(
    kind: str,
    entity_id: str,
    event_type: str,
    summary: str,
    root: Path | None = None,
    actor: str | None = None,
    details: dict[str, Any] | None = None,
) -> str | None:
    """Write a LogEntry as a side effect of a mechanical operation.

    Parameters
    ----------
    kind:
        The primitive kind of the entity being acted on (e.g. ``"WorkItem"``).
    entity_id:
        The ID of the entity being acted on (e.g. ``"BACK-20260409_1449-BoldVale"``).
    event_type:
        Dot-namespaced event type (e.g. ``"workitem.created"``,
        ``"decision.transitioned"``).
    summary:
        Short human-readable description of what happened.
    root:
        Project root (resolved automatically if omitted).
    actor:
        Optional actor ID (e.g. ``"ACTOR-claude"``).
    details:
        Optional extra key/value pairs added to ``spec.details``.

    Returns
    -------
    str | None
        The new LogEntry ID, or ``None`` if logging failed (the primary
        operation is never broken by a logging error).
    """
    try:
        from . import config, entity, ids, index, paths

        root = root or paths.find_project_root()
        cfg = config.load_config(root)

        db = index.open_db()
        try:
            existing = index.existing_ids(db, "LogEntry")
        finally:
            db.close()

        now = _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")
        log_id = ids.generate_id(
            "LogEntry",
            format=cfg.id_format,
            word_style=cfg.id_word_style,
            datetime_prefix=cfg.id_datetime_prefix,
            slug_text=event_type if cfg.id_slug else None,
            existing=existing,
        )

        spec: dict[str, Any] = {
            "event_type": event_type,
            "timestamp": now,
            "summary": summary,
            "subject": entity_id,
            "subject_kind": kind,
        }
        if actor:
            spec["actor"] = actor
        if details:
            spec["details"] = details

        log_ent = entity.new("LogEntry", log_id, spec)
        target = paths.entity_path("LogEntry", log_id, now, root)
        log_ent.write(target)

        db = index.open_db()
        try:
            index.upsert_entity(db, log_ent)
        finally:
            db.close()

        return log_id
    except Exception:
        return None
