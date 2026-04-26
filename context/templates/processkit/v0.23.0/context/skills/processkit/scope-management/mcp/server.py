#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit scope-management MCP server.

Tools:

    create_scope(name, kind, starts_at?, ends_at?, goals?, description?,
                 parent?)
        -> {id, path, state}

    get_scope(id) -> {id, name, kind, state, ...} | {error}

    transition_scope(id, to_state) -> {ok, from_state, to_state}

    list_scopes(kind?, state?, limit?) -> [scopes]

Scopes have a state machine: planned → active → completed (+ cancelled).
Reactivation from terminal states is intentionally not allowed — create
a new scope so historical references stay stable.
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

from processkit import config, entity, ids, index, log, paths, schema, state_machine  # noqa: E402

server = FastMCP("processkit-scope-management")

_VALID_KINDS = {"sprint", "milestone", "quarter", "project", "release", "other"}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_scope(root: Path, id: str) -> entity.Entity | None:
    scope_dir = paths.context_dir("Scope", root)
    candidate = scope_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Scope")
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
def create_scope(
    name: str,
    kind: str,
    starts_at: str | None = None,
    ends_at: str | None = None,
    goals: list[str] | None = None,
    description: str | None = None,
    parent: str | None = None,
) -> dict:
    """Create a new Scope entity in the planned state.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.

    Parameters
    ----------
    name:        human-readable name (e.g. "Sprint 42")
    kind:        sprint | milestone | quarter | project | release | other
    starts_at:   optional ISO date
    ends_at:     optional ISO date
    goals:       list of concrete outcomes
    description: longer context
    parent:      optional parent SCOPE-id
    """
    if kind not in _VALID_KINDS:
        return {"error": f"invalid kind {kind!r}; must be one of {sorted(_VALID_KINDS)}"}
    if parent is not None and not parent.startswith("SCOPE-"):
        return {"error": f"parent must start with SCOPE-, got {parent!r}"}

    root = paths.find_project_root()
    cfg = config.load_config(root)
    scope_dir = paths.context_dir("Scope", root)
    scope_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Scope")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Scope",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {"name": name, "kind": kind, "state": "planned"}
    if starts_at:
        spec["starts_at"] = starts_at
    if ends_at:
        spec["ends_at"] = ends_at
    if goals:
        spec["goals"] = list(goals)
    if description:
        spec["description"] = description
    if parent:
        spec["parent"] = parent

    errors = schema.validate_spec("Scope", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Scope", new_id, spec)
    target_path = scope_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Scope", new_id, "scope.created",
        f"Created Scope {new_id!r}: {name!r}",
        root=root,
        actor=new_id,
    )
    return {"id": new_id, "path": str(target_path), "state": "planned"}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_scope(id: str) -> dict:
    """Return the full Scope entity by ID."""
    root = paths.find_project_root()
    ent = _load_scope(root, id)
    if ent is None:
        return {"error": f"scope {id!r} not found"}
    return {
        "id": ent.id,
        "name": ent.spec.get("name"),
        "kind": ent.spec.get("kind"),
        "state": ent.spec.get("state"),
        "starts_at": ent.spec.get("starts_at"),
        "ends_at": ent.spec.get("ends_at"),
        "goals": ent.spec.get("goals", []),
        "description": ent.spec.get("description"),
        "parent": ent.spec.get("parent"),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def transition_scope(id: str, to_state: str) -> dict:
    """Transition a Scope to a new state per the scope state machine.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_scope(root, id)
    if ent is None:
        return {"error": f"scope {id!r} not found"}

    from_state = ent.spec.get("state")
    try:
        state_machine.validate_transition("scope", from_state, to_state)
    except state_machine.StateMachineError as e:
        return {"error": str(e)}

    ent.spec["state"] = to_state
    ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Scope", id, "scope.transitioned",
        f"Transitioned Scope {id!r} from {from_state!r} to {to_state!r}",
        root=root,
        actor=id,
    )
    return {"ok": True, "id": id, "from_state": from_state, "to_state": to_state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_scopes(
    kind: str | None = None,
    state: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List Scope entities with optional filters by kind and state."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Scope", state=state, limit=limit * 4)
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
        if kind and spec.get("kind") != kind:
            continue
        out.append({
            "id": full["id"],
            "name": spec.get("name"),
            "kind": spec.get("kind"),
            "state": spec.get("state"),
            "starts_at": spec.get("starts_at"),
            "ends_at": spec.get("ends_at"),
            "path": full.get("path"),
        })
        if len(out) >= limit:
            break
    return out


if __name__ == "__main__":
    server.run(transport="stdio")
