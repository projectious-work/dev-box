#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit artifact-management MCP server.

Tools:

    create_artifact(name, kind, location?, format?, version?, owner?,
                    produced_by?, tags?, description?)
        -> {id, path}

    get_artifact(id)
        -> artifact | error

    query_artifacts(kind?, owner?, tags?, limit?)
        -> [artifacts]

    update_artifact(id, name?, kind?, location?, format?, version?,
                    owner?, produced_by?, tags?)
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

from processkit import config, entity, ids, index, log, paths, schema  # noqa: E402

server = FastMCP("processkit-artifact-management")

_VALID_KINDS = {
    "document", "design", "dataset", "build", "slides",
    "video", "spec", "diagram", "url", "other",
}


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _load_artifact(root: Path, id: str) -> entity.Entity | None:
    art_dir = paths.context_dir("Artifact", root)
    candidate = art_dir / f"{id}.md"
    if candidate.is_file():
        return entity.load(candidate)
    db = index.open_db()
    try:
        row, _ = index.resolve_entity(db, id, kind="Artifact")
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
def create_artifact(
    name: str,
    kind: str,
    location: str | None = None,
    format: str | None = None,
    version: str | None = None,
    owner: str | None = None,
    produced_by: str | None = None,
    tags: list[str] | None = None,
    description: str | None = None,
) -> dict:
    """Register a new Artifact in context/artifacts/.

    ``kind`` must be one of: document, design, dataset, build, slides,
    video, spec, diagram, url, other.

    For pointer artifacts (content lives outside the repo), ``location``
    is required. For self-hosted artifacts (content in the Markdown
    body), ``location`` may be omitted.

    Returns ``{id, path}``. Prerequisite: call
    find_skill(task_description) or confirm you are already operating
    within a named processkit skill before using this tool.
    1% rule: call route_task first; commit in the same turn — deferred writes are dropped.
    """
    if kind not in _VALID_KINDS:
        return {
            "error": (
                f"invalid kind {kind!r}; must be one of "
                f"{sorted(_VALID_KINDS)}"
            )
        }

    root = paths.find_project_root()
    cfg = config.load_config(root)

    db = index.open_db()
    try:
        existing = index.existing_ids(db, "Artifact")
    finally:
        db.close()

    new_id = ids.generate_id(
        "Artifact",
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=name if cfg.id_slug else None,
        existing=existing,
    )

    spec: dict = {"name": name, "kind": kind}
    if location:
        spec["location"] = location
    if format:
        spec["format"] = format
    if version:
        spec["version"] = version
    if owner:
        spec["owner"] = owner
    if produced_by:
        spec["produced_by"] = produced_by
    if tags:
        spec["tags"] = tags
    spec["produced_at"] = _now_iso()

    errors = schema.validate_spec("Artifact", spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    body = description or ""
    ent = entity.new("Artifact", new_id, spec)
    if body:
        ent.body = body

    art_dir = paths.context_dir("Artifact", root)
    art_dir.mkdir(parents=True, exist_ok=True)
    target = paths.entity_path("Artifact", new_id, None, root)
    ent.write(target)

    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Artifact", new_id, "artifact.created",
        f"Created Artifact {new_id!r}: {name!r}",
        root=root,
    )
    return {"id": new_id, "path": str(target)}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_artifact(id: str) -> dict:
    """Fetch an Artifact by ID with its full spec.

    Accepts a full ID, a prefix (missing slug), or a bare word-pair.
    Returns ``{"error": "..."}`` if not found or ambiguous.
    """
    db = index.open_db()
    try:
        row, candidates = index.resolve_entity(db, id, kind="Artifact")
    finally:
        db.close()
    if candidates:
        return {"error": f"ambiguous ID {id!r}; candidates: {candidates}"}
    if row is None:
        return {"error": f"Artifact not found: {id!r}"}
    spec = row.get("spec", {})
    return {
        "id": row["id"],
        "name": spec.get("name") or row.get("title"),
        "kind": spec.get("kind"),
        "location": spec.get("location"),
        "format": spec.get("format"),
        "version": spec.get("version"),
        "checksum": spec.get("checksum"),
        "owner": spec.get("owner"),
        "produced_by": spec.get("produced_by"),
        "produced_at": spec.get("produced_at"),
        "tags": spec.get("tags", []),
        "path": row.get("path"),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_artifacts(
    kind: str | None = None,
    owner: str | None = None,
    tags: list[str] | None = None,
    limit: int = 50,
) -> list[dict]:
    """List Artifacts with optional filters.

    Filters: ``kind`` (exact match), ``owner`` (Actor ID),
    ``tags`` (all supplied tags must be present on the artifact).
    """
    if kind and kind not in _VALID_KINDS:
        return [{"error": (
            f"invalid kind {kind!r}; must be one of {sorted(_VALID_KINDS)}"
        )}]
    db = index.open_db()
    try:
        rows = index.query_entities(db, kind="Artifact", limit=limit * 4)
    finally:
        db.close()
    out = []
    for r in rows:
        full = get_artifact(r["id"])
        if "error" in full:
            continue
        if kind and full.get("kind") != kind:
            continue
        if owner and full.get("owner") != owner:
            continue
        if tags:
            artifact_tags = full.get("tags") or []
            if not all(t in artifact_tags for t in tags):
                continue
        out.append(full)
        if len(out) >= limit:
            break
    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def update_artifact(
    id: str,
    name: str | None = None,
    kind: str | None = None,
    location: str | None = None,
    format: str | None = None,
    version: str | None = None,
    owner: str | None = None,
    produced_by: str | None = None,
    tags: list[str] | None = None,
) -> dict:
    """Update metadata fields on an existing Artifact.

    Only the fields you supply are changed; omitted fields are
    preserved. Returns ``{ok, id}``. Prerequisite: call
    find_skill(task_description) or confirm you are already operating
    within a named processkit skill before using this tool.
    """
    if kind is not None and kind not in _VALID_KINDS:
        return {
            "error": (
                f"invalid kind {kind!r}; must be one of "
                f"{sorted(_VALID_KINDS)}"
            )
        }
    root = paths.find_project_root()
    ent = _load_artifact(root, id)
    if ent is None:
        return {"error": f"Artifact not found: {id!r}"}

    if name is not None:
        ent.spec["name"] = name
    if kind is not None:
        ent.spec["kind"] = kind
    if location is not None:
        ent.spec["location"] = location
    if format is not None:
        ent.spec["format"] = format
    if version is not None:
        ent.spec["version"] = version
    if owner is not None:
        ent.spec["owner"] = owner
    if produced_by is not None:
        ent.spec["produced_by"] = produced_by
    if tags is not None:
        ent.spec["tags"] = tags

    errors = schema.validate_spec("Artifact", ent.spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent.write()
    db = index.open_db()
    try:
        index.upsert_entity(db, ent)
    finally:
        db.close()

    log.log_side_effect(
        "Artifact", id, "artifact.updated",
        f"Updated Artifact {id!r}",
        root=root,
    )
    return {"ok": True, "id": id}


if __name__ == "__main__":
    server.run(transport="stdio")
