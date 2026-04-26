"""KeenFern regression: every seeded role must have a binding for
every level of the seniority ladder.

The roster's ladder is junior -> specialist -> expert -> senior -> principal
(5 levels). The default-bindings pack at
``context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml``
must contain a (role, seniority) seed for every combination of
seeded role and ladder level — otherwise ``resolve_model`` returns
``no viable model`` for the missing combinations.

Pre-KeenFern (v0.20.0) only 3 levels were seeded; 20 (10 roles x 2)
combinations failed. KeenFern (v0.21.0) closed the gap. This test
fires if anyone re-introduces the gap by removing a seed.

Run with:

    uv run --with pyyaml --with pytest \\
        pytest \\
        context/skills/processkit/model-recommender/scripts/test_default_bindings_coverage.py \\
        -v
"""
from __future__ import annotations

from pathlib import Path

import pytest
import yaml


SENIORITY_LADDER = ("junior", "specialist", "expert", "senior", "principal")

_HERE = Path(__file__).resolve().parent
_MANIFEST = _HERE.parent / "default-bindings" / "MANIFEST.yaml"


def _load_manifest() -> dict:
    docs = list(yaml.safe_load_all(_MANIFEST.read_text(encoding="utf-8")))
    if not docs:
        raise AssertionError(f"{_MANIFEST} is empty")
    return docs[0]


def test_manifest_loads():
    data = _load_manifest()
    assert data["kind"] == "DefaultBindingPack"
    assert isinstance(data.get("spec", {}).get("bindings"), list)


def test_every_seeded_role_covers_full_seniority_ladder():
    data = _load_manifest()
    bindings = data["spec"]["bindings"]

    by_role: dict[str, set[str]] = {}
    for b in bindings:
        if b.get("type") != "model-assignment":
            continue
        role = b["subject"]
        sen = b["conditions"]["seniority"]
        by_role.setdefault(role, set()).add(sen)

    assert by_role, "no model-assignment seeds found in manifest"

    gaps: list[str] = []
    for role, seens in sorted(by_role.items()):
        missing = set(SENIORITY_LADDER) - seens
        if missing:
            gaps.append(
                f"{role}: missing {sorted(missing)} (covered: {sorted(seens)})"
            )

    if gaps:
        pytest.fail(
            "Default binding pack has seniority-ladder gaps. "
            "resolve_model() will fail with 'no viable model' for these "
            "(role, seniority) pairs. Add seeds in MANIFEST.yaml.\n  "
            + "\n  ".join(gaps)
        )


def test_no_duplicate_role_seniority_seeds():
    data = _load_manifest()
    bindings = data["spec"]["bindings"]
    seen: dict[tuple[str, str], int] = {}
    for b in bindings:
        if b.get("type") != "model-assignment":
            continue
        if b["conditions"].get("rank") != 1:
            continue
        key = (b["subject"], b["conditions"]["seniority"])
        seen[key] = seen.get(key, 0) + 1
    duplicates = {k: v for k, v in seen.items() if v > 1}
    assert not duplicates, (
        f"duplicate rank-1 seeds for: {duplicates}"
    )
