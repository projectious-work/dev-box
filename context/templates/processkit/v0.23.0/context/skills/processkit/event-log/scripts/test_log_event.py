"""Tests for the log_event MCP tool's actor-required validation.

Run with:

    uv run --with pyyaml --with jsonschema --with pytest --with mcp \
        pytest context/skills/processkit/event-log/scripts/test_log_event.py -v

Covers BACK-20260425_1755-RapidDaisy: log_event must reject calls that
omit the schema-required `actor` field, with a clear error response and
no file written.
"""
from __future__ import annotations

import os
import shutil
import sys
import tempfile
from pathlib import Path

import pytest

# --- Path bootstrap so we can import the lib + the server module --------------
_REPO_ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(_REPO_ROOT / "context" / "skills" / "_lib"))

# Make the server module importable as plain `server` from its mcp/ dir.
_SERVER_DIR = _REPO_ROOT / "context" / "skills" / "processkit" / "event-log" / "mcp"
sys.path.insert(0, str(_SERVER_DIR))


@pytest.fixture
def project_root(tmp_path: Path, monkeypatch):
    """Create a minimal processkit project root + chdir into it."""
    (tmp_path / "aibox.toml").write_text("[processkit]\nversion = \"v0.0.0\"\n")
    (tmp_path / "context" / "logs").mkdir(parents=True)
    (tmp_path / "context" / "schemas").mkdir(parents=True)
    # Symlink the live LogEntry schema so validate_spec finds it.
    schema_src = _REPO_ROOT / "context" / "schemas" / "logentry.yaml"
    (tmp_path / "context" / "schemas" / "logentry.yaml").write_text(
        schema_src.read_text()
    )
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("PROCESSKIT_LIB_PATH", str(_REPO_ROOT / "context" / "skills" / "_lib"))
    yield tmp_path


def _import_server_fresh():
    """Re-import server after env/cwd setup so module-level state respects it."""
    if "server" in sys.modules:
        del sys.modules["server"]
    import server  # noqa: F401
    return sys.modules["server"]


def test_log_event_missing_actor_returns_error(project_root):
    server = _import_server_fresh()
    result = server.log_event(
        event_type="test.event",
        summary="missing actor should be rejected",
    )
    assert "error" in result, f"expected error response, got {result!r}"
    assert "actor" in result["error"].lower()
    # No file should have been written.
    log_files = list((project_root / "context" / "logs").rglob("*.md"))
    assert log_files == [], f"file written despite validation failure: {log_files}"


def test_log_event_with_actor_succeeds(project_root):
    server = _import_server_fresh()
    result = server.log_event(
        event_type="test.event",
        summary="actor present should succeed",
        actor="ACTOR-test",
    )
    assert "error" not in result, f"unexpected error: {result.get('error')}"
    assert "id" in result
    assert "path" in result
    assert Path(result["path"]).is_file()


def test_log_event_with_empty_actor_returns_error(project_root):
    """Empty-string actor is a common foot-gun — must be treated as missing."""
    server = _import_server_fresh()
    result = server.log_event(
        event_type="test.event",
        summary="empty actor should be rejected",
        actor="",
    )
    assert "error" in result
    assert "actor" in result["error"].lower()
