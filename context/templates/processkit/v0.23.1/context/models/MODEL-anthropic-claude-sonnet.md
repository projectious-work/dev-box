---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-anthropic-claude-sonnet
  created: '2026-04-22T00:00:00Z'
spec:
  provider: anthropic
  family: claude-sonnet
  versions:
  - version_id: '4.6'
    status: ga
    context_window: 200000
    pricing_usd_per_1m:
      input: 3.0
      output: 15.0
    pricing_note: Best price-capability ratio in the Claude family for engineering
      tasks
    vendor_model_id: claude-sonnet-4-6-20251015
    jurisdiction:
      vendor_hq_country: US
      applicable_legal_regimes:
      - EU-GDPR
      - US-CLOUD-Act
      - US-HIPAA
      data_residency_regions:
      - us
      - eu
    data_privacy:
      dpa_available: https://www.anthropic.com/legal/dpa
      data_retention_days: zero
      training_on_customer_data: never
      pii_eligible: true
      phi_hipaa_eligible: true
      gdpr_eligible: true
      sub_processors_url: https://www.anthropic.com/legal/subprocessors
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 5
    speed: 3
    breadth: 4
    reliability: 4
    governance: 5
  modalities:
  - text
  - vision
  - tools
  - computer-use
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.anthropic.com/
  rationale: Frontier mid-size
---
