---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-openai-gpt-4o
  created: '2026-04-22T00:00:00Z'
spec:
  provider: openai
  family: gpt-4o
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 2.5
      output: 10.0
    pricing_note: Strong value for multimodal tasks; audio in/out included in standard
      pricing
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
    engineering: 4
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
  equivalent_tier: xl
  status_page_url: https://status.openai.com/
  rationale: Frontier multi-modal
---
