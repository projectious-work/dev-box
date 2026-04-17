# processkit skill-gate hook scripts

WorkItem: `FEAT-20260414_1433-SteadyHand-provider-neutral-hook-scripts`

These scripts are shipped by processkit and wired into harness configs by
`aibox sync` (aibox issue 6.2).  They are **stdlib-only Python 3.10+**
with no third-party dependencies.

---

## Scripts

### `emit_compliance_contract.py`

**Event:** `SessionStart` and `UserPromptSubmit`

Reads `assets/compliance-contract.md` relative to its own location and
emits it to stdout, injecting it into the turn's context.

**Claude Code 2.1.0+ (preferred):** if any of `CLAUDE_CODE_ENTRYPOINT`,
`CLAUDE_CODE_VERSION`, or `ANTHROPIC_CLAUDE_CODE` is set in the
environment, emits JSON:

```json
{"hookSpecificOutput": {"additionalContext": "<contract text>"}}
```

**Fallback:** plain text on stdout (works for Claude Code legacy and Codex CLI).

**Exit codes:** `0` success, `1` contract file missing (stderr explains).

---

### `check_route_task_called.py`

**Event:** `PreToolUse`

Gates write-side processkit tools and generic file writes under
`context/` until `acknowledge_contract(version='v1')` has been called.

**Locked tool names** (always gated regardless of path):

```
create_workitem  transition_workitem  record_decision  link_entities
open_discussion  create_artifact      log_event        create_note
```

**Generic file tools** (`Write`, `Edit`, `MultiEdit`, `Bash`) are gated
only when the target path resolves under `context/`.

**Session-ID precedence:**
1. `session_id` field in hook input JSON
2. `PROCESSKIT_SESSION_ID` env var
3. `str(os.getpid())`

**Exit codes:** `0` pass, `2` block (stderr message shown to user).

---

## Wiring targets for `aibox sync`

### Claude Code — `.claude/settings.json`

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "python3 context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "python3 context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit|create_workitem|transition_workitem|record_decision|link_entities|open_discussion|create_artifact|log_event|create_note",
        "hooks": [
          {
            "type": "command",
            "command": "python3 context/skills/processkit/skill-gate/scripts/check_route_task_called.py"
          }
        ]
      }
    ]
  }
}
```

### Codex CLI — `.codex/hooks.json`

```json
{
  "hooks": {
    "session_start": {
      "command": "python3 context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py"
    },
    "user_prompt_submit": {
      "command": "python3 context/skills/processkit/skill-gate/scripts/emit_compliance_contract.py"
    }
  }
}
```

> **Known limitation (Codex):** as of 2026-04-14, Codex CLI `PreToolUse`
> only intercepts `Bash` calls (upstream openai/codex#16732).
> `check_route_task_called.py` is NOT wired for `PreToolUse` on Codex.
> `aibox sync` should log this limitation for Codex-only projects.
> The script remains deployable without change once Codex lifts the
> restriction.

---

## Running tests

```sh
python3 context/skills/processkit/skill-gate/scripts/test_hooks.py
```

All tests are stdlib-only and self-contained.
