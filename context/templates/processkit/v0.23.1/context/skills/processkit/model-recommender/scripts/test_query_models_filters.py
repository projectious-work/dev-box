"""Tests for query_models() RoyalFern governance filters (SolidWolf).

Run with:

    uv run --with pyyaml --with pytest --with mcp \
        pytest context/skills/processkit/model-recommender/scripts/test_query_models_filters.py -v

Covers BACK-20260426_1214-SolidWolf: query_models() must support filters
on the RoyalFern fields (jurisdiction, data_privacy, knowledge_cutoff,
vendor_model_id, latency_p50_ms) so the SKILL.md examples actually work
end-to-end.
"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

_REPO_ROOT = Path(__file__).resolve().parents[5]
_SERVER_DIR = _REPO_ROOT / "context" / "skills" / "processkit" / "model-recommender" / "mcp"
sys.path.insert(0, str(_SERVER_DIR))


def _make_model(
    *,
    mid: str = "test-model",
    R: int = 4,
    E: int = 4,
    S: int = 4,
    B: int = 4,
    L: int = 4,
    G: int = 4,
    data_privacy: dict | None = None,
    jurisdiction: dict | None = None,
    latency_p50_ms: int | None = None,
) -> dict:
    """Build a single legacy-shape model entry for unit testing the filter."""
    return {
        "id": mid,
        "name": mid,
        "provider": "Test",
        "tier": "test",
        "context_k": 200,
        "governance_warning": None,
        "pricing": {"input_per_1m": 1.0, "output_per_1m": 1.0, "currency": "USD", "hosting": "api"},
        "dimensions": {
            d: {"score": s, "sub": {}}
            for d, s in (("R", R), ("E", E), ("S", S), ("B", B), ("L", L), ("G", G))
        },
        "_estimated": False,
        "best_for": [],
        "avoid_for": [],
        "data_privacy": data_privacy or {},
        "jurisdiction": jurisdiction or {},
        "latency_p50_ms": latency_p50_ms,
        "vendor_model_id": None,
        "knowledge_cutoff": None,
    }


@pytest.fixture
def fake_scores(monkeypatch):
    """Inject a tiny fake roster covering the matrix the tests need."""
    if "server" in sys.modules:
        del sys.modules["server"]
    import server  # noqa: F401

    models = [
        # Anthropic-shape: HIPAA-eligible, US, no training, zero retention.
        _make_model(
            mid="hipaa-us-strict",
            G=5,
            jurisdiction={
                "vendor_hq_country": "US",
                "applicable_legal_regimes": ["EU-GDPR", "US-HIPAA"],
                "data_residency_regions": ["us", "eu"],
            },
            data_privacy={
                "dpa_available": "https://example.com/dpa",
                "data_retention_days": "zero",
                "training_on_customer_data": "never",
                "pii_eligible": True,
                "phi_hipaa_eligible": True,
                "gdpr_eligible": True,
            },
            latency_p50_ms=600,
        ),
        # OpenAI-shape: HIPAA-eligible but US-only, retention 30, opt-out.
        _make_model(
            mid="hipaa-us-30day",
            G=4,
            jurisdiction={
                "vendor_hq_country": "US",
                "applicable_legal_regimes": ["US-HIPAA"],
                "data_residency_regions": ["us"],
            },
            data_privacy={
                "dpa_available": True,
                "data_retention_days": 30,
                "training_on_customer_data": "opt-out",
                "pii_eligible": True,
                "phi_hipaa_eligible": True,
                "gdpr_eligible": False,
            },
            latency_p50_ms=400,
        ),
        # CN-shape: not HIPAA-eligible, CN HQ, training opt-out, unknown retention.
        _make_model(
            mid="cn-no-hipaa",
            G=2,
            jurisdiction={
                "vendor_hq_country": "CN",
                "applicable_legal_regimes": ["CN-DSL", "CN-PIPL"],
                "data_residency_regions": ["ap-east"],
            },
            data_privacy={
                "data_retention_days": "unknown",
                "training_on_customer_data": "always",
                "phi_hipaa_eligible": False,
                "pii_eligible": False,
                "gdpr_eligible": False,
            },
            latency_p50_ms=300,
        ),
        # Self-hosted-shape: missing data_privacy entirely (unknown posture).
        _make_model(mid="open-no-privacy"),
    ]

    monkeypatch.setattr(server, "_load_scores", lambda: {"_meta": {}, "models": models})
    monkeypatch.setattr(server, "_load_config", lambda: {})
    return server


def _ids(out):
    return [m["id"] for m in out]


def test_no_filter_returns_all(fake_scores):
    server = fake_scores
    out = server.query_models(apply_user_filter=False, limit=10)
    assert set(_ids(out)) == {
        "hipaa-us-strict",
        "hipaa-us-30day",
        "cn-no-hipaa",
        "open-no-privacy",
    }


def test_phi_hipaa_eligible_filter(fake_scores):
    """SKILL.md example: query_models(phi_hipaa_eligible=True)."""
    server = fake_scores
    out = server.query_models(
        phi_hipaa_eligible=True, apply_user_filter=False, limit=10
    )
    assert set(_ids(out)) == {"hipaa-us-strict", "hipaa-us-30day"}


def test_phi_hipaa_filter_rejects_missing_field(fake_scores):
    """A model with no data_privacy block must be REJECTED, not allowed."""
    server = fake_scores
    out = server.query_models(
        phi_hipaa_eligible=True, apply_user_filter=False, limit=10
    )
    assert "open-no-privacy" not in _ids(out)


def test_jurisdiction_country_in(fake_scores):
    """SKILL.md example: jurisdiction_country_in=['US','CA','FR']."""
    server = fake_scores
    out = server.query_models(
        jurisdiction_country_in=["US", "CA", "FR"],
        apply_user_filter=False,
        limit=10,
    )
    assert set(_ids(out)) == {"hipaa-us-strict", "hipaa-us-30day"}

    out_cn = server.query_models(
        jurisdiction_country_in=["CN"], apply_user_filter=False, limit=10
    )
    assert set(_ids(out_cn)) == {"cn-no-hipaa"}


def test_training_on_customer_data_filter(fake_scores):
    """SKILL.md example: training_on_customer_data='never'."""
    server = fake_scores
    out = server.query_models(
        training_on_customer_data="never",
        apply_user_filter=False,
        limit=10,
    )
    assert set(_ids(out)) == {"hipaa-us-strict"}


def test_data_retention_days_max_zero_string(fake_scores):
    """data_retention_days_max=0 must accept the literal 'zero' string but
    not 'unknown', and not numeric values > 0."""
    server = fake_scores
    out = server.query_models(
        data_retention_days_max=0, apply_user_filter=False, limit=10
    )
    assert set(_ids(out)) == {"hipaa-us-strict"}


def test_data_retention_days_max_30_includes_zero_and_30(fake_scores):
    server = fake_scores
    out = server.query_models(
        data_retention_days_max=30, apply_user_filter=False, limit=10
    )
    assert set(_ids(out)) == {"hipaa-us-strict", "hipaa-us-30day"}


def test_legal_regime_in_any_match(fake_scores):
    server = fake_scores
    # US-HIPAA matches both us-strict and us-30day.
    out = server.query_models(
        legal_regime_in=["US-HIPAA"], apply_user_filter=False, limit=10
    )
    assert set(_ids(out)) == {"hipaa-us-strict", "hipaa-us-30day"}
    # Multi-regime, any-match.
    out2 = server.query_models(
        legal_regime_in=["EU-GDPR", "CN-DSL"], apply_user_filter=False, limit=10
    )
    assert set(_ids(out2)) == {"hipaa-us-strict", "cn-no-hipaa"}


def test_data_residency_in(fake_scores):
    server = fake_scores
    out = server.query_models(
        data_residency_in=["eu"], apply_user_filter=False, limit=10
    )
    assert set(_ids(out)) == {"hipaa-us-strict"}


def test_max_latency_p50_ms(fake_scores):
    server = fake_scores
    out = server.query_models(
        max_latency_p50_ms=400, apply_user_filter=False, limit=10
    )
    assert set(_ids(out)) == {"hipaa-us-30day", "cn-no-hipaa"}


def test_filters_compose(fake_scores):
    """Multiple filters AND together — must require ALL to pass."""
    server = fake_scores
    out = server.query_models(
        phi_hipaa_eligible=True,
        training_on_customer_data="never",
        jurisdiction_country_in=["US"],
        apply_user_filter=False,
        limit=10,
    )
    assert _ids(out) == ["hipaa-us-strict"]


def test_existing_score_filters_still_work(fake_scores):
    """Pre-RoyalFern filters (R, E, G score minimums) must compose with
    the new ones, no regression."""
    server = fake_scores
    out = server.query_models(
        G=5, phi_hipaa_eligible=True, apply_user_filter=False, limit=10
    )
    assert _ids(out) == ["hipaa-us-strict"]


def test_profile_block_includes_royalfern_fields(fake_scores):
    """The summary profile each query result returns must surface the
    relevant RoyalFern fields when present, so callers can verify what
    they got without a second call."""
    server = fake_scores
    out = server.query_models(
        phi_hipaa_eligible=True, apply_user_filter=False, limit=10
    )
    by_id = {m["id"]: m for m in out}
    strict = by_id["hipaa-us-strict"]
    assert strict["data_privacy"]["phi_hipaa_eligible"] is True
    assert strict["jurisdiction"]["vendor_hq_country"] == "US"
    assert strict["latency_p50_ms"] == 600
