---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-google-gemini-3-flash
  created: '2026-04-22T00:00:00Z'
spec:
  provider: google
  family: gemini-3-flash
  versions:
  - version_id: '3'
    status: preview
    context_window: 2000000
    pricing_usd_per_1m:
      input: 0.05
      output: 0.2
    pricing_note: ESTIMATED — next-gen Google Flash. Run Workflow C to validate.
    vendor_model_id: gemini-3.0-flash
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
  dimensions:
    reasoning: 4
    engineering: 3
    speed: 5
    breadth: 5
    reliability: 3
    governance: 2
  modalities:
  - text
  - vision
  - audio
  - tools
  access_tier: public
  equivalent_tier: l
  status_page_url: https://status.cloud.google.com/
  rationale: Frontier fast (estimated)
---
