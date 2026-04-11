---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-LivelyBison-per-harness-mcp-server
  created: '2026-04-10T22:33:22+00:00'
spec:
  title: Per-harness MCP server registration — aibox writes provider config files
  state: accepted
  decision: 'v0.16.5 ships `cli/src/mcp_registration.rs` which walks the templates
    mirror at `context/templates/processkit/<version>/skills/*/mcp/mcp-config.json`
    and writes per-harness MCP server registration files keyed on `[ai].providers`.
    Writers: Claude/Mistral → `.mcp.json`; Cursor → `.cursor/mcp.json`; Gemini → `.gemini/settings.json`;
    Codex → `.codex/config.toml`; Continue → `.continue/mcpServers/processkit-<name>.json`
    (one file per server); Aider → no file written, sync emits a warning. Merge is
    non-destructive: the managed set is the JSON keys from per-skill `mcp-config.json`
    (prefixed `processkit-`); user-added entries survive every sync. Called from `cmd_init`
    and `cmd_sync` after `install_content_source`.'
  context: Every MCP-capable harness reads exactly one config file at session start.
    Different harnesses use different paths and formats (JSON vs TOML vs per-file
    directory). The per-skill `mcp-config.json` files use the Claude shape. aibox
    must translate these into each harness's native format at install/sync time so
    MCP servers are available without manual configuration.
  rationale: Translating the canonical per-skill MCP config into each harness format
    is aibox's job — it knows both the project's installed skills and the user's configured
    AI providers. Non-destructive merge keyed on JSON key (not directory name) prevents
    clobbering user-added servers. The `processkit-` prefix on all server names makes
    collisions structurally impossible. Best-effort (warn-and-continue on failure)
    prevents MCP registration glitches from breaking the rest of init/sync.
  alternatives:
  - option: Sidecar file at .aibox/mcp-managed-keys.json to track managed set
    rejected_because: Drifts out of sync; the templates mirror is already the per-version
      manifest
  - option: Comment-marker delimiters in harness file
    rejected_because: JSON does not support comments natively; requires JSONC for
      some harnesses
  - option: Always overwrite harness file unconditionally
    rejected_because: Clobbers user-added MCP servers outside the processkit managed
      set
  - option: Ship an aibox meta-MCP server that aggregates processkit servers
    rejected_because: Does not reduce the registration problem; loses per-skill encapsulation;
      new codebase to maintain
  consequences: 'New `AiProvider` variants: Cursor, Codex, Continue, Copilot. Sync
    perimeter extended to cover `.mcp.json`, `.cursor/`, `.gemini/`, `.codex/`, `.continue/`.
    [skills] filtering applies to MCP registration too — a filtered-out skill also
    has no registered server. Aider users get a warning on every sync until Aider
    ships native MCP support.'
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:33:22+00:00'
---
