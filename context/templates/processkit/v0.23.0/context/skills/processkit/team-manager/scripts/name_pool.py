"""Name pool management for team-manager.

The pool is a single YAML file (``data/name-pool.yaml``) shaped as a
processkit entity with ``kind: NamePool``. This module owns read/write
access with atomic rewrites.

Layout
------

    spec:
      names:
        feminine: [Alice, Aria, ...]
        masculine: [Adam, Alex, ...]
        neutral:   [Avery, Blake, ...]
      reserved:
        Alice: alice-chen
"""
from __future__ import annotations

import os
import random
import tempfile
from pathlib import Path
from typing import Any

import yaml


class NamePoolError(ValueError):
    """Raised on invalid reservation operations."""


_BUCKETS = ("feminine", "masculine", "neutral")


def load_pool(path: Path) -> dict[str, Any]:
    """Load the pool YAML. Returns the parsed dict."""
    text = path.read_text(encoding="utf-8")
    # The file is a YAML entity wrapped in leading/trailing `---`.
    # Parse the first YAML document that contains the `spec` key.
    docs = list(yaml.safe_load_all(text))
    for doc in docs:
        if isinstance(doc, dict) and "spec" in doc:
            return doc
    raise NamePoolError(f"no entity document found in {path}")


def all_names(pool: dict[str, Any]) -> list[str]:
    """Flat list of every name across every bucket, in document order."""
    spec = pool.get("spec") or {}
    names_map = spec.get("names") or {}
    out: list[str] = []
    for bucket in _BUCKETS:
        out.extend(names_map.get(bucket) or [])
    return out


def kind_of(pool: dict[str, Any], name: str) -> str | None:
    spec = pool.get("spec") or {}
    names_map = spec.get("names") or {}
    for bucket in _BUCKETS:
        if name in (names_map.get(bucket) or []):
            return bucket
    return None


def available(pool: dict[str, Any], kind: str | None = None) -> list[str]:
    """Names that are in the pool and not currently reserved."""
    spec = pool.get("spec") or {}
    reserved = spec.get("reserved") or {}
    out: list[str] = []
    names_map = spec.get("names") or {}
    buckets = (kind,) if kind else _BUCKETS
    for bucket in buckets:
        for name in names_map.get(bucket) or []:
            if name not in reserved:
                out.append(name)
    return out


def suggest(
    pool: dict[str, Any],
    kind: str | None = None,
    rng: random.Random | None = None,
) -> str | None:
    pool_available = available(pool, kind)
    if not pool_available:
        return None
    rng = rng or random.Random()
    return rng.choice(pool_available)


def reserve(path: Path, name: str, slug: str) -> dict[str, Any]:
    """Reserve a name for a slug. Atomic file rewrite."""
    pool = load_pool(path)
    spec = pool.setdefault("spec", {})
    names_map = spec.get("names") or {}
    flat = []
    for bucket in _BUCKETS:
        flat.extend(names_map.get(bucket) or [])
    if name not in flat:
        raise NamePoolError(f"name {name!r} is not in the pool")
    reserved = spec.setdefault("reserved", {}) or {}
    if not isinstance(reserved, dict):
        reserved = {}
        spec["reserved"] = reserved
    if name in reserved and reserved[name] != slug:
        raise NamePoolError(
            f"name {name!r} is already reserved by {reserved[name]!r}"
        )
    reserved[name] = slug
    spec["reserved"] = reserved
    _atomic_write(path, pool)
    return pool


def release(path: Path, name: str) -> dict[str, Any]:
    """Release a reservation. Idempotent (missing name is a no-op)."""
    pool = load_pool(path)
    spec = pool.setdefault("spec", {})
    reserved = spec.get("reserved") or {}
    if not isinstance(reserved, dict):
        reserved = {}
    if name in reserved:
        reserved.pop(name)
    spec["reserved"] = reserved
    _atomic_write(path, pool)
    return pool


def is_reserved(pool: dict[str, Any], name: str) -> str | None:
    """Return the slug that holds ``name``, or None if unreserved."""
    reserved = (pool.get("spec") or {}).get("reserved") or {}
    return reserved.get(name)


# ---------------------------------------------------------------------------
# Internals
# ---------------------------------------------------------------------------

def _atomic_write(path: Path, pool: dict[str, Any]) -> None:
    """Write the pool back with leading/trailing `---` to match the entity shape."""
    dump = yaml.safe_dump(pool, sort_keys=False, default_flow_style=False, width=100)
    text = f"---\n{dump}---\n"
    fd, tmp = tempfile.mkstemp(prefix=".name-pool.", suffix=".tmp", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            fh.write(text)
        os.replace(tmp, path)
    except Exception:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise
