---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-mistral-mistral-small
  created: '2026-04-22T00:00:00Z'
spec:
  provider: mistral
  family: mistral-small
  versions:
  - version_id: '3'
    status: ga
    context_window: 32000
    pricing_usd_per_1m:
      input: 0.1
      output: 0.3
    pricing_note: Mistral's entry-level model; very cheap, EU-based; good for classification
      and routing
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 2
    engineering: 3
    speed: 5
    breadth: 2
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: s
  status_page_url: https://status.mistral.ai/
  rationale: Small / fast
---
