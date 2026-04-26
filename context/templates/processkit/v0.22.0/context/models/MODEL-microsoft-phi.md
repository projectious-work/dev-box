---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-microsoft-phi
  created: '2026-04-22T00:00:00Z'
spec:
  provider: microsoft
  family: phi
  versions:
  - version_id: '4'
    status: ga
    context_window: 16000
    pricing_usd_per_1m:
      input: 0.07
      output: 0.14
    pricing_note: MIT license; cheapest STEM-capable model; via Azure AI Foundry with
      enterprise DPA available
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 4
    engineering: 3
    speed: 5
    breadth: 2
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://azure.status.microsoft/
  rationale: Small / efficient
---
