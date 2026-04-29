---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-minimax-minimax-m
  created: '2026-04-22T00:00:00Z'
spec:
  provider: minimax
  family: minimax-m
  versions:
  - version_id: '2.5'
    status: preview
    context_window: 1000000
    pricing_usd_per_1m:
      input: 0.2
      output: 1.1
    pricing_note: ESTIMATED — updated MiniMax. Run Workflow C to validate.
    governance_warning: 'Chinese company. Via MiniMax API: G:1. Self-hosted open weights
      raise governance to G:3.'
    vendor_model_id: MiniMax-M1
    jurisdiction:
      vendor_hq_country: CN
      applicable_legal_regimes:
      - CN-DSL
      - CN-PIPL
      data_residency_regions:
      - cn
    data_privacy:
      data_retention_days: unknown
      training_on_customer_data: unknown
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
    engineering: 3
    speed: 3
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://www.minimax.io/
  rationale: Long-context (estimated)
---
