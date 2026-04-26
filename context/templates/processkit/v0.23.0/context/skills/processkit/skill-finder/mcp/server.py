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

    catalog(category?, tag?, keyword?, columns?, sort_by?, output?)
        -> str (markdown table or fenced JSON/YAML) | list[dict] (json)
"""
from __future__ import annotations

import json
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
    """Extract name, description, category, and other fields from SKILL.md
    frontmatter."""
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
        provides = meta.get("provides", {}) or {}
        mcp_tools = provides.get("mcp_tools", []) or []
        tags = meta.get("tags", []) or []
        return {
            "name": fm.get("name", skill_md.parent.name),
            "description": (fm.get("description") or "").strip(),
            "category": meta.get("category", ""),
            "layer": meta.get("layer"),
            "version": meta.get("version", ""),
            "tags": tags if isinstance(tags, list) else [],
            "allowed_tools": mcp_tools if isinstance(mcp_tools, list) else [],
        }
    except Exception:
        return {
            "name": skill_md.parent.name,
            "description": "",
            "category": "",
        }


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


# ---------------------------------------------------------------------------
# Catalog helpers
# ---------------------------------------------------------------------------

_CATALOG_COLUMNS = (
    "name",
    "category",
    "description",
    "tags",
    "version",
    "layer",
    "allowed_tools",
    "has_mcp",
    "skill_md_path",
)

_DEFAULT_COLUMNS = ["name", "category", "description"]
_DEFAULT_SORT = "name"


def _skill_to_row(skill: dict, columns: list[str]) -> dict:
    """Extract requested columns from a skill dict."""
    row: dict = {}
    for col in columns:
        if col == "name":
            row["name"] = skill.get("name") or skill.get("skill", "")
        elif col == "description":
            # First line only for compact display
            desc = (skill.get("description") or "").split("\n")[0].strip()
            row["description"] = desc
        elif col == "tags":
            row["tags"] = skill.get("tags", [])
        elif col == "allowed_tools":
            row["allowed_tools"] = skill.get("allowed_tools", [])
        elif col == "has_mcp":
            row["has_mcp"] = skill.get("has_mcp", False)
        elif col == "layer":
            row["layer"] = skill.get("layer")
        elif col == "version":
            row["version"] = skill.get("version", "")
        elif col == "skill_md_path":
            row["skill_md_path"] = skill.get("skill_md_path", "")
        elif col == "category":
            row["category"] = skill.get("category", "")
        else:
            row[col] = skill.get(col)
    return row


def _render_markdown(rows: list[dict], columns: list[str]) -> str:
    """Render rows as a Markdown table."""
    if not rows:
        return "_No skills matched._"
    # Build header
    header = " | ".join(columns)
    sep = " | ".join(["---"] * len(columns))
    lines = [f"| {header} |", f"| {sep} |"]
    for row in rows:
        cells = []
        for col in columns:
            val = row.get(col)
            if isinstance(val, list):
                cell = ", ".join(str(v) for v in val)
            elif val is None:
                cell = ""
            else:
                cell = str(val)
            # Escape pipe chars inside cells
            cell = cell.replace("|", "\\|")
            cells.append(cell)
        lines.append("| " + " | ".join(cells) + " |")
    return "\n".join(lines)


def _render_yaml(rows: list[dict]) -> str:
    """Render rows as a YAML fenced block."""
    import yaml
    return "```yaml\n" + yaml.dump(rows, allow_unicode=True) + "```"


@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def catalog(
    category: str | None = None,
    tag: str | None = None,
    keyword: str | None = None,
    columns: list[str] | None = None,
    sort_by: str | None = None,
    output: str = "markdown",
) -> str | list[dict]:
    """Return a queryable view of the processkit skill catalog.

    Supports filtering, column selection, sorting, and multiple output
    formats. Use for user-facing skill discovery ("what skills do we
    have?", "show me all skills in engineering", "skills as JSON").

    Parameters
    ----------
    category:
        Filter to one category directory. One of: processkit,
        engineering, devops, data-ai, product, documents, design.
        Omit for all skills.
    tag:
        Filter to skills that carry this tag in their frontmatter
        tags list. Case-insensitive exact match.
    keyword:
        Case-insensitive substring search across skill name and
        first line of description.
    columns:
        Which columns to include. Allowed values: name, category,
        description, tags, version, layer, allowed_tools, has_mcp,
        skill_md_path. Default: [name, category, description].
    sort_by:
        Column to sort by. Default: name. Any column from the columns
        list is accepted; unknown columns fall back to name.
    output:
        One of "markdown" (default), "json", or "yaml".
        "markdown" returns a Markdown table string.
        "json" returns a list[dict] directly (no fencing).
        "yaml" returns a YAML fenced code block string.

    Returns
    -------
    str   for output="markdown" or output="yaml"
    list  for output="json"
    """
    cols = list(columns) if columns else _DEFAULT_COLUMNS
    # Validate columns
    unknown = [c for c in cols if c not in _CATALOG_COLUMNS]
    if unknown:
        return (
            f"Unknown column(s): {unknown}. "
            f"Allowed: {list(_CATALOG_COLUMNS)}"
        )

    sort_col = sort_by if sort_by in _CATALOG_COLUMNS else _DEFAULT_SORT

    # --- Gather skills via existing _all_skills() ---
    skills = _all_skills()

    # --- Apply filters ---
    if category:
        skills = [s for s in skills if s.get("category") == category]

    if tag:
        tag_lower = tag.lower()
        skills = [
            s for s in skills
            if any(
                t.lower() == tag_lower
                for t in (s.get("tags") or [])
            )
        ]

    if keyword:
        kw = keyword.lower()
        skills = [
            s for s in skills
            if kw in (s.get("name") or "").lower()
            or kw in (s.get("description") or "").lower()
        ]

    # --- Sort ---
    def _sort_key(s: dict):
        val = s.get(sort_col)
        if val is None:
            return ""
        if isinstance(val, list):
            return str(val)
        return str(val).lower()

    skills.sort(key=_sort_key)

    # --- Project to requested columns ---
    rows = [_skill_to_row(s, cols) for s in skills]

    # --- Render ---
    output_lower = (output or "markdown").lower()
    if output_lower == "json":
        return rows
    if output_lower == "yaml":
        return _render_yaml(rows)
    return _render_markdown(rows, cols)


if __name__ == "__main__":
    server.run(transport="stdio")
