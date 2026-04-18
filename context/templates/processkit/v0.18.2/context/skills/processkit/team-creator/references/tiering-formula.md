# Tiering Formula — Full Reference

## Dimensions and default weights

**Defaults rebalanced 2026-04-15 after Phase 3 dogfood (see ART-20260415_1545-TeamWeaver-team-creator-dogfood-diff §7).** Capability weight raised from 0.40 to 0.60 to prevent tier inversion on same-provider candidate sets where cost and capability are anti-correlated.

| Dim | Symbol | Source fields (from model-recommender) | Default weight |
|---|---|---|---|
| Capability | C | `(swe_bench_verified + swe_bench_pro + community_rating) / 3`, normalised 0–1 | **0.60** |
| Cost efficiency | K | `1 / (output_price_per_1M × quota_burn_vs_flagship)`, normalised 0–1 | **0.20** |
| Latency fit | L | `output_tokens_per_sec`, normalised 0–1 | **0.10** |
| Governance | G | G-score from model-recommender (0–5), divided by 5, normalised 0–1 | **0.10** |

Weights must sum to 1.00 ± 0.001. Owner may supply
`--weight-overrides '{"C":0.5,"K":0.3,"L":0.1,"G":0.1}'` on
`team-create`. The overrides are stored verbatim in the chartering
DecisionRecord's `inputs_snapshot.weights` and are reused by
`team-rebalance` unless a new `--weight-overrides` is supplied.

## Normalisation

Each raw metric is **min-max normalised across the accessible candidate
set** for the current run — not against a global roster. Normalisation
is per-run, so results are relative to what the owner can actually use.

```
norm(x) = (x − min_in_set) / (max_in_set − min_in_set)
          clamp to [0.01, 1.00]
```

The 0.01 floor prevents zero-division in single-model sets and
preserves ranking in near-degenerate sets.

## Composite tier score

```
TierScore(m) = 0.60 × norm(C(m))
             + 0.20 × norm(K(m))
             + 0.10 × norm(L(m))
             + 0.10 × norm(G(m))
```

## Classification thresholds

| TierScore range | Tier |
|---|---|
| ≥ 0.70 | **heavy** |
| 0.40 – 0.69 | **medium** |
| < 0.40 | **light** |

Thresholds are fixed and not owner-overridable — fixed thresholds
keep the formula auditable and results reproducible across runs.

**Fallback rule.** If no candidate clears 0.70 (e.g. only a single
accessible model, or every model is medium-tier), the highest-scoring
candidate is promoted to heavy regardless of its raw score.

**Tier-collapse rule.** If the accessible set produces fewer than 3
distinct tiers, the two highest-scoring light models are promoted to
medium. The skill never fails on tier-collapse.

## Computing `quota_burn_vs_flagship`

`quota_burn_vs_flagship` is the ratio of this model's token-cost
burn rate to the provider's flagship model on the same subscription
tier. For example:
- Flagship model (e.g. heaviest subscription-included model): 1.0×
- A lighter model that costs half as much per token: 0.5×
- A pay-per-token model outside the subscription: compute from
  effective $/token relative to flagship API price.

If the value is unavailable from `model-recommender`, default to 1.0×
(conservative: treats the model as equally expensive as the flagship).

## Governance (G) source

The G-score in `model-recommender` runs on a 0–5 integer scale.
For normalisation, divide by 5 before min-max normalisation:

```
G_raw(m) = model-recommender.G_score(m) / 5   # maps to [0, 1]
```

This means G is already 0–1 before the cross-set normalisation step.
Apply min-max normalisation across the candidate set as usual, with
the 0.01 floor.

## Worked example: two-model set (illustrative)

**Source data** (April 2026 landscape snapshot, Anthropic Max 5×
subscription, G-floor = 4):

| Metric | Model A (flagship) | Model B (budget) |
|---|---|---|
| SWE-bench Verified | 80.8% | ~80% |
| SWE-bench Pro | ~46% | 57.7% |
| Community rating | 9.5/10 | 9.0/10 |
| C raw avg | 0.739 | 0.759 |
| Output price /1M | $25 | $15 |
| Quota burn vs flagship | 1.0× | ~0.5× |
| K raw = 1/(price × burn) | 0.040 | 0.133 |
| Output tok/s | ~25 | ~82.5 |
| G score (raw 0–5) | 5 | 2 |
| G floor check | PASS (5 ≥ 4) | FAIL (2 < 4) |

Model B fails the G-floor gate check (G=2 < floor=4) and is excluded
**before** formula scoring. Model A is the sole survivor. The fallback
rule promotes it to heavy.

**With G-floor = 2 (both models accessible):**

| | Model A | Model B |
|---|---|---|
| norm(C) | 0.01 (min of set) | 1.00 (max of set) |
| norm(K) | 0.01 | 1.00 |
| norm(L) | 0.01 | 1.00 |
| norm(G) | 1.00 | 0.01 |
| **TierScore** | 0.60×0.01 + 0.20×0.01 + 0.10×0.01 + 0.10×1.00 = **0.110** | 0.60×1.00 + 0.20×1.00 + 0.10×1.00 + 0.10×0.01 = **0.901** |
| **Tier** | light | heavy |

**Interpretation.** In the G-floor=4 scenario, Model A (G=5, passes)
is the sole candidate and is promoted to heavy by the fallback rule —
reproducing the current team's flagship assignments for heavy roles
when constrained by governance. In the G-floor=2 scenario, Model B
scores 0.901 (heavy) vs Model A's 0.110 (light) — demonstrating full
provider-neutrality: the formula favours the better cost/latency model
when governance constraints permit.

**Important caveat.** A real run with 10–15 accessible models spreads
scores evenly; the two-model example is illustrative only. The extreme
normalised values (0.01 vs 1.00) arise purely from having just two
candidates in the set.

## Sensitivity to weight changes

| Weight change | Expected effect |
|---|---|
| Raise C (capability) | Favours frontier models with top benchmark scores |
| Raise K (cost) | Favours smaller, faster, cheaper models |
| Raise L (latency) | Favours high-throughput models regardless of cost |
| Raise G (governance) | Favours self-hosted or high-compliance models |

Owner-supplied overrides must sum to 1.00 ± 0.001. The skill rejects
overrides that violate this constraint with a clear error message.
