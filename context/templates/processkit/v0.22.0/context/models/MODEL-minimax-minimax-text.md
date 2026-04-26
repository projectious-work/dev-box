---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-minimax-minimax-text
  created: '2026-04-22T00:00:00Z'
spec:
  provider: minimax
  family: minimax-text
  versions:
  - version_id: '01'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 0.2
      output: 1.1
    pricing_note: 1M context at very low cost; open weights available for self-hosting
    governance_warning: 'Chinese company. Via MiniMax API: G:1 (same caution as DeepSeek).
      Self-hosted open weights raise governance to G:3 but no compliance certs available.'
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 3
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://www.minimax.io/
  rationale: Long-context specialist
---
