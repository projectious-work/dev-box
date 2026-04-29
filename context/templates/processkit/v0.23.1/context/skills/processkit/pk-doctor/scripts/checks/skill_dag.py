"""skill_dag check.

Walks every ``context/skills/<category>/<name>/SKILL.md`` in the repo,
builds the skill-dependency directed-acyclic-graph (DAG) from each
``metadata.processkit.uses[*].skill`` entry, and validates:

1. **No missing references** — every ``uses[*].skill`` names a skill that
   exists at ``context/skills/<any-category>/<name>/SKILL.md``.
2. **No cycles** — the directed graph (skill → dependency) is acyclic.
3. **Layer constraints** — a skill with ``layer = N`` only ``uses[]``
   skills with ``layer <= N``.

Findings:
- ERROR  skill.dag.missing-ref    — uses an unknown skill name
- ERROR  skill.dag.cycle          — cycle detected (path listed)
- ERROR  skill.dag.layer-violation — layer N references layer M > N
- INFO   skill.dag.summary        — summary of the walk
"""

from __future__ import annotations

from pathlib import Path
from typing import Iterator

import yaml

from .common import CheckResult


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _parse_skill_frontmatter(path: Path) -> dict | None:
    """Return parsed YAML frontmatter dict or None on any error."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


def _iter_skill_files(skills_root: Path) -> Iterator[tuple[str, Path]]:
    """Yield (skill_name, path) for every SKILL.md under skills_root.

    skill_name is the directory name (e.g. ``event-log``), which is the
    canonical short name used in ``uses[*].skill`` references.
    """
    if not skills_root.is_dir():
        return
    for category_dir in sorted(skills_root.iterdir()):
        if not category_dir.is_dir():
            continue
        for skill_dir in sorted(category_dir.iterdir()):
            if not skill_dir.is_dir():
                continue
            skill_file = skill_dir / "SKILL.md"
            if skill_file.is_file():
                yield skill_dir.name, skill_file


def _extract_processkit_meta(data: dict) -> dict:
    """Return the metadata.processkit sub-dict or {}."""
    md = data.get("metadata")
    if not isinstance(md, dict):
        return {}
    pk = md.get("processkit")
    if not isinstance(pk, dict):
        return {}
    return pk


def _extract_uses(pk: dict) -> list[str]:
    """Return the list of skill names referenced in uses[*].skill."""
    uses_raw = pk.get("uses")
    if not isinstance(uses_raw, list):
        return []
    out: list[str] = []
    for entry in uses_raw:
        if isinstance(entry, dict):
            skill_name = entry.get("skill")
            if isinstance(skill_name, str) and skill_name.strip():
                out.append(skill_name.strip())
        elif isinstance(entry, str) and entry.strip():
            out.append(entry.strip())
    return out


def _detect_cycles(graph: dict[str, list[str]]) -> list[list[str]]:
    """Return a list of cycle paths (each path is a list of node names).

    Uses iterative DFS with three-colour marking:
    - 0 = white (unvisited)
    - 1 = grey  (in current DFS stack)
    - 2 = black (fully processed)

    Each cycle is reported once, as the full path from the back-edge node
    back to itself (e.g. [a, b, c, a]).
    """
    color = {node: 0 for node in graph}
    # Also include nodes only seen as targets (not defined as skills).
    for deps in graph.values():
        for d in deps:
            if d not in color:
                color[d] = 0

    cycles: list[list[str]] = []
    reported: set[frozenset] = set()

    for start in list(color):
        if color[start] != 0:
            continue
        stack: list[tuple[str, list[str]]] = [(start, [start])]
        while stack:
            node, path = stack[-1]
            if color[node] == 0:
                color[node] = 1
            # Find the next unvisited neighbour.
            neighbours = graph.get(node, [])
            pushed = False
            for nb in neighbours:
                if color.get(nb, 0) == 1:
                    # Back edge → cycle.
                    cycle_start = path.index(nb)
                    cycle = path[cycle_start:] + [nb]
                    key = frozenset(cycle[:-1])
                    if key not in reported:
                        reported.add(key)
                        cycles.append(cycle)
                elif color.get(nb, 0) == 0:
                    stack.append((nb, path + [nb]))
                    pushed = True
                    break
            if not pushed:
                color[node] = 2
                stack.pop()

    return cycles


# ---------------------------------------------------------------------------
# Main check entry point
# ---------------------------------------------------------------------------

def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []

    skills_root = repo_root / "context" / "skills"

    # -----------------------------------------------------------------------
    # Pass 1: collect all skills, their layers, and their declared uses.
    # -----------------------------------------------------------------------
    # name → layer (int or None)
    skill_layer: dict[str, int | None] = {}
    # name → list of dependency skill names
    skill_uses: dict[str, list[str]] = {}

    for skill_name, skill_path in _iter_skill_files(skills_root):
        data = _parse_skill_frontmatter(skill_path)
        if data is None:
            # Unparseable — skip (schema_filename will catch this)
            continue
        pk = _extract_processkit_meta(data)
        layer_raw = pk.get("layer")
        layer: int | None = int(layer_raw) if isinstance(layer_raw, (int, float)) else None
        uses = _extract_uses(pk)

        skill_layer[skill_name] = layer
        skill_uses[skill_name] = uses

    total_skills = len(skill_layer)
    # Build the graph (skill → [dep1, dep2, ...]) for cycle detection.
    # Only include known skills as sources; unknown targets are caught by
    # missing-ref check below.
    graph: dict[str, list[str]] = {name: deps for name, deps in skill_uses.items()}

    # -----------------------------------------------------------------------
    # Pass 2: missing-reference check.
    # -----------------------------------------------------------------------
    missing_ref_count = 0
    for skill_name, deps in skill_uses.items():
        for dep in deps:
            if dep not in skill_layer:
                missing_ref_count += 1
                results.append(CheckResult(
                    severity="ERROR",
                    category="skill_dag",
                    id="skill.dag.missing-ref",
                    message=(
                        f"skill '{skill_name}' references unknown skill '{dep}'"
                    ),
                    entity_ref=skill_name,
                ))

    # -----------------------------------------------------------------------
    # Pass 3: cycle detection.
    # -----------------------------------------------------------------------
    cycles = _detect_cycles(graph)
    for cycle in cycles:
        path_str = " -> ".join(cycle)
        results.append(CheckResult(
            severity="ERROR",
            category="skill_dag",
            id="skill.dag.cycle",
            message=f"cycle detected: {path_str}",
        ))

    # -----------------------------------------------------------------------
    # Pass 4: layer-constraint check.
    # -----------------------------------------------------------------------
    layer_violation_count = 0
    for skill_name, deps in skill_uses.items():
        src_layer = skill_layer.get(skill_name)
        if src_layer is None:
            continue  # no layer declared — skip constraint
        for dep in deps:
            dep_layer = skill_layer.get(dep)
            if dep_layer is None:
                continue  # unknown dep — already caught by missing-ref
            if dep_layer > src_layer:
                layer_violation_count += 1
                results.append(CheckResult(
                    severity="ERROR",
                    category="skill_dag",
                    id="skill.dag.layer-violation",
                    message=(
                        f"skill '{skill_name}' (layer {src_layer}) references "
                        f"'{dep}' (layer {dep_layer}) — must be <= {src_layer}"
                    ),
                    entity_ref=skill_name,
                ))

    # -----------------------------------------------------------------------
    # Summary INFO line.
    # -----------------------------------------------------------------------
    results.append(CheckResult(
        severity="INFO",
        category="skill_dag",
        id="skill.dag.summary",
        message=(
            f"walked {total_skills} skill(s); "
            f"{len(cycles)} cycle(s), "
            f"{layer_violation_count} layer violation(s), "
            f"{missing_ref_count} missing ref(s)"
        ),
    ))

    return results
