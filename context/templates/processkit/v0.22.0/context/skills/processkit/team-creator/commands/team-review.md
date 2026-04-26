# Command: team-review

Read-only health-check. Compares current team assignments against the
latest landscape snapshot and surfaces tier-drift, unavailable models,
and new outperformers. No entities are written.

## Syntax

```
team-review
  [--landscape-artifact <ART-id>]  # explicit landscape override; skips discovery
  [--landscape <artifact-id>]      # alias for --landscape-artifact
  [--threshold <float>]            # flag if tier-score drifted > N pts, default 0.15
```

## Process (sequential, read-only)

### Step 1 — Load current team

1. Read `context/team/roster.md` to get the active role → model mapping.
2. Resolve the governing DecisionRecord (the most recent `team-create`
   DecisionRecord) via `decision-record.get_decision`.
3. Extract stored formula weights and tier scores from
   `spec.inputs_snapshot`. These are the baseline scores for the diff.
4. Extract `landscape_artifact` ID and date from `inputs_snapshot`.

### Step 2 — Load landscape snapshot

Landscape resolution follows the same three-level precedence as
`team-create` (see `references/landscape-resolution.md`):

1. If `--landscape-artifact <ART-id>` (or alias `--landscape`) is
   supplied, load that artifact directly.
2. Otherwise use the artifact ID stored in the governing DecisionRecord
   `inputs_snapshot.landscape_artifact`. If neither is available,
   apply the three-level discovery: project-tagged artifact first,
   then kit default `landscape-summary`.
3. If the artifact is older than 90 days, include a staleness warning
   in the output. Do not block.

### Step 3 — Re-score current assignments

For each currently-assigned model:
```
model-recommender.get_pricing(<model-id>)
model-recommender.get_profile(<model-id>)
```

Recompute `TierScore` using the weights from `inputs_snapshot`
(not from `--weight-overrides` — the baseline uses stored weights).
Apply the same normalisation across the current accessible candidate set.

### Step 4 — Query for new candidates

```
model-recommender.query_models(
  G_floor=<stored_governance_floor>,
  apply_user_filter=true
)
```

Score all accessible candidates with the stored weights. Identify any
model that:
- Would out-score the current assignment for its role by more than
  `--threshold` (default 0.15).
- Appeared in the landscape since the last `team-create` run.

### Step 5 — Detect unavailable models

```
model-recommender.check_availability()
```

Flag any currently-assigned model whose availability is `degraded`
or `major_outage`.

### Step 6 — Emit diff report

Output format (stdout only — no files written):

```
=== team-review — <date> ===
Baseline: DecisionRecord <DEC-id> (<team-create date>)
Landscape: <artifact-id> (<artifact date>) [STALE: >90 days if applicable]
Weights used: C=<C> K=<K> L=<K> G=<G>
Threshold: <threshold>

TIER-DRIFT (score delta > threshold):
  developer: <old-model-id> (baseline 0.62) → new score 0.44  ▼0.18
    → Best alternative: <new-model-id> (score 0.71, heavy)
    → Recommendation: rebalance this role

UNAVAILABLE:
  junior-developer: <model-id> — status: major_outage
    → Best fallback: <model-id> (score 0.38, light)

NEW OUTPERFORMERS:
  assistant: <new-model-id> (score 0.52) outperforms current
    <model-id> (score 0.31) by 0.21 — within light tier, no tier-shift

STABLE (no action needed):
  project-manager, senior-architect, senior-researcher,
  junior-architect, junior-researcher

SUMMARY:
  2 roles recommended for rebalance
  1 role urgently needs replacement (major_outage)
  Run: team-rebalance --roles developer,junior-developer --confirm \
       --reason "<reason>"
=============================
```

Omit empty sections. Always emit the SUMMARY even if no issues found.

## State side-effects

None. This command is fully read-only.

## Distinction from team-create

`team-review` reads and diffs; `team-create` writes. Running
`team-review` is always safe and reversible. Only run `team-create`
or `team-rebalance` when you have reviewed the diff and confirmed
the recommended changes.
