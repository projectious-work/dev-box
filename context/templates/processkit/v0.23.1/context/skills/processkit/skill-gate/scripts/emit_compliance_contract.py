"""
emit_compliance_contract.py — processkit SessionStart / UserPromptSubmit hook.

WorkItem: FEAT-20260414_1433-SteadyHand-provider-neutral-hook-scripts
Hooks guide: https://code.claude.com/docs/en/hooks-guide

Purpose
-------
Emit the canonical compliance contract (assets/compliance-contract.md)
to stdout so it lands in the current turn's context.  Wire this script
as a SessionStart AND UserPromptSubmit hook in your harness config.
aibox handles the wiring; see scripts/README.md for the target shape.

Claude Code detection
---------------------
Claude Code 2.1.0+ prefers a JSON envelope on stdout:
    {"hookSpecificOutput": {
         "hookEventName": "<SessionStart|UserPromptSubmit>",
         "additionalContext": "<text>"
    }}
We detect Claude Code by checking for any of these env vars (set by the
Claude Code runtime before invoking hook scripts):
    CLAUDE_CODE_ENTRYPOINT  — always present in Claude Code 2.x
    CLAUDE_CODE_VERSION     — present in Claude Code 2.1+
    ANTHROPIC_CLAUDE_CODE   — sometimes set by wrapper scripts

The hookEventName is read from the hook input JSON on stdin (every
Claude Code hook invocation includes it). If stdin is missing or
malformed, we fall back to "UserPromptSubmit" — the safer default since
SessionStart runs once per session while UserPromptSubmit runs on every
turn, so a mislabeled SessionStart only loses one initial emission.

Limitation: this heuristic can produce a false positive if a non-Claude
harness sets one of those vars, and a false negative on a very old Claude
Code or a customised wrapper that strips env vars.  The plain-stdout
fallback is safe on both Claude Code (legacy) and Codex CLI.

Exit codes
----------
0  contract emitted successfully
1  contract file missing (error on stderr, nothing useful on stdout)
"""

import hashlib
import json
import os
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Locate the contract relative to this script, not cwd.
# ---------------------------------------------------------------------------
_ASSETS_DIR = Path(__file__).parent.parent / "assets"
_CONTRACT_PATH = _ASSETS_DIR / "compliance-contract.md"

_CLAUDE_CODE_ENV_INDICATORS = (
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_VERSION",
    "ANTHROPIC_CLAUDE_CODE",
)


def _is_claude_code() -> bool:
    """Return True when we believe we're running inside Claude Code 2.1+."""
    return any(os.environ.get(var) for var in _CLAUDE_CODE_ENV_INDICATORS)


def _hook_event_name() -> str:
    """
    Return the hook event name from stdin input, defaulting to
    UserPromptSubmit if stdin is empty or unparseable.

    Claude Code sends a JSON object on stdin for every hook invocation.
    Relevant field: `hook_event_name` (snake_case on input; the response
    envelope uses camelCase `hookEventName`).
    """
    # Peek at stdin without consuming it permanently — SessionStart /
    # UserPromptSubmit scripts typically don't read stdin, but we do
    # need a line to extract the event name.
    try:
        if sys.stdin.isatty():
            return "UserPromptSubmit"
        raw = sys.stdin.read()
    except Exception:
        return "UserPromptSubmit"
    if not raw.strip():
        return "UserPromptSubmit"
    try:
        data = json.loads(raw)
    except json.JSONDecodeError:
        return "UserPromptSubmit"
    evt = data.get("hook_event_name")
    if isinstance(evt, str) and evt:
        return evt
    return "UserPromptSubmit"


def main() -> int:
    if not _CONTRACT_PATH.exists():
        print(
            f"ERROR: compliance-contract.md not found at {_CONTRACT_PATH}; "
            "processkit installation may be incomplete.",
            file=sys.stderr,
        )
        return 1

    contract_text = _CONTRACT_PATH.read_text(encoding="utf-8")

    if _is_claude_code():
        # Claude Code 2.1.0+ preferred form: JSON envelope.
        # hookEventName is REQUIRED — Claude Code rejects the envelope
        # with "hookSpecificOutput is missing required field hookEventName"
        # if omitted.
        output = json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": _hook_event_name(),
                    "additionalContext": contract_text,
                }
            },
            ensure_ascii=False,
        )
        print(output)
    else:
        # Plain stdout — works for Claude Code (legacy) and Codex CLI.
        print(contract_text, end="")

    return 0


if __name__ == "__main__":
    sys.exit(main())
