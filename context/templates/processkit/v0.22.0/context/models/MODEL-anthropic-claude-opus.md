---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-anthropic-claude-opus
  created: '2026-04-22T00:00:00Z'
spec:
  provider: anthropic
  family: claude-opus
  versions:
  - version_id: '4.6'
    status: ga
    context_window: 200000
    pricing_usd_per_1m:
      input: 15.0
      output: 75.0
    pricing_note: Most expensive Claude; reserve for tasks where quality justifies
      cost
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 5
    engineering: 5
    speed: 2
    breadth: 4
    reliability: 5
    governance: 5
  modalities:
  - text
  - vision
  - tools
  - computer-use
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.anthropic.com/
  rationale: Frontier flagship
---
