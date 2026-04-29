---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-deepseek-deepseek-v
  created: '2026-04-22T00:00:00Z'
spec:
  provider: deepseek
  family: deepseek-v
  versions:
  - version_id: '3'
    status: ga
    context_window: 64000
    pricing_usd_per_1m:
      input: 0.14
      output: 0.28
    pricing_note: Exceptional price-capability ratio; cheapest frontier-class model.
      G:1 severely limits safe use cases.
    governance_warning: G:1 — Chinese jurisdiction. Never use for PII, PHI, confidential
      IP, credentials, regulated data, or government/defense work.
    vendor_model_id: deepseek-v3
    jurisdiction:
      vendor_hq_country: CN
      applicable_legal_regimes:
      - CN-DSL
      - CN-PIPL
      data_residency_regions:
      - cn
    data_privacy:
      dpa_available: false
      data_retention_days: unknown
      training_on_customer_data: opt-out
      pii_eligible: false
      phi_hipaa_eligible: false
      gdpr_eligible: false
  efforts_supported:
  - none
  - low
  - medium
  - high
  - extra-high
  - max
  dimensions:
    reasoning: 4
    engineering: 4
    speed: 4
    breadth: 3
    reliability: 3
    governance: 1
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: xl
  status_page_url: https://status.deepseek.com/
  rationale: Frontier-class, low cost
---
