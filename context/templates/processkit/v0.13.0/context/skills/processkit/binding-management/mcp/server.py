#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit binding-management MCP server.

Tools:

    create_binding(type, subject, target, scope?, valid_from?, valid_until?,
                   conditions?, description?)
        -> {id, path}

    end_binding(id, end_date?)
        -> {ok}

    query_bindings(type?, subject?, target?, scope?, active_only?, limit?)
        -> [bindings]

    resolve_bindings_for(entity_id, type?, at_time?)
        -> [bindings]
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

from processkit import config, entity, ids, index, log, paths  # noqa: E402

server = FastMCP("processkit-binding-management")


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _today_iso() -> str:
    return _dt.date.today().isoformat()


def _load_binding(root: Path, id: str) -> entity.Entity | None:
    bind_dir = paths.context_dir("Binding", root)
    candidate = bind_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row = index.get_entity(db, id)
        if row and row.get("path"):
            return entity.load(row["path"])
    finally:
        db.close()
    return None


def _is_active(spec: dict, at: str | None = None) -> bool:
    """Return True if a binding is active at the given date (default today)."""
    at = at or _today_iso()
    valid_from = spec.get("valid_from")
    valid_until = spec.get("valid_until")
    if valid_from and str(valid_from) > at:
        return False
    if valid_until and str(valid_until) < at:
        return False
    return True


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_binding(
    type: str,
    subject: str,
    target: str,
    scope: str | None = None,
    valid_from: str | None = None,
    valid_until: str | None = None,
    conditions: dict | None = None,
    description: str | None = None,
) -> dict:
    """Create a new Binding entity.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool.

    Parameters
    ----------
    type:        binding type (e.g. "role-assignment", "process-gate")
    subject:    ID of the "from" entity
    target:     ID of the "to" entity
    scope:      optional scope ID limiting where the binding applies
    valid_from: optional ISO date when the binding starts
    valid_until: optional ISO date when the binding ends
    conditions: optional freeform conditions object
    description: optional one-line summary
    """
    root = paths.find_project_root()
    cfg = config.load_config(root)
    bind_dir = paths.context_dir("Binding", root)
    bind_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Binding")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Binding",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=type if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {
        "type": type,
        "subject": subject,
        "target": target,
    }
    if scope:
        spec["scope"] = scope
    if valid_from:
        spec["valid_from"] = valid_from
    if valid_until:
        spec["valid_until"] = valid_until
    if conditions:
        spec["conditions"] = conditions
    if description:
        spec["description"] = description

    ent = entity.new("Binding", new_id, spec)
    target_path = bind_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Binding", new_id, "binding.created",
        f"Created Binding {new_id!r}: {type!r} {subject!r} → {target!r}",
        root=root,
    )
    return {"id": new_id, "path": str(target_path)}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def end_binding(id: str, end_date: str | None = None) -> dict:
    """End a Binding by setting ``valid_until`` to ``end_date`` (default today).

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool.
    """
    root = paths.find_project_root()
    ent = _load_binding(root, id)
    if ent is None:
        return {"error": f"binding {id!r} not found"}
    ent.spec["valid_until"] = end_date or _today_iso()
    ent.write()
    root = paths.find_project_root()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Binding", id, "binding.ended",
        f"Ended Binding {id!r} (valid_until={ent.spec['valid_until']!r})",
        root=root,
    )
    return {"ok": True, "id": id, "valid_until": ent.spec["valid_until"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_bindings(
    type: str | None = None,
    subject: str | None = None,
    target: str | None = None,
    scope: str | None = None,
    active_only: bool = True,
    limit: int = 50,
) -> list[dict]:
    """Query Bindings with optional filters."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Binding", limit=limit * 4)
    finally:
        db.close()
    out = []
    for r in rows:
        full = _full_binding(r["id"])
        if not full:
            continue
        spec = full
        if type and spec.get("type") != type:
            continue
        if subject and spec.get("subject") != subject:
            continue
        if target and spec.get("target") != target:
            continue
        if scope and spec.get("scope") != scope:
            continue
        if active_only and not _is_active(spec):
            continue
        out.append(full)
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def resolve_bindings_for(
    entity_id: str,
    type: str | None = None,
    at_time: str | None = None,
) -> list[dict]:
    """Find all Bindings whose ``subject`` or ``target`` is ``entity_id``.

    Returns only bindings active at ``at_time`` (default today).
    """
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Binding", limit=200)
    finally:
        db.close()
    out = []
    for r in rows:
        full = _full_binding(r["id"])
        if not full:
            continue
        if full.get("subject") != entity_id and full.get("target") != entity_id:
            continue
        if type and full.get("type") != type:
            continue
        if not _is_active(full, at_time):
            continue
        out.append(full)
    return out


def _full_binding(id: str) -> dict | None:
    db = index.open_db()
    try:
        row = index.get_entity(db, id)
    finally:
        db.close()
    if not row or row.get("kind") != "Binding":
        return None
    spec = row.get("spec", {})
    return {
        "id": row["id"],
        "type": spec.get("type"),
        "subject": spec.get("subject"),
        "target": spec.get("target"),
        "scope": spec.get("scope"),
        "valid_from": spec.get("valid_from"),
        "valid_until": spec.get("valid_until"),
        "conditions": spec.get("conditions"),
        "description": spec.get("description"),
        "path": row.get("path"),
    }


if __name__ == "__main__":
    server.run(transport="stdio")
