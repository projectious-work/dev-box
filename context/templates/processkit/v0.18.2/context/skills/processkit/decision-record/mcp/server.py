#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit decision-record MCP server.

Tools:

    record_decision(title, decision, rationale?, context?, alternatives?,
                    consequences?, deciders?, related_workitems?, state?)
        -> {id, path, state}

    transition_decision(id, to_state)
        -> {ok, from_state, to_state}

    query_decisions(state?, limit?)
        -> [decisions]

    get_decision(id)
        -> decision | null

    supersede_decision(old_id, new_id)
        -> {ok}

    link_decision_to_workitem(decision_id, workitem_id)
        -> {ok}
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

server = FastMCP("processkit-decision-record")


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_decision(root: Path, id: str) -> entity.Entity | None:
    dec_dir = paths.context_dir("DecisionRecord", root)
    candidate = dec_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="DecisionRecord")
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
def record_decision(
    title: str,
    decision: str,
    rationale: str | None = None,
    context: str | None = None,
    alternatives: list[dict] | None = None,
    consequences: str | None = None,
    deciders: list[str] | None = None,
    related_workitems: list[str] | None = None,
    state: str = "proposed",
) -> dict:
    """Create a new DecisionRecord.

    The default state is ``proposed``. Set ``state="accepted"`` to record
    an already-finalized decision. Prerequisite: call
    find_skill(task_description) or confirm you are already operating
    within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    root = paths.find_project_root()
    cfg = config.load_config(root)
    dec_dir = paths.context_dir("DecisionRecord", root)
    dec_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "DecisionRecord")
    finally:
        db.close()

    new_id = ids.generate_id(
        "DecisionRecord",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=title if cfg.id_slug else None,
        existing=existing,
    )

    spec = {
        "title": title,
        "state": state,
        "decision": decision,
    }
    if context:
        spec["context"] = context
    if rationale:
        spec["rationale"] = rationale
    if alternatives:
        spec["alternatives"] = alternatives
    if consequences:
        spec["consequences"] = consequences
    if deciders:
        spec["deciders"] = deciders
    if related_workitems:
        spec["related_workitems"] = related_workitems
    if state == "accepted":
        spec["decided_at"] = _now_iso()

    errors = schema.validate_spec("DecisionRecord", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("DecisionRecord", new_id, spec)
    target = dec_dir / f"{new_id}.md"
    ent.write(target)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "DecisionRecord", new_id, "decision.created",
        f"Created DecisionRecord {new_id!r}: {title!r}",
        root=root,
    )
    return {"id": new_id, "path": str(target), "state": state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def transition_decision(id: str, to_state: str) -> dict:
    """Transition a DecisionRecord to a new state.

    Validates against the decisionrecord state machine. Prerequisite:
    call find_skill(task_description) or confirm you are already
    operating within a named processkit skill before using this tool.
    """
    root = paths.find_project_root()
    ent = _load_decision(root, id)
    if ent is None:
        return {"error": f"decision {id!r} not found"}
    from_state = ent.spec.get("state")
    try:
        state_machine.validate_transition("decisionrecord", from_state, to_state)
    except state_machine.StateMachineError as e:
        return {"error": str(e)}

    ent.spec["state"] = to_state
    if to_state == "accepted" and not ent.spec.get("decided_at"):
        ent.spec["decided_at"] = _now_iso()
    ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "DecisionRecord", id, "decision.transitioned",
        f"Transitioned DecisionRecord {id!r} from {from_state!r} to {to_state!r}",
        root=root,
    )
    return {"ok": True, "from_state": from_state, "to_state": to_state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_decisions(state: str | None = None, limit: int = 50) -> list[dict]:
    """List DecisionRecords with optional state filter."""
    db = index.open_db()
    try:
        return index.query_entities(db, kind="DecisionRecord", state=state, limit=limit)
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_decision(id: str) -> dict:
    """Fetch a DecisionRecord by ID.

    Accepts a full ID, a prefix (missing slug), or a bare word-pair.
    Returns ``{"error": "..."}`` if not found or ambiguous.
    """
    db = index.open_db()
    try:
        row, candidates = index.resolve_entity(db, id, kind="DecisionRecord")
    finally:
        db.close()
    if candidates:
        return {"error": f"ambiguous ID {id!r}; candidates: {candidates}"}
    if row is None:
        return {"error": f"DecisionRecord not found: {id!r}"}
    spec = row.get("spec", {})
    return {
        "id": row["id"],
        "title": row.get("title"),
        "state": row.get("state"),
        "decision": spec.get("decision"),
        "context": spec.get("context"),
        "rationale": spec.get("rationale"),
        "alternatives": spec.get("alternatives", []),
        "consequences": spec.get("consequences"),
        "deciders": spec.get("deciders", []),
        "supersedes": spec.get("supersedes"),
        "superseded_by": spec.get("superseded_by"),
        "related_workitems": spec.get("related_workitems", []),
        "decided_at": spec.get("decided_at"),
        "path": row.get("path"),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=True,
    openWorldHint=False,
))
def supersede_decision(old_id: str, new_id: str) -> dict:
    """Mark ``old_id`` as superseded by ``new_id``.

    Updates both DecisionRecords: the old one transitions to
    ``superseded`` and gets ``superseded_by``; the new one gets
    ``supersedes``. Prerequisite: call find_skill(task_description) or
    confirm you are already operating within a named processkit skill
    before using this tool.
    """
    root = paths.find_project_root()
    old = _load_decision(root, old_id)
    new = _load_decision(root, new_id)
    if old is None:
        return {"error": f"decision {old_id!r} not found"}
    if new is None:
        return {"error": f"decision {new_id!r} not found"}
    try:
        state_machine.validate_transition("decisionrecord", old.spec.get("state"), "superseded")
    except state_machine.StateMachineError as e:
        return {"error": str(e)}
    old.spec["state"] = "superseded"
    old.spec["superseded_by"] = new_id
    new.spec["supersedes"] = old_id
    old.write()
    new.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, old)
        index.upsert_entity(db, new)
    finally:
        db.close()

    log.log_side_effect(
        "DecisionRecord", old_id, "decision.superseded",
        f"DecisionRecord {old_id!r} superseded by {new_id!r}",
        root=root,
    )
    return {"ok": True, "old_id": old_id, "new_id": new_id}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def link_decision_to_workitem(decision_id: str, workitem_id: str) -> dict:
    """Add ``workitem_id`` to ``decision.spec.related_workitems``.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool.
    """
    root = paths.find_project_root()
    ent = _load_decision(root, decision_id)
    if ent is None:
        return {"error": f"decision {decision_id!r} not found"}
    related = ent.spec.get("related_workitems", []) or []
    if workitem_id not in related:
        related.append(workitem_id)
    ent.spec["related_workitems"] = related
    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()
    return {"ok": True}


if __name__ == "__main__":
    server.run(transport="stdio")
