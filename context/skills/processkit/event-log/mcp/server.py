#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit event-log MCP server.

Append-only LogEntry management. Tools:

    log_event(event_type, summary, actor?, subject?, subject_kind?, details?, timestamp?)
        -> {id, path}
    query_events(event_type?, subject?, actor?, limit?)
        -> [events]
    recent_events(limit?)
        -> [events]
"""
from __future__ import annotations

import datetime as _dt
import os
import sys
from pathlib import Path


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import config, entity, ids, index, paths  # noqa: E402

server = FastMCP("processkit-event-log")


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def log_event(
    event_type: str,
    summary: str,
    actor: str | None = None,
    subject: str | None = None,
    subject_kind: str | None = None,
    details: dict | None = None,
    timestamp: str | None = None,
    correlation_id: str | None = None,
) -> dict:
    """Append a new LogEntry to the project's event log.

    Returns ``{id, path}`` for the created file. Prerequisite: call
    find_skill(task_description) or confirm you are already operating
    within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    root = paths.find_project_root()
    cfg = config.load_config(root)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "LogEntry")
    finally:
        db.close()

    created_at = timestamp or _now_iso()
    new_id = ids.generate_id(
        "LogEntry",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=event_type if cfg.id_slug else None,
        existing=existing,
    )

    spec = {
        "event_type": event_type,
        "timestamp": created_at,
        "summary": summary,
    }
    if actor:
        spec["actor"] = actor
    if subject:
        spec["subject"] = subject
    if subject_kind:
        spec["subject_kind"] = subject_kind
    if details:
        spec["details"] = details
    if correlation_id:
        spec["correlation_id"] = correlation_id

    ent = entity.new("LogEntry", new_id, spec)
    # entity_path applies date-based sharding when configured
    target = paths.entity_path("LogEntry", new_id, created_at, root)
    ent.write(target)

    # Update the index in place
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    return {"id": new_id, "path": str(target)}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_events(
    event_type: str | None = None,
    subject: str | None = None,
    actor: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """Query indexed LogEntry events with filters."""
    db = index.open_db()
    try:
        return index.query_events(
            db, event_type=event_type, subject=subject, actor=actor, limit=limit
        )
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def recent_events(limit: int = 20) -> list[dict]:
    """Return the most recent events from the index."""
    db = index.open_db()
    try:
        return index.query_events(db, limit=limit)
    finally:
        db.close()


if __name__ == "__main__":
    server.run(transport="stdio")
