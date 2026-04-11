---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-SoftOwl-history-v0-16-5
  created: '2026-04-10T22:35:57+00:00'
spec:
  title: 'History: v0.16.5 bundle — MCP registration, render-mirror diff, skills filter,
    python/uv base (DEC-033 – DEC-036 summary)'
  state: accepted
  decision: 'DEC-033 (per-harness MCP registration), DEC-034 (render templated files
    into templates mirror), DEC-035 ([skills] filtering activation), and DEC-036 (python3
    + uv unconditionally in base) are each recorded individually as load-bearing decisions.
    This summary record captures the v0.16.5 bundle context: all four changes were
    designed together in a single session (2026-04-08) as interlocking pieces of the
    "processkit MCP servers just work" story. The `aibox kit` subcommand (BACK-123)
    — `aibox kit list`, `aibox kit skill list/ls/info/install/uninstall`, `aibox kit
    process list/ls/info` — was also shipped in this era, adding CLI-level visibility
    into the installed processkit content. Additionally: Copilot, Codex, Continue
    AI provider addons were added (BACK-064); `ai-mistral` kept as SDK addon with
    clarifying comment (BACK-065); `"latest"` sentinel supported as a tool version
    override (BACK-076); `aibox addon list` improved with category grouping and descriptions
    (BACK-077).'
  context: v0.16.5 was the last release before the aibox reset to processkit v0.8.0.
    The four interlocking changes resolved the main friction points discovered when
    running processkit MCP servers in real aibox projects.
  rationale: Recorded for historical completeness during the processkit v0.8.0 reset
    migration.
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:35:57+00:00'
---
