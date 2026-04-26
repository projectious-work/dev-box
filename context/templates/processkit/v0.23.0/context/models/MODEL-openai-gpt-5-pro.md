---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-openai-gpt-5-pro
  created: '2026-04-22T00:00:00Z'
spec:
  provider: openai
  family: gpt-5-pro
  versions:
  - version_id: '5.4'
    status: preview
    context_window: 256000
    pricing_usd_per_1m:
      input: 15.0
      output: 60.0
    pricing_note: ESTIMATED — premium/slower variant analogous to Opus 4.6. Run Workflow
      C to validate.
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
    speed: 2
    breadth: 5
    reliability: 5
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: xxl
  status_page_url: https://status.openai.com/
  rationale: Frontier flagship premium (estimated)
---
