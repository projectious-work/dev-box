"""
test_hooks.py — smoke tests for emit_compliance_contract.py,
check_route_task_called.py, and the skill-gate MCP server tools.

WorkItem: FEAT-20260414_1433-SteadyHand-provider-neutral-hook-scripts

Run from any directory:
    python3 context/skills/processkit/skill-gate/scripts/test_hooks.py

Or from inside the scripts/ directory:
    python3 test_hooks.py

Exit 0 = all tests passed.
Exit 1 = at least one test failed.
"""

import hashlib
import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

# ---------------------------------------------------------------------------
# Locate files relative to this script, not cwd.
# ---------------------------------------------------------------------------
_SCRIPTS_DIR = Path(__file__).parent
_FIXTURES_DIR = _SCRIPTS_DIR / "fixtures"
_EMIT_SCRIPT = _SCRIPTS_DIR / "emit_compliance_contract.py"
_CHECK_SCRIPT = _SCRIPTS_DIR / "check_route_task_called.py"
_CONTRACT = _SCRIPTS_DIR.parent / "assets" / "compliance-contract.md"

_PRETOOLUSE_FIXTURE = _FIXTURES_DIR / "claude-code-pretooluse-sample.json"

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"

failures: list[str] = []


def run(script: Path, stdin_text: str = "", env_override: dict | None = None) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    # Strip Claude Code env vars first so default mode is plain-stdout;
    # env_override wins if a test wants to exercise Claude Code JSON mode.
    for var in ("CLAUDE_CODE_ENTRYPOINT", "CLAUDE_CODE_VERSION", "ANTHROPIC_CLAUDE_CODE"):
        env.pop(var, None)
    if env_override:
        env.update(env_override)
    return subprocess.run(
        [sys.executable, str(script)],
        input=stdin_text,
        capture_output=True,
        text=True,
        env=env,
    )


def check(name: str, condition: bool, detail: str = "") -> None:
    if condition:
        print(f"  {PASS}  {name}")
    else:
        print(f"  {FAIL}  {name}" + (f": {detail}" if detail else ""))
        failures.append(name)


# ---------------------------------------------------------------------------
# Test 1: emit_compliance_contract.py — plain stdout mode
# ---------------------------------------------------------------------------
print("\n[1] emit_compliance_contract.py — plain stdout")

result = run(_EMIT_SCRIPT)
check("exit 0", result.returncode == 0, f"got {result.returncode}")
check("stdout contains contract marker", "pk-compliance v2" in result.stdout)
check("stderr empty", result.stderr.strip() == "", result.stderr.strip())

# ---------------------------------------------------------------------------
# Test 2: emit_compliance_contract.py — Claude Code JSON envelope mode
# ---------------------------------------------------------------------------
print("\n[2] emit_compliance_contract.py — Claude Code JSON envelope")

result_cc = run(
    _EMIT_SCRIPT,
    stdin_text=json.dumps({"hook_event_name": "UserPromptSubmit"}),
    env_override={"CLAUDE_CODE_ENTRYPOINT": "cli"},
)
check("exit 0", result_cc.returncode == 0)
try:
    payload = json.loads(result_cc.stdout)
    hso = payload.get("hookSpecificOutput", {})
    check("JSON envelope present", "hookSpecificOutput" in payload)
    check(
        "hookEventName present (required by Claude Code 2.1+)",
        hso.get("hookEventName") == "UserPromptSubmit",
        f"got {hso.get('hookEventName')!r}",
    )
    check(
        "additionalContext contains contract",
        "pk-compliance v2" in hso.get("additionalContext", ""),
    )
except json.JSONDecodeError as exc:
    check("JSON envelope present", False, str(exc))
    check("hookEventName present", False)
    check("additionalContext contains contract", False)

# ---------------------------------------------------------------------------
# Test 2b: SessionStart event name is echoed back
# ---------------------------------------------------------------------------
print("\n[2b] emit_compliance_contract.py — SessionStart event name is echoed back")

result_ss = run(
    _EMIT_SCRIPT,
    stdin_text=json.dumps({"hook_event_name": "SessionStart"}),
    env_override={"CLAUDE_CODE_ENTRYPOINT": "cli"},
)
try:
    payload = json.loads(result_ss.stdout)
    hso = payload.get("hookSpecificOutput", {})
    check(
        "hookEventName echoes SessionStart",
        hso.get("hookEventName") == "SessionStart",
        f"got {hso.get('hookEventName')!r}",
    )
except json.JSONDecodeError as exc:
    check("hookEventName echoes SessionStart", False, str(exc))

# ---------------------------------------------------------------------------
# Test 3: check_route_task_called.py — non-locked tool passes
# ---------------------------------------------------------------------------
print("\n[3] check_route_task_called.py — non-locked tool (Read) passes without marker")

non_locked_input = json.dumps(
    {
        "hook_event_name": "PreToolUse",
        "session_id": "test-session-fixture-999",
        "cwd": "/workspace",
        "tool_name": "Read",
        "tool_input": {"file_path": "context/workitems/foo.md"},
    }
)
result = run(_CHECK_SCRIPT, stdin_text=non_locked_input)
check("exit 0 (pass)", result.returncode == 0, f"got {result.returncode}")
check("stderr empty", result.stderr.strip() == "", result.stderr.strip())

# ---------------------------------------------------------------------------
# Test 4: check_route_task_called.py — fixture, no marker → exit 2
# ---------------------------------------------------------------------------
print("\n[4] check_route_task_called.py — fixture Write on context/ with no marker → exit 2")

fixture_text = _PRETOOLUSE_FIXTURE.read_text(encoding="utf-8")
fixture_data = json.loads(fixture_text)

# Redirect the fixture at an isolated tmp project root so real markers
# under /workspace/context/.state/ don't satisfy the "no marker" premise.
with tempfile.TemporaryDirectory() as _tmp4:
    _root4 = Path(_tmp4)
    (_root4 / "context").mkdir()
    _input4 = dict(fixture_data)
    _input4["cwd"] = str(_root4)
    _input4["tool_input"] = {"file_path": "context/workitems/FEAT-fixture.md", "content": "x"}
    result = run(_CHECK_SCRIPT, stdin_text=json.dumps(_input4))
    check("exit 2 (block)", result.returncode == 2, f"got {result.returncode}")
    check("remediation message on stderr", "acknowledge_contract" in result.stderr)
    # Remediation must reference the *current* on-disk contract version,
    # not a hard-coded literal. Parse it from the contract marker and
    # assert it appears in the remediation text. (SwiftLynx)
    _first_line = _CONTRACT.read_text(encoding="utf-8").splitlines()[0]
    _on_disk_v = _first_line[len("<!-- pk-compliance "):-len(" -->")].strip()
    check(
        f"remediation uses on-disk contract version {_on_disk_v!r}",
        f"version='{_on_disk_v}'" in result.stderr,
        f"stderr did not contain version='{_on_disk_v}':\n    {result.stderr.strip()}",
    )

# ---------------------------------------------------------------------------
# Test 5: check_route_task_called.py — valid marker → exit 0
# ---------------------------------------------------------------------------
print("\n[5] check_route_task_called.py — Write on context/ with valid marker → exit 0")

# Build a valid marker with a fresh timestamp (within TTL).
contract_hash = hashlib.sha256(_CONTRACT.read_bytes()).hexdigest()

with tempfile.TemporaryDirectory() as tmp:
    project_root = Path(tmp)
    (project_root / "context").mkdir()
    state_dir = project_root / "context" / ".state" / "skill-gate"
    state_dir.mkdir(parents=True)
    # Marker name no longer needs to match any session id — hook scans
    # for any valid marker. Use a stable literal for test readability.
    marker_path = state_dir / "session-test-fixture.ack"
    marker_path.write_text(
        json.dumps(
            {
                "contract_hash": contract_hash,
                "acknowledged_at": datetime.now(timezone.utc).isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    fixture_with_tmp_cwd = dict(fixture_data)
    fixture_with_tmp_cwd["cwd"] = str(project_root)
    fixture_with_tmp_cwd["tool_input"] = {"file_path": "context/workitems/FEAT-fixture.md", "content": "x"}

    result = run(_CHECK_SCRIPT, stdin_text=json.dumps(fixture_with_tmp_cwd))
    check("exit 0 (pass)", result.returncode == 0, f"got {result.returncode}\n    stderr: {result.stderr.strip()}")

# ---------------------------------------------------------------------------
# Test 5a: auto-renew — a valid-but-aging marker has its acknowledged_at
# bumped forward on a successful gate check (SwiftLynx)
# ---------------------------------------------------------------------------
print("\n[5a] check_route_task_called.py — valid marker is auto-renewed on pass")

with tempfile.TemporaryDirectory() as tmp:
    project_root = Path(tmp)
    (project_root / "context").mkdir()
    state_dir = project_root / "context" / ".state" / "skill-gate"
    state_dir.mkdir(parents=True)
    # 6h-old marker (within the 12h TTL, but clearly not "now").
    _old = datetime.now(timezone.utc) - timedelta(hours=6)
    marker_path = state_dir / "session-aging.ack"
    marker_path.write_text(
        json.dumps(
            {
                "contract_hash": contract_hash,
                "acknowledged_at": _old.isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    _input = dict(fixture_data)
    _input["cwd"] = str(project_root)
    _input["tool_input"] = {"file_path": "context/workitems/FEAT-fixture.md", "content": "x"}
    result = run(_CHECK_SCRIPT, stdin_text=json.dumps(_input))
    check("exit 0 (pass)", result.returncode == 0, f"got {result.returncode}")
    _after = json.loads(marker_path.read_text(encoding="utf-8"))
    _renewed = datetime.fromisoformat(_after["acknowledged_at"])
    if _renewed.tzinfo is None:
        _renewed = _renewed.replace(tzinfo=timezone.utc)
    check(
        "acknowledged_at auto-renewed to ~now (> original)",
        _renewed > _old,
        f"renewed={_renewed.isoformat()} old={_old.isoformat()}",
    )
    check(
        "contract_hash preserved across renew",
        _after.get("contract_hash") == contract_hash,
    )

# ---------------------------------------------------------------------------
# Test 5b: stale marker (outside TTL) → exit 2
# ---------------------------------------------------------------------------
print("\n[5b] check_route_task_called.py — stale marker (> 12h old) → exit 2")

with tempfile.TemporaryDirectory() as tmp:
    project_root = Path(tmp)
    (project_root / "context").mkdir()
    state_dir = project_root / "context" / ".state" / "skill-gate"
    state_dir.mkdir(parents=True)
    (state_dir / "session-stale.ack").write_text(
        json.dumps(
            {
                "contract_hash": contract_hash,
                "acknowledged_at": "2026-04-14T14:31:00+00:00",  # days old
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    _input = dict(fixture_data)
    _input["cwd"] = str(project_root)
    _input["tool_input"] = {"file_path": "context/workitems/FEAT-fixture.md", "content": "x"}
    result = run(_CHECK_SCRIPT, stdin_text=json.dumps(_input))
    check("exit 2 (block, TTL expired)", result.returncode == 2, f"got {result.returncode}")

# ---------------------------------------------------------------------------
# Test 5c: marker with stale contract_hash → exit 2
# ---------------------------------------------------------------------------
print("\n[5c] check_route_task_called.py — marker with mismatched contract_hash → exit 2")

with tempfile.TemporaryDirectory() as tmp:
    project_root = Path(tmp)
    (project_root / "context").mkdir()
    state_dir = project_root / "context" / ".state" / "skill-gate"
    state_dir.mkdir(parents=True)
    (state_dir / "session-bad-hash.ack").write_text(
        json.dumps(
            {
                "contract_hash": "0" * 64,
                "acknowledged_at": datetime.now(timezone.utc).isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    _input = dict(fixture_data)
    _input["cwd"] = str(project_root)
    _input["tool_input"] = {"file_path": "context/workitems/FEAT-fixture.md", "content": "x"}
    result = run(_CHECK_SCRIPT, stdin_text=json.dumps(_input))
    check("exit 2 (block, hash mismatch)", result.returncode == 2, f"got {result.returncode}")

# ---------------------------------------------------------------------------
# Test 6: check_route_task_called.py — locked processkit tool, no marker → exit 2
# ---------------------------------------------------------------------------
print("\n[6] check_route_task_called.py — create_workitem (locked MCP tool), no marker → exit 2")

with tempfile.TemporaryDirectory() as _tmp6:
    _root6 = Path(_tmp6)
    (_root6 / "context").mkdir()
    locked_mcp_input = json.dumps(
        {
            "hook_event_name": "PreToolUse",
            "session_id": "test-session-fixture-888",
            "cwd": str(_root6),
            "tool_name": "create_workitem",
            "tool_input": {"title": "New item", "type": "task"},
        }
    )
    result = run(_CHECK_SCRIPT, stdin_text=locked_mcp_input)
    check("exit 2 (block)", result.returncode == 2, f"got {result.returncode}")
    check("remediation on stderr", "acknowledge_contract" in result.stderr)

# ---------------------------------------------------------------------------
# Test 7: check_route_task_called.py — malformed JSON input → exit 0 (safe)
# ---------------------------------------------------------------------------
print("\n[7] check_route_task_called.py — malformed JSON → exit 0 (safe, non-blocking)")

result = run(_CHECK_SCRIPT, stdin_text="not valid json")
check("exit 0 (safe pass on bad input)", result.returncode == 0, f"got {result.returncode}")

# ---------------------------------------------------------------------------
# Tests 8–10: skip_decision_record marker semantics.
#
# The MCP server (server.py) uses `uv run` inline-script deps and cannot
# be imported directly in a plain Python subprocess.  Instead we test the
# marker file contract — identical to how tests 5/5b/5c test .ack markers —
# by writing .decision-skip files directly and asserting that
# check_decision_captured.py honours them.
#
# The MCP tool skip_decision_record() is integration-tested by the existing
# smoke-test-servers.py (scripts/smoke-test-servers.py) which runs the
# server via `uv run` and exercises all three tools over MCP.
# ---------------------------------------------------------------------------

_CHECK_DEC = _SCRIPTS_DIR / "check_decision_captured.py"

from datetime import timedelta as _td  # noqa: E402  (local to test section)


def _run_check_dec(
    input_json: str, block_mode: bool = False
) -> subprocess.CompletedProcess:
    """Run check_decision_captured.py with PYTHONPATH so it can import
    decision_markers from the same scripts/ directory."""
    cmd = [sys.executable, str(_CHECK_DEC)]
    if block_mode:
        cmd.append("--mode=block")
    env = os.environ.copy()
    for var in ("CLAUDE_CODE_ENTRYPOINT", "CLAUDE_CODE_VERSION", "ANTHROPIC_CLAUDE_CODE"):
        env.pop(var, None)
    env["PYTHONPATH"] = str(_SCRIPTS_DIR)
    return subprocess.run(cmd, input=input_json, capture_output=True, text=True, env=env)


# ---------------------------------------------------------------------------
# Test 8: valid .decision-skip marker written with expected fields
# ---------------------------------------------------------------------------
print("\n[8] skip_decision_record marker — field layout is correct")

with tempfile.TemporaryDirectory() as _tmp8:
    _root8 = Path(_tmp8)
    (_root8 / "context").mkdir()
    _state8 = _root8 / "context" / ".state" / "skill-gate"
    _state8.mkdir(parents=True)

    _now8 = datetime.now(timezone.utc)
    _marker8 = _state8 / "session-test8.decision-skip"
    _marker8.write_text(
        json.dumps(
            {
                "reason": "User said ok to dismiss a suggestion",
                "contract_hash": hashlib.sha256(_CONTRACT.read_bytes()).hexdigest(),
                "acknowledged_at": _now8.isoformat(),
                "expires_at": (_now8 + _td(hours=24)).isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    _mdata8 = json.loads(_marker8.read_text())
    check("marker file written", _marker8.is_file())
    check("marker has reason field", "reason" in _mdata8)
    check("marker has contract_hash", "contract_hash" in _mdata8)
    check("marker has acknowledged_at", "acknowledged_at" in _mdata8)
    check("marker has expires_at", "expires_at" in _mdata8)
    check(
        "marker reason is non-empty",
        bool(_mdata8.get("reason", "").strip()),
    )

# ---------------------------------------------------------------------------
# Test 9: empty-reason guard — skip marker without reason is rejected
#         (simulated: the gate ignores a marker with empty reason field)
# ---------------------------------------------------------------------------
print("\n[9] skip_decision_record — empty-reason marker not present (gate blocks)")

with tempfile.TemporaryDirectory() as _tmp9:
    _root9 = Path(_tmp9)
    (_root9 / "context").mkdir()
    _state9 = _root9 / "context" / ".state" / "skill-gate"
    _state9.mkdir(parents=True)

    # Deliberately do NOT write any skip marker, simulating what the MCP tool
    # does when reason="" (it rejects and writes nothing).
    _transcript9 = _root9 / "transcript.jsonl"
    _transcript9.write_text(
        json.dumps({"type": "user", "message": {"role": "user", "content": "approved"}})
        + "\n",
        encoding="utf-8",
    )
    _input9 = json.dumps(
        {
            "hook_event_name": "PreToolUse",
            "session_id": "test9",
            "cwd": str(_root9),
            "tool_name": "create_workitem",
            "tool_input": {"title": "x"},
            "transcript_path": str(_transcript9),
        }
    )
    _r9 = _run_check_dec(_input9, block_mode=True)
    check(
        "exit 2 (no skip marker, gate blocks in block mode)",
        _r9.returncode == 2,
        f"got {_r9.returncode}; stderr={_r9.stderr.strip()}",
    )

# ---------------------------------------------------------------------------
# Test 10: check_decision_captured — valid skip marker → exit 0 in block mode
# ---------------------------------------------------------------------------
print("\n[10] check_decision_captured — valid skip marker → exit 0 in block mode")

with tempfile.TemporaryDirectory() as _tmp10:
    _root10 = Path(_tmp10)
    (_root10 / "context").mkdir()
    _state10 = _root10 / "context" / ".state" / "skill-gate"
    _state10.mkdir(parents=True)

    _now10 = datetime.now(timezone.utc)
    (_state10 / "session-test10.decision-skip").write_text(
        json.dumps(
            {
                "reason": "User withdrew the proposal",
                "contract_hash": hashlib.sha256(_CONTRACT.read_bytes()).hexdigest(),
                "acknowledged_at": _now10.isoformat(),
                "expires_at": (_now10 + _td(hours=24)).isoformat(),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    # Transcript with Tier-A word — the gate would normally fire, but the
    # skip marker should suppress it.
    _transcript10 = _root10 / "transcript.jsonl"
    _transcript10.write_text(
        json.dumps({"type": "user", "message": {"role": "user", "content": "approved"}})
        + "\n",
        encoding="utf-8",
    )
    _input10 = json.dumps(
        {
            "hook_event_name": "PreToolUse",
            "session_id": "test10",
            "cwd": str(_root10),
            "tool_name": "create_workitem",
            "tool_input": {"title": "x"},
            "transcript_path": str(_transcript10),
        }
    )
    _r10 = _run_check_dec(_input10, block_mode=True)
    check(
        "exit 0 with skip marker in block mode",
        _r10.returncode == 0,
        f"got {_r10.returncode}; stderr={_r10.stderr.strip()}",
    )

# ---------------------------------------------------------------------------
# Test 11: VastLark — prefix-based locking covers tools that aren't on
#          the explicit allowlist (the auto-renew gap).
# ---------------------------------------------------------------------------
print("\n[11] check_route_task_called.py — VastLark prefix matcher catches create_binding et al.")

with tempfile.TemporaryDirectory() as _tmp_vl:
    _root_vl = Path(_tmp_vl)
    (_root_vl / "context").mkdir()
    # No marker present — every locked tool must be blocked.
    _vl_locked_cases = [
        ("create_binding", "the original VastLark repro"),
        ("create_actor", "actor-profile write"),
        ("create_team_member", "team-manager write"),
        ("create_scope", "scope-management write"),
        ("create_gate", "gate-management write"),
        ("create_role", "role-management write"),
        ("create_artifact", "artifact-management write"),
        ("transition_decision", "decision-record write"),
        ("transition_discussion", "discussion-management write"),
        ("link_workitems", "workitem-management linker"),
        ("link_decision_to_workitem", "decision-record linker"),
        ("supersede_decision", "decision-record supersede"),
        ("update_actor", "actor-profile update"),
        ("update_team_member", "team-manager update"),
        ("end_binding", "binding-management end"),
        ("deactivate_actor", "actor-profile deactivate"),
        ("reactivate_team_member", "team-manager reactivate"),
        ("reserve_name", "team-manager reservation"),
        ("release_name", "team-manager release"),
        ("import_team_member", "team-manager import"),
        ("apply_migration", "migration-management apply"),
        ("reject_migration", "migration-management reject"),
        ("start_migration", "migration-management start"),
        ("add_outcome", "discussion-management outcome"),
        ("evaluate_gate", "gate-management evaluation"),
    ]
    for _t, _desc in _vl_locked_cases:
        _r = run(
            _CHECK_SCRIPT,
            stdin_text=json.dumps({
                "hook_event_name": "PreToolUse",
                "tool_name": _t,
                "tool_input": {},
                "cwd": str(_root_vl),
            }),
        )
        check(
            f"{_t} blocked without marker ({_desc})",
            _r.returncode == 2,
            f"got exit {_r.returncode}, stderr={_r.stderr.strip()[:80]}",
        )

    # Read-side / unrelated tools must NOT be blocked.
    _vl_passthrough_cases = [
        ("get_entity", "index read"),
        ("get_workitem", "workitem read"),
        ("query_workitems", "workitem list"),
        ("query_events", "event-log read"),
        ("list_skills", "skill-finder read"),
        ("find_skill", "skill-finder lookup"),
        ("route_task", "task-router (read-only routing)"),
        ("acknowledge_contract", "the gate call itself"),
        ("check_contract_acknowledged", "gate status read"),
        ("recent_events", "event-log recent read"),
        ("search_entities", "index search"),
        ("resolve_model", "model-recommender read"),
        ("compare_models", "model-recommender compare"),
        ("explain_routing", "model-recommender explain"),
        ("Read", "harness read tool"),
    ]
    for _t, _desc in _vl_passthrough_cases:
        _r = run(
            _CHECK_SCRIPT,
            stdin_text=json.dumps({
                "hook_event_name": "PreToolUse",
                "tool_name": _t,
                "tool_input": {},
                "cwd": str(_root_vl),
            }),
        )
        check(
            f"{_t} passes without marker ({_desc})",
            _r.returncode == 0,
            f"got exit {_r.returncode}, stderr={_r.stderr.strip()[:80]}",
        )

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print()
if failures:
    print(f"FAILED ({len(failures)} test(s)):", ", ".join(failures))
    sys.exit(1)
else:
    print(f"All tests passed.")
    sys.exit(0)
