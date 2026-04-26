---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-openai-gpt-5
  created: '2026-04-22T00:00:00Z'
spec:
  provider: openai
  family: gpt-5
  versions:
  - version_id: '5'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 5.0
      output: 20.0
    pricing_note: ESTIMATED — scores based on extrapolation from GPT-4o trajectory.
      Run Workflow C to validate.
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
  status_page_url: https://status.openai.com/
  rationale: Frontier flagship (estimated)
---
