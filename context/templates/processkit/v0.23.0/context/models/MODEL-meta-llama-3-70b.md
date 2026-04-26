---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-meta-llama-3-70b
  created: '2026-04-22T00:00:00Z'
spec:
  provider: meta
  family: llama-3-70b
  versions:
  - version_id: '3.3'
    status: ga
    context_window: 128000
    pricing_note: 'No API cost; infra cost varies by hardware. Via third-party APIs
      (Groq, Together, etc.): ~$0.06-0.80/1M tokens depending on provider.'
    vendor_model_id: llama-3.3-70b-instruct
    jurisdiction:
      vendor_hq_country: US
      applicable_legal_regimes:
      - EU-GDPR
      - US-CLOUD-Act
      data_residency_regions: []
    data_privacy:
      data_retention_days: unknown
      training_on_customer_data: opt-out
      pii_eligible: false
      phi_hipaa_eligible: false
      gdpr_eligible: true
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
    breadth: 3
    reliability: 3
    governance: 5
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://ai.meta.com/
  rationale: Open-source
---
