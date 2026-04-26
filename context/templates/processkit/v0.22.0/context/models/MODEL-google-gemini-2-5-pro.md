---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemini-2-5-pro
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemini-2-5-pro
  versions:
  - version_id: '2.5'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 1.25
      output: 10.0
    pricing_note: Context >200K billed at higher rates; 1M context tasks can be expensive
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
    speed: 3
    breadth: 5
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier flagship
---
