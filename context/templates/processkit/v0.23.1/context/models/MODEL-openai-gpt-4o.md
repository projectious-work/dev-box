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
    vendor_model_id: gpt-4o-2024-08-06
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
