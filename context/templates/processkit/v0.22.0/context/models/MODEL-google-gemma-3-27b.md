---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemma-3-27b
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemma-3-27b
  versions:
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.15
      output: 0.45
    pricing_note: Open weights under Gemma Terms; self-hosting recommended for proprietary
      workloads
    governance_warning: 'Via Google API: G:2 (same as Gemini). Self-hosted open weights
      (Gemma Terms of Use): G:4 — data stays local, strong sovereignty, but no Anthropic-tier
      compliance certs.'
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 3
    engineering: 3
    speed: 4
    breadth: 4
    reliability: 3
    governance: 4
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cloud.google.com/
  rationale: Open-source frontier-class
---
