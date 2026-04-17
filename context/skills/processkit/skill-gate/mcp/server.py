#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
# ]
# ///
"""processkit skill-gate MCP server.

Tools:

    acknowledge_contract(version: str)
        -> {ok, contract_hash, expires_at, contract}

    check_contract_acknowledged()
        -> {acknowledged, session_id, age_seconds, contract_hash}

These tools implement Rail 3 of the processkit enforcement hybrid (see
ART-20260414_1430-SteadyBeacon): a provider-neutral MCP-based compliance
acknowledgement that works on any harness speaking MCP, without requiring
SessionStart / UserPromptSubmit hook support.

Session marker files
--------------------
Written to: context/.state/skill-gate/session-<SESSION_ID>.ack

SESSION_ID is resolved in this order:
  1. Environment variable PROCESSKIT_SESSION_ID (allows harness/test injection).
  2. os.getpid() — the PID of the uv-spawned server process, which is stable
     for the lifetime of the MCP server process (one process = one session on
     MCP stdio transport, since each harness session spawns a fresh server).

Marker file format (JSON):
  {
    "contract_hash": "<sha256 hex>",
    "acknowledged_at": "<ISO-8601 UTC>"
  }

The marker directory (context/.state/skill-gate/) is created on first write.
SteadyHand's check_route_task_called.py reads this directory to confirm
that a compliance acknowledgement exists for the current session.
"""
from __future__ import annotations

import hashlib
import json
import os
import sys
from datetime import datetime, timezone, timedelta
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

server = FastMCP("processkit-skill-gate")

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_COMPLIANCE_VERSION = "v1"
_CONTRACT_MARKER = f"<!-- pk-compliance {_COMPLIANCE_VERSION} -->"
_SESSION_ACK_SUBDIR = Path("context") / ".state" / "skill-gate"
_ACK_LIFETIME_HOURS = 12


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _contract_path() -> Path:
    """Absolute path to the canonical compliance-contract.md file."""
    return Path(__file__).resolve().parent.parent / "assets" / "compliance-contract.md"


def _session_id() -> str:
    """Return a stable session identifier for this server process."""
    return os.environ.get("PROCESSKIT_SESSION_ID") or str(os.getpid())


def _marker_path() -> Path:
    """Absolute path for the session acknowledgement marker file."""
    project_root = paths.find_project_root()
    return project_root / _SESSION_ACK_SUBDIR / f"session-{_session_id()}.ack"


def _hash_contract(content: str) -> str:
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


def _read_marker() -> dict | None:
    """Return parsed marker dict, or None if not present / unreadable."""
    p = _marker_path()
    if not p.is_file():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


def _write_marker(contract_hash: str, acknowledged_at: datetime) -> None:
    p = _marker_path()
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(
        json.dumps(
            {
                "contract_hash": contract_hash,
                "acknowledged_at": acknowledged_at.isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------


@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def acknowledge_contract(version: str) -> dict:
    """Acknowledge the processkit compliance contract for this session.

    Call once per session before any write-side processkit tool
    (create_*, transition_*, record_*, link_*, open_*). This is the
    1% rule checkpoint: if there is even a 1% chance a processkit skill
    covers your task, call this first, then route_task.

    On success, writes a session marker at
    context/.state/skill-gate/session-<SESSION_ID>.ack and returns the
    full contract text so you see it even if you skipped the description.

    Parameters
    ----------
    version :
        The contract version string you are acknowledging, e.g. "v1".
        Must match the on-disk version or the call returns ok=False.

    Returns
    -------
    ok              — True on successful acknowledgement
    contract_hash   — sha256 of the contract file content
    expires_at      — ISO-8601 UTC timestamp 12 h after acknowledgement
    contract        — full contract text (re-read even if ok=False)
    error           — present only when ok=False; explains the mismatch
    """
    try:
        contract_file = _contract_path()
        contract_text = contract_file.read_text(encoding="utf-8")
    except FileNotFoundError:
        return {
            "ok": False,
            "error": (
                f"compliance-contract.md not found at {_contract_path()}; "
                "processkit installation may be incomplete"
            ),
            "contract": "",
        }

    # Parse on-disk version from leading marker comment
    first_line = contract_text.splitlines()[0].strip() if contract_text.strip() else ""
    # Expected: <!-- pk-compliance v1 -->
    on_disk_version = ""
    if first_line.startswith("<!-- pk-compliance ") and first_line.endswith(" -->"):
        on_disk_version = first_line[len("<!-- pk-compliance "):-len(" -->")].strip()

    if version != on_disk_version:
        return {
            "ok": False,
            "error": (
                f"version mismatch: caller supplied {version!r} but "
                f"on-disk contract is {on_disk_version!r}. "
                f"Use version={on_disk_version!r} to acknowledge."
            ),
            "contract": contract_text,
        }

    contract_hash = _hash_contract(contract_text)
    now = datetime.now(timezone.utc)
    expires_at = now + timedelta(hours=_ACK_LIFETIME_HOURS)

    _write_marker(contract_hash, now)

    return {
        "ok": True,
        "contract_hash": contract_hash,
        "expires_at": expires_at.isoformat(),
        "contract": contract_text,
    }


@server.tool(
    annotations=ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    )
)
def check_contract_acknowledged() -> dict:
    """Check whether the compliance contract has been acknowledged this session.

    Read-only. Returns the acknowledgement status for the current session.
    Use this to guard write-side tools — enforce the 1% rule by confirming
    acknowledge_contract() was called before proceeding with any
    create_*, transition_*, or record_* operation.

    Returns
    -------
    acknowledged    — True if a valid session marker exists and its
                      contract_hash matches the current contract file
    session_id      — the session identifier used for the marker file
    age_seconds     — seconds since acknowledgement (None if not acknowledged)
    contract_hash   — hash from the marker (None if not acknowledged)
    """
    session_id = _session_id()
    marker = _read_marker()

    if marker is None:
        return {
            "acknowledged": False,
            "session_id": session_id,
            "age_seconds": None,
            "contract_hash": None,
        }

    # Verify the stored hash still matches the current contract file
    try:
        contract_text = _contract_path().read_text(encoding="utf-8")
        current_hash = _hash_contract(contract_text)
    except FileNotFoundError:
        return {
            "acknowledged": False,
            "session_id": session_id,
            "age_seconds": None,
            "contract_hash": None,
        }

    stored_hash = marker.get("contract_hash", "")
    if stored_hash != current_hash:
        return {
            "acknowledged": False,
            "session_id": session_id,
            "age_seconds": None,
            "contract_hash": stored_hash,
        }

    # Compute age
    age_seconds: int | None = None
    acked_at_str = marker.get("acknowledged_at")
    if acked_at_str:
        try:
            acked_at = datetime.fromisoformat(acked_at_str)
            now = datetime.now(timezone.utc)
            age_seconds = int((now - acked_at).total_seconds())
        except Exception:
            pass

    return {
        "acknowledged": True,
        "session_id": session_id,
        "age_seconds": age_seconds,
        "contract_hash": stored_hash,
    }


if __name__ == "__main__":
    server.run(transport="stdio")
