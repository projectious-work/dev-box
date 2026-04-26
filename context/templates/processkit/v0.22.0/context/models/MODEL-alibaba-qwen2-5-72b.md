---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-alibaba-qwen2-5-72b
  created: '2026-04-22T00:00:00Z'
spec:
  provider: alibaba
  family: qwen2-5-72b
  versions:
  - version_id: '2.5'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 0.4
      output: 1.2
    pricing_note: 'Self-hosted: GPU infra cost only. Via Together/Groq/etc.: ~$0.20–0.90/1M.
      Alibaba Cloud API available but G:1.'
    governance_warning: 'Via Alibaba Cloud API: G:1 (Chinese jurisdiction). Self-hosted
      (Apache 2.0 license): governance rises to G:4 — no data leaves your infra, but
      no compliance certs from Alibaba.'
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  dimensions:
    reasoning: 4
    engineering: 4
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
  rationale: Open-source frontier
---
