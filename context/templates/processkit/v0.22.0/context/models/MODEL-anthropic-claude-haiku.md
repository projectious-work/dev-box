---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-anthropic-claude-haiku
  created: '2026-04-22T00:00:00Z'
spec:
  provider: anthropic
  family: claude-haiku
  versions:
  - version_id: '4.5'
    status: ga
    context_window: 200000
    pricing_usd_per_1m:
      input: 0.25
      output: 1.25
    pricing_note: Cheapest Claude; excellent value for high-volume, low-complexity
      tasks
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 5
    breadth: 3
    reliability: 4
    governance: 5
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.anthropic.com/
  rationale: Frontier fast
---
