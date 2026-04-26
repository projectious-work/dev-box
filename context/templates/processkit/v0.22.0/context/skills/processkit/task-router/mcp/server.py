#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""processkit task-router MCP server.

Tools:

    route_task(task_description)
        -> {skill, skill_description_excerpt, process_override?,
            server, tool, tool_qualified, domain_group, confidence,
            routing_basis, candidate_tools[], also_consider_group?}

Two-phase heuristic routing (no LLM call required):
  Phase 1 — keyword match against domain groups (cheap classifier).
  Phase 2 — token overlap scoring within the matched group's tool list.
  Fallback — skill-finder trigger-phrase scan for cross-domain tasks.

Architecture note: each phase eliminates ~90 % of the candidate space
before passing a constrained set to the next stage (cascaded coarse-to-
fine pattern from MasRouter / Elastic Path). Routing never calls an LLM;
the caller can escalate to LLM confirmation when confidence < 0.5.
"""
from __future__ import annotations

import math
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

server = FastMCP("processkit-task-router")

# ---------------------------------------------------------------------------
# Domain group registry
# ---------------------------------------------------------------------------
# Maps domain name → {server, skill, keywords, tools[(name, description)]}.
# keywords: used in Phase 1 group scoring.
# tools:    used in Phase 2 tool scoring within the matched group.
#
# Naming convention for tool_qualified: "{server}__{tool_name}"
# (MetaMCP de-facto standard for collision-free aggregation).
# ---------------------------------------------------------------------------

DOMAIN_GROUPS: dict[str, dict] = {
    "workitem": {
        "server": "processkit-workitem-management",
        "skill": "workitem-management",
        "keywords": [
            "work item", "workitem", "ticket", "backlog", "task", "story",
            "bug", "issue", "todo", "sprint", "feature request",
            "create task", "track work", "open ticket",
        ],
        "tools": [
            ("create_workitem",
             "Create a new work item, ticket, or task in the backlog"),
            ("transition_workitem",
             "Move a work item to a new state: start, complete, close,"
             " reopen"),
            ("query_workitems",
             "Search or list work items by state, assignee, or label"),
            ("get_workitem",
             "Retrieve a specific work item by ID"),
            ("link_workitems",
             "Link two work items as blocking, related, or duplicate"),
        ],
    },
    "decision": {
        "server": "processkit-decision-record",
        "skill": "decision-record",
        "keywords": [
            "decision", "decide", "ADR", "architectural decision",
            "trade-off", "alternative", "rationale", "option",
            "record decision", "document decision", "chose",
        ],
        "tools": [
            ("record_decision",
             "Create a new decision record (ADR)"),
            ("get_decision",
             "Retrieve a specific decision by ID"),
            ("query_decisions",
             "Search decision records by state or keyword"),
            ("transition_decision",
             "Accept, supersede, or deprecate a decision"),
            ("supersede_decision",
             "Mark a decision as replaced by a newer one"),
            ("link_decision_to_workitem",
             "Link a decision to a related work item"),
        ],
    },
    "discussion": {
        "server": "processkit-discussion-management",
        "skill": "devils-advocate",
        "keywords": [
            "discuss", "deliberate", "debate", "open question",
            "brainstorm", "second opinion", "perspectives",
            "explore options", "conversation", "structured discussion",
            "open discussion",
        ],
        "tools": [
            ("open_discussion",
             "Open a new structured discussion with a driving question"),
            ("get_discussion",
             "Retrieve a specific discussion by ID"),
            ("list_discussions",
             "List discussions filtered by state or participant"),
            ("transition_discussion",
             "Resolve or close a discussion"),
            ("add_outcome",
             "Record the outcome or conclusion of a discussion"),
        ],
    },
    "event": {
        "server": "processkit-event-log",
        "skill": "event-log",
        "keywords": [
            "log", "audit", "event", "record event", "track activity",
            "append log", "audit trail", "history", "what happened",
            "log event", "write log",
        ],
        "tools": [
            ("log_event",
             "Append a new auditable log entry to the event log"),
            ("query_events",
             "Search log entries by event type, subject, or date range"),
            ("recent_events",
             "Return the most recent N log entries"),
        ],
    },
    "actor": {
        "server": "processkit-actor-profile",
        "skill": "onboarding-guide",
        "keywords": [
            "actor", "person", "team member", "participant",
            "agent profile", "user profile", "who is", "new member",
            "onboard person",
        ],
        "tools": [
            ("create_actor",
             "Create a new actor profile for a team member or agent"),
            ("get_actor",
             "Retrieve an actor profile by ID"),
            ("list_actors",
             "List all active actors in the project"),
            ("update_actor",
             "Update an actor's profile details"),
            ("deactivate_actor",
             "Deactivate an actor who has left the project"),
        ],
    },
    "scope": {
        "server": "processkit-scope-management",
        "skill": "estimation-planning",
        "keywords": [
            "scope", "milestone", "phase", "sprint scope",
            "project scope", "iteration", "roadmap item",
            "define scope", "scope boundary",
        ],
        "tools": [
            ("create_scope",
             "Create a new project scope, milestone, or sprint"),
            ("get_scope",
             "Retrieve a specific scope by ID"),
            ("list_scopes",
             "List all scopes, optionally filtered by state"),
            ("transition_scope",
             "Move a scope to active, completed, or cancelled"),
        ],
    },
    "index": {
        "server": "processkit-index-management",
        "skill": "status-briefing",
        "keywords": [
            "search", "find entity", "query entity", "index", "catalog",
            "discover", "full text search", "what exists", "reindex",
            "stats", "entity count",
        ],
        "tools": [
            ("get_entity",
             "Retrieve any processkit entity by its ID"),
            ("query_entities",
             "Filter entities by kind, state, label, or assignee"),
            ("search_entities",
             "Full-text search across all indexed entities"),
            ("reindex",
             "Rebuild the entity index after hand-edits to entity files"),
            ("stats",
             "Return entity counts and index health statistics"),
        ],
    },
    "skill": {
        "server": "processkit-skill-finder",
        "skill": "skill-finder",
        "keywords": [
            "skill", "which skill", "find skill", "route task",
            "what to do", "workflow", "capability", "skill catalog",
            "session start", "what skill applies",
        ],
        "tools": [
            ("find_skill",
             "Find the processkit skill matching a task description"),
            ("list_skills",
             "List all skills, optionally filtered by category"),
        ],
    },
    "gate": {
        "server": "processkit-gate-management",
        "skill": "tdd-workflow",
        "keywords": [
            "gate", "quality gate", "pass fail", "criterion",
            "threshold", "smoke test", "gate evaluation",
            "green gate", "check gate",
        ],
        "tools": [
            ("create_gate",
             "Create a quality gate with named pass/fail criteria"),
            ("evaluate_gate",
             "Record the result of a gate evaluation: passed or failed"),
            ("get_gate",
             "Retrieve a gate by ID"),
            ("list_gates",
             "List all gates, optionally filtered by state"),
        ],
    },
    "model": {
        "server": "processkit-model-recommender",
        "skill": "model-recommender",
        "keywords": [
            "model", "LLM", "AI model", "recommend model", "which model",
            "which Claude", "model selection", "compare models",
            "provider", "model pricing", "which LLM",
        ],
        "tools": [
            ("get_profile",
             "Get the capability profile for a specific AI model"),
            ("list_models",
             "List all models in the registry"),
            ("compare_models",
             "Compare two or more models across capability dimensions"),
            ("query_models",
             "Filter models by provider, capability, or context length"),
        ],
    },
    "id": {
        "server": "processkit-id-management",
        "skill": "workitem-management",
        "keywords": [
            "ID", "identifier", "generate ID", "unique ID", "new ID",
            "entity ID", "format ID", "pascal ID",
        ],
        "tools": [
            ("generate_id",
             "Generate a new processkit ID with pascal word-pair and"
             " datetime prefix"),
            ("validate_id",
             "Check whether a string is a valid processkit ID"),
            ("format_info",
             "Return the format specification for a given ID kind"),
            ("list_used_ids",
             "Return all IDs already assigned in this project"),
        ],
    },
    "role": {
        "server": "processkit-role-management",
        "skill": "onboarding-guide",
        "keywords": [
            "role", "responsibility", "RACI", "owner", "permission",
            "access", "assign role", "who owns", "role definition",
        ],
        "tools": [
            ("create_role",
             "Create a new role definition with responsibilities"),
            ("get_role",
             "Retrieve a role by ID"),
            ("list_roles",
             "List all roles in the project"),
            ("link_role_to_actor",
             "Assign a role to an actor"),
            ("update_role",
             "Update a role's definition or responsibilities"),
        ],
    },
    "retro": {
        "server": "processkit-event-log",
        "skill": "retrospective",
        "keywords": [
            "retrospective", "retro", "post-release", "post-mortem",
            "blameless", "what held", "what slipped", "release review",
            "lessons learned", "cycle review", "pk-retro",
        ],
        "tools": [
            ("log_event",
             "Emit the retro.completed LogEntry after the Artifact is saved"),
            ("query_events",
             "Query release.published and doctor.report events for signals"),
        ],
    },
    "binding": {
        "server": "processkit-binding-management",
        "skill": "workitem-management",
        "keywords": [
            "binding", "assign actor", "link actor to role",
            "role binding", "who is assigned", "who is responsible",
            "assignment", "active binding",
        ],
        "tools": [
            ("create_binding",
             "Bind an actor to a role or scope for a given period"),
            ("end_binding",
             "End an active binding when an actor leaves a role"),
            ("query_bindings",
             "List active bindings filtered by actor or role"),
            ("resolve_bindings_for",
             "Find all roles and scopes bound to a given actor"),
        ],
    },
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _project_root() -> Path:
    return paths.find_project_root()


def _score_tokens(task: str, phrases: list[str]) -> float:
    """Token-overlap score. Returns 1.0 on exact substring match."""
    task_lower = task.lower()
    task_tokens = set(re.split(r"\W+", task_lower)) - {""}
    best = 0.0
    for phrase in phrases:
        phrase_lower = phrase.lower()
        if phrase_lower in task_lower:
            return 1.0
        phrase_tokens = set(re.split(r"\W+", phrase_lower)) - {""}
        if not phrase_tokens:
            continue
        overlap = len(task_tokens & phrase_tokens) / len(phrase_tokens)
        if overlap > best:
            best = overlap
    return best


def _phase1_groups(task: str) -> list[tuple[float, str]]:
    """Score all domain groups. Returns [(score, group_name)] sorted desc."""
    scored = [
        (s, name)
        for name, g in DOMAIN_GROUPS.items()
        if (s := _score_tokens(task, g["keywords"])) > 0
    ]
    scored.sort(key=lambda x: -x[0])
    return scored


def _phase2_tools(
    task: str, group_name: str
) -> list[tuple[float, str, str]]:
    """Score tools within a group. Returns [(score, tool_name, rationale)]."""
    group = DOMAIN_GROUPS[group_name]
    server_name = group["server"]
    scored = []
    for tool_name, tool_desc in group["tools"]:
        phrases = [tool_name.replace("_", " "), tool_desc]
        s = _score_tokens(task, phrases)
        if s > 0:
            name_score = _score_tokens(task, [tool_name.replace("_", " ")])
            rationale = (
                f"task vocabulary matches tool name '{tool_name}'"
                if name_score >= 0.5
                else "task vocabulary matches tool description"
            )
            scored.append((s, tool_name, rationale))
    scored.sort(key=lambda x: -x[0])
    return scored


def _read_skill_description(skill_name: str) -> str:
    """Return first 150 chars of a skill's description from SKILL.md."""
    skills_root = _project_root() / "context" / "skills"
    for cat_dir in skills_root.iterdir():
        if not cat_dir.is_dir() or cat_dir.name.startswith("_"):
            continue
        skill_md = cat_dir / skill_name / "SKILL.md"
        if skill_md.exists():
            try:
                import yaml
                text = skill_md.read_text(encoding="utf-8")
                if text.startswith("---"):
                    end = text.index("---", 3)
                    fm = yaml.safe_load(text[3:end])
                    if isinstance(fm, dict):
                        desc = (fm.get("description") or "").strip()
                        # Flatten block scalars and truncate
                        desc = " ".join(desc.split())
                        return desc[:150]
            except Exception:
                pass
    return ""


def _find_process_override(skill_name: str) -> str | None:
    """Return relative path to context/processes/<name>.md if it exists."""
    processes_dir = _project_root() / "context" / "processes"
    if not processes_dir.exists():
        return None
    # Exact match first
    candidate = processes_dir / f"{skill_name}.md"
    if candidate.exists():
        return str(candidate.relative_to(_project_root()))
    # Prefix match: "release-semver" → "release.md"
    base = skill_name.split("-")[0]
    for p in sorted(processes_dir.glob(f"{base}*.md")):
        if p.name != "INDEX.md":
            return str(p.relative_to(_project_root()))
    return None


def _skill_finder_fallback(task: str) -> dict | None:
    """
    Fallback: parse skill-finder trigger table when group score < 0.3.
    Returns a partial result dict or None.
    """
    finder_md = (
        _project_root()
        / "context" / "skills" / "processkit" / "skill-finder" / "SKILL.md"
    )
    if not finder_md.exists():
        return None
    try:
        text = finder_md.read_text(encoding="utf-8")
    except Exception:
        return None

    rows: list[tuple[list[str], str]] = []
    in_table = False
    for line in text.splitlines():
        s = line.strip()
        if "If the user says" in s and "Skill to load" in s:
            in_table = True
            continue
        if not in_table:
            continue
        if s.startswith("|---") or s == "":
            continue
        if not s.startswith("|"):
            in_table = False
            continue
        parts = [p.strip() for p in s.strip("|").split("|")]
        if len(parts) < 2:
            continue
        skill = parts[1].strip("`").strip()
        if not skill or skill == "Skill to load":
            continue
        phrases = [
            p.strip().strip('"')
            for p in parts[0].split('", "')
            if p.strip().strip('"')
        ] or [parts[0].strip('"')]
        rows.append((phrases, skill))

    scored = [
        (sc, sk)
        for phrases, sk in rows
        if (sc := _score_tokens(task, phrases)) > 0
    ]
    if not scored:
        return None
    scored.sort(key=lambda x: -x[0])
    best_score, best_skill = scored[0]
    return {
        "skill": best_skill,
        "skill_description_excerpt": _read_skill_description(best_skill),
        "confidence": round(best_score, 2),
        "routing_basis": "skill_finder_trigger_table",
    }


# ---------------------------------------------------------------------------
# Tool
# ---------------------------------------------------------------------------


@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def route_task(task_description: str) -> dict:
    """Route a task to the matching processkit skill, process override,
    and MCP tool — in a single call, without an LLM.

    Prerequisite: call this tool at the start of any processkit domain
    task — before calling create_*, transition_*, link_*, or record_*
    tools — to confirm the right skill and any project-specific process
    override for this derived project.

    Two-phase heuristic routing:
      Phase 1 — keyword match to a domain group (eliminates ~90 %
                 of candidates; no LLM needed).
      Phase 2 — token-overlap scoring within the matched group's tools.
      Fallback — skill-finder trigger-phrase table for cross-domain
                 tasks not covered by any domain group.

    When ``confidence < 0.5`` (routing_basis == "needs_llm_confirm"),
    surface ``candidate_tools`` to the user or an LLM for confirmation
    before proceeding. The router never blocks — it always returns its
    best guess. 1% rule: if there is a 1% chance a processkit skill covers this task, call route_task before acting.

    Parameters
    ----------
    task_description :
        What the agent or user wants to do, in natural language.
        More specific phrasing → better match quality.

    Returns
    -------
    On match:
      skill                    — processkit skill name to load
      skill_description_excerpt — first 150 chars of skill description
      process_override         — path to project-specific process file
                                 (only present when one exists)
      server                   — MCP server to connect to
      tool                     — recommended tool name
      tool_qualified           — "{server}__{tool}" collision-safe form
      domain_group             — routing group (workitem, decision, …)
      confidence               — 0.0–1.0 combined routing confidence
      routing_basis            — keyword_match | skill_finder_trigger_table
                                 | needs_llm_confirm
      candidate_tools[]        — top-3 scored tools with rationales

    On no match:
      error, hint
    """
    task = task_description.strip()
    if not task:
        return {"error": "task_description must not be empty"}

    # ── Phase 1: domain group scoring ─────────────────────────────────────
    group_scores = _phase1_groups(task)

    if not group_scores or group_scores[0][0] < 0.3:
        # Fallback: skill-finder trigger table
        fb = _skill_finder_fallback(task)
        if fb:
            po = _find_process_override(fb["skill"])
            result = dict(fb)
            if po:
                result["process_override"] = po
            result["hint"] = (
                "No domain group matched; routed via skill-finder trigger "
                "table. Load the skill SKILL.md before proceeding."
            )
            return result
        return {
            "error": "no matching skill or tool found",
            "hint": (
                "Rephrase in natural language, e.g. 'create a work item "
                "for the login bug' or 'record a decision about the DB'."
            ),
        }

    best_group_score, best_group = group_scores[0]
    group = DOMAIN_GROUPS[best_group]
    server_name: str = group["server"]
    skill_name: str = group["skill"]

    # ── Phase 2: tool scoring within group ────────────────────────────────
    tool_scores = _phase2_tools(task, best_group)

    if tool_scores:
        best_tool_score, best_tool, best_rationale = tool_scores[0]
    else:
        best_tool, _ = group["tools"][0]
        best_tool_score = best_group_score * 0.5
        best_rationale = f"defaulted to first tool in '{best_group}' group"

    # Confidence = geometric mean of group and tool scores
    confidence = round(math.sqrt(best_group_score * best_tool_score), 2)
    routing_basis = (
        "keyword_match" if confidence >= 0.5 else "needs_llm_confirm"
    )

    # ── Skill description excerpt ──────────────────────────────────────────
    skill_excerpt = _read_skill_description(skill_name)

    # ── Process override ───────────────────────────────────────────────────
    process_override = _find_process_override(skill_name)

    # ── Candidate tools (top-3 for transparency + fallback) ───────────────
    candidates: list[dict] = []
    seen: set[str] = set()
    for score, tool_name, rationale in tool_scores[:3]:
        candidates.append({
            "tool_qualified": f"{server_name}__{tool_name}",
            "score": round(score, 2),
            "rationale": rationale,
        })
        seen.add(tool_name)

    # Pad to 3 with unscored tools from the group
    if len(candidates) < 3:
        for tool_name, _ in group["tools"]:
            if tool_name in seen:
                continue
            candidates.append({
                "tool_qualified": f"{server_name}__{tool_name}",
                "score": 0.0,
                "rationale": "in domain group; not vocabulary-matched",
            })
            if len(candidates) >= 3:
                break

    result: dict = {
        "skill": skill_name,
        "skill_description_excerpt": skill_excerpt,
        "server": server_name,
        "tool": best_tool,
        "tool_qualified": f"{server_name}__{best_tool}",
        "domain_group": best_group,
        "confidence": confidence,
        "routing_basis": routing_basis,
        "candidate_tools": candidates,
    }

    if process_override:
        result["process_override"] = process_override

    # Surface close runner-up group when scores are within 20 %
    if len(group_scores) > 1 and group_scores[1][0] >= best_group_score * 0.8:
        result["also_consider_group"] = group_scores[1][1]

    return result


if __name__ == "__main__":
    server.run(transport="stdio")
