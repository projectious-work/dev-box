---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260424_0114-JollyStream-phase-1-core-mcp
  created: '2026-04-24T01:14:24+00:00'
  updated: '2026-04-24T01:23:30+00:00'
spec:
  title: 'Phase 1: Core MCP infrastructure (pattern matching, McpConfig struct)'
  state: done
  type: task
  priority: high
  description: '**Scope:** Add pattern matching infrastructure to support glob-based
    MCP permission configuration.


    **Deliverables:**

    1. Add `McpConfig` struct in `cli/src/mcp_registration.rs` with fields: `default_mode`,
    `allow_patterns`, `deny_patterns`, `per_harness_overrides`

    2. Implement `expand_mcp_patterns(patterns: Vec<String>) -> Vec<String>` using
    glob matching

    3. Implement `first_match_wins(allow_patterns, deny_patterns, tool_name) -> bool`
    predicate

    4. Add unit tests for pattern expansion (e.g., `mcp__processkit-*` expands to
    all ~26 processkit tools)


    **Input:** aibox.toml `[mcp]` section parsed via `toml::from_str`

    **Output:** `Vec<String>` of concrete, allowed tool names


    **Estimated Tokens:** ~3K (struct definition, glob logic, test cases)

    **Blocker for:** Phase 2 (all harness generators depend on this)'
  parent: BACK-20260424_0058-ToughGrove-feature-global-mcp-permissions
  started_at: '2026-04-24T01:23:15+00:00'
  completed_at: '2026-04-24T01:23:30+00:00'
---

## Transition note (2026-04-24T01:23:15+00:00)

Core infrastructure implemented: McpConfig struct, expand_mcp_patterns(), first_match_wins(), glob_matches(). 8 comprehensive unit tests. All tests pass, clippy clean.


## Transition note (2026-04-24T01:23:21+00:00)

Core infrastructure complete. Ready for Phase 2 parallel execution.


## Transition note (2026-04-24T01:23:30+00:00)

Phase 1 implementation complete and tested. Phase 2 can proceed.
