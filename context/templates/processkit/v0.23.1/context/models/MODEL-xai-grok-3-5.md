---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-xai-grok-3-5
  created: '2026-04-22T00:00:00Z'
spec:
  provider: xai
  family: grok-3-5
  versions:
  - version_id: '3.5'
    status: preview
    context_window: 128000
    pricing_usd_per_1m:
      input: 3.0
      output: 15.0
    pricing_note: ESTIMATED — incremental improvement over Grok 3. Run Workflow C
      to validate.
    jurisdiction:
      vendor_hq_country: US
      applicable_legal_regimes:
      - EU-GDPR
      - US-CLOUD-Act
      data_residency_regions:
      - us
    data_privacy:
      dpa_available: https://x.ai/legal/dpa
      data_retention_days: 30
      training_on_customer_data: opt-out
      pii_eligible: true
      phi_hipaa_eligible: false
      gdpr_eligible: true
      sub_processors_url: https://x.ai/legal/sub-processors
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
    breadth: 4
    reliability: 4
    governance: 2
  modalities:
  - text
  - vision
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.x.ai/
  rationale: Frontier flagship (estimated)
---
