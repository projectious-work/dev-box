"""SQLite indexer for processkit entity files.

This module powers the ``index-management`` MCP server. It walks the
project's ``context/`` directory, parses every entity file, and writes a
SQLite database that other tools can query.

The schema is intentionally simple — one row per entity, with the most
common fields denormalized into columns and the full spec stashed as
JSON for queries we did not anticipate.
"""
from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

from . import entity as entity_mod
from . import paths


SCHEMA_DDL = """
CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    api_version TEXT NOT NULL,
    path TEXT NOT NULL,
    created TEXT NOT NULL,
    updated TEXT,
    title TEXT,
    state TEXT,
    labels_json TEXT,
    spec_json TEXT NOT NULL,
    body TEXT
);
CREATE INDEX IF NOT EXISTS idx_entities_kind ON entities (kind);
CREATE INDEX IF NOT EXISTS idx_entities_state ON entities (state);
CREATE INDEX IF NOT EXISTS idx_entities_created ON entities (created);

CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    actor TEXT,
    subject TEXT,
    subject_kind TEXT,
    summary TEXT,
    details_json TEXT,
    correlation_id TEXT,
    path TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_type ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_subject ON events (subject);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp);

CREATE TABLE IF NOT EXISTS errors (
    path TEXT PRIMARY KEY,
    message TEXT NOT NULL
);
"""


@dataclass
class IndexStats:
    entities: int
    events: int
    errors: int


def open_db(path: Path | None = None) -> sqlite3.Connection:
    db_path = Path(path) if path else paths.index_db_path()
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    # WAL mode lets multiple MCP servers read while one writes without
    # hitting "database is locked". Persists across connections — only
    # needs to be set once per database file, but PRAGMA is cheap to
    # re-issue. synchronous=NORMAL is the WAL-recommended pairing.
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=NORMAL")
    conn.executescript(SCHEMA_DDL)
    return conn


def reindex(root: Path | None = None, db: sqlite3.Connection | None = None) -> IndexStats:
    """Walk ``<root>/context/`` and rebuild the index from scratch."""
    root = root or paths.find_project_root()
    own = db is None
    if db is None:
        db = open_db()
    db.execute("DELETE FROM entities")
    db.execute("DELETE FROM events")
    db.execute("DELETE FROM errors")
    n_entities = 0
    n_events = 0
    n_errors = 0
    context_dir = root / "context"
    if not context_dir.is_dir():
        if own:
            db.commit()
            db.close()
        return IndexStats(0, 0, 0)
    for path in sorted(context_dir.rglob("*.md")):
        try:
            ent = entity_mod.load(path)
        except entity_mod.NotAnEntityError:
            # Not a processkit entity (no frontmatter, or no apiVersion key).
            # Silently skip — README, INDEX, SKILL.md, SERVER.md, etc.
            continue
        except entity_mod.EntityError as e:
            # Claims to be a processkit entity (has apiVersion) but is malformed.
            db.execute(
                "INSERT OR REPLACE INTO errors (path, message) VALUES (?, ?)",
                (str(path), str(e)),
            )
            n_errors += 1
            continue
        _insert_entity(db, ent)
        n_entities += 1
        if ent.kind == "LogEntry":
            _insert_event(db, ent)
            n_events += 1
    db.commit()
    if own:
        db.close()
    return IndexStats(n_entities, n_events, n_errors)


def _insert_entity(db: sqlite3.Connection, ent) -> None:
    spec = ent.spec or {}
    db.execute(
        """
        INSERT OR REPLACE INTO entities
        (id, kind, api_version, path, created, updated, title, state, labels_json, spec_json, body)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            ent.id,
            ent.kind,
            ent.apiVersion,
            str(ent.path) if ent.path else "",
            str(ent.created),
            str(ent.updated) if ent.updated else None,
            spec.get("title") or spec.get("name"),
            spec.get("state"),
            json.dumps(ent.labels) if ent.labels else None,
            json.dumps(spec, default=str),
            ent.body or None,
        ),
    )


def _insert_event(db: sqlite3.Connection, ent) -> None:
    spec = ent.spec or {}
    db.execute(
        """
        INSERT OR REPLACE INTO events
        (id, timestamp, event_type, actor, subject, subject_kind, summary, details_json, correlation_id, path)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            ent.id,
            str(spec.get("timestamp", ent.created)),
            spec.get("event_type", ""),
            spec.get("actor"),
            spec.get("subject"),
            spec.get("subject_kind"),
            spec.get("summary"),
            json.dumps(spec.get("details", {}), default=str),
            spec.get("correlation_id"),
            str(ent.path) if ent.path else "",
        ),
    )


def upsert_entity(db: sqlite3.Connection, ent) -> None:
    """Insert or update a single entity (used after a write)."""
    _insert_entity(db, ent)
    if ent.kind == "LogEntry":
        _insert_event(db, ent)
    db.commit()


def existing_ids(db: sqlite3.Connection, kind: str) -> set[str]:
    """Return all entity IDs of the given kind that the index currently knows.

    Used by id-management for collision avoidance during ID generation.
    Assumes the index is fresh — every MCP server upserts the index in
    place after each write, so this is true in normal operation. If
    files have been edited outside the MCP path, call ``reindex()``
    first.
    """
    rows = db.execute(
        "SELECT id FROM entities WHERE kind = ?",
        (kind,),
    ).fetchall()
    return {r["id"] for r in rows}


def query_entities(
    db: sqlite3.Connection,
    *,
    kind: str | None = None,
    state: str | None = None,
    limit: int = 50,
) -> list[dict[str, Any]]:
    sql = "SELECT id, kind, title, state, created, updated, path FROM entities WHERE 1=1"
    params: list[Any] = []
    if kind:
        sql += " AND kind = ?"
        params.append(kind)
    if state:
        sql += " AND state = ?"
        params.append(state)
    sql += " ORDER BY created DESC LIMIT ?"
    params.append(int(limit))
    return [dict(row) for row in db.execute(sql, params).fetchall()]


def get_entity(db: sqlite3.Connection, id: str) -> dict[str, Any] | None:
    row = db.execute(
        "SELECT id, kind, title, state, created, updated, path, spec_json, labels_json FROM entities WHERE id = ?",
        (id,),
    ).fetchone()
    if not row:
        return None
    out = dict(row)
    if out.get("spec_json"):
        try:
            out["spec"] = json.loads(out["spec_json"])
        except Exception:
            pass
    if out.get("labels_json"):
        try:
            out["labels"] = json.loads(out["labels_json"])
        except Exception:
            pass
    return out


def resolve_entity(
    db: sqlite3.Connection,
    partial_id: str,
    *,
    kind: str | None = None,
) -> tuple[dict[str, Any] | None, list[str]]:
    """Resolve a partial or word-pair ID to a single entity.

    Agents and humans often refer to entities by a shortened form — either
    omitting the slug suffix or using just the word-pair component. This
    function tries three resolution strategies in order:

    1. **Exact match** — ``id = partial_id``
    2. **Prefix match** — ``id LIKE '{partial_id}-%'`` (handles a missing slug)
    3. **Word-pair match** — ``id LIKE '%-{partial_id}-%' OR id LIKE
       '%-{partial_id}'`` (handles a bare word-pair like ``StoutCrow``)

    Returns ``(row, candidates)`` where:

    - ``(row, [])``   — exactly one match; ``row`` is the full entity dict
    - ``(None, [])``  — no match at all
    - ``(None, ids)`` — ambiguous; ``ids`` lists all matching entity IDs
    """

    def _escape_like(s: str) -> str:
        # Escape LIKE metacharacters so literal underscores in IDs (e.g.
        # "20260410_1050") are not treated as single-character wildcards.
        return s.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_")

    def _fetch(where: str, params: list[Any]) -> list[dict[str, Any]]:
        sql = (
            "SELECT id, kind, title, state, created, updated, path,"
            " spec_json, labels_json FROM entities WHERE " + where
        )
        if kind:
            sql += " AND kind = ?"
            params = list(params) + [kind]
        rows = db.execute(sql, params).fetchall()
        result = []
        for row in rows:
            out = dict(row)
            if out.get("spec_json"):
                try:
                    out["spec"] = json.loads(out["spec_json"])
                except Exception:
                    pass
            result.append(out)
        return result

    # 1. Exact match.
    rows = _fetch("id = ?", [partial_id])
    if rows:
        return rows[0], []

    # 2. Prefix match — handles a missing slug suffix, e.g.
    #    "BACK-20260410_1050-StoutCrow" → "BACK-20260410_1050-StoutCrow-create-..."
    escaped = _escape_like(partial_id)
    rows = _fetch("id LIKE ? ESCAPE '\\'", [escaped + "-%"])
    if len(rows) == 1:
        return rows[0], []
    if len(rows) > 1:
        return None, [r["id"] for r in rows]

    # 3. Word-pair / substring match — handles a bare word-pair, e.g.
    #    "StoutCrow" → "BACK-20260410_1050-StoutCrow-create-brand-design-skill"
    rows = _fetch(
        "(id LIKE ? ESCAPE '\\' OR id LIKE ? ESCAPE '\\')",
        ["%-" + escaped + "-%", "%-" + escaped],
    )
    if len(rows) == 1:
        return rows[0], []
    if len(rows) > 1:
        return None, [r["id"] for r in rows]

    return None, []


def search_entities(
    db: sqlite3.Connection,
    text: str,
    *,
    limit: int = 50,
) -> list[dict[str, Any]]:
    like = f"%{text}%"
    rows = db.execute(
        """
        SELECT id, kind, title, state, created, path
        FROM entities
        WHERE id LIKE ? OR title LIKE ? OR body LIKE ? OR spec_json LIKE ?
        ORDER BY created DESC LIMIT ?
        """,
        (like, like, like, like, int(limit)),
    ).fetchall()
    return [dict(r) for r in rows]


def query_events(
    db: sqlite3.Connection,
    *,
    event_type: str | None = None,
    subject: str | None = None,
    actor: str | None = None,
    limit: int = 50,
) -> list[dict[str, Any]]:
    sql = "SELECT id, timestamp, event_type, actor, subject, summary FROM events WHERE 1=1"
    params: list[Any] = []
    if event_type:
        sql += " AND event_type = ?"
        params.append(event_type)
    if subject:
        sql += " AND subject = ?"
        params.append(subject)
    if actor:
        sql += " AND actor = ?"
        params.append(actor)
    sql += " ORDER BY timestamp DESC LIMIT ?"
    params.append(int(limit))
    return [dict(row) for row in db.execute(sql, params).fetchall()]


def list_errors(db: sqlite3.Connection) -> list[dict[str, Any]]:
    return [dict(r) for r in db.execute("SELECT path, message FROM errors").fetchall()]
