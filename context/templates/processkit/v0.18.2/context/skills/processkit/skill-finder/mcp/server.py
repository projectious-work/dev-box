#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""processkit skill-finder MCP server.

Tools:

    find_skill(task_description)
        -> {skill, description, skill_md_path, category} | {error}

    list_skills(category?)
        -> [{skill, description, category, has_mcp}]
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

from processkit import paths  # noqa: E402

server = FastMCP("processkit-skill-finder")

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_SKILL_CATEGORIES = (
    "processkit",
    "engineering",
    "devops",
    "data-ai",
    "product",
    "documents",
    "design",
)


def _skills_root() -> Path:
    return paths.find_project_root() / "context" / "skills"


def _skill_finder_md() -> Path:
    return _skills_root() / "processkit" / "skill-finder" / "SKILL.md"


def _parse_trigger_table(md: str) -> list[tuple[list[str], str]]:
    """Return [(phrases, skill_name), ...] from the trigger-phrase table."""
    results: list[tuple[list[str], str]] = []
    in_table = False
    for line in md.splitlines():
        stripped = line.strip()
        if "If the user says" in stripped and "Skill to load" in stripped:
            in_table = True
            continue
        if not in_table:
            continue
        if stripped.startswith("|---"):
            continue
        if not stripped.startswith("|"):
            if in_table and stripped == "":
                continue
            in_table = False
            continue
        parts = [p.strip() for p in stripped.strip("|").split("|")]
        if len(parts) < 2:
            continue
        raw_phrases = parts[0]
        skill_name = parts[1].strip("`").strip()
        if not skill_name or skill_name == "Skill to load":
            continue
        phrases = [
            p.strip().strip('"')
            for p in raw_phrases.split('", "')
            if p.strip().strip('"')
        ]
        if not phrases:
            phrases = [raw_phrases.strip('"')]
        results.append((phrases, skill_name))
    return results


def _score(task: str, phrases: list[str]) -> float:
    """Simple keyword overlap score between task and trigger phrases."""
    task_lower = task.lower()
    task_tokens = set(re.split(r"\W+", task_lower))
    best = 0.0
    for phrase in phrases:
        phrase_lower = phrase.lower()
        # Substring match — highest weight
        if phrase_lower in task_lower:
            return 1.0
        phrase_tokens = set(re.split(r"\W+", phrase_lower))
        if not phrase_tokens:
            continue
        overlap = task_tokens & phrase_tokens
        score = len(overlap) / len(phrase_tokens)
        if score > best:
            best = score
    return best


def _read_skill_frontmatter(skill_md: Path) -> dict:
    """Extract name, description, category from SKILL.md frontmatter."""
    try:
        import yaml
        text = skill_md.read_text(encoding="utf-8")
        if not text.startswith("---"):
            return {}
        end = text.index("---", 3)
        fm = yaml.safe_load(text[3:end])
        if not isinstance(fm, dict):
            return {}
        meta = fm.get("metadata", {}).get("processkit", {})
        return {
            "name": fm.get("name", skill_md.parent.name),
            "description": (fm.get("description") or "").strip(),
            "category": meta.get("category", ""),
            "layer": meta.get("layer"),
        }
    except Exception:
        return {"name": skill_md.parent.name, "description": "", "category": ""}


def _all_skills() -> list[dict]:
    """Walk context/skills/ and return metadata for all skills."""
    root = _skills_root()
    skills = []
    for cat_dir in sorted(root.iterdir()):
        if cat_dir.name.startswith("_") or not cat_dir.is_dir():
            continue
        for skill_dir in sorted(cat_dir.iterdir()):
            if not skill_dir.is_dir():
                continue
            skill_md = skill_dir / "SKILL.md"
            if not skill_md.exists():
                continue
            fm = _read_skill_frontmatter(skill_md)
            fm["skill"] = skill_dir.name
            fm["category"] = cat_dir.name
            fm["has_mcp"] = (skill_dir / "mcp" / "server.py").exists()
            fm["skill_md_path"] = str(
                skill_md.relative_to(paths.find_project_root())
            )
            skills.append(fm)
    return skills


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------

@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def find_skill(task_description: str) -> dict:
    """Find the processkit skill that best matches a task description.

    Searches the trigger-phrase table in skill-finder and returns the
    skill name, its SKILL.md path, and a one-line description. Call this
    before acting on any processkit domain task to confirm you are using
    the right skill and conventions. 1% rule: if there is a 1% chance a processkit skill covers this task, call route_task before acting.

    Parameters
    ----------
    task_description:
        What the user asked for, in their words. The more natural the
        phrasing, the better the match.

    Returns
    -------
    {skill, description, skill_md_path, category}  on match
    {error, candidates}                              on no confident match
    """
    finder_md = _skill_finder_md()
    if not finder_md.exists():
        return {"error": "skill-finder SKILL.md not found at expected path"}

    text = finder_md.read_text(encoding="utf-8")
    table = _parse_trigger_table(text)
    if not table:
        return {"error": "could not parse trigger table from skill-finder"}

    scored: list[tuple[float, str]] = []
    for phrases, skill_name in table:
        s = _score(task_description, phrases)
        if s > 0:
            scored.append((s, skill_name))

    scored.sort(key=lambda x: -x[0])

    if not scored:
        return {
            "error": "no matching skill found",
            "hint": (
                "Try rephrasing in natural language, or call list_skills() "
                "to browse the full catalog."
            ),
        }

    best_score, best_skill = scored[0]

    # Find the SKILL.md path for the matched skill
    root = _skills_root()
    skill_md_path = None
    category = None
    for cat_dir in root.iterdir():
        if cat_dir.name.startswith("_") or not cat_dir.is_dir():
            continue
        candidate = cat_dir / best_skill / "SKILL.md"
        if candidate.exists():
            skill_md_path = str(
                candidate.relative_to(paths.find_project_root())
            )
            category = cat_dir.name
            break

    # Read description from SKILL.md
    description = ""
    if skill_md_path:
        fm = _read_skill_frontmatter(
            paths.find_project_root() / skill_md_path
        )
        description = fm.get("description", "").split("\n")[0]

    result: dict = {
        "skill": best_skill,
        "description": description,
        "skill_md_path": skill_md_path,
        "category": category,
        "match_confidence": round(best_score, 2),
    }

    # Surface close runners-up so the agent can sanity-check
    if len(scored) > 1 and scored[1][0] >= best_score * 0.8:
        result["also_consider"] = [s for _, s in scored[1:4]]

    return result


@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def list_skills(category: str | None = None) -> list[dict]:
    """List all processkit skills, optionally filtered by category.

    Returns name, one-line description, category, and whether the skill
    has an MCP server. Use this to browse the catalog when find_skill
    returns no confident match, or to discover what skills exist in a
    given domain.

    Parameters
    ----------
    category:
        Optional filter. One of: processkit, engineering, devops,
        data-ai, product, documents, design. Omit for all skills.

    Returns
    -------
    [{skill, description, category, has_mcp, skill_md_path}]
    """
    skills = _all_skills()
    if category:
        skills = [s for s in skills if s.get("category") == category]
    # Return a concise view — first line of description only
    return [
        {
            "skill": s["skill"],
            "description": (s.get("description") or "").split("\n")[0].strip(),
            "category": s.get("category", ""),
            "has_mcp": s.get("has_mcp", False),
            "skill_md_path": s.get("skill_md_path", ""),
        }
        for s in skills
    ]


if __name__ == "__main__":
    server.run(transport="stdio")
