---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemini-3-1-pro
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemini-3-1-pro
  versions:
  - version_id: '3.1'
    status: preview
    context_window: 2000000
    pricing_usd_per_1m:
      input: 2.0
      output: 8.0
    pricing_note: ESTIMATED — next-gen Google Pro. Run Workflow C to validate.
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
    reliability: 5
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier flagship (estimated)
---
