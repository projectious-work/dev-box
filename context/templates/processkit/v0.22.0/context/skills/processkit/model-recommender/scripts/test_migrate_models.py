#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
#   "pytest>=8.0",
# ]
# ///
"""Tests for migrate_models.py — v0.19.0 Phase 3 model extraction."""
from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest
import yaml

# ── module import ──────────────────────────────────────────────────────────

HERE = Path(__file__).parent
MIGRATE_PATH = HERE / "migrate_models.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("migrate_models", MIGRATE_PATH)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["migrate_models"] = mod
    spec.loader.exec_module(mod)
    return mod


mm = _load_module()


# ── sample data ────────────────────────────────────────────────────────────

SAMPLE_ENTRY_OPUS = {
    "id": "claude-opus-4.6",
    "name": "Claude Opus 4.6",
    "provider": "Anthropic",
    "tier": "Frontier flagship",
    "context_k": 200,
    "governance_warning": None,
    "pricing": {
        "input_per_1m": 15.0,
        "output_per_1m": 75.0,
        "currency": "USD",
        "hosting": "api",
        "note": "Flagship pricing",
    },
    "dimensions": {
        "R": {"score": 5, "sub": {"math_reasoning": 5}},
        "E": {"score": 5, "sub": {"function_codegen": 5, "tool_use": 5}},
        "S": {"score": 2, "sub": {"ttft": 2}},
        "B": {"score": 4, "sub": {"vision": 4, "audio": 1}},
        "L": {"score": 5, "sub": {"instruction_following": 5}},
        "G": {"score": 5, "sub": {"data_retention_policy": 5}},
    },
}

SAMPLE_ENTRY_OPUS_OLD = {
    **SAMPLE_ENTRY_OPUS,
    "id": "claude-opus-4.5",
    "name": "Claude Opus 4.5",
    "pricing": {**SAMPLE_ENTRY_OPUS["pricing"], "input_per_1m": 10.0},
}

SAMPLE_ENTRY_HAIKU = {
    "id": "claude-haiku-4.5",
    "name": "Claude Haiku 4.5",
    "provider": "Anthropic",
    "tier": "Frontier fast",
    "context_k": 200,
    "governance_warning": None,
    "pricing": {
        "input_per_1m": 0.25,
        "output_per_1m": 1.25,
        "currency": "USD",
        "hosting": "api",
    },
    "dimensions": {
        "R": {"score": 3, "sub": {}},
        "E": {"score": 3, "sub": {"tool_use": 3}},
        "S": {"score": 5, "sub": {}},
        "B": {"score": 3, "sub": {"vision": 3, "audio": 1}},
        "L": {"score": 4, "sub": {}},
        "G": {"score": 5, "sub": {}},
    },
}

SAMPLE_ENTRY_GEMINI_FLASH = {
    "id": "gemini-flash-2.0",
    "name": "Gemini Flash 2.0",
    "provider": "Google",
    "tier": "Frontier fast",
    "context_k": 1000,
    "governance_warning": None,
    "pricing": {
        "input_per_1m": 0.075,
        "output_per_1m": 0.30,
        "currency": "USD",
        "hosting": "api",
    },
    "dimensions": {
        "R": {"score": 3, "sub": {}},
        "E": {"score": 3, "sub": {"tool_use": 3}},
        "S": {"score": 5, "sub": {}},
        "B": {"score": 5, "sub": {"vision": 5, "audio": 5}},
        "L": {"score": 3, "sub": {}},
        "G": {"score": 2, "sub": {}},
    },
}

SAMPLE_ENTRY_ESTIMATED = {
    **SAMPLE_ENTRY_OPUS,
    "_estimated": True,
    "id": "gpt-5",
    "name": "GPT-5",
    "provider": "OpenAI",
    "tier": "Frontier flagship (estimated)",
    "context_k": 256,
}

SAMPLE_JSON = {
    "_meta": {"validated": "2026-Q1"},
    "models": [
        SAMPLE_ENTRY_OPUS,
        SAMPLE_ENTRY_OPUS_OLD,
        SAMPLE_ENTRY_HAIKU,
        SAMPLE_ENTRY_GEMINI_FLASH,
        SAMPLE_ENTRY_ESTIMATED,
    ],
}


# ── unit tests ─────────────────────────────────────────────────────────────

def test_parse_id_known_anthropic():
    family, version = mm.parse_id("claude-opus-4.6")
    assert family == "claude-opus"
    assert version == "4.6"


def test_parse_id_known_google():
    family, version = mm.parse_id("gemini-2.5-pro")
    assert family == "gemini-2-5-pro"
    assert version == "2.5"


def test_parse_id_unknown_fallback():
    # Arbitrary unseen id should still parse without raising.
    family, version = mm.parse_id("vendor-model-7.3")
    assert family and version


def test_equivalent_tier_mapping():
    assert mm.equivalent_tier(5, 5) == "xxl"
    assert mm.equivalent_tier(5, 4) == "xxl"  # avg 4.5
    assert mm.equivalent_tier(4, 4) == "xl"
    assert mm.equivalent_tier(4, 3) == "l"   # avg 3.5
    assert mm.equivalent_tier(3, 3) == "m"
    assert mm.equivalent_tier(3, 2) == "s"   # avg 2.5
    assert mm.equivalent_tier(2, 2) == "xs"


def test_efforts_for_family_small():
    # Haiku-class caps at medium
    assert mm.efforts_for_family("claude-haiku") == ["none", "low", "medium"]


def test_efforts_for_family_flagship():
    # Opus-class goes full ladder
    effs = mm.efforts_for_family("claude-opus")
    assert "max" in effs
    assert "extra-high" in effs


def test_build_entity_groups_versions():
    entity = mm.build_entity(
        "anthropic", "claude-opus",
        [SAMPLE_ENTRY_OPUS, SAMPLE_ENTRY_OPUS_OLD],
    )
    assert entity["kind"] == "Model"
    assert entity["metadata"]["id"] == "MODEL-anthropic-claude-opus"
    assert entity["spec"]["provider"] == "anthropic"
    assert entity["spec"]["family"] == "claude-opus"
    assert len(entity["spec"]["versions"]) == 2
    version_ids = {v["version_id"] for v in entity["spec"]["versions"]}
    assert version_ids == {"4.6", "4.5"}


def test_build_entity_dimensions_scalar():
    entity = mm.build_entity("anthropic", "claude-opus", [SAMPLE_ENTRY_OPUS])
    dims = entity["spec"]["dimensions"]
    # Flat scalar ints, not the nested {score, sub} form.
    assert dims["reasoning"] == 5
    assert dims["engineering"] == 5
    assert dims["speed"] == 2
    assert isinstance(dims["reasoning"], int)


def test_build_entity_tier_xxl_for_opus():
    entity = mm.build_entity("anthropic", "claude-opus", [SAMPLE_ENTRY_OPUS])
    assert entity["spec"]["equivalent_tier"] == "xxl"


def test_build_entity_modalities_includes_computer_use_for_opus():
    entity = mm.build_entity("anthropic", "claude-opus", [SAMPLE_ENTRY_OPUS])
    assert "computer-use" in entity["spec"]["modalities"]
    assert "vision" in entity["spec"]["modalities"]


def test_build_entity_estimated_is_preview():
    entity = mm.build_entity("openai", "gpt-5", [SAMPLE_ENTRY_ESTIMATED])
    assert entity["spec"]["versions"][0]["status"] == "preview"


def test_group_entries_maps_providers():
    grouped = mm.group_entries(SAMPLE_JSON["models"])
    keys = set(grouped.keys())
    assert ("anthropic", "claude-opus") in keys
    assert ("anthropic", "claude-haiku") in keys
    assert ("google", "gemini-flash") in keys
    assert ("openai", "gpt-5") in keys


def test_group_entries_family_grouping_combines_versions():
    grouped = mm.group_entries(SAMPLE_JSON["models"])
    # Opus 4.6 + 4.5 should be in one family entry.
    opus_entries = grouped[("anthropic", "claude-opus")]
    assert len(opus_entries) == 2
    # Newest first after internal sort.
    assert opus_entries[0]["id"] == "claude-opus-4.6"


# ── integration tests (file I/O) ──────────────────────────────────────────

@pytest.fixture
def tmp_migrate(tmp_path):
    scores = tmp_path / "scores.json"
    scores.write_text(json.dumps(SAMPLE_JSON))
    out = tmp_path / "models"
    src_out = tmp_path / "src_models"
    return scores, out, src_out


def test_migrate_writes_files(tmp_migrate):
    scores, out, src_out = tmp_migrate
    summary = mm.migrate(scores, out, src_out)
    assert summary["json_models"] == 5
    assert summary["families_written"] == 4
    assert (out / "MODEL-anthropic-claude-opus.md").exists()
    assert (out / "MODEL-google-gemini-flash.md").exists()


def test_migrate_dual_tree_mirror(tmp_migrate):
    scores, out, src_out = tmp_migrate
    mm.migrate(scores, out, src_out)
    primary_files = sorted(p.name for p in out.glob("*.md"))
    mirror_files = sorted(p.name for p in src_out.glob("*.md"))
    assert primary_files == mirror_files
    assert len(primary_files) > 0
    # Byte-identical content.
    for name in primary_files:
        assert (out / name).read_text() == (src_out / name).read_text()


def test_migrate_idempotent(tmp_migrate):
    scores, out, src_out = tmp_migrate
    mm.migrate(scores, out, src_out)
    first_snap = {p.name: p.read_text() for p in out.glob("*.md")}
    mm.migrate(scores, out, src_out)
    second_snap = {p.name: p.read_text() for p in out.glob("*.md")}
    assert first_snap == second_snap


def test_migrate_preserves_existing_created_timestamp(tmp_migrate):
    scores, out, src_out = tmp_migrate
    out.mkdir(parents=True)
    # Seed an existing file with a custom timestamp.
    (out / "MODEL-anthropic-claude-opus.md").write_text(
        "---\n"
        "apiVersion: processkit.projectious.work/v1\n"
        "kind: Model\n"
        "metadata:\n"
        "  id: MODEL-anthropic-claude-opus\n"
        "  created: '2025-01-01T00:00:00Z'\n"
        "spec:\n"
        "  provider: anthropic\n"
        "  family: claude-opus\n"
        "  versions: []\n"
        "---\n"
    )
    mm.migrate(scores, out, src_out)
    content = (out / "MODEL-anthropic-claude-opus.md").read_text()
    assert "2025-01-01T00:00:00Z" in content


def test_migrate_output_parses_as_valid_yaml(tmp_migrate):
    scores, out, src_out = tmp_migrate
    mm.migrate(scores, out, src_out)
    for p in out.glob("*.md"):
        parts = p.read_text().split("---")
        assert len(parts) >= 3, f"{p} missing YAML frontmatter delimiters"
        doc = yaml.safe_load(parts[1])
        assert doc["apiVersion"] == "processkit.projectious.work/v1"
        assert doc["kind"] == "Model"
        assert doc["spec"]["provider"]
        assert doc["spec"]["family"]
        assert doc["spec"]["versions"]
        assert doc["spec"]["equivalent_tier"] in {"xs", "s", "m", "l", "xl", "xxl"}


def test_modalities_for_adds_vision_when_breadth_strong():
    mods = mm.modalities_for(SAMPLE_ENTRY_GEMINI_FLASH)
    assert "vision" in mods
    assert "audio" in mods


def test_modalities_for_omits_audio_when_score_low():
    mods = mm.modalities_for(SAMPLE_ENTRY_OPUS)
    assert "audio" not in mods


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
