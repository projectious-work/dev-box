"""Tests for pk-retro — post-release retrospective generator.

Run with:
    uv run --with pytest --with pyyaml --with jsonschema \
        python3 -m pytest scripts/test_retro.py -v

Covers:
  - Each signal module with mock MCP data
  - Aggregator merges signals into correct section order
  - 80-line cap enforced when --verbose is off
  - --verbose bypasses cap
  - --auto-workitems branches (create_workitem called / not called)
  - Hallucination guard: no-entity slip item gets [uncertain] tag
  - Dual-emit: create_artifact and log_event both called on success
  - log_event NOT called if create_artifact fails (atomicity)
  - --dry-run skips both MCP writes, prints to stdout
"""

from __future__ import annotations

import sys
import types
from io import StringIO
from pathlib import Path
from typing import Any
from unittest.mock import MagicMock, call, patch

import pytest

# ---------------------------------------------------------------------------
# Path bootstrap so we can import from scripts/signals/ directly
# ---------------------------------------------------------------------------
_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

from signals import release_summary as rs_mod  # noqa: E402
from signals import timeline as tl_mod  # noqa: E402
from signals import workitems as wi_mod  # noqa: E402
from signals import drift as dr_mod  # noqa: E402
import pk_retro as retro  # noqa: E402


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture()
def repo_root(tmp_path: Path) -> Path:
    """Create a minimal fake repo structure."""
    (tmp_path / "context" / "logs" / "2026" / "04").mkdir(parents=True)
    (tmp_path / "context" / "workitems").mkdir(parents=True)
    return tmp_path


@pytest.fixture()
def mock_release_events() -> list[dict]:
    return [
        {
            "id": "LOG-20260418-Abc-release",
            "event_type": "release.published",
            "timestamp": "2026-04-18T10:00:00Z",
            "summary": "Release v0.18.2",
            "details": {"version": "v0.18.2"},
        },
        {
            "id": "LOG-20260415-Xyz-release",
            "event_type": "release.published",
            "timestamp": "2026-04-15T10:00:00Z",
            "summary": "Release v0.18.1",
            "details": {"version": "v0.18.1"},
        },
    ]


@pytest.fixture()
def mock_workitems_done() -> list[dict]:
    return [
        {
            "metadata": {"id": "BACK-001"},
            "spec": {
                "title": "Implement feature A",
                "state": "done",
                "type": "feature",
                "completed_at": "2026-04-17T09:00:00Z",
                "notes": "",
            },
        },
        {
            "metadata": {"id": "BACK-002"},
            "spec": {
                "title": "Fix login bug",
                "state": "done",
                "type": "bug",
                "completed_at": "2026-04-16T12:00:00Z",
                "notes": "",
            },
        },
    ]


@pytest.fixture()
def mock_workitems_deferred() -> list[dict]:
    return [
        {
            "metadata": {"id": "BACK-003"},
            "spec": {
                "title": "Refactor authentication",
                "state": "deferred",
                "type": "chore",
                "completed_at": "",
                "deferred_at": "2026-04-17T15:00:00Z",
                "notes": "Scope too large for this cycle",
            },
        },
    ]


@pytest.fixture()
def mock_workitems_superseded() -> list[dict]:
    return [
        {
            "metadata": {"id": "BACK-004"},
            "spec": {
                "title": "Old approach replaced by new design",
                "state": "superseded",
                "type": "feature",
                "completed_at": "",
                "deferred_at": "2026-04-16T10:00:00Z",
                "notes": "Superseded by BACK-010",
            },
        },
    ]


@pytest.fixture()
def mock_workitems_cancelled() -> list[dict]:
    return [
        {
            "metadata": {"id": "BACK-005"},
            "spec": {
                "title": "SunnyDawn — per-skill settings.toml (no longer needed)",
                "state": "cancelled",
                "type": "chore",
                "completed_at": "",
                "deferred_at": "2026-04-17T12:00:00Z",
                "notes": "Concern resolved via per-skill settings.toml",
            },
        },
    ]


@pytest.fixture()
def mock_doctor_reports() -> list[dict]:
    return [
        {
            "id": "LOG-20260415-Dr-first",
            "event_type": "doctor.report",
            "timestamp": "2026-04-15T08:00:00Z",
            "summary": "doctor 0 ERROR / 2 WARN",
            "details": {
                "categories": {
                    "drift": {"ERROR": 0, "WARN": 1, "INFO": 1},
                    "schema_filename": {"ERROR": 0, "WARN": 1, "INFO": 10},
                }
            },
        },
        {
            "id": "LOG-20260417-Dr-last",
            "event_type": "doctor.report",
            "timestamp": "2026-04-17T20:00:00Z",
            "summary": "doctor 1 ERROR / 3 WARN",
            "details": {
                "categories": {
                    "drift": {"ERROR": 1, "WARN": 2, "INFO": 1},
                    "schema_filename": {"ERROR": 0, "WARN": 1, "INFO": 10},
                }
            },
        },
    ]


@pytest.fixture()
def mock_session_events() -> list[dict]:
    return [
        {
            "id": "LOG-20260415-Ses1",
            "event_type": "session.started",
            "timestamp": "2026-04-15T09:00:00Z",
            "summary": "Session started",
        },
        {
            "id": "LOG-20260415-Han1",
            "event_type": "session.handover",
            "timestamp": "2026-04-15T18:00:00Z",
            "summary": "End of day handover",
        },
    ]


# ---------------------------------------------------------------------------
# Signal module: release_summary
# ---------------------------------------------------------------------------

class TestReleaseSummary:
    def _make_mcp(
        self,
        events: list[dict] | None = None,
        entities: list[dict] | None = None,
    ) -> dict:
        events = events or []
        entities = entities or []

        def query_events(event_type="", **kwargs):
            return [e for e in events if e.get("event_type") == event_type]

        def query_entities(kind="", state="", **kwargs):
            return [
                e for e in entities
                if e.get("metadata", {}).get("kind", "WorkItem") == kind
                or kind == "WorkItem"
            ]

        return {
            "query_events": query_events,
            "query_entities": query_entities,
        }

    def test_finds_release_event(
        self,
        repo_root: Path,
        mock_release_events: list[dict],
        mock_workitems_done: list[dict],
    ):
        mcp = self._make_mcp(
            events=mock_release_events,
            entities=mock_workitems_done,
        )
        ctx = {
            "repo_root": repo_root,
            "release": "v0.18.2",
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "verbose": False,
            "mcp": mcp,
        }
        result = rs_mod.collect(ctx)
        assert result["version"] == "v0.18.2"
        assert result["prior_release"] == "v0.18.1"

    def test_handles_no_release_event(self, repo_root: Path):
        ctx = {
            "repo_root": repo_root,
            "release": "v0.18.2",
            "since": None,
            "until": None,
            "verbose": False,
            "mcp": {},
        }
        result = rs_mod.collect(ctx)
        assert result["version"] == "v0.18.2"
        assert result["commit_count"] >= 0

    def test_returns_raw_block(
        self,
        repo_root: Path,
        mock_release_events: list[dict],
    ):
        mcp = self._make_mcp(events=mock_release_events)
        ctx = {
            "repo_root": repo_root,
            "release": "v0.18.2",
            "since": None,
            "until": None,
            "verbose": False,
            "mcp": mcp,
        }
        result = rs_mod.collect(ctx)
        assert "raw" in result
        assert isinstance(result["raw"], dict)


# ---------------------------------------------------------------------------
# Signal module: timeline
# ---------------------------------------------------------------------------

class TestTimeline:
    def test_builds_sessions_from_events(
        self,
        repo_root: Path,
        mock_session_events: list[dict],
    ):
        def query_events(event_type="", **kwargs):
            return [
                e for e in mock_session_events
                if e.get("event_type") == event_type
            ]

        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "verbose": False,
            "mcp": {"query_events": query_events},
        }
        result = tl_mod.collect(ctx)
        assert result["session_count"] >= 0
        assert "sessions" in result
        assert "milestones" in result
        assert "raw" in result

    def test_handles_empty_events(self, repo_root: Path):
        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "verbose": False,
            "mcp": {"query_events": lambda **kw: []},
        }
        result = tl_mod.collect(ctx)
        assert result["session_count"] == 0
        assert result["sessions"] == []

    def test_extracts_milestones(
        self,
        repo_root: Path,
        mock_release_events: list[dict],
        mock_session_events: list[dict],
    ):
        all_events = mock_release_events + mock_session_events

        def query_events(event_type="", **kwargs):
            return [e for e in all_events if e.get("event_type") == event_type]

        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "verbose": False,
            "mcp": {"query_events": query_events},
        }
        result = tl_mod.collect(ctx)
        assert len(result["milestones"]) == 2  # both release events


# ---------------------------------------------------------------------------
# Signal module: workitems
# ---------------------------------------------------------------------------

class TestWorkitems:
    def _make_mcp(
        self,
        items: list[dict] | None = None,
    ) -> dict:
        items = items or []

        def query_entities(kind="", state="", **kwargs):
            return [
                e for e in items
                if e.get("spec", {}).get("state") == state
            ]

        return {"query_entities": query_entities}

    def test_partitions_held_and_slipped(
        self,
        repo_root: Path,
        mock_workitems_done: list[dict],
        mock_workitems_deferred: list[dict],
    ):
        mcp = self._make_mcp(items=mock_workitems_done + mock_workitems_deferred)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        assert result["held_count"] == 2
        assert result["slipped_count"] == 1
        assert result["total_closed"] == 3
        assert result["bug_closed"] == 1  # only BACK-002 is type=bug

    def test_deferred_appears_in_slipped(
        self,
        repo_root: Path,
        mock_workitems_deferred: list[dict],
    ):
        mcp = self._make_mcp(items=mock_workitems_deferred)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        assert result["slipped_count"] == 1
        assert result["held_count"] == 0
        slipped_ids = [w["id"] for w in result["slipped"]]
        assert "BACK-003" in slipped_ids

    def test_superseded_appears_in_slipped(
        self,
        repo_root: Path,
        mock_workitems_superseded: list[dict],
    ):
        mcp = self._make_mcp(items=mock_workitems_superseded)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        assert result["slipped_count"] == 1
        slipped_ids = [w["id"] for w in result["slipped"]]
        assert "BACK-004" in slipped_ids

    def test_done_appears_in_held(
        self,
        repo_root: Path,
        mock_workitems_done: list[dict],
    ):
        mcp = self._make_mcp(items=mock_workitems_done)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        assert result["held_count"] == 2
        assert result["slipped_count"] == 0
        held_ids = [w["id"] for w in result["held"]]
        assert "BACK-001" in held_ids
        assert "BACK-002" in held_ids

    def test_cancelled_excluded_from_both_buckets(
        self,
        repo_root: Path,
        mock_workitems_cancelled: list[dict],
    ):
        """cancelled WorkItems must not appear in held OR slipped."""
        mcp = self._make_mcp(items=mock_workitems_cancelled)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        assert result["held_count"] == 0
        assert result["slipped_count"] == 0
        all_ids = (
            [w["id"] for w in result["held"]]
            + [w["id"] for w in result["slipped"]]
        )
        assert "BACK-005" not in all_ids

    def test_cancelled_not_flagged_as_slipped_regression(
        self,
        repo_root: Path,
        mock_workitems_cancelled: list[dict],
        mock_workitems_done: list[dict],
    ):
        """Regression: BraveReef false positive — /pk-retro v0.18.2 flagged SunnyDawn
        as slipped because it was cancelled, even though the cancellation note said
        the concern was resolved via per-skill settings.toml. See DEC-20260421_2025-KindFrog."""
        # Mix cancelled with done items: cancelled must stay out of slipped
        mcp = self._make_mcp(items=mock_workitems_done + mock_workitems_cancelled)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        slipped_ids = [w["id"] for w in result["slipped"]]
        assert "BACK-005" not in slipped_ids, (
            "BraveReef regression: cancelled item BACK-005 (SunnyDawn) "
            "must not appear in slipped"
        )
        # done items are unaffected
        assert result["held_count"] == 2

    def test_empty_returns_zeros(self, repo_root: Path):
        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "mcp": {"query_entities": lambda **kw: []},
        }
        result = wi_mod.collect(ctx)
        assert result["held_count"] == 0
        assert result["slipped_count"] == 0
        assert result["bug_closed"] == 0
        assert "raw" in result

    def test_held_items_have_ids(
        self,
        repo_root: Path,
        mock_workitems_done: list[dict],
    ):
        mcp = self._make_mcp(items=mock_workitems_done)
        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "mcp": mcp,
        }
        result = wi_mod.collect(ctx)
        for item in result["held"]:
            assert item["id"], "held item must have an ID"


# ---------------------------------------------------------------------------
# Regression: BraveReef false positive (standalone)
# ---------------------------------------------------------------------------

def test_cancelled_workitem_not_flagged_as_slipped():
    """Regression: BraveReef false positive — /pk-retro v0.18.2 flagged SunnyDawn
    as slipped because it was cancelled, even though the cancellation note said
    the concern was resolved via per-skill settings.toml. See DEC-20260421_2025-KindFrog."""
    import tempfile
    from pathlib import Path as _Path

    cancelled_item = {
        "metadata": {"id": "BACK-SunnyDawn"},
        "spec": {
            "title": "SunnyDawn — per-skill settings.toml",
            "state": "cancelled",
            "type": "chore",
            "completed_at": "",
            "deferred_at": "2026-04-17T12:00:00Z",
            "notes": "Concern resolved via per-skill settings.toml",
        },
    }

    def query_entities(kind="", state="", **kwargs):
        if state == "cancelled":
            return [cancelled_item]
        return []

    with tempfile.TemporaryDirectory() as tmp:
        repo_root = _Path(tmp)
        (repo_root / "context" / "workitems").mkdir(parents=True)
        ctx = {
            "repo_root": repo_root,
            "since": "2026-04-15T00:00:00Z",
            "until": "2026-04-18T23:59:59Z",
            "mcp": {"query_entities": query_entities},
        }
        result = wi_mod.collect(ctx)

    slipped_ids = [w["id"] for w in result["slipped"]]
    held_ids = [w["id"] for w in result["held"]]
    assert "BACK-SunnyDawn" not in slipped_ids, (
        "BraveReef regression: cancelled SunnyDawn must not appear in slipped"
    )
    assert "BACK-SunnyDawn" not in held_ids, (
        "BraveReef regression: cancelled SunnyDawn must not appear in held"
    )
    assert result["slipped_count"] == 0
    assert result["held_count"] == 0


# ---------------------------------------------------------------------------
# Signal module: drift
# ---------------------------------------------------------------------------

class TestDrift:
    def test_computes_error_delta(
        self,
        repo_root: Path,
        mock_doctor_reports: list[dict],
    ):
        def query_events(event_type="", **kwargs):
            if event_type == "doctor.report":
                return mock_doctor_reports
            return []

        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "mcp": {"query_events": query_events},
        }
        result = dr_mod.collect(ctx)
        # First: 0 ERROR, Last: 1 ERROR → delta = +1
        assert result["doctor_error_delta"] == 1
        assert result["doctor_warn_delta"] == 1  # 2→3 WARN

    def test_uncertain_when_no_entity(self, repo_root: Path):
        """Drift signals with no entity ID get certain=False."""
        def query_events(event_type="", **kwargs):
            if event_type == "doctor.report":
                return [
                    {
                        "id": "",
                        "event_type": "doctor.report",
                        "timestamp": "2026-04-15T10:00:00Z",
                        "details": {
                            "categories": {
                                "drift": {"ERROR": 0, "WARN": 0}
                            }
                        },
                    },
                    {
                        "id": "",
                        "event_type": "doctor.report",
                        "timestamp": "2026-04-17T10:00:00Z",
                        "details": {
                            "categories": {
                                "drift": {"ERROR": 2, "WARN": 0}
                            }
                        },
                    },
                ]
            return []

        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "mcp": {"query_events": query_events},
        }
        result = dr_mod.collect(ctx)
        # The signal should exist but entity=None → certain=False
        degradation_sigs = [
            s for s in result["slipped_signals"]
            if "ERROR count" in s.get("description", "")
        ]
        assert degradation_sigs, "Expected an ERROR degradation signal"
        sig = degradation_sigs[0]
        assert sig["certain"] is False, (
            "Signal with no entity ID must be uncertain"
        )

    def test_no_reports_returns_zero_deltas(self, repo_root: Path):
        ctx = {
            "repo_root": repo_root,
            "since": None,
            "until": None,
            "mcp": {"query_events": lambda **kw: []},
        }
        result = dr_mod.collect(ctx)
        assert result["doctor_error_delta"] == 0
        assert result["doctor_warn_delta"] == 0


# ---------------------------------------------------------------------------
# Aggregator: section order + line cap
# ---------------------------------------------------------------------------

class TestAggregator:
    def _build_signals(self) -> dict[str, dict]:
        return {
            "release_summary": {
                "version": "v0.18.2",
                "release_date": "2026-04-18",
                "commit_count": 42,
                "contributors": ["Alice", "Bob"],
                "top_deliverables": ["Feature A", "Fix B"],
                "prior_release": "v0.18.1",
                "prior_release_date": "2026-04-15",
                "raw": {},
            },
            "timeline": {
                "sessions": [
                    {"start": "2026-04-15", "end": "2026-04-18", "summary": ""},
                ],
                "milestones": [
                    {
                        "timestamp": "2026-04-18",
                        "description": "Release v0.18.2",
                        "entity": "LOG-release",
                    }
                ],
                "session_count": 1,
                "raw": {},
            },
            "workitems": {
                "held": [
                    {"id": "BACK-001", "title": "Feature A",
                     "state": "done", "completed_at": "2026-04-17",
                     "deferred_at": "", "reason": ""},
                ],
                "slipped": [
                    {"id": "BACK-003", "title": "Refactor auth",
                     "state": "deferred", "completed_at": "",
                     "deferred_at": "2026-04-17", "reason": "Scope too large"},
                ],
                "held_count": 1,
                "slipped_count": 1,
                "total_closed": 2,
                "bug_closed": 0,
                "raw": {},
            },
            "drift": {
                "doctor_error_delta": 0,
                "doctor_warn_delta": 0,
                "skip_marker_count": 0,
                "skip_marker_ids": [],
                "first_doctor": None,
                "last_doctor": None,
                "slipped_signals": [],
                "raw": {},
            },
        }

    def test_section_order(self):
        signals = self._build_signals()
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        sections = [
            "## Release Summary",
            "## Timeline",
            "## Signals",
            "## What Held",
            "## What Slipped",
            "## Action Items (proposed)",
        ]
        last_pos = -1
        for sec in sections:
            pos = body.find(sec)
            assert pos > last_pos, (
                f"Section '{sec}' is out of order or missing"
            )
            last_pos = pos

    def test_80_line_cap_enforced(self):
        """Non-verbose output must not exceed 80 lines."""
        signals = self._build_signals()
        # Inflate held items to push past budget
        signals["workitems"]["held"] = [
            {"id": f"BACK-{i:03d}", "title": f"Item {i}",
             "state": "done", "completed_at": "2026-04-17",
             "deferred_at": "", "reason": ""}
            for i in range(30)
        ]
        signals["workitems"]["held_count"] = 30
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        line_count = len(body.splitlines())
        assert line_count <= 80 + 2, (  # +2 for truncation notice lines
            f"Expected ≤80 lines, got {line_count}"
        )

    def test_verbose_bypasses_cap(self):
        """Verbose output is allowed to exceed 80 lines."""
        signals = self._build_signals()
        signals["workitems"]["held"] = [
            {"id": f"BACK-{i:03d}", "title": f"Item {i}",
             "state": "done", "completed_at": "2026-04-17",
             "deferred_at": "", "reason": ""}
            for i in range(30)
        ]
        signals["workitems"]["held_count"] = 30
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=True,
            auto_workitems=False,
            notes=None,
        )
        line_count = len(body.splitlines())
        assert line_count > 80, (
            "Verbose mode should allow output longer than 80 lines"
        )

    def test_learned_section_included_with_notes(self):
        signals = self._build_signals()
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes="Teams communicated better this cycle\nCI improved",
        )
        assert "## Learned" in body

    def test_learned_section_absent_without_notes(self):
        signals = self._build_signals()
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        assert "## Learned" not in body


# ---------------------------------------------------------------------------
# Hallucination guard
# ---------------------------------------------------------------------------

class TestHallucinationGuard:
    def test_uncertain_tag_for_no_entity_slip(self):
        """WorkItem slips without an ID get [uncertain] in What Slipped."""
        signals = {
            "release_summary": {
                "version": "v0.18.2", "release_date": "2026-04-18",
                "commit_count": 5, "contributors": [],
                "top_deliverables": [], "prior_release": None,
                "prior_release_date": None, "raw": {},
            },
            "timeline": {
                "sessions": [], "milestones": [],
                "session_count": 0, "raw": {},
            },
            "workitems": {
                "held": [],
                "slipped": [
                    {"id": "", "title": "Orphan item",
                     "state": "deferred", "completed_at": "",
                     "deferred_at": "2026-04-17", "reason": ""},
                ],
                "held_count": 0, "slipped_count": 1,
                "total_closed": 1, "bug_closed": 0, "raw": {},
            },
            "drift": {
                "doctor_error_delta": 0, "doctor_warn_delta": 0,
                "skip_marker_count": 0, "skip_marker_ids": [],
                "first_doctor": None, "last_doctor": None,
                "slipped_signals": [], "raw": {},
            },
        }
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        assert "[uncertain" in body, (
            "Item without entity ID must get [uncertain] tag"
        )

    def test_drift_uncertain_tag(self):
        """Drift signals with certain=False get [uncertain] prefix."""
        signals = {
            "release_summary": {
                "version": "v0.18.2", "release_date": "",
                "commit_count": 0, "contributors": [],
                "top_deliverables": [], "prior_release": None,
                "prior_release_date": None, "raw": {},
            },
            "timeline": {
                "sessions": [], "milestones": [],
                "session_count": 0, "raw": {},
            },
            "workitems": {
                "held": [], "slipped": [],
                "held_count": 0, "slipped_count": 0,
                "total_closed": 0, "bug_closed": 0, "raw": {},
            },
            "drift": {
                "doctor_error_delta": 2, "doctor_warn_delta": 0,
                "skip_marker_count": 0, "skip_marker_ids": [],
                "first_doctor": None, "last_doctor": None,
                "slipped_signals": [
                    {
                        "description": "ERROR count increased by 2",
                        "entity": None,
                        "certain": False,
                    }
                ],
                "raw": {},
            },
        }
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        assert "[uncertain" in body


# ---------------------------------------------------------------------------
# Dual-emit atomicity
# ---------------------------------------------------------------------------

class TestDualEmit:
    def _minimal_signals_ctx(self, repo_root: Path) -> dict:
        return {
            "repo_root": repo_root,
            "release": "v0.18.2",
            "since": None,
            "until": None,
            "verbose": False,
            "mcp": {},
        }

    def test_both_called_on_success(self, repo_root: Path):
        artifact_result = {
            "metadata": {"id": "ART-20260418-Test-retro-v0.18.2"},
            "id": "ART-20260418-Test-retro-v0.18.2",
        }
        create_artifact = MagicMock(return_value=artifact_result)
        log_event = MagicMock(return_value=None)

        mcp = {
            "create_artifact": create_artifact,
            "log_event": log_event,
            "query_events": lambda **kw: [],
            "query_entities": lambda **kw: [],
        }
        rc = retro.main(
            ["--release", "v0.18.2", "--repo-root", str(repo_root)],
            mcp_overrides=mcp,
        )
        assert rc == 0
        create_artifact.assert_called_once()
        log_event.assert_called()
        # First log_event call must be retro.completed
        first_call_kwargs = log_event.call_args_list[0][1]
        assert first_call_kwargs.get("event_type") == "retro.completed"

    def test_log_event_not_called_if_create_artifact_fails(
        self, repo_root: Path
    ):
        create_artifact = MagicMock(return_value=None)  # simulate failure
        log_event = MagicMock()

        mcp = {
            "create_artifact": create_artifact,
            "log_event": log_event,
            "query_events": lambda **kw: [],
            "query_entities": lambda **kw: [],
        }
        rc = retro.main(
            ["--release", "v0.18.2", "--repo-root", str(repo_root)],
            mcp_overrides=mcp,
        )
        assert rc == 1
        log_event.assert_not_called()

    def test_dry_run_skips_both_writes(
        self, repo_root: Path, capsys
    ):
        create_artifact = MagicMock()
        log_event = MagicMock()

        mcp = {
            "create_artifact": create_artifact,
            "log_event": log_event,
            "query_events": lambda **kw: [],
            "query_entities": lambda **kw: [],
        }
        rc = retro.main(
            [
                "--release", "v0.18.2",
                "--repo-root", str(repo_root),
                "--dry-run",
            ],
            mcp_overrides=mcp,
        )
        assert rc == 0
        create_artifact.assert_not_called()
        log_event.assert_not_called()
        # stdout should have the rendered artifact
        captured = capsys.readouterr()
        assert "# Retrospective" in captured.out


# ---------------------------------------------------------------------------
# --auto-workitems (Phase 2)
# ---------------------------------------------------------------------------

class TestAutoWorkitems:
    def test_create_workitem_called_per_action_item(self, repo_root: Path):
        artifact_result = {"metadata": {"id": "ART-test"}, "id": "ART-test"}
        wi_result = {"metadata": {"id": "BACK-new"}, "id": "BACK-new"}

        create_artifact = MagicMock(return_value=artifact_result)
        create_workitem = MagicMock(return_value=wi_result)
        log_event = MagicMock()

        # Provide a slipped WorkItem with an ID so action items get derived
        def query_entities(kind="", state="", **kw):
            if state == "deferred":
                return [{
                    "metadata": {"id": "BACK-003"},
                    "spec": {
                        "title": "Refactor auth",
                        "state": "deferred",
                        "type": "chore",
                        "completed_at": "",
                        "deferred_at": "2026-04-17T15:00:00Z",
                        "notes": "",
                    },
                }]
            return []

        mcp = {
            "create_artifact": create_artifact,
            "create_workitem": create_workitem,
            "log_event": log_event,
            "query_events": lambda **kw: [],
            "query_entities": query_entities,
        }
        rc = retro.main(
            [
                "--release", "v0.18.2",
                "--repo-root", str(repo_root),
                "--auto-workitems",
            ],
            mcp_overrides=mcp,
        )
        assert rc == 0
        create_workitem.assert_called()
        # Verify title is truncated to 80 chars
        call_kwargs = create_workitem.call_args[1]
        assert len(call_kwargs.get("title", "")) <= 80
        assert call_kwargs.get("type") == "chore"
        assert call_kwargs.get("priority") == "medium"
        assert call_kwargs.get("state") == "backlog"

    def test_create_workitem_not_called_without_flag(self, repo_root: Path):
        artifact_result = {"metadata": {"id": "ART-test"}, "id": "ART-test"}
        create_artifact = MagicMock(return_value=artifact_result)
        create_workitem = MagicMock()
        log_event = MagicMock()

        mcp = {
            "create_artifact": create_artifact,
            "create_workitem": create_workitem,
            "log_event": log_event,
            "query_events": lambda **kw: [],
            "query_entities": lambda **kw: [],
        }
        rc = retro.main(
            ["--release", "v0.18.2", "--repo-root", str(repo_root)],
            mcp_overrides=mcp,
        )
        assert rc == 0
        create_workitem.assert_not_called()

    def test_auto_workitems_hint_when_flag_off(self):
        """Artifact body includes hint when --auto-workitems is off."""
        signals = {
            "release_summary": {
                "version": "v0.18.2", "release_date": "2026-04-18",
                "commit_count": 0, "contributors": [],
                "top_deliverables": [], "prior_release": None,
                "prior_release_date": None, "raw": {},
            },
            "timeline": {
                "sessions": [], "milestones": [],
                "session_count": 0, "raw": {},
            },
            "workitems": {
                "held": [], "slipped": [],
                "held_count": 0, "slipped_count": 0,
                "total_closed": 0, "bug_closed": 0, "raw": {},
            },
            "drift": {
                "doctor_error_delta": 0, "doctor_warn_delta": 0,
                "skip_marker_count": 0, "skip_marker_ids": [],
                "first_doctor": None, "last_doctor": None,
                "slipped_signals": [], "raw": {},
            },
        }
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        assert "--auto-workitems" in body


# ---------------------------------------------------------------------------
# --verbose appendix
# ---------------------------------------------------------------------------

class TestVerboseMode:
    def test_appendix_a_present_in_verbose(self):
        signals = {
            "release_summary": {
                "version": "v0.18.2", "release_date": "2026-04-18",
                "commit_count": 5, "contributors": ["Alice"],
                "top_deliverables": [], "prior_release": None,
                "prior_release_date": None,
                "raw": {"git_stats": "sample"},
            },
            "timeline": {
                "sessions": [], "milestones": [],
                "session_count": 0,
                "raw": {"event_count": 0},
            },
            "workitems": {
                "held": [], "slipped": [],
                "held_count": 0, "slipped_count": 0,
                "total_closed": 0, "bug_closed": 0,
                "raw": {"held_ids": [], "slipped_ids": []},
            },
            "drift": {
                "doctor_error_delta": 0, "doctor_warn_delta": 0,
                "skip_marker_count": 0, "skip_marker_ids": [],
                "first_doctor": None, "last_doctor": None,
                "slipped_signals": [],
                "raw": {"doctor_report_count": 0},
            },
        }
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=True,
            auto_workitems=False,
            notes=None,
        )
        assert "Appendix A" in body
        assert "git_stats" in body

    def test_appendix_absent_in_normal_mode(self):
        signals = {
            k: {"raw": {}}
            for k in ["release_summary", "timeline", "workitems", "drift"]
        }
        signals["release_summary"].update({
            "version": "v0.18.2", "release_date": "", "commit_count": 0,
            "contributors": [], "top_deliverables": [],
            "prior_release": None, "prior_release_date": None,
        })
        signals["timeline"].update({
            "sessions": [], "milestones": [], "session_count": 0,
        })
        signals["workitems"].update({
            "held": [], "slipped": [], "held_count": 0, "slipped_count": 0,
            "total_closed": 0, "bug_closed": 0,
        })
        signals["drift"].update({
            "doctor_error_delta": 0, "doctor_warn_delta": 0,
            "skip_marker_count": 0, "skip_marker_ids": [],
            "first_doctor": None, "last_doctor": None, "slipped_signals": [],
        })
        body = retro._render_artifact(
            release="v0.18.2",
            signals=signals,
            verbose=False,
            auto_workitems=False,
            notes=None,
        )
        assert "Appendix A" not in body


# ---------------------------------------------------------------------------
# Production MCP loader
# ---------------------------------------------------------------------------

class TestProductionMcpLoader:
    def test_load_production_mcp_returns_callables(self):
        """_load_production_mcp() returns a dict with the 3 expected keys.

        Skipped when the ``mcp`` package is not installed (test env uses
        ``--with pytest --with pyyaml --with jsonschema`` only; add
        ``--with 'mcp[cli]>=1.0'`` to run this test in-process).
        """
        pytest.importorskip("mcp", reason="mcp package not installed")
        result = retro._load_production_mcp()
        assert set(result.keys()) == {
            "create_artifact", "log_event", "create_workitem"
        }
        for key, fn in result.items():
            assert callable(fn), f"{key} must be callable"

    def test_main_uses_production_loader_when_no_overrides(
        self, repo_root: Path
    ):
        """main() calls _load_production_mcp when mcp_overrides is None
        and neither --dry-run nor --no-mcp is set."""
        artifact_result = {
            "metadata": {"id": "ART-test-prod"},
            "id": "ART-test-prod",
        }
        mock_create_artifact = MagicMock(return_value=artifact_result)
        mock_log_event = MagicMock(return_value=None)
        mock_mcp = {
            "create_artifact": mock_create_artifact,
            "log_event": mock_log_event,
            "query_events": lambda **kw: [],
            "query_entities": lambda **kw: [],
        }

        with patch.object(retro, "_load_production_mcp", return_value=mock_mcp) \
                as mock_loader:
            rc = retro.main(
                ["--release", "v0.18.2", "--repo-root", str(repo_root)],
                mcp_overrides=None,
            )

        mock_loader.assert_called_once()
        assert rc == 0
        mock_create_artifact.assert_called_once()
        mock_log_event.assert_called()

    def test_main_fails_cleanly_when_loader_raises(
        self, repo_root: Path, capsys
    ):
        """main() returns 1 and prints a warning when the loader raises."""
        with patch.object(
            retro, "_load_production_mcp",
            side_effect=ImportError("test server missing"),
        ):
            rc = retro.main(
                ["--release", "v0.18.2", "--repo-root", str(repo_root)],
                mcp_overrides=None,
            )

        assert rc == 1
        captured = capsys.readouterr()
        assert "WARNING: in-process MCP loader failed" in captured.err
        assert "test server missing" in captured.err
        assert "--dry-run" in captured.err

    def test_no_mcp_flag_skips_loader(self, repo_root: Path):
        """--no-mcp forces empty mcp dict even when mcp_overrides is None."""
        with patch.object(retro, "_load_production_mcp") as mock_loader:
            # --no-mcp + no overrides → loader is never called;
            # mcp is empty, so create_artifact warning fires and returns 1.
            rc = retro.main(
                [
                    "--release", "v0.18.2",
                    "--repo-root", str(repo_root),
                    "--no-mcp",
                ],
                mcp_overrides=None,
            )

        mock_loader.assert_not_called()
        # With empty mcp and no dry-run, create_artifact fails → rc == 1
        assert rc == 1
