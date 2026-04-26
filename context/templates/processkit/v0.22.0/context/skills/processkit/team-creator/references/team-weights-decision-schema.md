# DEC-*-TeamWeights — Schema Reference

## Purpose

A `DEC-*-TeamWeights` DecisionRecord persists per-project formula
weights and tier thresholds. `team-create` discovers it automatically
at startup and applies it when no CLI `--weight-overrides` or
`--threshold-overrides` are supplied. The record is separate from the
chartering team DecisionRecord — it governs **policy**, not a run.

## Discovery rule (tag-only)

Query the decision index for DecisionRecords tagged
`team-weights-override` with `spec.state == "accepted"`, ordered by
`metadata.created` descending. The most-recently-accepted record wins.

Tag-only discovery is the sole mechanism. Do NOT rely on arbitrary
`spec.*` field queries (e.g. `spec.applies_to == "team-creator"`) —
these are not guaranteed portable across index implementations.

## Full schema

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-<generated-id>
  created: <ISO-8601>
  tags: [team-weights-override]        # REQUIRED for discovery
spec:
  title: "Team weight override — <project-slug> — <YYYY-MM-DD>"
  state: accepted                      # only accepted records are applied
  context: "<why this project needs non-default weights>"
  decision: "Override team-creator formula weights and/or tier thresholds"
  rationale: |
    <REQUIRED — at minimum one sentence per dimension changed.
    Must explain the project context that motivates each override.>
  alternatives:
    - option: "Keep kit defaults (C=0.60, K=0.20, L=0.10, G=0.10)"
      rejected_because: "<reason>"
  consequences: |
    <Expected effects on tier classification for this project's model set.>
  deciders: [<ACTOR-id>, ...]
  decided_at: <ISO-8601>

  # ── OpenWeave extension fields ──────────────────────────────────────
  applies_to: team-creator             # discriminator; must be this value

  weight_overrides:                    # OPTIONAL block; omit to keep defaults
    C: <float>   # Capability weight
    K: <float>   # Cost efficiency weight
    L: <float>   # Latency fit weight
    G: <float>   # Governance weight
    # All four keys must be present if the block appears.
    # Values must sum to 1.00 ± 0.001; violation is a hard error.

  tier_thresholds:                     # OPTIONAL block; omit to keep defaults
    heavy_min: <float>   # default 0.70; TierScore >= this → heavy
    medium_min: <float>  # default 0.40; TierScore >= this → medium
                         # light is implicit: TierScore < medium_min
    rationale: |
      <REQUIRED — minimum two sentences:
       (1) why these thresholds fit this project's model set;
       (2) what analysis was done to arrive at the values.>
```

## Precedence chains

### Weights

```
CLI --weight-overrides <json>
  > DEC-*-TeamWeights.weight_overrides (most recent accepted, tag query)
    > skill defaults in references/tiering-formula.md
```

### Thresholds

```
CLI --threshold-overrides <json>
  > DEC-*-TeamWeights.tier_thresholds (most recent accepted, tag query)
    > skill defaults (heavy_min=0.70, medium_min=0.40)
```

The source used is recorded in the chartering DecisionRecord:
- `inputs_snapshot.weights_source`: `"cli"` | `"dec-team-weights"` | `"skill-default"`
- `inputs_snapshot.tier_thresholds_source`: `"cli"` | `"dec-team-weights"` | `"skill-default"`
- Actual values applied are stored in `inputs_snapshot.weights` and
  `inputs_snapshot.tier_thresholds`.

## Threshold validation rules

The following combinations are hard errors (reject with a clear message):

| Condition | Error |
|---|---|
| `heavy_min <= medium_min` | Tier ordering violated |
| `heavy_min > 0.95` | No practical heavy band remains |
| `medium_min < 0.10` | Light tier effectively eliminated |
| `heavy_min - medium_min < 0.10` | Band too narrow; rounding instability |
| Missing or empty `rationale` in `tier_thresholds` | Hard error, not a warning |

Valid range: `0.10 ≤ medium_min < heavy_min ≤ 0.95`.

## team-rebalance interaction

On startup, `team-rebalance` runs the same tag-based DEC-*-TeamWeights
discovery query. **If the TeamWeights DEC is newer than the governing
team DecisionRecord**, `team-rebalance` emits a warning and suggests
`--roles all`:

```
WARNING: A newer DEC-*-TeamWeights record (created <date>) post-dates
the governing team DecisionRecord (created <date>). Weight policy has
changed. Stored weights from the governing DEC are being used for this
targeted rebalance. To apply the updated weights across all roles, run:
  team-rebalance --roles all --confirm --reason "<reason>"
```

`team-rebalance` does NOT silently apply the newer TeamWeights DEC to
a targeted (`--roles <list>`) run. CLI `--weight-overrides` always
wins regardless of DEC age.

## Audit trail

Every run records which override source was active. Querying the
chartering team DecisionRecord fully reconstructs the run's weight
and threshold configuration.
