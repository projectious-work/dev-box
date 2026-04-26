"""Consistency checks for team-manager.

10 checks per team-member. Each returns findings shaped as
``{severity, code, team_member, path, message}``.

Check codes:

    team.drift.schema              error    — entity validates against SCHEMA-team-member
    team.drift.tier_missing        error    — expected tier subdirs exist
    team.drift.dangling_ref        error    — default_role + relationships[].with resolve
    team.name_collision            error    — no two active members share name or slug
    team.name.off_pool             warning  — ai-agent name in pool
    team.drift.orphan_file         warning  — no unexpected top-level files
    team.sensitivity.leak_risk     warning  — confidential/pii files live under private/
    team.private.not_ignored       warning  — private/ covered by .gitignore
    team.memory.bad_header         warning  — memory files carry required frontmatter
    team.card.stale                warning  — card.json name/role/seniority match entity
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import yaml

try:
    from processkit import entity as _entity_mod
    from processkit import schema as _schema_mod
except Exception:  # pragma: no cover — defensive; server injects sys.path
    _entity_mod = None  # type: ignore[assignment]
    _schema_mod = None  # type: ignore[assignment]


EXPECTED_TIERS = ("knowledge", "journal", "skills", "relations", "lessons")
EXPECTED_TOP_FILES = {"persona.md", "card.json", "team-member.md"}
EXPECTED_TOP_DIRS = set(EXPECTED_TIERS) | {"working", "private"}


def _finding(severity: str, code: str, tm: str, path: Path | str | None, message: str) -> dict:
    return {
        "severity": severity,
        "code": code,
        "team_member": tm,
        "path": str(path) if path else "",
        "message": message,
    }


def _load_entity(tm_md: Path):
    if _entity_mod is None:
        return None
    try:
        return _entity_mod.load(tm_md)
    except Exception:
        return None


# ---------------------------------------------------------------------------
# Single-member checks
# ---------------------------------------------------------------------------

def check_one(
    root: Path,
    tm_dir: Path,
    slug: str,
    pool_path: Path,
    assets_dir: Path,
    all_members: list[dict] | None = None,
) -> list[dict]:
    findings: list[dict] = []

    if not tm_dir.is_dir():
        findings.append(_finding(
            "error", "team.drift.tier_missing", slug, tm_dir,
            f"team-member directory does not exist: {tm_dir}",
        ))
        return findings

    tm_md = tm_dir / "team-member.md"
    ent = _load_entity(tm_md) if tm_md.is_file() else None

    # 1. team.drift.schema
    if ent is None:
        findings.append(_finding(
            "error", "team.drift.schema", slug, tm_md,
            "team-member.md missing or unreadable",
        ))
    else:
        if _schema_mod is not None:
            errs = _schema_mod.validate_spec("TeamMember", ent.spec or {})
            for err in errs:
                findings.append(_finding(
                    "error", "team.drift.schema", slug, tm_md,
                    f"schema validation failed: {err}",
                ))

    # 2. team.drift.tier_missing
    for tier in EXPECTED_TIERS:
        sub = tm_dir / tier
        if not sub.is_dir():
            findings.append(_finding(
                "error", "team.drift.tier_missing", slug, sub,
                f"expected tier subdir missing: {tier}/",
            ))

    # 3. team.drift.dangling_ref
    if ent is not None:
        spec = ent.spec or {}
        default_role = spec.get("default_role")
        if default_role:
            if not _role_exists(root, default_role):
                findings.append(_finding(
                    "error", "team.drift.dangling_ref", slug, tm_md,
                    f"default_role {default_role!r} does not resolve to an existing Role",
                ))
        for rel in spec.get("relationships") or []:
            with_ref = (rel or {}).get("with")
            if not with_ref:
                continue
            if not _team_member_exists(root, with_ref):
                findings.append(_finding(
                    "error", "team.drift.dangling_ref", slug, tm_md,
                    f"relationship target {with_ref!r} does not resolve",
                ))

    # 4. team.name_collision (computed at aggregate layer; local check vs other members)
    if all_members is not None and ent is not None:
        spec = ent.spec or {}
        for other in all_members:
            if other["slug"] == slug:
                continue
            if not other.get("active", True):
                continue
            if not spec.get("active", True):
                continue
            if other.get("name") and other["name"] == spec.get("name"):
                findings.append(_finding(
                    "error", "team.name_collision", slug, tm_md,
                    f"active name collision with {other['slug']!r}: {spec.get('name')!r}",
                ))
            if other.get("slug") == spec.get("slug"):
                findings.append(_finding(
                    "error", "team.name_collision", slug, tm_md,
                    f"active slug collision with {other['slug']!r}",
                ))

    # 5. team.name.off_pool
    if ent is not None:
        spec = ent.spec or {}
        if spec.get("type") == "ai-agent":
            pool_names = _load_pool_names(pool_path)
            name = spec.get("name", "")
            if name and name not in pool_names:
                findings.append(_finding(
                    "warning", "team.name.off_pool", slug, tm_md,
                    f"ai-agent name {name!r} is not in the name pool",
                ))

    # 6. team.drift.orphan_file
    for child in tm_dir.iterdir():
        if child.is_dir() and child.name not in EXPECTED_TOP_DIRS:
            findings.append(_finding(
                "warning", "team.drift.orphan_file", slug, child,
                f"unexpected top-level directory: {child.name}/",
            ))
        elif child.is_file() and child.name not in EXPECTED_TOP_FILES:
            # tolerate hidden files like .gitignore
            if child.name.startswith("."):
                continue
            findings.append(_finding(
                "warning", "team.drift.orphan_file", slug, child,
                f"unexpected top-level file: {child.name}",
            ))

    # 7. team.sensitivity.leak_risk
    for tier in EXPECTED_TIERS + ("working",):
        sub = tm_dir / tier
        if not sub.is_dir():
            continue
        for md in sub.rglob("*.md"):
            header = _read_front_matter(md)
            if header is None:
                continue
            sens = header.get("sensitivity")
            if sens in {"confidential", "pii"}:
                findings.append(_finding(
                    "warning", "team.sensitivity.leak_risk", slug, md,
                    f"{sens} file outside private/: {md.relative_to(tm_dir)}",
                ))

    # 8. team.private.not_ignored
    priv = tm_dir / "private"
    if priv.is_dir():
        if not _is_gitignored(root, priv):
            findings.append(_finding(
                "warning", "team.private.not_ignored", slug, priv,
                "private/ directory is not covered by a .gitignore rule",
            ))

    # 9. team.memory.bad_header
    for tier in EXPECTED_TIERS:
        sub = tm_dir / tier
        if not sub.is_dir():
            continue
        for md in sub.rglob("*.md"):
            header = _read_front_matter(md)
            if header is None:
                findings.append(_finding(
                    "warning", "team.memory.bad_header", slug, md,
                    "memory file missing YAML frontmatter",
                ))
                continue
            required = ("tier", "source", "sensitivity", "created")
            missing = [k for k in required if k not in header]
            if missing:
                findings.append(_finding(
                    "warning", "team.memory.bad_header", slug, md,
                    f"memory file missing required keys: {missing}",
                ))

    # 10. team.card.stale
    card_path = tm_dir / "card.json"
    if ent is not None and card_path.is_file():
        try:
            card = json.loads(card_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            findings.append(_finding(
                "warning", "team.card.stale", slug, card_path,
                "card.json is not valid JSON",
            ))
        else:
            spec = ent.spec or {}
            mismatches = []
            if card.get("name") and spec.get("name") and card["name"] != spec["name"]:
                mismatches.append(f"name({card['name']!r}!={spec['name']!r})")
            if spec.get("default_role") and card.get("role") and card["role"] != spec["default_role"]:
                mismatches.append(f"role({card['role']!r}!={spec['default_role']!r})")
            if spec.get("default_seniority") and card.get("seniority") and card["seniority"] != spec["default_seniority"]:
                mismatches.append(f"seniority({card['seniority']!r}!={spec['default_seniority']!r})")
            if mismatches:
                findings.append(_finding(
                    "warning", "team.card.stale", slug, card_path,
                    f"card.json out of sync with team-member.md: {', '.join(mismatches)}",
                ))

    return findings


# ---------------------------------------------------------------------------
# Aggregate
# ---------------------------------------------------------------------------

def check_all(
    root: Path,
    tm_root: Path,
    pool_path: Path,
    assets_dir: Path,
) -> dict[str, Any]:
    members_summary: list[dict[str, Any]] = []
    if tm_root.is_dir():
        for child in sorted(tm_root.iterdir()):
            if not child.is_dir():
                continue
            tm_md = child / "team-member.md"
            if not tm_md.is_file():
                continue
            ent = _load_entity(tm_md)
            if ent is None or ent.kind != "TeamMember":
                continue
            spec = ent.spec or {}
            members_summary.append({
                "slug": spec.get("slug") or child.name,
                "name": spec.get("name"),
                "active": spec.get("active", True),
            })

    per_member: dict[str, list[dict]] = {}
    error_count = 0
    warning_count = 0
    if tm_root.is_dir():
        for child in sorted(tm_root.iterdir()):
            if not child.is_dir():
                continue
            if not (child / "team-member.md").is_file():
                continue
            slug = child.name
            findings = check_one(root, child, slug, pool_path, assets_dir, members_summary)
            per_member[slug] = findings
            for f in findings:
                if f["severity"] == "error":
                    error_count += 1
                else:
                    warning_count += 1

    return {
        "members": per_member,
        "summary": {
            "count": len(per_member),
            "errors": error_count,
            "warnings": warning_count,
        },
    }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _read_front_matter(path: Path) -> dict[str, Any] | None:
    try:
        text = path.read_text(encoding="utf-8")
    except Exception:
        return None
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


def _load_pool_names(pool_path: Path) -> set[str]:
    if not pool_path.is_file():
        return set()
    try:
        docs = list(yaml.safe_load_all(pool_path.read_text(encoding="utf-8")))
    except yaml.YAMLError:
        return set()
    for doc in docs:
        if isinstance(doc, dict) and "spec" in doc:
            names_map = (doc.get("spec") or {}).get("names") or {}
            out: set[str] = set()
            for bucket in ("feminine", "masculine", "neutral"):
                for name in names_map.get(bucket) or []:
                    out.add(name)
            return out
    return set()


def _role_exists(root: Path, role_id: str) -> bool:
    """True if a Role entity exists with the given ID.

    Looks in ``context/roles/`` first (fast path); falls back to the
    index if present.
    """
    roles_dir = root / "context" / "roles"
    if roles_dir.is_dir():
        for md in roles_dir.rglob("*.md"):
            header = _read_front_matter(md)
            if not header:
                continue
            meta = header.get("metadata") or {}
            if meta.get("id") == role_id and header.get("kind") == "Role":
                return True
    return False


def _team_member_exists(root: Path, ref: str) -> bool:
    tm_root = root / "context" / "team-members"
    if not tm_root.is_dir():
        return False
    slug = ref[len("TEAMMEMBER-"):] if ref.startswith("TEAMMEMBER-") else ref
    return (tm_root / slug / "team-member.md").is_file()


def _is_gitignored(root: Path, path: Path) -> bool:
    """Ask git whether the path is ignored. Fallback: text-match common patterns."""
    try:
        rel = path.relative_to(root)
    except ValueError:
        return False
    try:
        res = subprocess.run(
            ["git", "-C", str(root), "check-ignore", "-q", str(rel) + "/"],
            capture_output=True,
            timeout=5,
        )
        if res.returncode == 0:
            return True
        if res.returncode == 1:
            return False
        # other return code → git unavailable / not a repo — fall through
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    # Fallback: scan .gitignore for a 'private/' or the specific rel path.
    gi = root / ".gitignore"
    if not gi.is_file():
        return False
    patterns = [
        line.strip()
        for line in gi.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    ]
    rel_str = str(rel).replace(os.sep, "/")
    for pat in patterns:
        if pat in (rel_str, rel_str + "/", "private/", "**/private/", "*/private/"):
            return True
    return False


import os  # noqa: E402 — used by _is_gitignored fallback
