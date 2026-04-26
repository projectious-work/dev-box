#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit gate-management MCP server.

Tools:

    create_gate(name, description, kind, validator, validator_command?,
                required_roles?, blocking?, evidence_required?)
        -> {id, path}

    get_gate(id) -> {...} | {error}

    list_gates(kind?, blocking?, limit?) -> [gates]

    evaluate_gate(id, outcome, actor?, evidence?, reason?)
        -> {ok, log_id}

evaluate_gate emits a LogEntry rather than mutating the Gate (the gate
is the rule; evaluations are events). outcome must be one of
"passed", "failed", or "waived"; "waived" requires a reason.
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

server = FastMCP("processkit-gate-management")

_VALID_KINDS = {"manual", "automated", "hybrid"}
_VALID_OUTCOMES = {"passed", "failed", "waived"}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_gate(root: Path, id: str) -> entity.Entity | None:
    gate_dir = paths.context_dir("Gate", root)
    candidate = gate_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Gate")
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
def create_gate(
    name: str,
    description: str,
    kind: str,
    validator: str,
    validator_command: str | None = None,
    required_roles: list[str] | None = None,
    blocking: bool = True,
    evidence_required: bool = False,
) -> dict:
    """Create a new Gate entity.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.

    Parameters
    ----------
    name:               kebab-case identifier
    description:        what this gate checks
    kind:               manual | automated | hybrid
    validator:          prose description of the check
    validator_command:  optional CLI for automated/hybrid kinds
    required_roles:     ROLE-ids authorized to sign off
    blocking:           true = work cannot proceed if failed
    evidence_required:  true = pass events must include a link/artifact
    """
    if kind not in _VALID_KINDS:
        return {"error": f"invalid kind {kind!r}; must be one of {sorted(_VALID_KINDS)}"}

    root = paths.find_project_root()
    cfg = config.load_config(root)
    gate_dir = paths.context_dir("Gate", root)
    gate_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Gate")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Gate",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {
        "name": name,
        "description": description,
        "kind": kind,
        "validator": validator,
        "blocking": blocking,
        "evidence_required": evidence_required,
    }
    if validator_command:
        spec["validator_command"] = validator_command
    if required_roles:
        spec["required_roles"] = list(required_roles)

    errors = schema.validate_spec("Gate", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Gate", new_id, spec)
    target_path = gate_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Gate", new_id, "gate.created",
        f"Created Gate {new_id!r}: {name!r}",
        root=root,
        actor=new_id,
    )
    return {"id": new_id, "path": str(target_path), "name": name}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_gate(id: str) -> dict:
    """Return the full Gate entity by ID."""
    root = paths.find_project_root()
    ent = _load_gate(root, id)
    if ent is None:
        return {"error": f"gate {id!r} not found"}
    return {
        "id": ent.id,
        "name": ent.spec.get("name"),
        "description": ent.spec.get("description"),
        "kind": ent.spec.get("kind"),
        "validator": ent.spec.get("validator"),
        "validator_command": ent.spec.get("validator_command"),
        "required_roles": ent.spec.get("required_roles", []),
        "blocking": ent.spec.get("blocking", True),
        "evidence_required": ent.spec.get("evidence_required", False),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_gates(
    kind: str | None = None,
    blocking: bool | None = None,
    limit: int = 50,
) -> list[dict]:
    """List Gate entities, optionally filtered by kind and blocking flag."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Gate", limit=limit * 4)
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
        if blocking is not None and bool(spec.get("blocking", True)) != blocking:
            continue
        out.append({
            "id": full["id"],
            "name": spec.get("name"),
            "kind": spec.get("kind"),
            "blocking": spec.get("blocking", True),
            "path": full.get("path"),
        })
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def evaluate_gate(
    id: str,
    outcome: str,
    actor: str | None = None,
    evidence: str | None = None,
    reason: str | None = None,
) -> dict:
    """Record a gate evaluation as a LogEntry.

    Gates are immutable rules; evaluations are events. The Gate file is
    not modified. ``outcome`` must be one of "passed", "failed", or
    "waived"; a "waived" outcome requires a non-empty ``reason`` and
    typically an ``actor`` who waived it.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    if outcome not in _VALID_OUTCOMES:
        return {"error": f"invalid outcome {outcome!r}; must be one of {sorted(_VALID_OUTCOMES)}"}
    if outcome == "waived" and not reason:
        return {"error": "waived outcome requires a reason"}

    root = paths.find_project_root()
    ent = _load_gate(root, id)
    if ent is None:
        return {"error": f"gate {id!r} not found"}
    if (
        outcome == "passed"
        and ent.spec.get("evidence_required")
        and not evidence
    ):
        return {"error": f"gate {id!r} requires evidence on pass"}

    details: dict = {}
    if evidence:
        details["evidence"] = evidence
    if reason:
        details["reason"] = reason
    if outcome == "waived" and actor:
        details["waived_by"] = actor

    log_id = log.log_side_effect(
        "Gate", id, f"gate.{outcome}",
        f"Gate {ent.spec.get('name', id)} {outcome}",
        root=root,
        actor=actor,
        details=details or None,
    )

    return {
        "ok": True,
        "log_id": log_id,
        "gate_id": id,
        "outcome": outcome,
    }


if __name__ == "__main__":
    server.run(transport="stdio")
