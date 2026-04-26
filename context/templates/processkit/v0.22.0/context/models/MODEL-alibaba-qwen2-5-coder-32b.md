---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-alibaba-qwen2-5-coder-32b
  created: '2026-04-22T00:00:00Z'
spec:
  provider: alibaba
  family: qwen2-5-coder-32b
  versions:
  - version_id: '2.5'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.2
      output: 0.6
    pricing_note: Smaller than 72B so cheaper to host; very cost-effective coding
      model
    governance_warning: 'Via Alibaba Cloud API: G:1. Self-hosted (Apache 2.0): G:4.
      Recommended deployment is self-hosted for any proprietary code.'
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
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
  status_page_url: https://status.aliyun.com/
  rationale: Open-source coding specialist
---
