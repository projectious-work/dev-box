#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit id-management MCP server.

Allocates unique entity IDs following the project's configured format
(``word``/``uuid`` × with/without slug × camel/kebab × datetime prefix).
Other entity-creating servers call into the same lib function
(`processkit.ids.generate_id`) directly for performance — this MCP
server exposes the same capability to agents that want to call it
explicitly (preview, validate, list, audit).

Tools:

    generate_id(kind, slug_text?)         -> {id, kind}
    validate_id(id)                       -> {valid, kind?, prefix?, body?, reason?}
    list_used_ids(kind?, limit?)          -> [{id, kind}]
    format_info()                         -> {format, word_style, datetime_prefix,
                                              slug, prefixes}
"""
from __future__ import annotations

import os
import re
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

from processkit import KIND_PREFIXES, config, ids, index  # noqa: E402

server = FastMCP("processkit-id-management")

# Reverse registry: prefix -> kind
_PREFIX_TO_KIND = {prefix: kind for kind, prefix in KIND_PREFIXES.items()}

# Body patterns
# Old kebab format:  adj-noun[-slug...]
_KEBAB_BODY = re.compile(r"^[a-z]+-[a-z]+(?:-[a-z0-9]+)*$")
# New camel format:  [YYYYMMDD_HHMM-]CamelCase[-slug...]
_CAMEL_BODY = re.compile(
    r"^(?:(\d{8}_\d{4})-)?([A-Z][a-z]+[A-Z][a-z]+)(?:-([a-z0-9][a-z0-9-]*))?$"
)
# UUID format
_UUID_BODY = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}(?:-[0-9a-f]{4})?(?:-[a-z0-9]+)*$")


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def generate_id(kind: str, slug_text: str | None = None) -> dict:
    """Generate a fresh, collision-free ID for the given primitive kind.

    Reads ``word_style``, ``datetime_prefix``, ``format``, and ``slug``
    from the project's id-management settings. Queries the index for
    existing IDs of the same kind to avoid collisions.
    """
    if kind not in KIND_PREFIXES:
        return {"error": f"unknown kind {kind!r}; valid: {sorted(KIND_PREFIXES.keys())}"}

    cfg = config.load_config()
    db = index.open_db()
    try:
        existing = index.existing_ids(db, kind)
    finally:
        db.close()

    new_id = ids.generate_id(
        kind,
        format=cfg.id_format,
        word_style=cfg.id_word_style,
        datetime_prefix=cfg.id_datetime_prefix,
        slug_text=slug_text if cfg.id_slug else None,
        existing=existing,
    )
    return {"id": new_id, "kind": kind}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def validate_id(id: str) -> dict:
    """Validate an ID's format and decompose it into kind/prefix/body.

    Accepts both old kebab format (``PREFIX-adj-noun``) and new formats
    (``PREFIX-YYYYMMDD_HHMM-CamelCase[-slug]``, ``PREFIX-CamelCase[-slug]``).
    Does NOT check whether the ID is currently in use.
    """
    if not isinstance(id, str) or "-" not in id:
        return {"valid": False, "reason": "id must be a string of the form PREFIX-body"}
    prefix, body = id.split("-", 1)
    kind = _PREFIX_TO_KIND.get(prefix)
    if kind is None:
        return {"valid": False, "reason": f"unknown prefix {prefix!r}"}
    if not body:
        return {"valid": False, "reason": "empty body"}

    # Try new camel format first
    m = _CAMEL_BODY.match(body)
    if m:
        dt_part, word_pair, slug = m.group(1), m.group(2), m.group(3)
        return {
            "valid": True,
            "kind": kind,
            "prefix": prefix,
            "body": body,
            "format_guess": "word",
            "word_style": "camel",
            "datetime_part": dt_part,
            "slug": slug,
            "has_slug": slug is not None,
        }

    # Try old kebab format
    if _KEBAB_BODY.match(body):
        has_slug = body.count("-") > 1
        return {
            "valid": True,
            "kind": kind,
            "prefix": prefix,
            "body": body,
            "format_guess": "word",
            "word_style": "kebab",
            "datetime_part": None,
            "has_slug": has_slug,
        }

    # Try UUID format
    if _UUID_BODY.match(body):
        return {
            "valid": True,
            "kind": kind,
            "prefix": prefix,
            "body": body,
            "format_guess": "uuid",
            "word_style": None,
            "datetime_part": None,
            "has_slug": body.count("-") > 2,
        }

    return {
        "valid": False,
        "reason": f"body {body!r} does not match a known format",
        "kind": kind,
        "prefix": prefix,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_used_ids(kind: str | None = None, limit: int = 200) -> list[dict]:
    """List IDs already in use, optionally filtered by kind.

    Returns up to ``limit`` rows. The result is read from the index, so
    it reflects whatever was last upserted. Run ``reindex()`` first if
    you suspect the index is stale.
    """
    db = index.open_db()
    try:
        if kind:
            rows = db.execute(
                "SELECT id, kind FROM entities WHERE kind = ? ORDER BY id LIMIT ?",
                (kind, int(limit)),
            ).fetchall()
        else:
            rows = db.execute(
                "SELECT id, kind FROM entities ORDER BY kind, id LIMIT ?",
                (int(limit),),
            ).fetchall()
        return [dict(r) for r in rows]
    finally:
        db.close()


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def format_info() -> dict:
    """Return the project's ID configuration and the prefix registry."""
    cfg = config.load_config()
    return {
        "format": cfg.id_format,
        "word_style": cfg.id_word_style,
        "datetime_prefix": cfg.id_datetime_prefix,
        "slug": cfg.id_slug,
        "example": (
            "BACK-20260409_1449-BoldVale-fts5-search"
            if cfg.id_datetime_prefix and cfg.id_word_style == "camel"
            else "BACK-bold-vale-fts5-search"
        ),
        "prefixes": dict(KIND_PREFIXES),
    }


if __name__ == "__main__":
    server.run(transport="stdio")
