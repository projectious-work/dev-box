# Example: Task Router Output

**Input:** A sprint plan for a healthcare SaaS platform migration

**Tasks provided:**
1. Design the new multi-tenant database schema
2. Audit all current patient data flows for HIPAA compliance
3. Migrate 2.3M legacy patient records to new schema format
4. Generate test fixtures for all 340 API endpoints
5. Implement JWT refresh token rotation with sliding window
6. Refactor the monolithic auth service into three bounded-context services
7. Write the architecture decision record for the new schema
8. Analyze the attached system architecture diagram and identify bottlenecks
9. Review all 47 PRs in the migration branch
10. Generate FHIR-compliant HL7 bundles for all migrated records
11. Prove that the new schema is backwards-compatible with the legacy API contract
12. Set up GitHub Actions CI pipeline for the new services

---

## Task Routing Analysis

```
══════════════════════════════════════════════════════════════════

Cluster 1 — Privacy-First (Governance)                           [3 tasks]
  Profile:  R:3  E:3  S:3  B:3  L:4  G:5
  Model:    Llama 3.3 70B self-hosted
  Fallback: Mistral Large 3 (Le Chat Enterprise, EU region)

  NOTE: These tasks involve PHI (patient data) and HIPAA-governed workflows.
  Route to a G:5 model regardless of capability trade-offs. Do not use any
  cloud-hosted frontier model without a signed HIPAA BAA and regional DPA.

  Tasks:
    • #2  Audit all current patient data flows for HIPAA compliance
    • #3  Migrate 2.3M legacy patient records to new schema format
    • #10 Generate FHIR-compliant HL7 bundles for all migrated records

──────────────────────────────────────────────────────────────────

Cluster 2 — Deep-Think (Reasoning + Engineering)                 [3 tasks]
  Profile:  R:5  E:5  S:2  B:3  L:5  G:5
  Model:    Claude Opus 4.6
  Fallback: Claude Sonnet 4.6 (for tasks 6 and 7; overkill only on task 11)

  Tasks:
    • #1  Design the new multi-tenant database schema
    • #6  Refactor the monolithic auth service into three bounded-context services
    • #11 Prove the new schema is backwards-compatible with the legacy API contract

──────────────────────────────────────────────────────────────────

Cluster 3 — Production-Coder (Engineering)                       [3 tasks]
  Profile:  R:3  E:5  S:3  B:3  L:4  G:5
  Model:    Claude Sonnet 4.6
  Fallback: GPT-4o (only if Sonnet is rate-limited and data is non-PHI)

  Tasks:
    • #5  Implement JWT refresh token rotation with sliding window
    • #9  Review all 47 PRs in the migration branch
    • #12 Set up GitHub Actions CI pipeline for the new services

──────────────────────────────────────────────────────────────────

Cluster 4 — Multimodal + Analysis (Breadth)                      [1 task]
  Profile:  R:4  E:3  S:3  B:5  L:4  G:5
  Model:    Claude Sonnet 4.6 (vision, G:5 required — cannot use Gemini)
  Fallback: Claude Opus 4.6 (if architectural analysis needs deeper reasoning)

  NOTE: Gemini 2.5 Pro would be ideal for vision (B:5) but scores G:2.
  Since the architecture diagram likely contains internal system details,
  we apply the governance ceiling rule and stay with Claude.

  Tasks:
    • #8  Analyze the attached system architecture diagram and identify bottlenecks

──────────────────────────────────────────────────────────────────

Cluster 5 — High-Volume (Speed)                                  [2 tasks]
  Profile:  R:2  E:3  S:5  B:3  L:3  G:5
  Model:    Claude Haiku 4.5
  Fallback: Gemini Flash 2.0 (only if data is confirmed non-PHI)

  NOTE: Verify task #4 does not include real patient identifiers in fixtures.
  If fixtures contain synthetic data only, Haiku is appropriate. If real
  record structure leaks PHI into fixture generation context, route to
  Cluster 1 instead.

  Tasks:
    • #4  Generate test fixtures for all 340 API endpoints
    • #7  Write the architecture decision record for the new schema

══════════════════════════════════════════════════════════════════
Total: 12 tasks across 5 clusters.

Suggested sequence:
  1. Cluster 2 (architecture and schema design) — decisions made here
     constrain all downstream work; use Opus for quality.
  2. Cluster 1 (PHI-sensitive tasks) — patient data work in parallel
     with implementation once schema is locked.
  3. Cluster 3 (implementation) — Sonnet on feature work and reviews.
  4. Cluster 4 (diagram analysis) — can run in parallel with Cluster 3.
  5. Cluster 5 (high-volume) — fixture generation and ADR writing after
     schema and API contracts are finalized.

Governance note: 5 of 12 tasks require G:5. Confirm that Llama 3.3 is
provisioned and accessible before starting Cluster 1. Do not begin
patient data work until the self-hosted model is validated.
══════════════════════════════════════════════════════════════════
```
