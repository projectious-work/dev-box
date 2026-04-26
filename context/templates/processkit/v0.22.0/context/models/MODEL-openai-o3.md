---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-openai-o3
  created: '2026-04-22T00:00:00Z'
spec:
  provider: openai
  family: o3
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 10.0
      output: 40.0
    pricing_note: Extended thinking output is billed at output rates; total cost per
      task can be very high
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
    speed: 1
    breadth: 3
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.openai.com/
  rationale: Frontier reasoning flagship
---
