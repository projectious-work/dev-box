---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemini-flash
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemini-flash
  versions:
  - version_id: '2.0'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 0.075
      output: 0.3
    pricing_note: Extremely low cost; 1M context sweep costs cents; best throughput/dollar
      of any listed model
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 5
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier fast
---
