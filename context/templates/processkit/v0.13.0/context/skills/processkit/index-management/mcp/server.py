#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit index-management MCP server.

Walks the consumer project's ``context/`` directory, parses every entity
file, and writes a SQLite index that other MCP servers (and the agent)
can query.

Tools provided:
    reindex()                              -> stats
    query_entities(kind?, state?, limit?)  -> [entities]
    get_entity(id)                         -> entity | null
    search_entities(text, limit?)          -> [entities]
    query_events(event_type?, subject?, actor?, limit?) -> [events]
    list_errors()                          -> [{path, message}]
    stats()                                -> counts

Run from a processkit checkout (development) or installed by aibox into
``context/skills/index-management/mcp/server.py`` (consumer install,
provider-neutral path).

Lib resolution: walks up from this file to find ``src/lib/processkit/``
(checkout) or ``_lib/processkit/`` (consumer-install layout — resolves
to ``context/skills/_lib/processkit/`` after the walk-up).
"""
from __future__ import annotations

import os
import sys
from pathlib import Path


def _find_lib() -> Path:
    """Locate the processkit shared library directory.

    Search order:
      1. PROCESSKIT_LIB_PATH environment variable
      2. <ancestor>/src/lib/processkit/__init__.py (running from
         a processkit checkout)
      3. <ancestor>/_lib/processkit/__init__.py    (consumer install,
         e.g. <project>/context/skills/_lib/processkit/__init__.py
         when invoked as <project>/context/skills/<name>/mcp/server.py)
    """
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for candidate in (
            here / "src" / "lib",
            here / "_lib",
        ):
            if (candidate / "processkit" / "__init__.py").is_file():
                return candidate
        if here.parent == here:
            raise RuntimeError(
                "could not locate processkit shared library "
                "(set PROCESSKIT_LIB_PATH or run from a processkit checkout)"
            )
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import index, paths  # noqa: E402

server = FastMCP("processkit-index-management")


def _open() -> tuple[Path, "index.sqlite3.Connection"]:  # noqa: F821
    root = paths.find_project_root()
    db = index.open_db()
    return root, db


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def reindex() -> dict:
    """Rebuild the SQLite index from scratch.

    Walks ``<project-root>/context/`` recursively, parses every Markdown
    file with YAML frontmatter, and replaces the index contents. Returns
    counts of entities, events, and parse errors.
    """
    root, db = _open()
    stats = index.reindex(root, db)
    db.close()
    return {
        "entities": stats.entities,
        "events": stats.events,
        "errors": stats.errors,
        "project_root": str(root),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_entities(
    kind: str | None = None,
    state: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List entities matching the filters.

    Parameters
    ----------
    kind: optional primitive kind (e.g. "WorkItem", "DecisionRecord").
    state: optional state to filter by.
    limit: maximum rows to return (default 50).
    """
    _, db = _open()
    try:
        return index.query_entities(db, kind=kind, state=state, limit=limit)
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_entity(id: str) -> dict:
    """Fetch a single entity by ID.

    Accepts a full ID, a prefix (missing slug), or a bare word-pair.
    Returns ``{"error": "..."}`` if not found or ambiguous.
    """
    _, db = _open()
    try:
        row, candidates = index.resolve_entity(db, id)
    finally:
        db.close()
    if candidates:
        return {"error": f"ambiguous ID {id!r}; candidates: {candidates}"}
    if row is None:
        return {"error": f"entity not found: {id!r}"}
    return row


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def search_entities(text: str, limit: int = 50) -> list[dict]:
    """Full-text search across entity IDs, titles, bodies, and specs."""
    _, db = _open()
    try:
        return index.search_entities(db, text, limit=limit)
    finally:
        db.close()


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
    """Query the LogEntry events table."""
    _, db = _open()
    try:
        return index.query_events(
            db,
            event_type=event_type,
            subject=subject,
            actor=actor,
            limit=limit,
        )
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_errors() -> list[dict]:
    """Return files that failed to parse during the last reindex."""
    _, db = _open()
    try:
        return index.list_errors(db)
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def stats() -> dict:
    """Return counts of indexed entities/events/errors."""
    _, db = _open()
    try:
        rows = {
            "entities": db.execute("SELECT COUNT(*) FROM entities").fetchone()[0],
            "events": db.execute("SELECT COUNT(*) FROM events").fetchone()[0],
            "errors": db.execute("SELECT COUNT(*) FROM errors").fetchone()[0],
        }
        return rows
    finally:
        db.close()


if __name__ == "__main__":
    server.run(transport="stdio")
