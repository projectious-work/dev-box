"""
test_hooks.py — smoke tests for emit_compliance_contract.py and
check_route_task_called.py.

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
check("stdout contains contract marker", "pk-compliance v1" in result.stdout)
check("stderr empty", result.stderr.strip() == "", result.stderr.strip())

# ---------------------------------------------------------------------------
# Test 2: emit_compliance_contract.py — Claude Code JSON envelope mode
# ---------------------------------------------------------------------------
print("\n[2] emit_compliance_contract.py — Claude Code JSON envelope")

result_cc = run(_EMIT_SCRIPT, env_override={"CLAUDE_CODE_ENTRYPOINT": "cli"})
check("exit 0", result_cc.returncode == 0)
try:
    payload = json.loads(result_cc.stdout)
    check("JSON envelope present", "hookSpecificOutput" in payload)
    check(
        "additionalContext contains contract",
        "pk-compliance v1" in payload.get("hookSpecificOutput", {}).get("additionalContext", ""),
    )
except json.JSONDecodeError as exc:
    check("JSON envelope present", False, str(exc))
    check("additionalContext contains contract", False)

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
result = run(_CHECK_SCRIPT, stdin_text=fixture_text)
check("exit 2 (block)", result.returncode == 2, f"got {result.returncode}")
check("remediation message on stderr", "acknowledge_contract" in result.stderr)

# ---------------------------------------------------------------------------
# Test 5: check_route_task_called.py — valid marker → exit 0
# ---------------------------------------------------------------------------
print("\n[5] check_route_task_called.py — Write on context/ with valid marker → exit 0")

# Build a valid marker.
contract_hash = hashlib.sha256(_CONTRACT.read_bytes()).hexdigest()
fixture_data = json.loads(fixture_text)
session_id = fixture_data["session_id"]

with tempfile.TemporaryDirectory() as tmp:
    # Create a fake project root with context/.state/skill-gate/
    project_root = Path(tmp)
    (project_root / "context").mkdir()
    state_dir = project_root / "context" / ".state" / "skill-gate"
    state_dir.mkdir(parents=True)
    marker_path = state_dir / f"session-{session_id}.ack"
    marker_path.write_text(
        json.dumps({"contract_hash": contract_hash, "acknowledged_at": "2026-04-14T14:31:00+00:00"}, indent=2)
        + "\n",
        encoding="utf-8",
    )

    # Point the fixture's cwd at the temp project root.
    fixture_with_tmp_cwd = dict(fixture_data)
    fixture_with_tmp_cwd["cwd"] = str(project_root)
    # Also fix the file_path to be relative so the context check matches.
    fixture_with_tmp_cwd["tool_input"] = {"file_path": "context/workitems/FEAT-fixture.md", "content": "x"}

    result = run(_CHECK_SCRIPT, stdin_text=json.dumps(fixture_with_tmp_cwd))
    check("exit 0 (pass)", result.returncode == 0, f"got {result.returncode}\n    stderr: {result.stderr.strip()}")

# ---------------------------------------------------------------------------
# Test 6: check_route_task_called.py — locked processkit tool, no marker → exit 2
# ---------------------------------------------------------------------------
print("\n[6] check_route_task_called.py — create_workitem (locked MCP tool), no marker → exit 2")

locked_mcp_input = json.dumps(
    {
        "hook_event_name": "PreToolUse",
        "session_id": "test-session-fixture-888",
        "cwd": "/workspace",
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
# Summary
# ---------------------------------------------------------------------------
print()
if failures:
    print(f"FAILED ({len(failures)} test(s)):", ", ".join(failures))
    sys.exit(1)
else:
    print(f"All tests passed.")
    sys.exit(0)
