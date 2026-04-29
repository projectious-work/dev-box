"""Entity file model and I/O.

A processkit entity is a Markdown file with YAML frontmatter following the
shape defined in `src/primitives/FORMAT.md`:

    ---
    apiVersion: processkit.projectious.work/v1
    kind: <Kind>
    metadata:
      id: <PREFIX>-<id>
      created: <iso8601>
      updated: <iso8601 | omitted>
      labels: { ... }
    spec:
      ...
    ---

    <Markdown body>

This module reads, writes, and validates entity files at the structural
level. Schema-aware validation lives in `processkit.schema`.
"""
from __future__ import annotations

import datetime as _dt
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from . import API_VERSION
from .frontmatter import FrontmatterError, parse, render


class EntityError(ValueError):
    """Raised when an entity file is structurally invalid."""


class NotAnEntityError(EntityError):
    """Raised when a file is not a processkit entity and should be silently skipped.

    This covers two cases:
    - The file has no YAML frontmatter block at all (README, INDEX, …).
    - The file has frontmatter but none of the processkit top-level keys
      (``apiVersion``, ``kind``, ``metadata``, ``spec``), meaning it is some
      other YAML-frontmatter file (e.g. a SKILL.md in Agent Skills format).

    Distinct from ``EntityError``, which is reserved for files that *claim*
    to be processkit entities (``apiVersion`` present) but are malformed.
    """


@dataclass
class Entity:
    """A parsed entity file.

    The four top-level keys are split into named fields. Anything extra
    in the frontmatter is preserved in `extra` and round-trips when the
    entity is rendered back to text.
    """

    apiVersion: str
    kind: str
    metadata: dict[str, Any]
    spec: dict[str, Any]
    body: str = ""
    extra: dict[str, Any] = field(default_factory=dict)
    path: Path | None = None  # set when loaded from disk

    # ------------------------------------------------------------------
    # Derived helpers
    # ------------------------------------------------------------------
    @property
    def id(self) -> str:
        return self.metadata["id"]

    @property
    def created(self) -> str:
        return self.metadata["created"]

    @property
    def updated(self) -> str | None:
        return self.metadata.get("updated")

    @property
    def labels(self) -> dict[str, str]:
        return self.metadata.get("labels", {}) or {}

    # ------------------------------------------------------------------
    # Round-trip
    # ------------------------------------------------------------------
    def to_dict(self) -> dict[str, Any]:
        data = {
            "apiVersion": self.apiVersion,
            "kind": self.kind,
            "metadata": self.metadata,
            "spec": self.spec,
        }
        if self.extra:
            for k, v in self.extra.items():
                data[k] = v
        return data

    def to_text(self) -> str:
        return render(self.to_dict(), self.body)

    def write(self, path: Path | str | None = None) -> Path:
        """Write the entity to disk. Sets `metadata.updated` to now.

        If `path` is provided it overrides `self.path`. The parent
        directory is created if missing.
        """
        target = Path(path) if path else self.path
        if target is None:
            raise EntityError("Entity has no path; pass one to write()")
        target.parent.mkdir(parents=True, exist_ok=True)
        # Stamp updated only if the entity already had a created time
        # different from now (avoid stamping on initial create).
        if self.metadata.get("created"):
            now = _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")
            if self.metadata.get("created") != now:
                self.metadata["updated"] = now
        target.write_text(self.to_text(), encoding="utf-8")
        self.path = target
        return target

    def touch(self) -> None:
        """Update `metadata.updated` to now without writing."""
        self.metadata["updated"] = _now_iso()


# ---------------------------------------------------------------------------
# Module-level constructors
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def from_text(text: str, path: Path | str | None = None) -> Entity:
    """Parse a string of markdown+frontmatter into an Entity."""
    try:
        data, body = parse(text)
    except FrontmatterError as e:
        # No YAML frontmatter at all — not an entity, skip silently.
        raise NotAnEntityError(str(e)) from e
    return _from_dict(data, body, Path(path) if path else None)


def load(path: Path | str) -> Entity:
    """Load an entity from a file path."""
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    try:
        return from_text(text, p)
    except NotAnEntityError:
        raise  # preserve type so callers can silently skip non-entities
    except EntityError as e:
        raise EntityError(f"{p}: {e}") from e


def _from_dict(data: dict[str, Any], body: str, path: Path | None) -> Entity:
    _ENTITY_KEYS = ("apiVersion", "kind", "metadata", "spec")
    missing = [k for k in _ENTITY_KEYS if k not in data]
    if missing:
        if "apiVersion" not in data:
            # No apiVersion — this is not a processkit entity (e.g. SKILL.md
            # in Agent Skills format, or another YAML-frontmatter file).
            raise NotAnEntityError(f"missing required keys: {', '.join(missing)}")
        # Has apiVersion but is otherwise malformed — a real error worth recording.
        raise EntityError(f"missing required keys: {', '.join(missing)}")
    metadata = data["metadata"]
    spec = data["spec"]
    if not isinstance(metadata, dict):
        raise EntityError("metadata must be a mapping")
    if not isinstance(spec, dict):
        raise EntityError("spec must be a mapping")
    if "id" not in metadata:
        raise EntityError("metadata.id is required")
    if "created" not in metadata:
        # Tolerate creation-in-progress: allow missing created and stamp it
        metadata["created"] = _now_iso()
    extra = {
        k: v for k, v in data.items()
        if k not in ("apiVersion", "kind", "metadata", "spec")
    }
    return Entity(
        apiVersion=data["apiVersion"],
        kind=data["kind"],
        metadata=metadata,
        spec=spec,
        body=body,
        extra=extra,
        path=path,
    )


def new(
    kind: str,
    id: str,
    spec: dict[str, Any],
    *,
    labels: dict[str, str] | None = None,
    body: str = "",
) -> Entity:
    """Construct a fresh entity ready to be written."""
    metadata: dict[str, Any] = {
        "id": id,
        "created": _now_iso(),
    }
    if labels:
        metadata["labels"] = labels
    return Entity(
        apiVersion=API_VERSION,
        kind=kind,
        metadata=metadata,
        spec=spec,
        body=body,
    )
