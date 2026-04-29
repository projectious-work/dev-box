---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-openai-o3
  created: '2026-04-22T00:00:00Z'
spec:
  provider: openai
  family: o3
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 10.0
      output: 40.0
    pricing_note: Extended thinking output is billed at output rates; total cost per
      task can be very high
    vendor_model_id: o3-2025-04-16
    jurisdiction:
      vendor_hq_country: US
      applicable_legal_regimes:
      - EU-GDPR
      - US-CLOUD-Act
      - US-HIPAA
      data_residency_regions:
      - us
      - eu
      - ap
    data_privacy:
      dpa_available: https://openai.com/policies/data-processing-addendum
      data_retention_days: 30
      training_on_customer_data: opt-in
      pii_eligible: true
      phi_hipaa_eligible: true
      gdpr_eligible: true
      sub_processors_url: https://openai.com/policies/subprocessors
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
    breadth: 3
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.openai.com/
  rationale: Frontier reasoning flagship
---
