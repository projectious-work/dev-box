#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit role-management MCP server.

Tools:

    create_role(name, description, responsibilities?, skills_required?,
                default_scope?)
        -> {id, path}

    get_role(id) -> {id, name, description, ...} | {error}

    update_role(id, **fields) -> {ok, id, updated}

    list_roles(default_scope?, limit?) -> [roles]

    link_role_to_actor(role_id, actor_id, scope?, valid_from?,
                       valid_until?, description?)
        -> {ok, binding_id}

The link tool creates a Binding (type: role-assignment) so that
Actor-to-Role assignments stay scoped and time-bounded. processkit
roles are descriptive, not restrictive (DEC-017).
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

server = FastMCP("processkit-role-management")

_VALID_SCOPES = {"project", "sprint", "permanent"}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _today_iso() -> str:
    return _dt.date.today().isoformat()


def _load_role(root: Path, id: str) -> entity.Entity | None:
    role_dir = paths.context_dir("Role", root)
    candidate = role_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Role")
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
def create_role(
    name: str,
    description: str,
    responsibilities: list[str] | None = None,
    skills_required: list[str] | None = None,
    default_scope: str | None = None,
) -> dict:
    """Create a new Role entity.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.

    Parameters
    ----------
    name:             kebab-case identifier (matches metadata.id suffix)
    description:      one-sentence purpose statement
    responsibilities: list of imperative bullet points
    skills_required:  optional skill IDs/names (advisory)
    default_scope:    "project" | "sprint" | "permanent"
    """
    if default_scope is not None and default_scope not in _VALID_SCOPES:
        return {
            "error": (
                f"invalid default_scope {default_scope!r}; "
                f"must be one of {sorted(_VALID_SCOPES)}"
            )
        }

    root = paths.find_project_root()
    cfg = config.load_config(root)
    role_dir = paths.context_dir("Role", root)
    role_dir.mkdir(parents=True, exist_ok=True)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Role")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Role",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {"name": name, "description": description}
    if responsibilities:
        spec["responsibilities"] = list(responsibilities)
    if skills_required:
        spec["skills_required"] = list(skills_required)
    if default_scope:
        spec["default_scope"] = default_scope

    errors = schema.validate_spec("Role", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Role", new_id, spec)
    target_path = role_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Role", new_id, "role.created",
        f"Created Role {new_id!r}: {name!r}",
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
def get_role(id: str) -> dict:
    """Return the full Role entity by ID."""
    root = paths.find_project_root()
    ent = _load_role(root, id)
    if ent is None:
        return {"error": f"role {id!r} not found"}
    return {
        "id": ent.id,
        "name": ent.spec.get("name"),
        "description": ent.spec.get("description"),
        "responsibilities": ent.spec.get("responsibilities", []),
        "skills_required": ent.spec.get("skills_required", []),
        "default_scope": ent.spec.get("default_scope"),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def update_role(
    id: str,
    description: str | None = None,
    responsibilities: list[str] | None = None,
    skills_required: list[str] | None = None,
    default_scope: str | None = None,
) -> dict:
    """Update fields on an existing Role. Only supplied fields change.

    Note: ``name`` is intentionally NOT updatable (the metadata.id suffix
    must match the name; renaming a role means superseding it via a new
    entity that sets ``spec.supersedes``).

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    if default_scope is not None and default_scope not in _VALID_SCOPES:
        return {
            "error": (
                f"invalid default_scope {default_scope!r}; "
                f"must be one of {sorted(_VALID_SCOPES)}"
            )
        }

    root = paths.find_project_root()
    ent = _load_role(root, id)
    if ent is None:
        return {"error": f"role {id!r} not found"}

    updated: list[str] = []
    if description is not None:
        ent.spec["description"] = description
        updated.append("description")
    if responsibilities is not None:
        ent.spec["responsibilities"] = list(responsibilities)
        updated.append("responsibilities")
    if skills_required is not None:
        ent.spec["skills_required"] = list(skills_required)
        updated.append("skills_required")
    if default_scope is not None:
        ent.spec["default_scope"] = default_scope
        updated.append("default_scope")

    if not updated:
        return {"ok": True, "id": id, "updated": []}

    errors = schema.validate_spec("Role", ent.spec)
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
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_roles(
    default_scope: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List Role entities, optionally filtered by default_scope."""
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Role", limit=limit * 4)
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
        if default_scope and spec.get("default_scope") != default_scope:
            continue
        out.append({
            "id": full["id"],
            "name": spec.get("name"),
            "description": spec.get("description"),
            "default_scope": spec.get("default_scope"),
            "path": full.get("path"),
        })
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def link_role_to_actor(
    role_id: str,
    actor_id: str,
    scope: str | None = None,
    valid_from: str | None = None,
    valid_until: str | None = None,
    description: str | None = None,
) -> dict:
    """Bind an Actor to a Role by creating a Binding (type: role-assignment).

    Use this rather than editing the Actor's ``spec.roles`` field when
    the assignment has scope, time, or its own attributes. The created
    binding can be queried via binding-management's
    resolve_bindings_for.    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    if not actor_id.startswith("ACTOR-"):
        return {"error": f"actor_id must start with ACTOR-, got {actor_id!r}"}
    if not role_id.startswith("ROLE-"):
        return {"error": f"role_id must start with ROLE-, got {role_id!r}"}

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
        slug_text="role-assignment" if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {
        "type": "role-assignment",
        "subject": actor_id,
        "subject_kind": "Actor",
        "target": role_id,
        "target_kind": "Role",
    }
    if scope:
        spec["scope"] = scope
    if valid_from:
        spec["valid_from"] = valid_from
    if valid_until:
        spec["valid_until"] = valid_until
    if description:
        spec["description"] = description

    errors = schema.validate_spec("Binding", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("Binding", new_id, spec)
    target_path = bind_dir / f"{new_id}.md"
    ent.write(target_path)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Binding", new_id, "binding.role-assigned",
        f"Bound Actor {actor_id!r} to Role {role_id!r} via {new_id!r}",
        root=root,
        actor=new_id,
    )
    return {
        "ok": True,
        "binding_id": new_id,
        "actor_id": actor_id,
        "role_id": role_id,
        "path": str(target_path),
    }


if __name__ == "__main__":
    server.run(transport="stdio")
