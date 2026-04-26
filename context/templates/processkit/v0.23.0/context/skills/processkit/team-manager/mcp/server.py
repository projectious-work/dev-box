#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit team-manager MCP server.

Owns TeamMember lifecycle, a curated international name pool, memory-tree
scaffolding, A2A Agent Card export/import, and 10 consistency checks.
Replaces the deprecated actor-profile server (DEC-20260422_0233-SpryTulip).

Tools
-----

Lifecycle:
    create_team_member(name, type, slug, default_role?, default_seniority?,
                       personality?, email?, handle?, joined_at?) -> {id, path}
    get_team_member(id) -> {...} | {error}
    list_team_members(type?, active_only?, role?, limit?) -> [members]
    update_team_member(id, **fields) -> {ok, id, updated}
    deactivate_team_member(id, left_at?) -> {ok, id, left_at}
    reactivate_team_member(id) -> {ok, id}

Name pool:
    reserve_name(name, team_member_slug) -> {ok, name, slug}
    release_name(name) -> {ok, name}
    list_available_names(kind?) -> [names]
    suggest_name(kind?) -> {name, kind}

Memory tree:
    init_memory_tree(slug) -> {slug, path, created}

Export/import:
    export_team_member(slug, output_path?) -> {path, redacted}
    import_team_member(tarball_path) -> {slug, path}

Consistency:
    check_consistency(slug) -> {slug, findings}
    check_all_consistency() -> {members: {...}, summary}
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

# Expose scripts/ for reuse of helper modules
_SCRIPTS_DIR = Path(__file__).resolve().parent.parent / "scripts"
if _SCRIPTS_DIR.is_dir():
    sys.path.insert(0, str(_SCRIPTS_DIR))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import config, entity, index, log, paths, schema  # noqa: E402

import consistency as _consistency  # noqa: E402
import memory_tree as _memory_tree  # noqa: E402
import name_pool as _name_pool  # noqa: E402
import export_import as _export_import  # noqa: E402


server = FastMCP("processkit-team-manager")

_VALID_TYPES = {"human", "ai-agent", "service"}
_VALID_SENIORITY = {"junior", "specialist", "expert", "senior", "principal"}
_UPDATABLE_FIELDS = {
    "name", "email", "handle", "default_role", "default_seniority",
    "personality", "memory", "relationships", "exportable",
    "export_policy", "active", "joined_at", "left_at",
}

_SLUG_RE = __import__("re").compile(r"^[a-z][a-z0-9-]*$")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _tm_dir(root: Path, slug: str) -> Path:
    return paths.context_dir("TeamMember", root) / slug


def _tm_entity_path(root: Path, slug: str) -> Path:
    return _tm_dir(root, slug) / "team-member.md"


def _load_tm_by_slug(root: Path, slug: str) -> entity.Entity | None:
    p = _tm_entity_path(root, slug)
    if p.is_file():
        return entity.load(p)
    return None


def _load_tm(root: Path, id_or_slug: str) -> entity.Entity | None:
    slug = id_or_slug[len("TEAMMEMBER-"):] if id_or_slug.startswith("TEAMMEMBER-") else id_or_slug
    ent = _load_tm_by_slug(root, slug)
    if ent is not None:
        return ent
    # Fallback: try index
    try:
        db = index.open_db()
        try:
            row = index.get_entity(db, id_or_slug if id_or_slug.startswith("TEAMMEMBER-") else f"TEAMMEMBER-{slug}")
        finally:
            db.close()
        if row and row.get("path"):
            return entity.load(row["path"])
    except Exception:
        pass
    return None


def _pool_path() -> Path:
    return Path(__file__).resolve().parent.parent / "data" / "name-pool.yaml"


# ---------------------------------------------------------------------------
# Lifecycle tools
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def create_team_member(
    name: str,
    type: str,
    slug: str,
    default_role: str | None = None,
    default_seniority: str | None = None,
    personality: dict | None = None,
    email: str | None = None,
    handle: str | None = None,
    joined_at: str | None = None,
) -> dict:
    """Create a new TeamMember entity.

    Parameters
    ----------
    name:               display name (e.g. "Atlas" or "Alice Chen")
    type:               "human", "ai-agent", or "service"
    slug:               canonical kebab-case slug; becomes TEAMMEMBER-<slug>
    default_role:       optional ROLE-<id>
    default_seniority:  optional enum (junior|specialist|expert|senior|principal)
    personality:        optional dict (communication_style, voice, ...)
    email:              optional; humans only
    handle:             optional; GitHub/Slack handle
    joined_at:          ISO datetime; defaults to now

    For type=ai-agent, slug must match a name in the name pool and will be
    auto-reserved on success.
    """
    if type not in _VALID_TYPES:
        return {"error": f"invalid type {type!r}; must be one of {sorted(_VALID_TYPES)}"}
    if default_seniority is not None and default_seniority not in _VALID_SENIORITY:
        return {"error": f"invalid seniority {default_seniority!r}"}
    if not _SLUG_RE.match(slug):
        return {"error": f"invalid slug {slug!r}; must match ^[a-z][a-z0-9-]*$"}

    root = paths.find_project_root()
    tm_path = _tm_entity_path(root, slug)
    if tm_path.is_file():
        return {"error": f"team-member {slug!r} already exists at {tm_path}"}

    # For ai-agent, verify slug is reserveable from pool
    pool_reservation_done = False
    if type == "ai-agent":
        pool = _name_pool.load_pool(_pool_path())
        # slug derived from a pool name — lower-cased
        pool_names = _name_pool.all_names(pool)
        match = next((n for n in pool_names if n.lower() == slug.lower()), None)
        if match is None:
            return {
                "error": f"slug {slug!r} is not in the name pool; "
                "call list_available_names(kind?) or suggest_name(kind?) first"
            }
        reserved = pool["spec"].get("reserved") or {}
        if match in reserved:
            return {"error": f"name {match!r} is already reserved by slug {reserved[match]!r}"}
        _name_pool.reserve(_pool_path(), match, slug)
        pool_reservation_done = True

    new_id = f"TEAMMEMBER-{slug}"

    spec: dict = {
        "type": type,
        "name": name,
        "slug": slug,
        "active": True,
        "joined_at": joined_at or _now_iso(),
    }
    if email:
        spec["email"] = email
    if handle:
        spec["handle"] = handle
    if default_role:
        spec["default_role"] = default_role
    if default_seniority:
        spec["default_seniority"] = default_seniority
    if personality:
        spec["personality"] = dict(personality)

    errors = schema.validate_spec("TeamMember", spec)
    if errors:
        if pool_reservation_done:
            _name_pool.release(_pool_path(), match)
        return {"error": "schema validation failed", "details": errors}

    ent = entity.new("TeamMember", new_id, spec)
    ent.write(tm_path)

    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass

    log.log_side_effect(
        "TeamMember", new_id, "team_member.created",
        f"Created TeamMember {new_id!r}: {name!r} ({type})",
        root=root,
        actor=new_id,
    )
    return {"id": new_id, "path": str(tm_path), "type": type, "name": name, "slug": slug}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_team_member(id: str) -> dict:
    """Return the full TeamMember entity by ID or slug."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    return {
        "id": ent.id,
        "type": ent.spec.get("type"),
        "name": ent.spec.get("name"),
        "slug": ent.spec.get("slug"),
        "email": ent.spec.get("email"),
        "handle": ent.spec.get("handle"),
        "default_role": ent.spec.get("default_role"),
        "default_seniority": ent.spec.get("default_seniority"),
        "personality": ent.spec.get("personality", {}),
        "memory": ent.spec.get("memory", {}),
        "relationships": ent.spec.get("relationships", []),
        "active": ent.spec.get("active", True),
        "joined_at": ent.spec.get("joined_at"),
        "left_at": ent.spec.get("left_at"),
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_team_members(
    type: str | None = None,
    active_only: bool = True,
    role: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """List TeamMembers with optional filters by type, active state, default_role."""
    root = paths.find_project_root()
    tm_root = paths.context_dir("TeamMember", root)
    if not tm_root.is_dir():
        return []
    out: list[dict] = []
    for child in sorted(tm_root.iterdir()):
        if not child.is_dir():
            continue
        p = child / "team-member.md"
        if not p.is_file():
            continue
        try:
            ent = entity.load(p)
        except Exception:
            continue
        if ent.kind != "TeamMember":
            continue
        spec = ent.spec or {}
        if type and spec.get("type") != type:
            continue
        if active_only and not spec.get("active", True):
            continue
        if role and spec.get("default_role") != role:
            continue
        out.append({
            "id": ent.id,
            "type": spec.get("type"),
            "name": spec.get("name"),
            "slug": spec.get("slug"),
            "default_role": spec.get("default_role"),
            "default_seniority": spec.get("default_seniority"),
            "active": spec.get("active", True),
            "path": str(ent.path) if ent.path else None,
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
def update_team_member(
    id: str,
    name: str | None = None,
    email: str | None = None,
    handle: str | None = None,
    default_role: str | None = None,
    default_seniority: str | None = None,
    personality: dict | None = None,
    memory: dict | None = None,
    relationships: list | None = None,
    exportable: bool | None = None,
    export_policy: dict | None = None,
    active: bool | None = None,
) -> dict:
    """Update fields on an existing TeamMember. Only supplied fields change."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}

    updated: list[str] = []
    locals_map = {
        "name": name, "email": email, "handle": handle,
        "default_role": default_role, "default_seniority": default_seniority,
        "personality": personality, "memory": memory,
        "relationships": relationships, "exportable": exportable,
        "export_policy": export_policy, "active": active,
    }
    for k, v in locals_map.items():
        if v is None:
            continue
        if k == "default_seniority" and v not in _VALID_SENIORITY:
            return {"error": f"invalid seniority {v!r}"}
        ent.spec[k] = v
        updated.append(k)

    if not updated:
        return {"ok": True, "id": ent.id, "updated": []}

    errors = schema.validate_spec("TeamMember", ent.spec)
    if errors:
        return {"error": "schema validation failed", "details": errors}

    ent.write()
    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass
    return {"ok": True, "id": ent.id, "updated": updated}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=True,
    openWorldHint=False,
))
def deactivate_team_member(id: str, left_at: str | None = None) -> dict:
    """Mark a TeamMember inactive. Sets active=false and left_at."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    ent.spec["active"] = False
    ent.spec["left_at"] = left_at or _now_iso()
    ent.write()
    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass
    log.log_side_effect(
        "TeamMember", ent.id, "team_member.deactivated",
        f"Deactivated TeamMember {ent.id!r}",
        root=root,
        actor=ent.id,
    )
    return {"ok": True, "id": ent.id, "left_at": ent.spec["left_at"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def reactivate_team_member(id: str) -> dict:
    """Reactivate a deactivated TeamMember (active=true, clears left_at)."""
    root = paths.find_project_root()
    ent = _load_tm(root, id)
    if ent is None:
        return {"error": f"team-member {id!r} not found"}
    ent.spec["active"] = True
    ent.spec.pop("left_at", None)
    ent.write()
    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        pass
    log.log_side_effect(
        "TeamMember", ent.id, "team_member.reactivated",
        f"Reactivated TeamMember {ent.id!r}",
        root=root,
        actor=ent.id,
    )
    return {"ok": True, "id": ent.id}


# ---------------------------------------------------------------------------
# Name-pool tools
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def reserve_name(name: str, team_member_slug: str) -> dict:
    """Reserve a pool name for a team-member slug."""
    try:
        _name_pool.reserve(_pool_path(), name, team_member_slug)
    except _name_pool.NamePoolError as e:
        return {"error": str(e)}
    return {"ok": True, "name": name, "slug": team_member_slug}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=True,
    openWorldHint=False,
))
def release_name(name: str) -> dict:
    """Release a pool name reservation."""
    try:
        _name_pool.release(_pool_path(), name)
    except _name_pool.NamePoolError as e:
        return {"error": str(e)}
    return {"ok": True, "name": name}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_available_names(kind: str | None = None) -> list[str]:
    """List pool names that are not reserved. Optional kind filter:
    feminine | masculine | neutral."""
    pool = _name_pool.load_pool(_pool_path())
    return _name_pool.available(pool, kind)


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def suggest_name(kind: str | None = None) -> dict:
    """Suggest one available pool name at random. Does not reserve."""
    pool = _name_pool.load_pool(_pool_path())
    chosen = _name_pool.suggest(pool, kind)
    if chosen is None:
        return {"error": f"no available names in pool (kind={kind!r})"}
    bucket = _name_pool.kind_of(pool, chosen)
    return {"name": chosen, "kind": bucket}


# ---------------------------------------------------------------------------
# Memory-tree tool
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def init_memory_tree(slug: str) -> dict:
    """Scaffold the memory-tree layout for an existing team-member slug.

    Creates tier subdirectories with .gitkeep, plus persona.md, card.json,
    and team-member.md (if the entity does not already exist). Idempotent.
    """
    root = paths.find_project_root()
    tm_dir = _tm_dir(root, slug)
    created = _memory_tree.init_tree(tm_dir, slug, _load_tm_by_slug(root, slug))
    return {"slug": slug, "path": str(tm_dir), "created": created}


# ---------------------------------------------------------------------------
# Export / import
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def export_team_member(slug: str, output_path: str | None = None) -> dict:
    """Build a tar.gz bundle for a team-member.

    Includes persona.md, card.json, team-member.md, and knowledge/,
    skills/, lessons/. Excludes journal/, relations/, private/. Redacts
    memory files whose frontmatter declares sensitivity=confidential|pii.
    """
    root = paths.find_project_root()
    tm_dir = _tm_dir(root, slug)
    if not tm_dir.is_dir():
        return {"error": f"team-member directory not found: {tm_dir}"}
    dest = Path(output_path) if output_path else Path.cwd() / f"{slug}-export-{_dt.date.today().isoformat()}.tar.gz"
    summary = _export_import.export(tm_dir, dest)
    log.log_side_effect(
        "TeamMember", f"TEAMMEMBER-{slug}", "team_member.exported",
        f"Exported TeamMember {slug!r} to {dest}",
        root=root,
        actor=f"TEAMMEMBER-{slug}",
    )
    return {"path": str(dest), "redacted": summary["redacted"], "included": summary["included"]}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def import_team_member(tarball_path: str) -> dict:
    """Import a team-member bundle. Validates schema + signature-field presence."""
    root = paths.find_project_root()
    tm_root = paths.context_dir("TeamMember", root)
    assets = Path(__file__).resolve().parent.parent / "assets"
    try:
        result = _export_import.import_bundle(Path(tarball_path), tm_root, assets)
    except _export_import.ImportError as e:
        return {"error": str(e)}
    slug = result["slug"]
    log.log_side_effect(
        "TeamMember", f"TEAMMEMBER-{slug}", "team_member.imported",
        f"Imported TeamMember {slug!r} from {tarball_path}",
        root=root,
        actor=f"TEAMMEMBER-{slug}",
    )
    return result


# ---------------------------------------------------------------------------
# Consistency
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def check_consistency(slug: str) -> dict:
    """Run the 10 consistency checks on a single team-member."""
    root = paths.find_project_root()
    tm_dir = _tm_dir(root, slug)
    assets = Path(__file__).resolve().parent.parent / "assets"
    findings = _consistency.check_one(root, tm_dir, slug, _pool_path(), assets)
    return {"slug": slug, "findings": findings}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def check_all_consistency() -> dict:
    """Run the 10 consistency checks across every team-member."""
    root = paths.find_project_root()
    tm_root = paths.context_dir("TeamMember", root)
    assets = Path(__file__).resolve().parent.parent / "assets"
    return _consistency.check_all(root, tm_root, _pool_path(), assets)


if __name__ == "__main__":
    server.run(transport="stdio")
