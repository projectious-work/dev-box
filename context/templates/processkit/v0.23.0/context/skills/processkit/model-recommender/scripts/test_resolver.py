"""Tests for the 8-layer model-assignment binding resolver.

Run with:

    uv run --with pyyaml --with jsonschema --with pytest --with mcp \
        pytest context/skills/processkit/model-recommender/scripts/test_resolver.py -v

Covers:
  - Each of the 8 layers in isolation (8 tests)
  - Higher layers override lower (3 tests)
  - Capability filter: vision, tool-use, computer-use (3 tests)
  - Effort clamping: floor + ceiling (2 tests)
  - Version pin resolution (1 test)
  - Stale binding skip via valid_until past (1 test)
  - Dedupe across layers (1 test)
  - Tie-breakers: provider-pref, cost, version recency (3 tests)
  - Shim fallback emits the warning event (1 test)
  - NoViableModelError when no candidates anywhere (1 test)
  - Explain mode returns trace (1 test)
"""
from __future__ import annotations

import datetime as _dt
import sys
from pathlib import Path
from typing import Any

import pytest


# ---------------------------------------------------------------------------
# Path bootstrap
# ---------------------------------------------------------------------------

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))


def _find_lib() -> Path:
    here = _SCRIPTS_DIR
    while True:
        for c in (here / "src" / "lib", here / "_lib",
                  here / "context" / "skills" / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


_LIB = _find_lib()
if str(_LIB) not in sys.path:
    sys.path.insert(0, str(_LIB))


import resolver as R  # noqa: E402


# ---------------------------------------------------------------------------
# Fixtures: synthetic project tree with Role, Model, Binding entities
# ---------------------------------------------------------------------------


def _write_entity(dir_: Path, kind: str, id: str, spec: dict,
                  created: str = "2026-04-22T00:00:00Z") -> Path:
    dir_.mkdir(parents=True, exist_ok=True)
    import yaml
    frontmatter = {
        "apiVersion": "processkit.projectious.work/v1",
        "kind": kind,
        "metadata": {"id": id, "created": created},
        "spec": spec,
    }
    text = "---\n" + yaml.safe_dump(frontmatter, sort_keys=False) + "---\n"
    p = dir_ / f"{id}.md"
    p.write_text(text, encoding="utf-8")
    return p


def _mk_model(id: str, provider: str, family: str, equivalent_tier: str,
              *, efforts: list[str] | None = None,
              modalities: list[str] | None = None,
              versions: list[dict] | None = None,
              dims: dict[str, int] | None = None) -> dict:
    return {
        "provider": provider,
        "family": family,
        "versions": versions or [
            {"version_id": "1.0", "status": "ga", "released_at": "2026-01-01",
             "context_window": 128000,
             "pricing_usd_per_1m": {"input": 3.0, "output": 15.0}},
        ],
        "efforts_supported": efforts or ["none", "low", "medium", "high"],
        "modalities": modalities or ["text", "tools"],
        "dimensions": dims or {
            "reasoning": 4, "engineering": 4, "speed": 3,
            "breadth": 4, "reliability": 4, "governance": 5,
        },
        "access_tier": "public",
        "equivalent_tier": equivalent_tier,
    }


@pytest.fixture(autouse=True)
def _reset_cache():
    R.clear_cache()
    yield
    R.clear_cache()


@pytest.fixture()
def project_root(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Create a throwaway project root with a populated context/ tree."""
    (tmp_path / "AGENTS.md").write_text("# test project\n")
    ctx = tmp_path / "context"
    (ctx / "models").mkdir(parents=True)
    (ctx / "roles").mkdir(parents=True)
    (ctx / "bindings").mkdir(parents=True)
    (ctx / "logs").mkdir(parents=True)
    # Minimal role file
    _write_entity(ctx / "roles", "Role", "ROLE-software-engineer", {
        "name": "software-engineer",
        "description": "Test role",
        "default_seniority": "senior",
    })
    # Default models
    _write_entity(ctx / "models", "Model", "MODEL-anthropic-claude-sonnet",
                  _mk_model("MODEL-anthropic-claude-sonnet", "anthropic",
                            "claude-sonnet", "xxl",
                            efforts=["none", "low", "medium", "high",
                                     "extra-high", "max"],
                            modalities=["text", "vision", "tools",
                                        "computer-use"],
                            dims={"reasoning": 4, "engineering": 5,
                                  "speed": 3, "breadth": 4,
                                  "reliability": 4, "governance": 5}))
    _write_entity(ctx / "models", "Model", "MODEL-anthropic-claude-opus",
                  _mk_model("MODEL-anthropic-claude-opus", "anthropic",
                            "claude-opus", "xxl",
                            efforts=["none", "low", "medium", "high",
                                     "extra-high", "max"],
                            modalities=["text", "vision", "tools",
                                        "computer-use"],
                            versions=[{"version_id": "4.6", "status": "ga",
                                       "released_at": "2026-02-01",
                                       "pricing_usd_per_1m": {"input": 15.0,
                                                              "output": 75.0}}],
                            dims={"reasoning": 5, "engineering": 5,
                                  "speed": 2, "breadth": 4,
                                  "reliability": 5, "governance": 5}))
    _write_entity(ctx / "models", "Model", "MODEL-anthropic-claude-haiku",
                  _mk_model("MODEL-anthropic-claude-haiku", "anthropic",
                            "claude-haiku", "m",
                            efforts=["none", "low", "medium"],
                            modalities=["text", "tools"],
                            versions=[{"version_id": "4.5", "status": "ga",
                                       "released_at": "2026-01-15",
                                       "pricing_usd_per_1m": {"input": 0.25,
                                                              "output": 1.25}}],
                            dims={"reasoning": 3, "engineering": 3,
                                  "speed": 5, "breadth": 3,
                                  "reliability": 4, "governance": 5}))
    _write_entity(ctx / "models", "Model", "MODEL-openai-gpt-5",
                  _mk_model("MODEL-openai-gpt-5", "openai", "gpt-5", "xxl",
                            efforts=["none", "low", "medium", "high"],
                            modalities=["text", "tools"],
                            dims={"reasoning": 5, "engineering": 4,
                                  "speed": 3, "breadth": 4,
                                  "reliability": 4, "governance": 3}))
    _write_entity(ctx / "models", "Model", "MODEL-google-gemini-flash",
                  _mk_model("MODEL-google-gemini-flash", "google",
                            "gemini-flash", "m",
                            efforts=["none", "low", "medium"],
                            modalities=["text", "vision", "tools"],
                            versions=[{"version_id": "2.0", "status": "ga",
                                       "released_at": "2025-12-01",
                                       "pricing_usd_per_1m": {"input": 0.08,
                                                              "output": 0.30}}],
                            dims={"reasoning": 3, "engineering": 3,
                                  "speed": 5, "breadth": 4,
                                  "reliability": 3, "governance": 2}))

    monkeypatch.chdir(tmp_path)
    # Force the index DB into tmp_path
    cache = tmp_path / "context" / ".cache" / "processkit"
    cache.mkdir(parents=True, exist_ok=True)
    # Reindex so the resolver's queries see our fixtures
    from processkit import index as _idx
    _idx.reindex(tmp_path)
    return tmp_path


def _add_binding(project_root: Path, id: str, type_: str, subject: str,
                 target: str, conditions: dict | None = None,
                 valid_from: str | None = None,
                 valid_until: str | None = None) -> None:
    """Write a binding file + reindex."""
    spec = {"type": type_, "subject": subject, "target": target}
    if conditions:
        spec["conditions"] = conditions
    if valid_from:
        spec["valid_from"] = valid_from
    if valid_until:
        spec["valid_until"] = valid_until
    _write_entity(project_root / "context" / "bindings",
                  "Binding", id, spec)
    from processkit import index as _idx
    _idx.reindex(project_root)
    R.clear_cache()


def _add_role_default(project_root: Path, role_id: str,
                      default_model: str) -> None:
    import yaml
    f = project_root / "context" / "roles" / f"{role_id}.md"
    text = f.read_text()
    # Rewrite with default_model in spec.
    parts = text.split("---")
    data = yaml.safe_load(parts[1])
    data["spec"]["default_model"] = default_model
    new_text = "---\n" + yaml.safe_dump(data, sort_keys=False) + "---\n"
    f.write_text(new_text)
    from processkit import index as _idx
    _idx.reindex(project_root)
    R.clear_cache()


def _add_model(project_root: Path, **kwargs) -> None:
    """Add a fresh MODEL-* entity; kwargs passed to ``_mk_model``."""
    model_id = kwargs.pop("id")
    _write_entity(project_root / "context" / "models", "Model", model_id,
                  _mk_model(model_id, **kwargs))
    from processkit import index as _idx
    _idx.reindex(project_root)
    R.clear_cache()


# ---------------------------------------------------------------------------
# Layer-in-isolation tests (8)
# ---------------------------------------------------------------------------


def test_layer_1_task_pin_short_circuits(project_root):
    # Even when role bindings exist, a task-pin wins.
    _add_binding(project_root, "BIND-l6", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1, "rationale": "role default"})
    cands = R.resolve("ROLE-software-engineer",
                      task_hints={"model_pin": "MODEL-openai-gpt-5"})
    assert len(cands) == 1
    assert cands[0].model_id == "MODEL-openai-gpt-5"
    assert cands[0].source_layer == 1


def test_layer_2_team_member_preference(project_root):
    _add_binding(project_root, "BIND-l2", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1, "rationale": "atlas loves opus"})
    cands = R.resolve("ROLE-software-engineer", team_member="TEAMMEMBER-atlas")
    assert len(cands) == 1
    assert cands[0].model_id == "MODEL-anthropic-claude-opus"
    assert cands[0].source_layer == 2


def test_layer_3_project_veto_blocks_model(project_root):
    # Team-member likes opus; project vetoes opus; a role-default sonnet
    # binding is still viable, so the resolver should return sonnet.
    _add_binding(project_root, "BIND-l2", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-l3", "model-assignment",
                 "SCOPE-myproj", "MODEL-anthropic-claude-opus",
                 conditions={"blocked": True,
                             "rationale": "no opus this project"})
    _add_binding(project_root, "BIND-l6", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    cands = R.resolve("ROLE-software-engineer",
                      team_member="TEAMMEMBER-atlas",
                      scope="SCOPE-myproj")
    # Opus was vetoed; sonnet is the survivor.
    assert all(c.model_id != "MODEL-anthropic-claude-opus" for c in cands)
    assert cands[0].model_id == "MODEL-anthropic-claude-sonnet"


def test_layer_4_capability_filter_drops_non_vision(project_root):
    # Haiku on this fixture has no 'vision' modality.
    _add_binding(project_root, "BIND-l2", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-haiku",
                 conditions={"rank": 1})
    with pytest.raises(R.NoViableModelError):
        R.resolve("ROLE-software-engineer",
                  team_member="TEAMMEMBER-atlas",
                  task_hints={"requires_vision": True})


def test_layer_5_role_seniority_binding_matches(project_root):
    _add_binding(project_root, "BIND-l5", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"seniority": "senior", "rank": 1})
    cands = R.resolve("ROLE-software-engineer", seniority="senior")
    assert cands[0].model_id == "MODEL-anthropic-claude-sonnet"
    assert cands[0].source_layer == 5


def test_layer_5_role_seniority_binding_skipped_if_mismatch(project_root):
    _add_binding(project_root, "BIND-l5", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"seniority": "senior", "rank": 1})
    # principal request → the senior binding won't fire at layer 5;
    # expect NoViableModelError because no layer 6 either and no shim.
    with pytest.raises(R.NoViableModelError):
        R.resolve("ROLE-software-engineer", seniority="principal")


def test_layer_6_role_default_binding_fires_when_no_seniority_set(project_root):
    _add_binding(project_root, "BIND-l6", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1, "rationale": "role default"})
    cands = R.resolve("ROLE-software-engineer")
    assert cands[0].model_id == "MODEL-anthropic-claude-sonnet"
    assert cands[0].source_layer == 6


def test_layer_7_project_bias_reorders(project_root):
    # Two bindings at layer 6 with the same rank; project-bias prefers Google.
    _add_binding(project_root, "BIND-a", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-b", "model-assignment",
                 "ROLE-software-engineer", "MODEL-google-gemini-flash",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-bias", "model-assignment",
                 "SCOPE-myproj", "MODEL-google-gemini-flash",
                 conditions={"provider_preference": ["google", "anthropic"]})
    cands = R.resolve("ROLE-software-engineer", scope="SCOPE-myproj")
    assert cands[0].model_id == "MODEL-google-gemini-flash"


def test_layer_8_shim_fallback_fires_and_emits_warning(project_root):
    # No bindings anywhere; role.default_model = haiku.
    _add_role_default(project_root, "ROLE-software-engineer",
                      "MODEL-anthropic-claude-haiku")
    cands, trace = R.resolve("ROLE-software-engineer", explain=True)
    assert len(cands) == 1
    assert cands[0].source_layer == 8
    assert cands[0].model_id == "MODEL-anthropic-claude-haiku"
    # Trace includes the shim-fallback marker.
    shim_steps = [t for t in trace if t["action"] == "shim_fallback"]
    assert shim_steps
    assert shim_steps[0]["details"].get("warning") == "model.resolved.shim_fallback"


# ---------------------------------------------------------------------------
# Layer precedence: higher layers override lower (3)
# ---------------------------------------------------------------------------


def test_team_member_overrides_role_binding(project_root):
    _add_binding(project_root, "BIND-role", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"seniority": "senior", "rank": 1})
    _add_binding(project_root, "BIND-tm", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1})
    cands = R.resolve("ROLE-software-engineer", seniority="senior",
                      team_member="TEAMMEMBER-atlas")
    # Both contribute; the team-member binding is higher precedence (layer 2)
    # and must sort first.
    assert cands[0].model_id == "MODEL-anthropic-claude-opus"
    assert cands[0].source_layer == 2


def test_project_veto_beats_team_member_pref(project_root):
    _add_binding(project_root, "BIND-tm", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-veto", "model-assignment",
                 "SCOPE-p", "MODEL-anthropic-claude-opus",
                 conditions={"blocked": True})
    _add_binding(project_root, "BIND-fallback", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"seniority": "senior", "rank": 1})
    cands = R.resolve("ROLE-software-engineer", seniority="senior",
                      team_member="TEAMMEMBER-atlas", scope="SCOPE-p")
    # Opus is vetoed at layer 3 — Sonnet wins via layer 5.
    assert all(c.model_id != "MODEL-anthropic-claude-opus" for c in cands)
    assert cands[0].model_id == "MODEL-anthropic-claude-sonnet"


def test_seniority_beats_role_default(project_root):
    _add_binding(project_root, "BIND-sen", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-opus",
                 conditions={"seniority": "principal", "rank": 1})
    _add_binding(project_root, "BIND-def", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    cands = R.resolve("ROLE-software-engineer", seniority="principal")
    # Seniority layer (5) beats default layer (6).
    assert cands[0].model_id == "MODEL-anthropic-claude-opus"
    assert cands[0].source_layer == 5


# ---------------------------------------------------------------------------
# Capability filter: vision / tool-use / computer-use (3)
# ---------------------------------------------------------------------------


def test_capability_requires_vision(project_root):
    # Only models with 'vision' modality should survive.
    _add_binding(project_root, "BIND-s", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-g5", "model-assignment",
                 "ROLE-software-engineer", "MODEL-openai-gpt-5",
                 conditions={"rank": 2})
    cands = R.resolve("ROLE-software-engineer",
                      task_hints={"requires_vision": True})
    ids = [c.model_id for c in cands]
    assert "MODEL-anthropic-claude-sonnet" in ids
    assert "MODEL-openai-gpt-5" not in ids  # no vision in fixture


def test_capability_requires_tool_use(project_root):
    # Add a model without tool support.
    _add_model(project_root, id="MODEL-notool", provider="fake", family="f",
               equivalent_tier="m", modalities=["text"])
    _add_binding(project_root, "BIND-ok", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-bad", "model-assignment",
                 "ROLE-software-engineer", "MODEL-notool",
                 conditions={"rank": 2})
    cands = R.resolve("ROLE-software-engineer",
                      task_hints={"requires_tool_use": True})
    ids = [c.model_id for c in cands]
    assert "MODEL-anthropic-claude-sonnet" in ids
    assert "MODEL-notool" not in ids


def test_capability_requires_computer_use(project_root):
    _add_binding(project_root, "BIND-opus", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-haiku", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-haiku",
                 conditions={"rank": 2})
    cands = R.resolve("ROLE-software-engineer",
                      task_hints={"requires_computer_use": True})
    ids = [c.model_id for c in cands]
    assert "MODEL-anthropic-claude-opus" in ids
    assert "MODEL-anthropic-claude-haiku" not in ids


# ---------------------------------------------------------------------------
# Effort clamping (2)
# ---------------------------------------------------------------------------


def test_effort_ceiling_clamps_to_model_max(project_root):
    # Haiku in our fixture supports efforts up to 'medium'. Ceiling 'max'
    # should clamp to 'medium' and note the clamp in rationale.
    _add_binding(project_root, "BIND-h", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-haiku",
                 conditions={"rank": 1, "effort_ceiling": "max"})
    cands = R.resolve("ROLE-software-engineer")
    assert cands[0].effort == "medium"
    assert "clamp" in cands[0].rationale.lower()


def test_effort_floor_raises_effort(project_root):
    _add_binding(project_root, "BIND-o", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1, "effort_floor": "high"})
    cands = R.resolve("ROLE-software-engineer")
    # Default effort was 'medium' but floor='high' kicks it up.
    assert cands[0].effort == "high"


# ---------------------------------------------------------------------------
# Version pin (1)
# ---------------------------------------------------------------------------


def test_version_pin_resolution(project_root):
    # Add a model with two versions (new + old).
    _add_model(project_root, id="MODEL-multi", provider="fake", family="multi",
               equivalent_tier="l",
               versions=[
                   {"version_id": "3.0", "status": "ga",
                    "released_at": "2026-04-01"},
                   {"version_id": "2.0", "status": "ga",
                    "released_at": "2025-10-01"},
               ])
    _add_binding(project_root, "BIND-pin", "model-assignment",
                 "ROLE-software-engineer", "MODEL-multi",
                 conditions={"rank": 1, "version_pin": "2.0"})
    cands = R.resolve("ROLE-software-engineer")
    assert cands[0].version_id == "2.0"


# ---------------------------------------------------------------------------
# Stale binding (valid_until past) is skipped (1)
# ---------------------------------------------------------------------------


def test_stale_binding_skipped(project_root):
    # Stale team-member binding (expired) MUST be skipped so only the
    # role binding fires.
    _add_binding(project_root, "BIND-stale", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1},
                 valid_until="2020-01-01")
    _add_binding(project_root, "BIND-fresh", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"seniority": "senior", "rank": 1})
    cands = R.resolve("ROLE-software-engineer", seniority="senior",
                      team_member="TEAMMEMBER-atlas")
    assert cands[0].model_id == "MODEL-anthropic-claude-sonnet"
    # No opus showed up (stale binding ignored).
    assert all(c.model_id != "MODEL-anthropic-claude-opus" for c in cands)


# ---------------------------------------------------------------------------
# Dedupe across layers (1)
# ---------------------------------------------------------------------------


def test_dedupe_keeps_highest_precedence(project_root):
    # Same (model, effort) appears at layer 2 and layer 6 — layer 2 wins.
    _add_binding(project_root, "BIND-tm", "model-assignment",
                 "TEAMMEMBER-atlas", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1, "effort_floor": "medium",
                             "effort_ceiling": "medium"})
    _add_binding(project_root, "BIND-role", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1, "effort_floor": "medium",
                             "effort_ceiling": "medium"})
    cands = R.resolve("ROLE-software-engineer",
                      team_member="TEAMMEMBER-atlas")
    # Only one copy should remain after dedupe, and it must be from layer 2.
    sonnet_entries = [c for c in cands
                      if c.model_id == "MODEL-anthropic-claude-sonnet"]
    assert len(sonnet_entries) == 1
    assert sonnet_entries[0].source_layer == 2


# ---------------------------------------------------------------------------
# Tie-breakers (3)
# ---------------------------------------------------------------------------


def test_tiebreak_prefers_provider(project_root):
    # Two layer-6 bindings; project-bias provider_preference=[google,...].
    _add_binding(project_root, "BIND-a", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-g", "model-assignment",
                 "ROLE-software-engineer", "MODEL-google-gemini-flash",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-bias", "model-assignment",
                 "SCOPE-p", "MODEL-google-gemini-flash",
                 conditions={"provider_preference": ["google"]})
    cands = R.resolve("ROLE-software-engineer", scope="SCOPE-p")
    assert cands[0].model_id == "MODEL-google-gemini-flash"


def test_tiebreak_prefers_lower_cost(project_root):
    # Two same-tier same-layer bindings; no provider pref; cheapest wins.
    _add_binding(project_root, "BIND-o", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-opus",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-h", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-haiku",
                 conditions={"rank": 1})
    cands = R.resolve("ROLE-software-engineer")
    # Haiku is far cheaper than Opus (1.25 vs 75 output per 1M).
    assert cands[0].model_id == "MODEL-anthropic-claude-haiku"


def test_tiebreak_prefers_recent_version(project_root):
    # Two providers with identical price; more recent release wins.
    _add_model(project_root, id="MODEL-old", provider="prov", family="old",
               equivalent_tier="l",
               versions=[{"version_id": "1.0", "status": "ga",
                          "released_at": "2023-01-01",
                          "pricing_usd_per_1m": {"input": 1.0,
                                                 "output": 2.0}}])
    _add_model(project_root, id="MODEL-new", provider="prov", family="new",
               equivalent_tier="l",
               versions=[{"version_id": "1.0", "status": "ga",
                          "released_at": "2026-03-01",
                          "pricing_usd_per_1m": {"input": 1.0,
                                                 "output": 2.0}}])
    _add_binding(project_root, "BIND-old", "model-assignment",
                 "ROLE-software-engineer", "MODEL-old",
                 conditions={"rank": 1})
    _add_binding(project_root, "BIND-new", "model-assignment",
                 "ROLE-software-engineer", "MODEL-new",
                 conditions={"rank": 1})
    cands = R.resolve("ROLE-software-engineer")
    assert cands[0].model_id == "MODEL-new"


# ---------------------------------------------------------------------------
# No viable model (1)
# ---------------------------------------------------------------------------


def test_no_viable_model_raises(project_root):
    # No bindings anywhere, no default_model shim.
    with pytest.raises(R.NoViableModelError):
        R.resolve("ROLE-software-engineer", seniority="senior")


# ---------------------------------------------------------------------------
# Explain mode (1)
# ---------------------------------------------------------------------------


def test_explain_mode_returns_trace(project_root):
    _add_binding(project_root, "BIND-l6", "model-assignment",
                 "ROLE-software-engineer", "MODEL-anthropic-claude-sonnet",
                 conditions={"rank": 1})
    out = R.resolve("ROLE-software-engineer", explain=True)
    assert isinstance(out, tuple)
    cands, trace = out
    assert cands
    # Each layer (1..8) is represented at least once in the trace.
    layers = {t["layer"] for t in trace}
    for L in (3, 4, 5, 6, 7, 8):
        assert L in layers, f"layer {L} missing from trace"
    # Steps include required fields
    for t in trace:
        for key in ("step", "layer", "action",
                    "count_before", "count_after", "details"):
            assert key in t
