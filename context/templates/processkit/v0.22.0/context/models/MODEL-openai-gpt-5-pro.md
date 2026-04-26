---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-openai-gpt-5-pro
  created: '2026-04-22T00:00:00Z'
spec:
  provider: openai
  family: gpt-5-pro
  versions:
  - version_id: '5.4'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 15.0
      output: 60.0
    pricing_note: ESTIMATED — premium/slower variant analogous to Opus 4.6. Run Workflow
      C to validate.
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
    speed: 2
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
  status_page_url: https://status.openai.com/
  rationale: Frontier flagship premium (estimated)
---
