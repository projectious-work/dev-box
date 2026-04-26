---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-deepseek-deepseek-r
  created: '2026-04-22T00:00:00Z'
spec:
  provider: deepseek
  family: deepseek-r
  versions:
  - version_id: '1'
    status: ga
    context_window: 64000
    pricing_usd_per_1m:
      input: 0.55
      output: 2.19
    pricing_note: ~20x cheaper than o3 for comparable reasoning; G:1 severely limits
      safe use cases.
    governance_warning: G:1 — Chinese jurisdiction. Never use for PII, PHI, confidential
      IP, credentials, regulated data, or government/defense work.
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 5
    engineering: 4
    speed: 3
    breadth: 3
    reliability: 3
    governance: 1
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.deepseek.com/
  rationale: Frontier reasoning, low cost
---
