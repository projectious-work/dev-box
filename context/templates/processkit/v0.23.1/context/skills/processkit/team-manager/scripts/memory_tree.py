"""Memory-tree scaffolding for team-manager.

Creates the on-disk layout for a TeamMember:

    context/team-members/<slug>/
        team-member.md
        persona.md
        card.json
        knowledge/.gitkeep
        journal/.gitkeep
        skills/.gitkeep
        relations/.gitkeep
        lessons/.gitkeep
        working/.gitkeep
        private/.gitkeep

Idempotent: missing parts are filled in; existing files are not modified.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

TIER_SUBDIRS = ("working", "journal", "knowledge", "skills", "relations", "lessons")
_PRIVATE = "private"


def init_tree(tm_dir: Path, slug: str, entity_obj: Any | None = None) -> list[str]:
    """Create the tree. Returns the list of paths created (relative)."""
    created: list[str] = []
    tm_dir.mkdir(parents=True, exist_ok=True)

    for sub in TIER_SUBDIRS + (_PRIVATE,):
        sub_dir = tm_dir / sub
        if not sub_dir.is_dir():
            sub_dir.mkdir(parents=True, exist_ok=True)
            created.append(f"{sub}/")
        keep = sub_dir / ".gitkeep"
        if not keep.exists():
            keep.write_text("", encoding="utf-8")
            created.append(f"{sub}/.gitkeep")

    persona = tm_dir / "persona.md"
    if not persona.exists():
        name = _display_name(entity_obj, slug)
        persona.write_text(_persona_template(name, slug), encoding="utf-8")
        created.append("persona.md")

    card = tm_dir / "card.json"
    if not card.exists():
        card.write_text(_card_template(entity_obj, slug), encoding="utf-8")
        created.append("card.json")

    tm_md = tm_dir / "team-member.md"
    if not tm_md.exists() and entity_obj is None:
        # Write a minimal templated stub; if entity exists elsewhere, skip.
        tm_md.write_text(_entity_stub(slug), encoding="utf-8")
        created.append("team-member.md")

    return created


def _display_name(entity_obj: Any | None, slug: str) -> str:
    if entity_obj is None:
        return slug
    try:
        return entity_obj.spec.get("name") or slug
    except Exception:
        return slug


def _persona_template(name: str, slug: str) -> str:
    return (
        f"# {name}\n\n"
        f"Team-member slug: `{slug}`.\n\n"
        "## Identity\n\n"
        "TODO — one-paragraph identity statement, voice, and communication style.\n\n"
        "## Boundaries\n\n"
        "TODO — explicit behavioral no-go areas.\n\n"
        "## Declared expertise\n\n"
        "TODO — depth areas.\n"
    )


def _card_template(entity_obj: Any | None, slug: str) -> str:
    card: dict[str, Any] = {
        "schemaVersion": "a2a/v0.3",
        "name": slug,
        "role": "ROLE-unassigned",
        "seniority": "specialist",
        "skills": [],
        "modalities": ["text"],
        "provider": "processkit",
        "contact": {},
        "signature": {"alg": "none", "value": ""},
    }
    if entity_obj is not None:
        try:
            spec = entity_obj.spec or {}
            card["name"] = spec.get("name") or slug
            if spec.get("default_role"):
                card["role"] = spec["default_role"]
            if spec.get("default_seniority"):
                card["seniority"] = spec["default_seniority"]
        except Exception:
            pass
    return json.dumps(card, indent=2) + "\n"


def _entity_stub(slug: str) -> str:
    return (
        "---\n"
        "apiVersion: processkit.projectious.work/v1\n"
        "kind: TeamMember\n"
        "metadata:\n"
        f"  id: TEAMMEMBER-{slug}\n"
        "  created: 1970-01-01T00:00:00+00:00\n"
        "spec:\n"
        "  type: human\n"
        f"  name: {slug}\n"
        f"  slug: {slug}\n"
        "  active: true\n"
        "---\n\n"
        "Stub. Replace via team-manager.create_team_member().\n"
    )
