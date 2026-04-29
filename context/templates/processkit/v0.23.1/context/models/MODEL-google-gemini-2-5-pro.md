---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemini-2-5-pro
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemini-2-5-pro
  versions:
  - version_id: '2.5'
    status: ga
    context_window: 1000000
    pricing_usd_per_1m:
      input: 1.25
      output: 10.0
    pricing_note: Context >200K billed at higher rates; 1M context tasks can be expensive
    knowledge_cutoff: 2025-01-01
    vendor_model_id: gemini-2.5-pro
    jurisdiction:
      vendor_hq_country: US
      applicable_legal_regimes:
      - EU-GDPR
      - US-CLOUD-Act
      - US-HIPAA
      data_residency_regions:
      - us
      - eu
      - asia
      - multi-region
    data_privacy:
      dpa_available: https://cloud.google.com/terms/data-processing-addendum
      data_retention_days: zero
      training_on_customer_data: opt-out
      pii_eligible: true
      phi_hipaa_eligible: true
      gdpr_eligible: true
      sub_processors_url: https://cloud.google.com/terms/subprocessors
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
  equivalent_tier: xxl
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier flagship
---
