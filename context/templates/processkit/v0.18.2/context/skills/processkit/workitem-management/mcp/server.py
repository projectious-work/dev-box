#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit workitem-management MCP server.

Tools:

    create_workitem(title, type?, priority?, assignee?, description?,
                    parent?, scope?, labels?)
        -> {id, path, state}

    transition_workitem(id, to_state, note?)
        -> {ok, from_state, to_state}

    query_workitems(state?, type?, assignee?, limit?)
        -> [workitems]

    link_workitems(from_id, to_id, relation)
        -> {ok}

    get_workitem(id)
        -> workitem | null
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

from processkit import config, entity, ids, index, paths, log, schema, state_machine  # noqa: E402

server = FastMCP("processkit-workitem-management")

_VALID_TYPES = {"task", "story", "bug", "epic", "spike", "chore"}
_VALID_PRIORITIES = {"critical", "high", "medium", "low"}
_VALID_RELATIONS = {"blocks", "blocked_by", "parent", "children", "related_decisions"}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_workitem(root: Path, id: str) -> entity.Entity | None:
    wi_dir = paths.context_dir("WorkItem", root)
    candidate = wi_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    # Try the index; resolve_entity handles partial IDs and word-pair lookups.
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="WorkItem")
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
def create_workitem(
    title: str,
    type: str = "task",
    priority: str = "medium",
    assignee: str | None = None,
    description: str | None = None,
    parent: str | None = None,
    scope: str | None = None,
    labels: dict | None = None,
) -> dict:
    """Create a new WorkItem in the project's context/workitems/ directory.

    Validates ``type``, ``priority``, and the initial state from the
    workitem state machine. Returns ``{id, path, state}``. Prerequisite:
    call find_skill(task_description) or confirm you are already
    operating within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    if type not in _VALID_TYPES:
        return {"error": f"invalid type {type!r}; must be one of {sorted(_VALID_TYPES)}"}
    if priority not in _VALID_PRIORITIES:
        return {"error": f"invalid priority {priority!r}"}

    root = paths.find_project_root()
    cfg = config.load_config(root)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "WorkItem")
    finally:
        db.close()

    new_id = ids.generate_id(
        "WorkItem",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=title if cfg.id_slug else None,
        existing=existing,
    )

    initial_state = "backlog"
    try:
        sm = state_machine.load("workitem")
        initial_state = sm.initial
    except state_machine.StateMachineError:
        pass

    spec = {
        "title": title,
        "state": initial_state,
        "type": type,
        "priority": priority,
    }
    if assignee:
        spec["assignee"] = assignee
    if description:
        spec["description"] = description
    if parent:
        spec["parent"] = parent
    if scope:
        spec["scope"] = scope

    errors = schema.validate_spec("WorkItem", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("WorkItem", new_id, spec, labels=labels)
    target = paths.entity_path("WorkItem", new_id, None, root)
    ent.write(target)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "WorkItem", new_id, "workitem.created",
        f"Created WorkItem {new_id!r}: {title!r}",
        root=root,
    )
    return {"id": new_id, "path": str(target), "state": initial_state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def transition_workitem(id: str, to_state: str, note: str | None = None) -> dict:
    """Transition a WorkItem to a new state.

    Validates the transition against the workitem state machine. Sets
    ``spec.started_at`` on first entry to ``in-progress`` and
    ``spec.completed_at`` on entering a terminal state. Prerequisite:
    call find_skill(task_description) or confirm you are already
    operating within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_workitem(root, id)
    if ent is None:
        return {"error": f"workitem {id!r} not found"}
    from_state = ent.spec.get("state")
    try:
        state_machine.validate_transition("workitem", from_state, to_state)
    except state_machine.StateMachineError as e:
        return {"error": str(e)}

    ent.spec["state"] = to_state
    if to_state == "in-progress" and not ent.spec.get("started_at"):
        ent.spec["started_at"] = _now_iso()
    try:
        sm = state_machine.load("workitem")
        if sm.is_terminal(to_state):
            ent.spec["completed_at"] = _now_iso()
    except state_machine.StateMachineError:
        pass

    if note:
        ent.body = (ent.body or "") + f"\n\n## Transition note ({_now_iso()})\n\n{note}\n"

    ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "WorkItem", id, "workitem.transitioned",
        f"Transitioned WorkItem {id!r} from {from_state!r} to {to_state!r}",
        root=root,
    )
    return {"ok": True, "id": id, "from_state": from_state, "to_state": to_state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_workitems(
    state: str | None = None,
    type: str | None = None,
    assignee: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """Query WorkItems via the index. Filters: state, type, assignee."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="WorkItem", state=state, limit=limit * 4)
    finally:
        db.close()
    if type:
        rows = [r for r in rows if (r.get("title") and True)]  # placeholder, type filter applied below
    out = []
    for r in rows:
        full = get_workitem(r["id"])
        if full is None or "error" in full:
            continue
        if type and full.get("type") != type:
            continue
        if assignee and full.get("assignee") != assignee:
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
def get_workitem(id: str) -> dict:
    """Fetch a WorkItem by ID with its full spec.

    Accepts a full ID, a prefix (missing slug), or a bare word-pair.
    Returns ``{"error": "..."}`` if not found or ambiguous.
    """
    db = index.open_db()
    try:
        row, candidates = index.resolve_entity(db, id, kind="WorkItem")
    finally:
        db.close()
    if candidates:
        return {"error": f"ambiguous ID {id!r}; candidates: {candidates}"}
    if row is None:
        return {"error": f"WorkItem not found: {id!r}"}
    spec = row.get("spec", {})
    return {
        "id": row["id"],
        "title": row.get("title"),
        "state": row.get("state"),
        "type": spec.get("type"),
        "priority": spec.get("priority"),
        "assignee": spec.get("assignee"),
        "scope": spec.get("scope"),
        "parent": spec.get("parent"),
        "blocks": spec.get("blocks", []),
        "blocked_by": spec.get("blocked_by", []),
        "related_decisions": spec.get("related_decisions", []),
        "path": row.get("path"),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def link_workitems(from_id: str, to_id: str, relation: str) -> dict:
    """Add a typed cross-reference between two WorkItems.

    ``relation`` must be one of: blocks, blocked_by, parent, children,
    related_decisions. Bidirectional relations (blocks/blocked_by,
    parent/children) are NOT auto-mirrored — call link_workitems on
    both sides if you want them. Prerequisite: call
    find_skill(task_description) or confirm you are already operating
    within a named processkit skill before using this tool.
    """
    if relation not in _VALID_RELATIONS:
        return {"error": f"invalid relation {relation!r}; valid: {sorted(_VALID_RELATIONS)}"}
    root = paths.find_project_root()
    ent = _load_workitem(root, from_id)
    if ent is None:
        return {"error": f"workitem {from_id!r} not found"}
    if relation == "parent":
        ent.spec["parent"] = to_id
    else:
        existing = ent.spec.get(relation, []) or []
        if to_id not in existing:
            existing.append(to_id)
        ent.spec[relation] = existing
    ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()
    return {"ok": True, "from": from_id, "to": to_id, "relation": relation}


if __name__ == "__main__":
    server.run(transport="stdio")
