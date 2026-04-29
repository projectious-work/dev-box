#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit actor-profile MCP server.

Tools:

    create_actor(name, type, email?, handle?, expertise?, roles?,
                 preferences?, joined_at?)
        -> {id, path}

    get_actor(id)
        -> {id, type, name, ...} | {error}

    update_actor(id, **fields)
        -> {ok, id, updated}

    deactivate_actor(id, left_at?)
        -> {ok, id, left_at}

    list_actors(type?, active_only?, expertise?, limit?)
        -> [actors]

Actors are descriptive, not restrictive. Roles listed on the Actor are
project-wide; for scoped role assignments use a Binding via
binding-management.
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

from processkit import config, entity, ids, index, log, paths, schema  # noqa: E402

server = FastMCP("processkit-actor-profile")

_VALID_TYPES = {"human", "ai-agent", "service"}
_UPDATABLE_FIELDS = {
    "name", "email", "handle", "expertise", "roles",
    "preferences", "active", "joined_at", "left_at",
}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _today_iso() -> str:
    return _dt.date.today().isoformat()


def _load_actor(root: Path, id: str) -> entity.Entity | None:
    actor_dir = paths.context_dir("Actor", root)
    candidate = actor_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Actor")
        if row and row.get("path"):
            return entity.load(row["path"])
    finally:
        db.close()
    return None


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_actor(
    name: str,
    type: str,
    email: str | None = None,
    handle: str | None = None,
    expertise: list[str] | None = None,
    roles: list[str] | None = None,
    preferences: dict | None = None,
    joined_at: str | None = None,
) -> dict:
    """Create a new Actor entity.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.

    Parameters
    ----------
    name:        display name
    type:        "human", "ai-agent", or "service"
    email:       optional, humans only
    handle:      optional GitHub/Slack handle
    expertise:   list of tag strings
    roles:       list of unscoped Role IDs (use Binding for scoped)
    preferences: free-form per-actor preferences
    joined_at:   ISO datetime; defaults to now
    """
    if type not in _VALID_TYPES:
        return {"error": f"invalid type {type!r}; must be one of {sorted(_VALID_TYPES)}"}

    root = paths.find_project_root()
    cfg = config.load_config(root)
    actor_dir = paths.context_dir("Actor", root)
    actor_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Actor")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Actor",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {
        "type": type,
        "name": name,
        "active": True,
        "joined_at": joined_at or _now_iso(),
    }
    if email:
        spec["email"] = email
    if handle:
        spec["handle"] = handle
    if expertise:
        spec["expertise"] = list(expertise)
    if roles:
        spec["roles"] = list(roles)
    if preferences:
        spec["preferences"] = dict(preferences)

    errors = schema.validate_spec("Actor", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Actor", new_id, spec)
    target_path = actor_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    # Emit the actor.created LogEntry AFTER the Actor is persisted, using
    # the newly-created Actor's own ID as the actor (self-attribution) so
    # the emitted LogEntry satisfies the LogEntry schema's required
    # `spec.actor` field. The actor that was just created IS logically the
    # subject — the tool acted in its name.
    log.log_side_effect(
        "Actor", new_id, "actor.created",
        f"Created Actor {new_id!r}: {name!r}",
        root=root,
        actor=new_id,
    )
    return {"id": new_id, "path": str(target_path), "type": type, "name": name}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_actor(id: str) -> dict:
    """Return the full Actor entity by ID."""
    root = paths.find_project_root()
    ent = _load_actor(root, id)
    if ent is None:
        return {"error": f"actor {id!r} not found"}
    return {
        "id": ent.id,
        "type": ent.spec.get("type"),
        "name": ent.spec.get("name"),
        "email": ent.spec.get("email"),
        "handle": ent.spec.get("handle"),
        "expertise": ent.spec.get("expertise", []),
        "roles": ent.spec.get("roles", []),
        "preferences": ent.spec.get("preferences", {}),
        "active": ent.spec.get("active", True),
        "joined_at": ent.spec.get("joined_at"),
        "left_at": ent.spec.get("left_at"),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def update_actor(
    id: str,
    name: str | None = None,
    email: str | None = None,
    handle: str | None = None,
    expertise: list[str] | None = None,
    roles: list[str] | None = None,
    preferences: dict | None = None,
    active: bool | None = None,
) -> dict:
    """Update fields on an existing Actor. Only supplied fields change.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_actor(root, id)
    if ent is None:
        return {"error": f"actor {id!r} not found"}

    updated: list[str] = []
    if name is not None:
        ent.spec["name"] = name
        updated.append("name")
    if email is not None:
        ent.spec["email"] = email
        updated.append("email")
    if handle is not None:
        ent.spec["handle"] = handle
        updated.append("handle")
    if expertise is not None:
        ent.spec["expertise"] = list(expertise)
        updated.append("expertise")
    if roles is not None:
        ent.spec["roles"] = list(roles)
        updated.append("roles")
    if preferences is not None:
        ent.spec["preferences"] = dict(preferences)
        updated.append("preferences")
    if active is not None:
        ent.spec["active"] = bool(active)
        updated.append("active")

    if not updated:
        return {"ok": True, "id": id, "updated": []}

    errors = schema.validate_spec("Actor", ent.spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()
    return {"ok": True, "id": id, "updated": updated}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=True,
    openWorldHint=False,
))
def deactivate_actor(id: str, left_at: str | None = None) -> dict:
    """Mark an Actor inactive. Sets active=false and left_at.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_actor(root, id)
    if ent is None:
        return {"error": f"actor {id!r} not found"}
    ent.spec["active"] = False
    ent.spec["left_at"] = left_at or _now_iso()
    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Actor", id, "actor.deactivated",
        f"Deactivated Actor {id!r}",
        root=root,
        actor=id,
    )
    return {"ok": True, "id": id, "left_at": ent.spec["left_at"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_actors(
    type: str | None = None,
    active_only: bool = True,
    expertise: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List Actors with optional filters by type, active state, expertise tag."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Actor", limit=limit * 4)
    finally:
        db.close()
    out: list[dict] = []
    for r in rows:
        db = index.open_db()
        try:
            full = index.get_entity(db, r["id"])
        finally:
            db.close()
        if not full:
            continue
        spec = full.get("spec", {})
        if type and spec.get("type") != type:
            continue
        if active_only and not spec.get("active", True):
            continue
        if expertise and expertise not in (spec.get("expertise") or []):
            continue
        out.append({
            "id": full["id"],
            "type": spec.get("type"),
            "name": spec.get("name"),
            "expertise": spec.get("expertise", []),
            "active": spec.get("active", True),
            "path": full.get("path"),
        })
        if len(out) >= limit:
            break
    return out


if __name__ == "__main__":
    server.run(transport="stdio")
