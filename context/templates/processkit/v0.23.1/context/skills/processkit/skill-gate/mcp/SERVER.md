# processkit-skill-gate MCP server

## Tools

### `acknowledge_contract(version: str) -> dict`

Acknowledge the processkit compliance contract for the current session.
Call once per session before any write-side processkit tool
(`create_*`, `transition_*`, `record_*`, `link_*`, `open_*`).

**Parameters**

| Name | Type | Description |
|---|---|---|
| `version` | string | Contract version string to acknowledge, e.g. `"v1"`. Must match the on-disk contract version. |

**Returns — success (`ok=True`)**

```json
{
  "ok": true,
  "contract_hash": "a3f2...c9d1",
  "expires_at": "2026-04-15T02:31:00+00:00",
  "contract": "<!-- pk-compliance v1 -->\n# processkit Compliance Contract\n..."
}
```

**Returns — version mismatch (`ok=False`)**

```json
{
  "ok": false,
  "error": "version mismatch: caller supplied 'v999' but on-disk contract is 'v1'. Use version='v1' to acknowledge.",
  "contract": "<!-- pk-compliance v1 -->\n# processkit Compliance Contract\n..."
}
```

**Returns — contract file missing**

```json
{
  "ok": false,
  "error": "compliance-contract.md not found at .../assets/compliance-contract.md; processkit installation may be incomplete",
  "contract": ""
}
```

| Field | Type | Notes |
|---|---|---|
| `ok` | bool | `true` on successful acknowledgement |
| `contract_hash` | string | sha256 hex of the contract file content; present only when `ok=true` |
| `expires_at` | string | ISO-8601 UTC timestamp 12 hours after acknowledgement; present only when `ok=true` |
| `contract` | string | Full contract text (returned even when `ok=false` so the agent can read it) |
| `error` | string | Explanation; present only when `ok=false` |

**Example call**

```json
{
  "tool": "acknowledge_contract",
  "arguments": { "version": "v1" }
}
```

**Example return**

```json
{
  "ok": true,
  "contract_hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576cddcff3b9a1b7879",
  "expires_at": "2026-04-15T14:31:00+00:00",
  "contract": "<!-- pk-compliance v1 -->\n# processkit Compliance Contract\n..."
}
```

---

### `check_contract_acknowledged() -> dict`

Check whether the compliance contract has been acknowledged in the
current session. Read-only; safe to call at any time.

**Parameters**

None.

**Returns — acknowledged**

```json
{
  "acknowledged": true,
  "session_id": "42187",
  "age_seconds": 37,
  "contract_hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576cddcff3b9a1b7879"
}
```

**Returns — not acknowledged**

```json
{
  "acknowledged": false,
  "session_id": "42187",
  "age_seconds": null,
  "contract_hash": null
}
```

| Field | Type | Notes |
|---|---|---|
| `acknowledged` | bool | `true` if a valid session marker exists and its hash matches the current contract |
| `session_id` | string | Session identifier used to locate the marker file |
| `age_seconds` | int \| null | Seconds since acknowledgement; `null` if not acknowledged |
| `contract_hash` | string \| null | Hash from the marker; `null` if not acknowledged (or `string` if marker exists but hash is stale) |

**Example call**

```json
{
  "tool": "check_contract_acknowledged",
  "arguments": {}
}
```

**Example return**

```json
{
  "acknowledged": true,
  "session_id": "42187",
  "age_seconds": 120,
  "contract_hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576cddcff3b9a1b7879"
}
```

---

## Session marker file

### Path

```
<project_root>/context/.state/skill-gate/session-<SESSION_ID>.ack
```

`SESSION_ID` is resolved in this order:

1. Environment variable `PROCESSKIT_SESSION_ID` — allows harness or test
   injection of a stable, human-readable session identifier.
2. `os.getpid()` — the PID of the `uv`-spawned server process.

The marker directory (`context/.state/skill-gate/`) is created on first
write with `mkdir(parents=True, exist_ok=True)`.

> **Validity rule (revised 2026-04-17).** The PreToolUse hook no longer
> matches markers by SESSION_ID. The MCP protocol does not propagate
> the harness session UUID to tool calls, so the hook (which reads the
> harness session_id from stdin) and the MCP server (which uses its own
> pid) wrote disjoint identifiers, making acknowledge_contract()
> unsatisfiable in practice. The hook now scans the marker directory
> and accepts any marker whose `contract_hash` matches the current
> contract file and whose `acknowledged_at` is within 12 h. SESSION_ID
> remains in the filename purely for human auditing and cleanup.

### Format

```json
{
  "contract_hash": "<sha256 hex of compliance-contract.md>",
  "acknowledged_at": "2026-04-14T14:31:00+00:00"
}
```

The file is UTF-8, pretty-printed (2-space indent), with a trailing
newline.

### Contract that consumers may rely on

`SteadyHand`'s `check_route_task_called.py` and any other consumer that
inspects the marker directory may rely on the following invariants:

- If the file is present and parseable JSON, the fields `contract_hash`
  and `acknowledged_at` are always present as strings.
- `acknowledged_at` is always a valid ISO-8601 datetime string (may be
  timezone-aware or naive UTC; use `datetime.fromisoformat()` to parse).
- The file is not written (and the directory is not created) when
  `acknowledge_contract` returns `ok=false`.
- `check_contract_acknowledged` re-validates the stored hash against the
  current contract file on every call; a stale hash (contract updated
  after acknowledgement) returns `acknowledged=false`.
- Marker files are never deleted by the server itself. The CI harness or
  test fixtures are responsible for cleanup.
