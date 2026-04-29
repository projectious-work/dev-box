---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-mistral-mistral-large
  created: '2026-04-22T00:00:00Z'
spec:
  provider: mistral
  family: mistral-large
  versions:
  - version_id: '3'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 2.0
      output: 6.0
    pricing_note: EU-based pricing in EUR also available; Le Chat Enterprise has volume
      discounts
    vendor_model_id: mistral-large-2411
    jurisdiction:
      vendor_hq_country: FR
      applicable_legal_regimes:
      - EU-GDPR
      - FR-LIL
      - US-CLOUD-Act
      data_residency_regions:
      - eu
      - us
    data_privacy:
      dpa_available: https://mistral.ai/terms/dpa
      data_retention_days: zero
      training_on_customer_data: never
      pii_eligible: true
      phi_hipaa_eligible: false
      gdpr_eligible: true
      sub_processors_url: https://mistral.ai/terms/sub-processors
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 3
    engineering: 4
    speed: 4
    breadth: 3
    reliability: 3
    governance: 4
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://status.mistral.ai/
  rationale: Mid-tier frontier
---
