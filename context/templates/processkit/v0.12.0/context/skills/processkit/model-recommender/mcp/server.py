#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "httpx>=0.27",
# ]
# ///
"""
model-recommender MCP server

Provides structured queries over the model roster and user config.
Data lives in model_scores.json and user_config.json alongside this file.
"""

import json
import sys
from pathlib import Path
from typing import Optional

from mcp.server.fastmcp import FastMCP
from mcp.types import ToolAnnotations

HERE = Path(__file__).parent
SCORES_FILE = HERE / "model_scores.json"
CONFIG_FILE = HERE / "user_config.json"

server = FastMCP("processkit-model-recommender")

# ── helpers ────────────────────────────────────────────────────────────────

def _load_scores() -> dict:
    with open(SCORES_FILE) as f:
        return json.load(f)

def _load_config() -> dict:
    if CONFIG_FILE.exists():
        with open(CONFIG_FILE) as f:
            return json.load(f)
    return {"available_models": [], "blocked_models": [], "require_governance_min": 0, "budget_tier": "any", "preferred_providers": []}

def _save_config(cfg: dict):
    with open(CONFIG_FILE, "w") as f:
        json.dump(cfg, f, indent=2)

def _bar(score: int) -> str:
    filled = score * 2
    empty = 10 - filled
    return "▓" * filled + "░" * empty

def _label(score: int) -> str:
    return {5: "Exceptional", 4: "Strong", 3: "Moderate", 2: "Limited", 1: "Minimal"}.get(score, "—")

def _apply_user_filter(models: list[dict], cfg: dict) -> list[dict]:
    available = cfg.get("available_models", [])
    blocked = cfg.get("blocked_models", [])
    preferred = cfg.get("preferred_providers", [])
    gov_min = cfg.get("require_governance_min", 0)
    budget = cfg.get("budget_tier", "any")

    result = []
    for m in models:
        if blocked and m["id"] in blocked:
            continue
        if available and m["id"] not in available:
            continue
        if gov_min and m["dimensions"]["G"]["score"] < gov_min:
            continue
        if budget == "low" and m["dimensions"]["S"]["sub"]["cost_efficiency"] < 3:
            continue
        result.append(m)

    # Prefer specified providers (stable sort — keep original order within groups)
    if preferred:
        result.sort(key=lambda m: 0 if m["provider"] in preferred else 1)
    return result

def _profile_block(m: dict, scope: str = "summary") -> dict:
    dims = m["dimensions"]
    top = {d: {"score": dims[d]["score"], "label": _label(dims[d]["score"]), "bar": _bar(dims[d]["score"])} for d in dims}
    out = {
        "id": m["id"],
        "name": m["name"],
        "provider": m["provider"],
        "tier": m["tier"],
        "context_k": m["context_k"],
        "governance_warning": m.get("governance_warning"),
        "dimensions": top,
    }
    if scope == "full":
        out["sub_dimensions"] = {d: dims[d]["sub"] for d in dims}
        out["best_for"] = m.get("best_for", [])
        out["avoid_for"] = m.get("avoid_for", [])
    elif scope in ("R", "E", "S", "B", "L", "G"):
        out["sub_dimensions"] = {scope: dims[scope]["sub"]}
    return out

# ── tools ──────────────────────────────────────────────────────────────────

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def query_models(
    R: int = 0,
    E: int = 0,
    S: int = 0,
    B: int = 0,
    L: int = 0,
    G: int = 0,
    sub_R: Optional[dict] = None,
    sub_E: Optional[dict] = None,
    sub_B: Optional[dict] = None,
    min_context_k: int = 0,
    max_price_per_1m_output: Optional[float] = None,
    apply_user_filter: bool = True,
    include_estimated: bool = True,
    limit: int = 5,
) -> list[dict]:
    """
    Find models that meet minimum dimension scores, ranked by overall fit.

    min_context_k: minimum context window in thousands of tokens (e.g. 200 for 200K).
      Use this when the task requires a specific context window size.
    max_price_per_1m_output: maximum acceptable output price in USD per 1M tokens.
      Self-hosted models (null price) are always included regardless of this filter.
    include_estimated: if True (default), includes models with estimated/unvalidated
      scores (marked _estimated in model_scores.json). Set False for production routing.

    Find models that meet minimum dimension scores, ranked by overall fit.

    Top-level requirements: pass minimum scores per dimension (1–5, 0 = no requirement).
    Sub-dimension requirements: pass dicts like sub_R={"math_reasoning": 4} to filter on
    specific sub-dimensions within Reasoning. Supported: sub_R, sub_E, sub_B.
    apply_user_filter: if True (default), respects user_config.json access/preference settings.
    limit: max results to return (default 5).

    Returns ranked list of matching model profiles (summary level).
    """
    data = _load_scores()
    cfg = _load_config() if apply_user_filter else {}
    mins = {"R": R, "E": E, "S": S, "B": B, "L": L, "G": G}
    sub_mins = {"R": sub_R or {}, "E": sub_E or {}, "B": sub_B or {}}

    candidates = []
    for m in data["models"]:
        dims = m["dimensions"]
        # Filter estimated models if requested
        if not include_estimated and m.get("_estimated"):
            continue
        # Check context window
        if min_context_k > 0 and m.get("context_k", 0) < min_context_k:
            continue
        # Check price ceiling (self-hosted models with null price always pass)
        if max_price_per_1m_output is not None:
            price = m.get("pricing", {}).get("output_per_1m")
            if price is not None and price > max_price_per_1m_output:
                continue
        # Check top-level minimums
        if any(dims[d]["score"] < mins[d] for d in mins if mins[d] > 0):
            continue
        # Check sub-dimension minimums
        skip = False
        for dim, sub_req in sub_mins.items():
            for sub_key, min_val in sub_req.items():
                if dims[dim]["sub"].get(sub_key, 0) < min_val:
                    skip = True
                    break
            if skip:
                break
        if skip:
            continue
        # Compute fit score: sum of (actual - required) for required dims
        fit = sum(dims[d]["score"] - mins[d] for d in mins if mins[d] > 0)
        candidates.append((fit, m))

    candidates.sort(key=lambda x: -x[0])
    models = [m for _, m in candidates[:limit]]
    if apply_user_filter and cfg:
        models = _apply_user_filter(models, cfg)

    return [_profile_block(m, "summary") for m in models]


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_profile(model_id: str, scope: str = "full") -> dict:
    """
    Return the profile for a specific model.

    model_id: the model's id field (e.g. "claude-sonnet-4.6")
    scope: "summary" (top-level only), "full" (all sub-dimensions + best/avoid lists),
           or a single dimension letter "R", "E", "S", "B", "L", "G" (drill into that
           dimension's sub-scores only).

    Returns the profile dict or an error if the model is not found.
    """
    data = _load_scores()
    for m in data["models"]:
        if m["id"] == model_id:
            return _profile_block(m, scope)
    return {"error": f"Model '{model_id}' not found. Call list_models() to see all IDs."}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_models(apply_user_filter: bool = True) -> list[dict]:
    """
    List all models in the roster with their top-level dimension scores.

    apply_user_filter: if True (default), only returns models accessible per user_config.json.
    Set to False to see the full roster regardless of user config.

    Returns a compact list: id, name, provider, tier, context_k, dimension scores.
    """
    data = _load_scores()
    models = data["models"]
    cfg = _load_config() if apply_user_filter else {}
    if apply_user_filter and cfg.get("available_models") or cfg.get("blocked_models") or cfg.get("require_governance_min"):
        models = _apply_user_filter(models, cfg)

    return [
        {
            "id": m["id"],
            "name": m["name"],
            "provider": m["provider"],
            "tier": m["tier"],
            "context_k": m["context_k"],
            "scores": {d: m["dimensions"][d]["score"] for d in m["dimensions"]},
            "governance_warning": m.get("governance_warning"),
        }
        for m in models
    ]


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def compare_models(model_ids: list[str], scope: str = "top_level") -> dict:
    """
    Compare two or more models side by side.

    model_ids: list of model id strings (e.g. ["claude-sonnet-4.6", "gemini-2.5-pro"])
    scope: "top_level" (6 dimension scores), or a single dimension letter
           "R", "E", "S", "B", "L", "G" (compare all sub-dimensions within that dim).

    Returns a comparison dict with rows = dimensions/sub-dimensions, columns = models.
    """
    data = _load_scores()
    by_id = {m["id"]: m for m in data["models"]}

    missing = [mid for mid in model_ids if mid not in by_id]
    if missing:
        return {"error": f"Models not found: {missing}. Call list_models() to see all IDs."}

    models = [by_id[mid] for mid in model_ids]
    meta = _load_scores()["_meta"]

    if scope == "top_level":
        rows = {}
        for d in ("R", "E", "S", "B", "L", "G"):
            rows[meta["dimension_names"][d]] = {
                m["name"]: {"score": m["dimensions"][d]["score"], "label": _label(m["dimensions"][d]["score"])}
                for m in models
            }
        return {
            "scope": "top_level",
            "models": [m["name"] for m in models],
            "rows": rows,
            "winner_per_dimension": {
                meta["dimension_names"][d]: max(models, key=lambda m: m["dimensions"][d]["score"])["name"]
                for d in ("R", "E", "S", "B", "L", "G")
            },
        }
    elif scope in ("R", "E", "S", "B", "L", "G"):
        sub_keys = meta["sub_dimensions"][scope]
        rows = {}
        for sk in sub_keys:
            rows[sk] = {
                m["name"]: m["dimensions"][scope]["sub"].get(sk, 0)
                for m in models
            }
        return {
            "scope": f"sub-dimension:{scope}",
            "dimension": meta["dimension_names"][scope],
            "models": [m["name"] for m in models],
            "rows": rows,
            "top_level_scores": {m["name"]: m["dimensions"][scope]["score"] for m in models},
        }
    else:
        return {"error": f"Invalid scope '{scope}'. Use 'top_level' or a dimension letter: R E S B L G"}


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_pricing(model_ids: Optional[list[str]] = None, sort_by: str = "output_per_1m") -> list[dict]:
    """
    Return pricing data for models, with a value score (capability per dollar).

    model_ids: optional list of model IDs to include. If omitted, returns all
               models accessible per user_config.
    sort_by: "output_per_1m" (default, cheapest output first), "input_per_1m",
             "value_score" (capability per dollar, highest first), "name".

    Value score = (R + E + L) / 3 divided by output cost per 1M tokens.
    Higher value score = more capability per dollar. Self-hosted models (null price)
    are listed separately since their cost depends on your infrastructure.

    Returns a list sorted by the requested field.
    """
    data = _load_scores()
    cfg = _load_config()

    if model_ids:
        models = [m for m in data["models"] if m["id"] in model_ids]
    else:
        models = _apply_user_filter(data["models"], cfg) if cfg else data["models"]

    results = []
    for m in models:
        p = m.get("pricing", {})
        inp = p.get("input_per_1m")
        out = p.get("output_per_1m")
        dims = m["dimensions"]
        # Capability score: average of capability-focused dimensions
        cap = (dims["R"]["score"] + dims["E"]["score"] + dims["L"]["score"]) / 3
        value_score = None
        if out and out > 0:
            value_score = round(cap / out * 10, 2)  # scale for readability

        results.append({
            "id": m["id"],
            "name": m["name"],
            "provider": m["provider"],
            "input_per_1m_usd": inp,
            "output_per_1m_usd": out,
            "hosting": p.get("hosting", "api"),
            "capability_score": round(cap, 1),
            "value_score": value_score,
            "value_score_note": "capability / output_cost * 10; higher = more capability per dollar; null for self-hosted",
            "pricing_note": p.get("note", ""),
            "governance_warning": m.get("governance_warning"),
        })

    # Sort
    def sort_key(x):
        if sort_by == "value_score":
            v = x["value_score"]
            return -(v if v is not None else -999)
        elif sort_by == "input_per_1m":
            v = x["input_per_1m_usd"]
            return v if v is not None else 999
        elif sort_by == "name":
            return x["name"]
        else:  # output_per_1m
            v = x["output_per_1m_usd"]
            return v if v is not None else 999

    results.sort(key=sort_key)
    return results


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=True,
))
def check_availability(model_ids: Optional[list[str]] = None) -> list[dict]:
    """
    Check provider status pages for current API availability.

    model_ids: optional list of model IDs to check. If omitted, checks all providers
               in the roster (filtered by user_config if set).

    Returns a list of {model_id, provider, status, detail} dicts.
    Status values: "operational", "degraded", "major_outage", "unknown", "no_status_page".

    Note: status is fetched from public status pages on a best-effort basis.
    Rate limits, quota exhaustion, and subscription expiry cannot be detected
    here — the user must verify those manually.
    """
    import httpx

    data = _load_scores()
    meta = data["_meta"]
    by_id = {m["id"]: m for m in data["models"]}

    if model_ids:
        targets = {mid: by_id[mid] for mid in model_ids if mid in by_id}
    else:
        cfg = _load_config()
        models = _apply_user_filter(data["models"], cfg) if cfg else data["models"]
        targets = {m["id"]: m for m in models}

    # Group by provider to avoid duplicate fetches
    provider_status: dict[str, dict] = {}
    for provider, url in meta["status_pages"].items():
        if url is None:
            provider_status[provider] = {"status": "no_status_page", "detail": "No public status page; check provider docs manually."}
            continue
        try:
            r = httpx.get(url + "/api/v2/status.json", timeout=5.0)
            if r.status_code == 200:
                body = r.json()
                indicator = body.get("status", {}).get("indicator", "unknown")
                description = body.get("status", {}).get("description", "")
                status_map = {"none": "operational", "minor": "degraded", "major": "major_outage", "critical": "major_outage"}
                provider_status[provider] = {"status": status_map.get(indicator, "unknown"), "detail": description}
            else:
                provider_status[provider] = {"status": "unknown", "detail": f"HTTP {r.status_code} from status page"}
        except Exception as e:
            provider_status[provider] = {"status": "unknown", "detail": f"Could not reach status page: {e}"}

    results = []
    for mid, m in targets.items():
        ps = provider_status.get(m["provider"], {"status": "unknown", "detail": "Provider not in status map"})
        results.append({
            "model_id": mid,
            "model_name": m["name"],
            "provider": m["provider"],
            "status": ps["status"],
            "detail": ps["detail"],
            "note": "Rate limits, quota, and subscription status must be checked manually via your API dashboard.",
        })

    return results


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_config() -> dict:
    """
    Return the current user configuration (accessible models, blocked models,
    governance floor, budget tier, preferred providers).
    """
    cfg = _load_config()
    data = _load_scores()
    by_id = {m["id"]: m["name"] for m in data["models"]}

    return {
        "available_models": [{"id": mid, "name": by_id.get(mid, mid)} for mid in cfg.get("available_models", [])],
        "blocked_models": [{"id": mid, "name": by_id.get(mid, mid)} for mid in cfg.get("blocked_models", [])],
        "require_governance_min": cfg.get("require_governance_min", 0),
        "budget_tier": cfg.get("budget_tier", "any"),
        "preferred_providers": cfg.get("preferred_providers", []),
        "note": "Empty available_models means all models are included (unfiltered). Add model IDs to restrict.",
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=True,
    idempotentHint=False,
    openWorldHint=False,
))
def set_config(
    available_models: Optional[list[str]] = None,
    blocked_models: Optional[list[str]] = None,
    require_governance_min: Optional[int] = None,
    budget_tier: Optional[str] = None,
    preferred_providers: Optional[list[str]] = None,
) -> dict:
    """
    Update user configuration. Only fields provided are changed; omitted fields keep current values.

    available_models: list of model IDs the user has API access to. Empty list = all models.
      Example: ["claude-sonnet-4.6", "claude-haiku-4.5", "gpt-4o"]
    blocked_models: model IDs to never recommend regardless of scores.
      Example: ["deepseek-v3", "deepseek-r1"]
    require_governance_min: minimum G score for all recommendations (0–5).
      Set to 5 to enforce full data sovereignty across all tasks.
    budget_tier: "any" | "low" (prefer cost_efficiency >= 3) | "medium" | "high"
    preferred_providers: list of providers to prioritize in rankings.
      Example: ["Anthropic", "Mistral"]

    Returns the updated configuration.
    """
    data = _load_scores()
    valid_ids = {m["id"] for m in data["models"]}
    valid_providers = {m["provider"] for m in data["models"]}

    cfg = _load_config()

    if available_models is not None:
        bad = [mid for mid in available_models if mid not in valid_ids]
        if bad:
            return {"error": f"Unknown model IDs: {bad}. Call list_models(apply_user_filter=False) to see all valid IDs."}
        cfg["available_models"] = available_models

    if blocked_models is not None:
        bad = [mid for mid in blocked_models if mid not in valid_ids]
        if bad:
            return {"error": f"Unknown model IDs: {bad}."}
        cfg["blocked_models"] = blocked_models

    if require_governance_min is not None:
        if not (0 <= require_governance_min <= 5):
            return {"error": "require_governance_min must be 0–5."}
        cfg["require_governance_min"] = require_governance_min

    if budget_tier is not None:
        if budget_tier not in ("any", "low", "medium", "high"):
            return {"error": "budget_tier must be 'any', 'low', 'medium', or 'high'."}
        cfg["budget_tier"] = budget_tier

    if preferred_providers is not None:
        bad = [p for p in preferred_providers if p not in valid_providers]
        if bad:
            return {"error": f"Unknown providers: {bad}. Valid: {sorted(valid_providers)}"}
        cfg["preferred_providers"] = preferred_providers

    _save_config(cfg)
    return {"ok": True, "config": cfg}


# ── main ───────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    server.run(transport="stdio")
