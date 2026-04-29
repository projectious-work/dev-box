---
apiVersion: processkit.projectious.work/v1
kind: Model
metadata:
  id: MODEL-cohere-command-r-plus
  created: '2026-04-22T00:00:00Z'
spec:
  provider: cohere
  family: command-r-plus
  versions:
  - version_id: '1'
    status: ga
    context_window: 128000
    pricing_usd_per_1m:
      input: 2.5
      output: 10.0
    pricing_note: Weights available for self-hosting; Cohere is SOC 2 certified; HIPAA
      BAA available for enterprise
    vendor_model_id: command-r-plus-08-2024
    jurisdiction:
      vendor_hq_country: CA
      applicable_legal_regimes:
      - EU-GDPR
      - US-CLOUD-Act
      - CA-PIPEDA
      data_residency_regions:
      - us
      - eu
    data_privacy:
      dpa_available: https://cohere.com/legal/dpa
      data_retention_days: zero
      training_on_customer_data: never
      pii_eligible: true
      phi_hipaa_eligible: true
      gdpr_eligible: true
      sub_processors_url: https://cohere.com/legal/sub-processors
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
    reliability: 4
    governance: 3
  modalities:
  - text
  - tools
  access_tier: public
  equivalent_tier: m
  status_page_url: https://status.cohere.com/
  rationale: Enterprise RAG specialist
---
