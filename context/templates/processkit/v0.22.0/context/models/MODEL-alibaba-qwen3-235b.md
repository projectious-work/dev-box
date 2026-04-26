---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-alibaba-qwen3-235b
  created: '2026-04-22T00:00:00Z'
spec:
  provider: alibaba
  family: qwen3-235b
  versions:
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_note: 'Massive MoE (235B params, 22B active); open weights; GPU cluster
      required for self-hosting. Via third-party APIs: pricing varies (~$0.50-2.00/1M).'
    governance_warning: 'Via Alibaba Cloud API: G:1. Self-hosted (Apache 2.0): G:4.'
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 5
    engineering: 5
    speed: 3
    breadth: 3
    reliability: 4
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.aliyun.com/
  rationale: Open-source frontier flagship
---
