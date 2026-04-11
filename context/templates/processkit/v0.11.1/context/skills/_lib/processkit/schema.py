"""Load primitive schemas and validate entity ``spec`` blocks.

Schemas live in ``src/primitives/schemas/<kind-lowercase>.yaml`` and are
themselves processkit entities (``kind: Schema``). Each schema's
``spec.spec_schema`` is a JSON Schema (draft 2020-12) for the target
primitive's ``spec`` block.

This module is the bridge between schema files on disk and runtime
validation in MCP servers. Validation uses the ``jsonschema`` library
(declared in each server's PEP 723 deps).
"""
from __future__ import annotations

from functools import lru_cache
from pathlib import Path
from typing import Any

import yaml

from . import paths


class SchemaError(ValueError):
    """Raised when a schema cannot be loaded or applied."""


@lru_cache(maxsize=64)
def load_schema(kind: str, schemas_dir: Path | None = None) -> dict[str, Any]:
    """Load the schema entity for ``kind`` and return its spec block.

    The returned dict has keys: ``description``, ``id_prefix``,
    ``state_machine``, ``default_directory``, ``spec_schema``.
    """
    schemas_dir = schemas_dir or paths.primitive_schemas_dir()
    if schemas_dir is None:
        raise SchemaError("no schemas directory found")
    candidate = schemas_dir / f"{kind.lower()}.yaml"
    if not candidate.is_file():
        raise SchemaError(f"no schema for kind={kind!r} at {candidate}")
    text = candidate.read_text()
    try:
        data = yaml.safe_load(text)
    except yaml.YAMLError as e:
        raise SchemaError(f"invalid YAML in {candidate}: {e}") from e
    if not isinstance(data, dict) or "spec" not in data:
        raise SchemaError(f"{candidate} is not a Schema entity")
    return data["spec"]


def validate_spec(kind: str, spec: dict[str, Any]) -> list[str]:
    """Validate ``spec`` against the schema for ``kind``.

    Returns a list of human-readable error messages. An empty list means
    the spec is valid. If no schema exists for the kind, returns an empty
    list (i.e., schemas are advisory until they exist for all primitives).
    """
    try:
        schema = load_schema(kind)
    except SchemaError:
        return []
    json_schema = schema.get("spec_schema")
    if not json_schema:
        return []
    try:
        import jsonschema
    except ModuleNotFoundError:
        return []
    validator = jsonschema.Draft202012Validator(json_schema)
    errors = sorted(validator.iter_errors(spec), key=lambda e: list(e.absolute_path))
    return [_format_error(e) for e in errors]


def list_known_kinds(schemas_dir: Path | None = None) -> list[str]:
    """Return all primitive kinds for which a schema file exists."""
    schemas_dir = schemas_dir or paths.primitive_schemas_dir()
    if schemas_dir is None:
        return []
    out: list[str] = []
    for f in sorted(schemas_dir.glob("*.yaml")):
        try:
            text = f.read_text()
            data = yaml.safe_load(text)
            if isinstance(data, dict) and data.get("kind") == "Schema":
                target = data.get("metadata", {}).get("target_kind")
                if target:
                    out.append(target)
        except Exception:
            continue
    return out


def _format_error(error: Any) -> str:
    path = ".".join(str(p) for p in error.absolute_path) or "<root>"
    return f"{path}: {error.message}"
