"""Locate the project root, processkit checkout, and per-kind directories.

Project layout (consumer side):

    <project-root>/
      AGENTS.md            ← provider-neutral root marker (preferred)
      aibox.toml           ← aibox installer config (accepted as fallback marker)
      context/
        skills/<name>/...           ← installed processkit skills
        skills/_lib/processkit/...  ← installed processkit lib
        schemas/<f>.yaml            ← installed primitive schemas
        state-machines/<f>.yaml     ← installed state machines
        processes/<f>.md            ← installed process definitions
        templates/processkit/<v>/   ← full upstream reference templates
        workitems/, decisions/, logs/, ...
        .cache/processkit/          ← gitignored runtime cache (index DB)

Processkit checkout layout (this repo, when running servers from
a checkout):

    <processkit-root>/
      AGENTS.md
      src/
        primitives/
        skills/
        lib/
      context/

Servers may be invoked in either context. The functions here resolve
paths by walking upward from cwd looking for a project root marker.
AGENTS.md is the preferred marker; aibox.toml is accepted as a
fallback for projects that have not yet adopted AGENTS.md.
"""
from __future__ import annotations

import os
from pathlib import Path

from . import DEFAULT_DIRS

# Markers tried in order. First match wins.
_PROJECT_MARKERS = ("AGENTS.md", "aibox.toml")


def find_project_root(start: Path | str | None = None) -> Path:
    """Walk upward from ``start`` (default: cwd) looking for a project marker.

    Tries AGENTS.md, processkit.toml, and aibox.toml in that order.
    Returns the directory containing the first marker found. If none is
    found, returns the starting directory so callers can still proceed
    with sensible defaults rather than crashing.
    """
    here = Path(start) if start else Path.cwd()
    here = here.resolve()
    if here.is_file():
        here = here.parent
    candidate = here
    while True:
        for marker in _PROJECT_MARKERS:
            if (candidate / marker).is_file():
                return candidate
        if candidate.parent == candidate:
            return here
        candidate = candidate.parent


def find_processkit_root(server_path: Path | str) -> Path | None:
    """Locate the processkit checkout root containing ``server_path``.

    Looks for ``src/lib/processkit/__init__.py`` upward from the given
    server script. Returns the directory containing ``src/`` or None if
    the script is not running from a processkit checkout (e.g. installed
    by aibox into ``context/skills/``).
    """
    here = Path(server_path).resolve().parent
    while True:
        if (here / "src" / "lib" / "processkit" / "__init__.py").is_file():
            return here
        if here.parent == here:
            return None
        here = here.parent


def context_dir(kind: str, root: Path | str | None = None) -> Path:
    """Return the base directory under ``context/`` for entities of ``kind``.

    Reads directory overrides from processkit.toml (or aibox.toml legacy)
    via config.load_config(); otherwise uses the default subdirectory from
    DEFAULT_DIRS.

    This returns the *root* of the kind's directory, without any sharding
    subdirectories. Use ``entity_path()`` when writing a new entity so that
    sharding is applied automatically.
    """
    from . import config  # local import to avoid circular dependency at module level
    root = Path(root) if root else find_project_root()
    cfg = config.load_config(root)
    override = cfg.directory_for(kind)
    sub = override if override else DEFAULT_DIRS.get(kind, kind.lower())
    return root / "context" / sub


def entity_path(
    kind: str,
    entity_id: str,
    created_at: str | None = None,
    root: Path | str | None = None,
) -> Path:
    """Return the full file path for a new entity, applying sharding rules.

    When date-based sharding is configured for ``kind``, entities are
    written into ``context/<subdir>/{year}/{month}/`` rather than the flat
    ``context/<subdir>/`` directory. The ``created_at`` parameter (an ISO
    8601 string) is used to derive the year and month. If ``created_at``
    is None, the current UTC date is used.

    Falls back to ``context_dir(kind) / entity_id.md`` when sharding is
    not configured.
    """
    from . import config  # local import to avoid circular dependency
    from datetime import datetime, timezone

    root = Path(root) if root else find_project_root()
    base = context_dir(kind, root)
    cfg = config.load_config(root)

    shard = cfg.sharding.get(kind)
    if shard and shard.get("scheme") == "date":
        if created_at:
            # Parse ISO 8601 — tolerate both "Z" and "+00:00" suffixes.
            ts_str = created_at.replace("Z", "+00:00")
            try:
                ts = datetime.fromisoformat(ts_str)
            except ValueError:
                ts = datetime.now(timezone.utc)
        else:
            ts = datetime.now(timezone.utc)
        shard_dir = base / f"{ts.year:04d}" / f"{ts.month:02d}"
        return shard_dir / f"{entity_id}.md"

    return base / f"{entity_id}.md"


def primitive_schemas_dir(root: Path | str | None = None) -> Path | None:
    """Where the JSON-Schema YAML files live.

    Tries the consumer's installed location (``context/schemas/``)
    first, then the processkit checkout (``src/primitives/schemas/``).
    Returns None if neither exists.
    """
    root = Path(root) if root else find_project_root()
    consumer = root / "context" / "schemas"
    if consumer.is_dir():
        return consumer
    processkit = root / "src" / "primitives" / "schemas"
    if processkit.is_dir():
        return processkit
    return None


def state_machines_dir(root: Path | str | None = None) -> Path | None:
    """Where state machine YAML files live.

    Tries the consumer's installed location (``context/state-machines/``)
    first, then the processkit checkout
    (``src/primitives/state-machines/``). Returns None if neither
    exists.
    """
    root = Path(root) if root else find_project_root()
    consumer = root / "context" / "state-machines"
    if consumer.is_dir():
        return consumer
    processkit = root / "src" / "primitives" / "state-machines"
    if processkit.is_dir():
        return processkit
    return None


def index_db_path(root: Path | str | None = None) -> Path:
    """Path to the SQLite index database (gitignored runtime cache).

    Reads an override from processkit.toml ``[index] path`` if set.
    Otherwise uses the default ``context/.cache/processkit/index.sqlite``.
    Creates the parent directory as a side effect.
    """
    from . import config  # local import to avoid circular at module level
    root = Path(root) if root else find_project_root()
    cfg = config.load_config(root)
    if cfg.index_path:
        db = root / cfg.index_path
    else:
        db = root / "context" / ".cache" / "processkit" / "index.sqlite"
    db.parent.mkdir(parents=True, exist_ok=True)
    return db


def env_override(name: str) -> Path | None:
    """Return an env-var path override if set, else None."""
    value = os.environ.get(name)
    return Path(value) if value else None
