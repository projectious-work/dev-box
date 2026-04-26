"""Export / import team-member bundles.

Export produces a tar.gz with:

    <slug>/
        persona.md
        card.json
        team-member.md
        knowledge/...
        skills/...
        lessons/...

Excludes: journal/, relations/, private/. Redacts memory files whose
frontmatter declares ``sensitivity: confidential`` or ``sensitivity: pii``.

Import extracts into a temporary directory, validates:
  - ``team-member.md`` frontmatter against the TeamMember schema
  - ``card.json`` against the A2A Agent Card schema
  - presence of a ``signature`` object on the card
Signature cryptographic verification is deferred.
"""
from __future__ import annotations

import json
import shutil
import tarfile
import tempfile
from pathlib import Path
from typing import Any

import yaml


class ImportError(ValueError):
    """Raised when a tarball fails validation during import."""


INCLUDE_DIRS = ("knowledge", "skills", "lessons")
EXCLUDE_DIRS = ("journal", "relations", "private", "working")
TOP_FILES = ("persona.md", "card.json", "team-member.md")
REDACT_SENSITIVITY = {"confidential", "pii"}


# ---------------------------------------------------------------------------
# Export
# ---------------------------------------------------------------------------

def export(tm_dir: Path, dest: Path) -> dict[str, Any]:
    """Build a tar.gz bundle. Returns {included, redacted}."""
    slug = tm_dir.name
    included: list[str] = []
    redacted: list[str] = []

    with tempfile.TemporaryDirectory() as tmp:
        staging = Path(tmp) / slug
        staging.mkdir(parents=True)

        for fname in TOP_FILES:
            src = tm_dir / fname
            if src.is_file():
                shutil.copy2(src, staging / fname)
                included.append(fname)

        for sub in INCLUDE_DIRS:
            src_dir = tm_dir / sub
            if not src_dir.is_dir():
                continue
            dest_dir = staging / sub
            dest_dir.mkdir(parents=True, exist_ok=True)
            for child in src_dir.rglob("*"):
                if child.is_dir():
                    continue
                if child.name == ".gitkeep":
                    # Preserve structure markers
                    rel = child.relative_to(tm_dir)
                    out = staging / rel
                    out.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(child, out)
                    continue
                if _is_redactable(child):
                    redacted.append(str(child.relative_to(tm_dir)))
                    continue
                rel = child.relative_to(tm_dir)
                out = staging / rel
                out.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(child, out)
                included.append(str(rel))

        dest.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(dest, "w:gz") as tar:
            tar.add(staging, arcname=slug)

    return {"included": included, "redacted": redacted}


def _is_redactable(path: Path) -> bool:
    if path.suffix.lower() not in {".md", ".markdown"}:
        return False
    try:
        text = path.read_text(encoding="utf-8")
    except Exception:
        return False
    header = _parse_front_matter(text)
    if not header:
        return False
    sens = header.get("sensitivity")
    return sens in REDACT_SENSITIVITY


def _parse_front_matter(text: str) -> dict[str, Any] | None:
    if not text.startswith("---"):
        return None
    try:
        _, rest = text.split("---", 1)
        block, _body = rest.split("---", 1)
    except ValueError:
        return None
    try:
        data = yaml.safe_load(block)
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


# ---------------------------------------------------------------------------
# Import
# ---------------------------------------------------------------------------

def import_bundle(tarball: Path, tm_root: Path, assets_dir: Path) -> dict[str, Any]:
    """Extract + validate + install a tarball."""
    if not tarball.is_file():
        raise ImportError(f"tarball not found: {tarball}")

    with tempfile.TemporaryDirectory() as tmp:
        staging = Path(tmp)
        with tarfile.open(tarball, "r:gz") as tar:
            _safe_extract(tar, staging)
        candidates = [p for p in staging.iterdir() if p.is_dir()]
        if len(candidates) != 1:
            raise ImportError(
                f"expected exactly one top-level directory in tarball, got {len(candidates)}"
            )
        bundle = candidates[0]
        slug = bundle.name
        tm_md = bundle / "team-member.md"
        if not tm_md.is_file():
            raise ImportError("team-member.md missing from bundle")
        card_json = bundle / "card.json"
        if not card_json.is_file():
            raise ImportError("card.json missing from bundle")

        # Validate entity frontmatter
        tm_header = _parse_front_matter(tm_md.read_text(encoding="utf-8"))
        if not tm_header:
            raise ImportError("team-member.md frontmatter missing or invalid")
        if tm_header.get("kind") != "TeamMember":
            raise ImportError("team-member.md is not kind=TeamMember")
        spec = tm_header.get("spec") or {}
        _validate_tm_spec(spec)

        # Validate card.json
        try:
            card = json.loads(card_json.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            raise ImportError(f"card.json is not valid JSON: {e}") from e
        _validate_card(card, assets_dir)
        if "signature" not in card or not isinstance(card["signature"], dict):
            raise ImportError("card.json is missing a signature object")

        # Install
        dest = tm_root / slug
        if dest.exists():
            raise ImportError(f"destination {dest} already exists")
        tm_root.mkdir(parents=True, exist_ok=True)
        shutil.copytree(bundle, dest)

    return {"slug": slug, "path": str(dest)}


def _validate_tm_spec(spec: dict[str, Any]) -> None:
    required = ("type", "name", "slug")
    missing = [k for k in required if k not in spec]
    if missing:
        raise ImportError(f"team-member.md spec is missing required keys: {missing}")
    if spec.get("type") not in {"human", "ai-agent", "service"}:
        raise ImportError(f"team-member.md spec.type invalid: {spec.get('type')!r}")


def _validate_card(card: dict[str, Any], assets_dir: Path) -> None:
    schema_path = assets_dir / "agent-card.schema.json"
    if not schema_path.is_file():
        return
    try:
        import jsonschema
    except ModuleNotFoundError:
        return
    schema_obj = json.loads(schema_path.read_text(encoding="utf-8"))
    try:
        jsonschema.Draft202012Validator(schema_obj).validate(card)
    except jsonschema.ValidationError as e:
        raise ImportError(f"card.json failed schema validation: {e.message}") from e


def _safe_extract(tar: tarfile.TarFile, dest: Path) -> None:
    """Extract with path-traversal protection."""
    dest = dest.resolve()
    for member in tar.getmembers():
        target = (dest / member.name).resolve()
        if not str(target).startswith(str(dest)):
            raise ImportError(f"unsafe path in tarball: {member.name}")
    tar.extractall(dest)
