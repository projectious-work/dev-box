---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-mistral-mistral-medium
  created: '2026-04-22T00:00:00Z'
spec:
  provider: mistral
  family: mistral-medium
  versions:
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.4
      output: 2.0
    pricing_note: Between Small and Large; good balance of speed and capability
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
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.mistral.ai/
  rationale: Mid-tier
---
