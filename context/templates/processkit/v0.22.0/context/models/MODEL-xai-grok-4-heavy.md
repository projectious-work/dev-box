---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-xai-grok-4-heavy
  created: '2026-04-22T00:00:00Z'
spec:
  provider: xai
  family: grok-4-heavy
  versions:
  - version_id: '4'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 10.0
      output: 40.0
    pricing_note: ESTIMATED — compute-heavy reasoning variant analogous to o3. Run
      Workflow C to validate.
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
    breadth: 4
    reliability: 5
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.x.ai/
  rationale: Frontier reasoning flagship (estimated)
---
