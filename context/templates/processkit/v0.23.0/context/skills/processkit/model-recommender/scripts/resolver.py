#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
# ]
# ///
"""resolver.py — 8-layer model-assignment binding resolver.

Implements the precedence ladder from DEC-20260422_0234-LoyalComet:

    1. Task-pinned override           (task_hints.model_pin)
    2. Team-member preference         (subject=TEAMMEMBER-*, type=model-assignment)
    3. Project veto                   (subject=SCOPE-*, conditions.blocked=true)
    4. Capability filter              (task_hints.requires_*)
    5. Role + seniority bindings      (subject=ROLE-*, conditions.seniority match)
    6. Role default bindings          (subject=ROLE-*, no seniority)
    7. Project bias                   (subject=SCOPE-*, provider_preference/cost_bias)
    8. Shim fallback                  (role.spec.default_model)

Each layer contributes ResolvedCandidate entries or reorders/filters the
running list; ties within a layer are broken by provider preference,
cost, version recency, then reliability.

The resolver is read-only: it queries bindings, models, and roles via
`processkit.index` and returns candidates without persisting anything.
A small in-module cache with a 60-second TTL keyed on the call inputs
amortises repeated lookups. Callers that mutate context should clear
the cache via ``clear_cache()`` — event-driven invalidation is a
future enhancement.
"""
from __future__ import annotations

import datetime as _dt
import hashlib
import json
import os
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Iterable


# ---------------------------------------------------------------------------
# Path bootstrap (lets scripts/ + tests run without uv/package install)
# ---------------------------------------------------------------------------


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib",
                  here / "context" / "skills" / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))


from processkit import index as _index  # noqa: E402
from processkit import paths as _paths  # noqa: E402


# ---------------------------------------------------------------------------
# Public types
# ---------------------------------------------------------------------------

EFFORT_ORDER = ["none", "low", "medium", "high", "extra-high", "max"]
_EFFORT_INDEX = {e: i for i, e in enumerate(EFFORT_ORDER)}

_SENIORITY_ORDER = ["junior", "specialist", "expert", "senior", "principal"]


class NoViableModelError(RuntimeError):
    """Raised when no candidate model can be resolved after all 8 layers."""


@dataclass
class ResolvedCandidate:
    model_id: str
    version_id: str
    effort: str
    rank: int
    source_layer: int
    rationale: str = ""

    def key(self) -> tuple[str, str]:
        """Dedupe key: (model_id, effort)."""
        return (self.model_id, self.effort)


# ---------------------------------------------------------------------------
# Cache
# ---------------------------------------------------------------------------

_CACHE_TTL = 60.0
_cache: dict[tuple, tuple[float, Any]] = {}


def clear_cache() -> None:
    _cache.clear()


def _cache_key(
    role: str,
    seniority: str | None,
    team_member: str | None,
    scope: str | None,
    task_hints: dict | None,
    explain: bool,
) -> tuple:
    h = hashlib.sha1()
    h.update(json.dumps(task_hints or {}, sort_keys=True, default=str).encode())
    return (role, seniority, team_member, scope, h.hexdigest(), bool(explain))


# ---------------------------------------------------------------------------
# Entity loading helpers
# ---------------------------------------------------------------------------


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _today_iso() -> str:
    return _dt.date.today().isoformat()


def _load_bindings_for_subject(subject: str, binding_type: str = "model-assignment") -> list[dict]:
    """Return active bindings whose subject is ``subject`` and type matches."""
    db = _index.open_db()
    try:
        rows = _index.query_entities(db, kind="Binding", limit=1000)
    finally:
        db.close()
    out: list[dict] = []
    today = _today_iso()
    for r in rows:
        full = _get_entity_spec(r["id"])
        if not full:
            continue
        if full.get("type") != binding_type:
            continue
        if full.get("subject") != subject:
            continue
        valid_until = full.get("valid_until")
        valid_from = full.get("valid_from")
        if valid_from and str(valid_from) > today:
            continue
        if valid_until and str(valid_until) < today:
            continue
        full["_entity_id"] = r["id"]
        out.append(full)
    return out


def _get_entity_spec(entity_id: str) -> dict | None:
    """Return the full spec of an entity by ID."""
    db = _index.open_db()
    try:
        row = _index.get_entity(db, entity_id)
    finally:
        db.close()
    if not row:
        return None
    return row.get("spec") or {}


def _get_role(role_id: str) -> dict | None:
    spec = _get_entity_spec(role_id)
    return spec


def _get_model(model_id: str) -> dict | None:
    spec = _get_entity_spec(model_id)
    if not spec:
        return None
    spec["_id"] = model_id
    return spec


def _model_versions_sorted(model: dict) -> list[dict]:
    """Return ``versions[]`` ordered newest-first by released_at (then input order)."""
    versions = list(model.get("versions") or [])
    # Stable sort: newer released_at wins; missing dates sink to the end.
    def _k(v):
        return v.get("released_at") or ""
    return sorted(versions, key=_k, reverse=True)


def _pick_version(model: dict, pin: str | None) -> tuple[str | None, str | None]:
    """Return (version_id, warning). Prefers version_pin → latest GA → latest any."""
    versions = _model_versions_sorted(model)
    if not versions:
        return None, f"model {model.get('_id', '')!r} has no versions"
    if pin:
        for v in versions:
            if v.get("version_id") == pin:
                return pin, None
        return pin, f"version_pin '{pin}' not found; using pin literally"
    # Prefer latest GA
    for v in versions:
        if v.get("status") == "ga":
            return v.get("version_id"), None
    # Fallback: latest release regardless of status; warn
    v = versions[0]
    return v.get("version_id"), (
        f"model {model.get('_id', '')!r} has no GA version; "
        f"using {v.get('status', 'unknown')} version {v.get('version_id', '?')}"
    )


def _clamp_effort(
    requested: str | None,
    floor: str | None,
    ceiling: str | None,
    supported: Iterable[str],
) -> tuple[str, list[str]]:
    """Clamp an effort to the model's supported range, respecting floor/ceiling.

    Returns (effort, notes). ``requested`` defaults to ``medium`` when None.
    """
    notes: list[str] = []
    sup = [e for e in supported if e in _EFFORT_INDEX]
    if not sup:
        return (requested or "none"), [
            "model declares no efforts_supported; using requested effort verbatim",
        ]
    sup_sorted = sorted(sup, key=lambda e: _EFFORT_INDEX[e])
    sup_min, sup_max = sup_sorted[0], sup_sorted[-1]

    # Start from floor if present, else requested, else 'medium'.
    eff = requested or "medium"

    if floor and floor in _EFFORT_INDEX:
        if _EFFORT_INDEX[floor] > _EFFORT_INDEX[eff]:
            eff = floor
    if ceiling and ceiling in _EFFORT_INDEX:
        if _EFFORT_INDEX[ceiling] < _EFFORT_INDEX[eff]:
            eff = ceiling

    # Clamp against model's supported range.
    if _EFFORT_INDEX[eff] < _EFFORT_INDEX[sup_min]:
        notes.append(
            f"floor-clamped to model minimum {sup_min!r} (requested {eff!r})"
        )
        eff = sup_min
    if _EFFORT_INDEX[eff] > _EFFORT_INDEX[sup_max]:
        notes.append(
            f"ceiling-clamped to model maximum {sup_max!r} (requested {eff!r})"
        )
        eff = sup_max

    # Final safety: if effort still not supported, pick nearest.
    if eff not in sup:
        # nearest supported
        nearest = min(sup, key=lambda e: abs(_EFFORT_INDEX[e] - _EFFORT_INDEX[eff]))
        notes.append(f"effort {eff!r} not in efforts_supported; snapped to {nearest!r}")
        eff = nearest

    # Honour an explicit effort_ceiling above the model's max (clamp with note).
    if ceiling and ceiling in _EFFORT_INDEX and _EFFORT_INDEX[ceiling] > _EFFORT_INDEX[sup_max]:
        notes.append(
            f"effort_ceiling {ceiling!r} exceeds model's max {sup_max!r}; clamped"
        )
    if floor and floor in _EFFORT_INDEX and _EFFORT_INDEX[floor] < _EFFORT_INDEX[sup_min]:
        notes.append(
            f"effort_floor {floor!r} below model's min {sup_min!r}; clamped"
        )
    return eff, notes


# ---------------------------------------------------------------------------
# Layer builders
# ---------------------------------------------------------------------------


def _binding_to_candidate(
    binding: dict,
    layer: int,
    default_effort: str | None = None,
) -> ResolvedCandidate | tuple[None, str]:
    """Convert a model-assignment binding to a ResolvedCandidate, or (None, reason)."""
    target = binding.get("target", "")
    if not target.startswith("MODEL-"):
        return None, f"binding target {target!r} is not a MODEL id"
    model = _get_model(target)
    if model is None:
        return None, f"binding target {target!r} not found in index"

    conds = binding.get("conditions") or {}
    pin = conds.get("version_pin")
    floor = conds.get("effort_floor")
    ceiling = conds.get("effort_ceiling")

    ver, ver_warn = _pick_version(model, pin)
    eff, effort_notes = _clamp_effort(
        requested=default_effort,
        floor=floor,
        ceiling=ceiling,
        supported=model.get("efforts_supported") or [],
    )

    rationale_parts: list[str] = []
    if conds.get("rationale"):
        rationale_parts.append(str(conds["rationale"]))
    if ver_warn:
        rationale_parts.append(ver_warn)
    if effort_notes:
        rationale_parts.extend(effort_notes)

    return ResolvedCandidate(
        model_id=target,
        version_id=ver or "",
        effort=eff,
        rank=int(conds.get("rank", 1) or 1),
        source_layer=layer,
        rationale="; ".join(rationale_parts),
    )


def _capability_ok(model: dict, hints: dict | None) -> tuple[bool, str]:
    """Return (ok, reason). Reason is empty if ok is True."""
    if not hints:
        return True, ""
    modalities = set(model.get("modalities") or [])
    if hints.get("requires_vision") and "vision" not in modalities:
        return False, "model lacks 'vision' modality"
    if hints.get("requires_tool_use") and "tools" not in modalities:
        return False, "model lacks 'tools' modality"
    if hints.get("requires_computer_use") and "computer-use" not in modalities:
        return False, "model lacks 'computer-use' modality"
    return True, ""


# ---------------------------------------------------------------------------
# Tie-breaker comparators
# ---------------------------------------------------------------------------


def _tiebreak_key(
    cand: ResolvedCandidate,
    preferred_providers: list[str],
    expected_tokens: int | None,
):
    """Lower tuple wins. Composed of (layer, rank, provider-pref, cost, -recency, -reliability)."""
    model = _get_model(cand.model_id) or {}
    provider = model.get("provider", "")
    prov_rank = preferred_providers.index(provider) if provider in preferred_providers else len(preferred_providers) + 1

    # Cost: sum of input+output per 1M tokens for the selected version.
    cost = 999999.0
    released_at = ""
    for v in model.get("versions") or []:
        if v.get("version_id") == cand.version_id:
            p = v.get("pricing_usd_per_1m") or {}
            inp = p.get("input") or 0.0
            out = p.get("output") or 0.0
            if expected_tokens:
                cost = (inp + out) * (expected_tokens / 1_000_000.0)
            else:
                cost = out or (inp or 0.0) or cost
            released_at = v.get("released_at") or ""
            break
    # Reliability score (higher = better); negate so lower wins.
    dims = model.get("dimensions") or {}
    reliability = -int(dims.get("reliability", 0) or 0)

    # Recency: more-recent released_at should sort earlier. Since Python
    # sorts lexicographically asc, invert by using the tuple key with a
    # "max-like" sentinel — we negate by prepending a ZZZ marker. Simpler:
    # compute the negation via a descending tuple element.
    # We'll return released_at in a form where 'greater date' is 'less key'.
    # Approach: use a tuple (-year, -month, -day). Fall back to 0 when unknown.
    try:
        year, month, day = (int(x) for x in released_at.split("-")[:3])
    except Exception:
        year = month = day = 0
    recency_key = (-year, -month, -day)

    # Use layer first so higher-precedence layers stay on top.
    return (cand.source_layer, cand.rank, prov_rank, cost,
            recency_key, reliability)


# ---------------------------------------------------------------------------
# Main resolver
# ---------------------------------------------------------------------------


def resolve(
    role: str,
    seniority: str | None = None,
    team_member: str | None = None,
    scope: str | None = None,
    task_hints: dict | None = None,
    explain: bool = False,
):
    """Resolve a role (+ optional seniority/team_member/scope) to a ranked list of models.

    See the module docstring for the 8-layer precedence. When ``explain`` is
    True, returns ``(candidates, trace)``; otherwise returns ``candidates``.
    """
    task_hints = task_hints or {}

    key = _cache_key(role, seniority, team_member, scope, task_hints, explain)
    cached = _cache.get(key)
    if cached and (time.time() - cached[0]) < _CACHE_TTL:
        return cached[1]

    trace: list[dict] = []
    candidates: list[ResolvedCandidate] = []
    preferred_providers: list[str] = []
    expected_tokens: int | None = task_hints.get("expected_tokens")

    # -- Layer 1: Task-pinned override ----------------------------------------
    model_pin = task_hints.get("model_pin")
    if model_pin:
        model = _get_model(model_pin)
        if model is None:
            trace.append({
                "step": 1, "layer": 1, "action": "task_pin_missing",
                "count_before": 0, "count_after": 0,
                "details": {"pin": model_pin, "reason": "model not found in index"},
            })
            raise NoViableModelError(
                f"task_hints.model_pin {model_pin!r} not found"
            )
        ver, ver_warn = _pick_version(model, task_hints.get("version_pin"))
        eff, notes = _clamp_effort(
            task_hints.get("effort"),
            None,
            None,
            model.get("efforts_supported") or [],
        )
        rationale = "task-pinned"
        if ver_warn:
            rationale += f"; {ver_warn}"
        if notes:
            rationale += "; " + "; ".join(notes)
        pinned = ResolvedCandidate(
            model_id=model_pin,
            version_id=ver or "",
            effort=eff,
            rank=1,
            source_layer=1,
            rationale=rationale,
        )
        trace.append({
            "step": 1, "layer": 1, "action": "task_pin_applied",
            "count_before": 0, "count_after": 1,
            "details": {"pin": model_pin},
        })
        result = ([pinned], trace) if explain else [pinned]
        _cache[key] = (time.time(), result)
        return result

    # -- Layer 2: Team-member preference --------------------------------------
    if team_member:
        tm_bindings = _load_bindings_for_subject(team_member)
        tm_bindings.sort(key=lambda b: int((b.get("conditions") or {}).get("rank", 99)))
        before = len(candidates)
        for b in tm_bindings:
            rc = _binding_to_candidate(b, layer=2)
            if isinstance(rc, tuple):
                continue
            candidates.append(rc)
        trace.append({
            "step": 2, "layer": 2, "action": "team_member_preference",
            "count_before": before, "count_after": len(candidates),
            "details": {
                "team_member": team_member,
                "bindings_considered": len(tm_bindings),
            },
        })

    # -- Layer 3: Project veto (blocked list) ---------------------------------
    blocked: set[str] = set()
    if scope:
        proj_bindings = _load_bindings_for_subject(scope)
        for b in proj_bindings:
            conds = b.get("conditions") or {}
            if conds.get("blocked") is True:
                blocked.add(b.get("target", ""))
        before = len(candidates)
        candidates = [c for c in candidates if c.model_id not in blocked]
        trace.append({
            "step": 3, "layer": 3, "action": "project_veto",
            "count_before": before, "count_after": len(candidates),
            "details": {"blocked_models": sorted(blocked)},
        })
    else:
        trace.append({
            "step": 3, "layer": 3, "action": "project_veto",
            "count_before": len(candidates), "count_after": len(candidates),
            "details": {"blocked_models": [], "scope": None},
        })

    # -- Layer 4: Capability filter (plus project blocked list for later adds) -
    def _cap_filter(cands: list[ResolvedCandidate]) -> tuple[list[ResolvedCandidate], list[dict]]:
        kept: list[ResolvedCandidate] = []
        dropped: list[dict] = []
        for c in cands:
            if c.model_id in blocked:
                dropped.append({"model_id": c.model_id, "reason": "project-blocked"})
                continue
            model = _get_model(c.model_id)
            if model is None:
                dropped.append({"model_id": c.model_id, "reason": "model not found"})
                continue
            ok, reason = _capability_ok(model, task_hints)
            if not ok:
                dropped.append({"model_id": c.model_id, "reason": reason})
                continue
            kept.append(c)
        return kept, dropped

    before = len(candidates)
    candidates, dropped = _cap_filter(candidates)
    trace.append({
        "step": 4, "layer": 4, "action": "capability_filter",
        "count_before": before, "count_after": len(candidates),
        "details": {"dropped": dropped, "requirements": {
            k: v for k, v in task_hints.items()
            if k.startswith("requires_")
        }},
    })

    # -- Layer 5: Role + seniority bindings -----------------------------------
    role_bindings = _load_bindings_for_subject(role)
    added_5 = 0
    if seniority:
        before = len(candidates)
        for b in role_bindings:
            conds = b.get("conditions") or {}
            if conds.get("seniority") == seniority:
                rc = _binding_to_candidate(b, layer=5)
                if isinstance(rc, tuple):
                    continue
                if rc.model_id in blocked:
                    continue
                model = _get_model(rc.model_id)
                if model is None:
                    continue
                ok, _ = _capability_ok(model, task_hints)
                if not ok:
                    continue
                candidates.append(rc)
                added_5 += 1
        trace.append({
            "step": 5, "layer": 5, "action": "role_seniority_bindings",
            "count_before": before, "count_after": len(candidates),
            "details": {"role": role, "seniority": seniority, "added": added_5},
        })
    else:
        trace.append({
            "step": 5, "layer": 5, "action": "role_seniority_bindings",
            "count_before": len(candidates), "count_after": len(candidates),
            "details": {"role": role, "seniority": None, "added": 0},
        })

    # -- Layer 6: Role default bindings (no seniority) ------------------------
    before = len(candidates)
    added_6 = 0
    for b in role_bindings:
        conds = b.get("conditions") or {}
        if conds.get("seniority"):
            continue
        rc = _binding_to_candidate(b, layer=6)
        if isinstance(rc, tuple):
            continue
        if rc.model_id in blocked:
            continue
        model = _get_model(rc.model_id)
        if model is None:
            continue
        ok, _ = _capability_ok(model, task_hints)
        if not ok:
            continue
        candidates.append(rc)
        added_6 += 1
    trace.append({
        "step": 6, "layer": 6, "action": "role_default_bindings",
        "count_before": before, "count_after": len(candidates),
        "details": {"role": role, "added": added_6},
    })

    # -- Layer 7: Project bias (reorder only) ---------------------------------
    if scope:
        proj_bindings = _load_bindings_for_subject(scope)
        for b in proj_bindings:
            conds = b.get("conditions") or {}
            pref = conds.get("provider_preference")
            if pref:
                preferred_providers = list(pref)
                break
        trace.append({
            "step": 7, "layer": 7, "action": "project_bias",
            "count_before": len(candidates), "count_after": len(candidates),
            "details": {
                "provider_preference": preferred_providers,
                "cost_bias": _collect_cost_bias(proj_bindings),
            },
        })
    else:
        trace.append({
            "step": 7, "layer": 7, "action": "project_bias",
            "count_before": len(candidates), "count_after": len(candidates),
            "details": {"provider_preference": [], "cost_bias": None},
        })

    # -- Dedupe by (model_id, effort); keep the highest-precedence entry -----
    # Lower source_layer = higher precedence (layer 2 > layer 5 > layer 6).
    seen: dict[tuple[str, str], ResolvedCandidate] = {}
    # Iterate in natural insertion order (which mirrors precedence: layer 2,
    # then 5, then 6 were appended in that order).
    for c in candidates:
        k = c.key()
        if k not in seen or c.source_layer < seen[k].source_layer:
            seen[k] = c
    before = len(candidates)
    candidates = list(seen.values())
    if before != len(candidates):
        trace.append({
            "step": 7.5, "layer": 7, "action": "dedupe",
            "count_before": before, "count_after": len(candidates),
            "details": {"kept_keys": [list(k) for k in seen.keys()]},
        })

    # -- Apply tie-breakers and sort ------------------------------------------
    candidates.sort(key=lambda c: _tiebreak_key(c, preferred_providers, expected_tokens))

    # -- Layer 8: Shim fallback ----------------------------------------------
    if not candidates:
        role_ent = _get_role(role)
        default_model = (role_ent or {}).get("default_model")
        if default_model:
            model = _get_model(default_model)
            if model is None:
                trace.append({
                    "step": 8, "layer": 8, "action": "shim_fallback_missing",
                    "count_before": 0, "count_after": 0,
                    "details": {"default_model": default_model,
                                "reason": "model not found"},
                })
                raise NoViableModelError(
                    f"role {role!r} default_model {default_model!r} not found"
                )
            ok, reason = _capability_ok(model, task_hints)
            if not ok:
                trace.append({
                    "step": 8, "layer": 8, "action": "shim_fallback_capability_fail",
                    "count_before": 0, "count_after": 0,
                    "details": {"default_model": default_model, "reason": reason},
                })
                raise NoViableModelError(
                    f"role {role!r} shim-fallback model {default_model!r} "
                    f"failed capability check: {reason}"
                )
            ver, ver_warn = _pick_version(model, None)
            eff, notes = _clamp_effort(
                task_hints.get("effort"),
                None,
                None,
                model.get("efforts_supported") or [],
            )
            rationale = "shim fallback (role.default_model)"
            if ver_warn:
                rationale += f"; {ver_warn}"
            if notes:
                rationale += "; " + "; ".join(notes)
            shim = ResolvedCandidate(
                model_id=default_model,
                version_id=ver or "",
                effort=eff,
                rank=1,
                source_layer=8,
                rationale=rationale,
            )
            candidates.append(shim)
            _emit_shim_warning(role, default_model)
            trace.append({
                "step": 8, "layer": 8, "action": "shim_fallback",
                "count_before": 0, "count_after": 1,
                "details": {
                    "default_model": default_model,
                    "warning": "model.resolved.shim_fallback",
                },
            })
        else:
            trace.append({
                "step": 8, "layer": 8, "action": "shim_fallback_empty",
                "count_before": 0, "count_after": 0,
                "details": {"role": role},
            })
            raise NoViableModelError(
                f"no viable model for role={role!r} seniority={seniority!r} "
                f"team_member={team_member!r} scope={scope!r}"
            )
    else:
        trace.append({
            "step": 8, "layer": 8, "action": "shim_fallback",
            "count_before": len(candidates), "count_after": len(candidates),
            "details": {"skipped": "candidates already present"},
        })

    result = (candidates, trace) if explain else candidates
    _cache[key] = (time.time(), result)
    return result


def _collect_cost_bias(bindings: list[dict]) -> float | None:
    for b in bindings:
        conds = b.get("conditions") or {}
        if "cost_bias" in conds:
            return float(conds["cost_bias"])
    return None


def _emit_shim_warning(role: str, default_model: str) -> None:
    """Write a LogEntry recording the shim fallback.

    Silent-on-failure: never let logging break resolution. Matches the
    pattern used by processkit.log.log_side_effect.
    """
    try:
        from processkit import log
        log.log_side_effect(
            "Role", role, "model.resolved.shim_fallback",
            f"Shim fallback to {default_model!r} for {role!r}",
            actor=role,
        )
    except Exception:
        pass  # never fail resolution because of logging


# ---------------------------------------------------------------------------
# CLI entry-point (very small; used by tests and pk-explain-routing rendering)
# ---------------------------------------------------------------------------


def _render_text(candidates: list[ResolvedCandidate], trace: list[dict]) -> str:
    lines: list[str] = []
    lines.append("Resolution trace:")
    for t in trace:
        lines.append(
            f"  step {t['step']:>3} L{t['layer']}: {t['action']} "
            f"[{t['count_before']} → {t['count_after']}] {t.get('details') or ''}"
        )
    lines.append("")
    lines.append("Final candidates:")
    if not candidates:
        lines.append("  (none)")
    for c in candidates:
        lines.append(
            f"  [L{c.source_layer}] {c.model_id} v{c.version_id} "
            f"effort={c.effort} rank={c.rank} — {c.rationale}"
        )
    return "\n".join(lines)


if __name__ == "__main__":  # pragma: no cover
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("role")
    ap.add_argument("--seniority")
    ap.add_argument("--team-member")
    ap.add_argument("--scope")
    ap.add_argument("--explain", action="store_true")
    args = ap.parse_args()
    try:
        res = resolve(
            args.role, seniority=args.seniority, team_member=args.team_member,
            scope=args.scope, explain=True,
        )
        cands, trace = res if isinstance(res, tuple) else (res, [])
        print(_render_text(cands, trace))
    except NoViableModelError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(2)
