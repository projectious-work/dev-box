---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-mistral-codestral
  created: '2026-04-22T00:00:00Z'
spec:
  provider: mistral
  family: codestral
  versions:
  - version_id: '1'
    status: ga
    context_window: 256000
    pricing_usd_per_1m:
      input: 0.3
      output: 0.9
    pricing_note: Mistral's code-focused model; weights available for self-hosting;
      strong fill-in-the-middle
  efforts_supported:
  - none
  - low
  - medium
  dimensions:
    reasoning: 3
    engineering: 5
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.mistral.ai/
  rationale: Coding specialist
---
