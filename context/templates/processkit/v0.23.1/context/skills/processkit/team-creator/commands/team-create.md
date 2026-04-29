# Command: team-create

Full team derivation from scratch. Writes 8 Role + 8 Actor + 8 Binding
entities, rewrites `context/team/roster.md`, and emits a chartering
DecisionRecord that supersedes the prior team DecisionRecord.

## Syntax

```
team-create
  --subscription <provider>:<tier>   # e.g. anthropic:max-5x
  --providers <list|"any">           # comma-separated; "any" = all accessible
  --parallelism-cap <int>            # max clones per role, default 5
  --governance-floor <0-5>           # G-score floor, default 3
  [--weight-overrides <json>]        # {"C":0.60,"K":0.20,"L":0.10,"G":0.10}
                                     # CLI > DEC-*-TeamWeights > skill defaults
  [--threshold-overrides <json>]     # {"heavy_min":0.70,"medium_min":0.40}
                                     # CLI > DEC-*-TeamWeights > skill defaults
  [--landscape-artifact <ART-id>]    # explicit landscape override; skips discovery
  [--landscape <artifact-id>]        # alias for --landscape-artifact (kept for compat)
  [--dry-run]                        # print plan; write nothing
```

## Process (sequential)

### Step 1 — Resolve landscape snapshot

Three-level precedence (see `references/landscape-resolution.md`):

1. If `--landscape-artifact <ART-id>` (or alias `--landscape`) is
   supplied, load that artifact directly. Record
   `inputs_snapshot.landscape_artifact_source: "explicit"`.
2. Else query artifact index for the most-recently-created artifact
   tagged both `landscape-summary` **and** `landscape-summary-project`
   whose `spec.project_id` matches the current project context.
   Record `inputs_snapshot.landscape_artifact_source: "project-tag"`.
3. Else fall back to the most-recently-created artifact tagged
   `landscape-summary` (kit default). Record
   `inputs_snapshot.landscape_artifact_source: "kit-default"`.

If no artifact is found at any level, **abort** with:
> "No landscape-summary artifact found. Run the Haiku landscape-ingest
> task to produce an ART-*-LandscapeSnapshot artifact before running
> team-create."

If the resolved artifact is older than 90 days, emit a warning but
continue. Record the artifact ID and its `created` date in the
DecisionRecord `inputs_snapshot`.

### Step 2 — Query accessible models

```
model-recommender.check_availability()
model-recommender.query_models(
  providers=<--providers>,
  G_floor=<--governance-floor>,
  apply_user_filter=true
)
model-recommender.get_pricing(sort_by="value_score")
```

Filter out any model with availability != "operational" unless the
owner explicitly passes `--allow-degraded`.

### Step 3 — Resolve weights + thresholds, then apply tiering formula

**Weight resolution (before scoring):**

```
CLI --weight-overrides <json>
  > DEC-*-TeamWeights.weight_overrides (tag: team-weights-override,
      most recent accepted; see references/team-weights-decision-schema.md)
    > skill defaults: {C:0.60, K:0.20, L:0.10, G:0.10}
```

**Threshold resolution (before classification):**

```
CLI --threshold-overrides <json>
  > DEC-*-TeamWeights.tier_thresholds (same DEC; validated on load)
    > skill defaults: {heavy_min:0.70, medium_min:0.40}
```

Validate any threshold override against the rules in
`references/team-weights-decision-schema.md` (§Threshold validation
rules). Violations are hard errors — abort before scoring.

Record `inputs_snapshot.weights_source` and
`inputs_snapshot.tier_thresholds_source` with the resolution level
used (`"cli"`, `"dec-team-weights"`, or `"skill-default"`), and
store the actual values in `inputs_snapshot.weights` and
`inputs_snapshot.tier_thresholds`.

**Scoring:**

For each accessible candidate model `m`:

1. Compute raw dimension values:
   - `C_raw(m)` = (swe_bench_verified + swe_bench_pro +
                   community_rating) / 3
   - `K_raw(m)` = 1 / (output_price_per_1M × quota_burn_vs_flagship)
   - `L_raw(m)` = output_tokens_per_sec
   - `G_raw(m)` = G-score from model-recommender (0–5 scale,
                  normalise to 0–1 by dividing by 5)

2. Min-max normalise each dimension across the full candidate set:
   `norm(x) = clamp((x - min) / (max - min), 0.01, 1.00)`

3. Compute TierScore using resolved weights `{C, K, L, G}`:
   `TierScore(m) = C×norm(C) + K×norm(K) + L×norm(L) + G×norm(G)`

4. Classify using resolved thresholds:
   - TierScore ≥ `heavy_min` → **heavy**
   - TierScore ≥ `medium_min` and < `heavy_min` → **medium**
   - TierScore < `medium_min` → **light**

5. Tier-collapse rule: if fewer than 3 distinct tiers result, promote
   the two highest-scoring light models to medium.

See `references/tiering-formula.md` for the full formula and worked
example.

### Step 4 — Load role-archetypes override (if present), then map archetypes

**Layer 4 override (eager validation — before archetype mapping):**

If `context/team/role-archetypes.yaml` exists, load it now and
validate it immediately against the rules in
`references/role-archetypes-override.md` (§Validation invariants).
Validation failures are hard errors — abort before any mapping begins.
This includes: PM pin must remain heavy, PM clone_cap_override ≤ 1,
all rationales non-empty, all 8 archetypes present in `replace` mode.

Record `inputs_snapshot.archetype_override_file: "present"` (or
`"absent"`) and list each overridden role with its new pin and
rationale summary in `inputs_snapshot.archetype_overrides`.

**Archetype mapping:**

For each of the 8 role archetypes:

1. Select the archetype's pinned tier — from the (possibly-overridden)
   archetype table. Kit defaults (`references/role-archetypes.md`)
   apply for any archetype not listed in a delta override, or when no
   override file is present.
2. From the models classified into that tier, pick the highest TierScore.
3. If preferred_providers are supplied and two models score within 0.05
   of each other, prefer the model from the preferred provider.
4. If the pinned tier has no candidates: apply the override-when rule
   from the active archetype table; if no override applies, fail
   with a clear message.

### Step 5 — Deactivate prior team (if re-running)

If `context/team/roster.md` already exists:

1. Parse the prior team's Actor and Binding IDs from roster.md.
2. For each prior Binding:
   `binding-management.end_binding(id, reason=
     "superseded by team-create run <ISO-timestamp>")`
3. For each prior Actor whose model ID does NOT appear in the new
   team assignments:
   `actor-profile.deactivate_actor(id)`
4. For each prior Actor whose model IS being re-assigned to the SAME
   role in the new team: identify the canonical seed by
   `is_template: true` (not by heuristics such as name or ID prefix).
   REUSE that Actor ID — do not create a new Actor entity. This
   ensures reuse targets the authoritative template, never a clone.

### Step 6 — Write entities

Skip if `--dry-run`.

Per-archetype values for `primary_contact`, `clone_cap`, and
`cap_escalation` come from `references/role-archetypes.md`.
Only `project-manager` has `primary_contact: true` and
`clone_cap: 1`; all others use `primary_contact: false` and
`clone_cap: 5`. `cap_escalation` is `"owner"` for all roles.
Seed Actors always receive `is_template: true, templated_from: null`.

For each of the 8 archetypes:
```
role-management.create_role(
  id=ROLE-<archetype-name>,
  name=<archetype-name>,
  description=<one-line responsibility>,
  responsibilities=[...],
  default_scope="permanent",
  primary_contact=<bool per archetype table>,
  clone_cap=<int per archetype table>,
  cap_escalation="owner"
)
actor-profile.create_actor(
  id=ACTOR-<archetype-alias>,     # reuse existing if same model
  type="ai-agent",
  name=<model-display-name>,      # from model-recommender, not hardcoded
  active=true,
  is_template=true,
  templated_from=null
)
binding-management.create_binding(
  type="role-assignment",
  subject=<ACTOR-id>,
  target=<ROLE-id>,
  valid_from=<today>,
  description="<model-id> fills <archetype-name>"
)
```

### Step 7 — Write roster.md

Write `context/team/roster.md` with:
- Narrative preamble (team charter, subscription, governance floor)
- Routing table: role → actor → model-id → tier → tier-score
- Parallelism cap and PM clone-cap note
- Weights used (from inputs or defaults)
- Landscape artifact ID and date
- DecisionRecord ID (written in step 8)

### Step 8 — Write chartering DecisionRecord

```
decision-record-write(
  title="Team composition — <subscription> — <date>",
  state="accepted",
  supersedes=[<prior-team-decisionrecord-id>],  # if re-running
  inputs_snapshot={
    subscription: ...,
    governance_floor: ...,
    parallelism_cap: ...,
    weights: {C, K, L, G},
    weights_source: "cli" | "dec-team-weights" | "skill-default",
    tier_thresholds: {heavy_min, medium_min},
    tier_thresholds_source: "cli" | "dec-team-weights" | "skill-default",
    landscape_artifact: <ART-id>,
    landscape_artifact_date: <date>,
    landscape_artifact_source: "explicit" | "project-tag" | "kit-default",
    tier_scores: {<model-id>: <score>, ...},
    weight_overrides_applied: <bool>,
    archetype_override_file: "present" | "absent",
    archetype_override_semantics: "delta" | "replace" | null,
    archetype_overrides: [...]   # list of {role, kit_default_pin, override_pin}
  }
)
```

The `inputs_snapshot.weights` block is the canonical weight store
that `team-rebalance` will read on future runs.

## Dry-run output format

```
=== team-create DRY RUN ===
Subscription: <value>   Governance floor: <value>
Landscape: <artifact-id> (<date>)
Weights: C=0.60 K=0.20 L=0.10 G=0.10

Candidate models scored:
  <model-id>  TierScore=0.92  → heavy
  <model-id>  TierScore=0.61  → medium
  ...

Role assignments:
  project-manager  → <model-id>  (heavy, score=0.92)
    Role fields: primary_contact=true  clone_cap=1
                 cap_escalation="owner"
    Actor fields: is_template=true  templated_from=null
  senior-architect → <model-id>  (heavy, score=0.87)
    Role fields: primary_contact=false clone_cap=5
                 cap_escalation="owner"
    Actor fields: is_template=true  templated_from=null
  ...

Entities to write (skipped in dry-run):
  8 Role entities, 8 Actor entities, 8 Binding entities
  context/team/roster.md
  DecisionRecord superseding <DEC-id>
===========================
```

## State side-effects (non-dry-run)

Creates: 8 Role + 8 Actor + 8 Binding entities, roster.md,
1 DecisionRecord. Deactivates: prior Bindings (end_binding) and
prior Actors not re-used.
