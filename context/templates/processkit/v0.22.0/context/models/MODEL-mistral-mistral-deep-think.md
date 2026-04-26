---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-mistral-mistral-deep-think
  created: '2026-04-22T00:00:00Z'
spec:
  provider: mistral
  family: mistral-deep-think
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 1.5
      output: 6.0
    pricing_note: Mistral's reasoning-mode model; EU-based alternative to o3/DeepSeek
      R1 for regulated data
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 2
    breadth: 3
    reliability: 4
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.mistral.ai/
  rationale: Reasoning / extended thinking
---
