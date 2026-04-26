---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemini-3-flash
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemini-3-flash
  versions:
  - version_id: '3'
    status: preview
    context_window: 2000000
    pricing_usd_per_1m:
      input: 0.05
      output: 0.2
    pricing_note: ESTIMATED — next-gen Google Flash. Run Workflow C to validate.
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 4
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
  equivalent_tier: l
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier fast (estimated)
---
