#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit discussion-management MCP server.

Tools:

    open_discussion(question, participants?, related?, body?)
        -> {id, path, state}

    get_discussion(id)
        -> {id, question, state, ...} | {error}

    transition_discussion(id, to_state)
        -> {ok, from_state, to_state}

    add_outcome(id, decision_id)
        -> {ok, outcomes}

    list_discussions(state?, limit?) -> [discussions]

Discussions have a state machine: active ↔ resolved → archived.
Reopening a resolved discussion is intentional. Use a Discussion when
you don't yet know the answer; use a DecisionRecord when you do.
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

server = FastMCP("processkit-discussion-management")


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_discussion(root: Path, id: str) -> entity.Entity | None:
    disc_dir = paths.context_dir("Discussion", root)
    candidate = disc_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Discussion")
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
def open_discussion(
    question: str,
    participants: list[str] | None = None,
    related: list[str] | None = None,
    body: str | None = None,
) -> dict:
    """Open a new Discussion in the active state.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn — deferred writes are dropped.

    Parameters
    ----------
    question:     the driving question, one crisp sentence
    participants: ACTOR-ids of participants
    related:     other DISC-ids related to this discussion
    body:         optional initial Markdown body
    """
    root = paths.find_project_root()
    cfg = config.load_config(root)
    disc_dir = paths.context_dir("Discussion", root)
    disc_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Discussion")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Discussion",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=question if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {
        "question": question,
        "state": "active",
        "opened_at": _now_iso(),
    }
    if participants:
        spec["participants"] = list(participants)
    if related:
        spec["related"] = list(related)

    errors = schema.validate_spec("Discussion", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Discussion", new_id, spec, body=body or "")
    target_path = disc_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Discussion", new_id, "discussion.opened",
        f"Opened Discussion {new_id!r}: {question!r}",
        root=root,
    )
    return {"id": new_id, "path": str(target_path), "state": "active"}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_discussion(id: str) -> dict:
    """Return the full Discussion entity by ID."""
    root = paths.find_project_root()
    ent = _load_discussion(root, id)
    if ent is None:
        return {"error": f"discussion {id!r} not found"}
    return {
        "id": ent.id,
        "question": ent.spec.get("question"),
        "state": ent.spec.get("state"),
        "participants": ent.spec.get("participants", []),
        "related": ent.spec.get("related", []),
        "outcomes": ent.spec.get("outcomes", []),
        "opened_at": ent.spec.get("opened_at"),
        "closed_at": ent.spec.get("closed_at"),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def transition_discussion(id: str, to_state: str) -> dict:
    """Transition a Discussion through the discussion state machine.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool.
    """
    root = paths.find_project_root()
    ent = _load_discussion(root, id)
    if ent is None:
        return {"error": f"discussion {id!r} not found"}

    from_state = ent.spec.get("state")
    try:
        state_machine.validate_transition("discussion", from_state, to_state)
    except state_machine.StateMachineError as e:
        return {"error": str(e)}

    ent.spec["state"] = to_state
    if to_state == "resolved" and "closed_at" not in ent.spec:
        ent.spec["closed_at"] = _now_iso()
    ent.write()

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Discussion", id, "discussion.transitioned",
        f"Transitioned Discussion {id!r} from {from_state!r} to {to_state!r}",
        root=root,
    )
    return {"ok": True, "id": id, "from_state": from_state, "to_state": to_state}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def add_outcome(id: str, decision_id: str) -> dict:
    """Append a DecisionRecord ID to the discussion's outcomes list.

    The DecisionRecord must already exist (use decision-record's
    record_decision first). Idempotent: re-adding the same decision is
    a no-op. Prerequisite: call find_skill(task_description) or confirm
    you are already operating within a named processkit skill before
    using this tool.
    """
    if not decision_id.startswith("DEC-"):
        return {"error": f"decision_id must start with DEC-, got {decision_id!r}"}

    root = paths.find_project_root()
    ent = _load_discussion(root, id)
    if ent is None:
        return {"error": f"discussion {id!r} not found"}

    outcomes = list(ent.spec.get("outcomes") or [])
    if decision_id not in outcomes:
        outcomes.append(decision_id)
    ent.spec["outcomes"] = outcomes

    errors = schema.validate_spec("Discussion", ent.spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    return {"ok": True, "id": id, "outcomes": outcomes}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_discussions(state: str | None = None, limit: int = 50) -> list[dict]:
    """List Discussion entities, optionally filtered by state."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Discussion", state=state, limit=limit * 4)
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
        out.append({
            "id": full["id"],
            "question": spec.get("question"),
            "state": spec.get("state"),
            "outcomes": spec.get("outcomes", []),
            "path": full.get("path"),
        })
        if len(out) >= limit:
            break
    return out


if __name__ == "__main__":
    server.run(transport="stdio")
