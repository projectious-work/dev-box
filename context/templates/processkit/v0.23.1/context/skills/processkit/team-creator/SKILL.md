---
name: team-creator
description: |
  Compose a provider-neutral AI team by tiering accessible models on
  cost-efficiency, capability, latency, and governance, then mapping
  the 8 processkit role archetypes onto those tiers. Use when
  bootstrapping a new project team, rotating after a provider change,
  or quarterly rebalancing. Triggers: "create my AI team",
  "compose a team", "rebalance the team", "review the team".
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-team-creator
    version: "1.2.0"
    created: 2026-04-15T00:00:00Z
    category: processkit
    layer: 3
    uses:
      - skill: model-recommender
        purpose: >
          Query, score, and filter accessible models via
          query_models, get_pricing, check_availability.
      - skill: role-management
        purpose: Create Role entities via create_role.
      - skill: actor-profile
        purpose: >
          Create Actor entities (type: ai-agent) via create_actor;
          deactivate replaced actors via deactivate_actor.
      - skill: binding-management
        purpose: >
          Create role-assignment Bindings via create_binding;
          end superseded Bindings via end_binding.
      - skill: decision-record
        purpose: >
          Write the chartering DecisionRecord; read stored
          formula weights on rebalance via get_decision.
    provides:
      primitives: []
      mcp_tools: []
    commands:
      - name: team-create
        args: "--subscription <provider>:<tier> --providers <list|any> --parallelism-cap <int> --governance-floor <0-5>"
        description: "Derive a full team from scratch: roles, actors, bindings, roster, and chartering DecisionRecord"
      - name: team-review
        args: "[--landscape-artifact <ART-id>] [--threshold <float>]"
        description: "Read-only team health-check against the latest landscape snapshot"
      - name: team-rebalance
        args: "--roles <list|all> --confirm --reason <string>"
        description: "Apply a team-review recommendation: targeted re-tiering of one or more roles"
---

# Team Creator

## Intro

`team-creator` derives a provider-neutral AI team by scoring all
accessible models on four weighted dimensions (Capability, Cost,
Latency, Governance) and mapping the 8 processkit role archetypes
onto the resulting heavy / medium / light tiers. It composes 5
existing skills — no new primitives or MCP tools are introduced.

## Overview

### When to use

| Trigger | Command |
|---|---|
| New project or provider subscription | `team-create` |
| Quarterly landscape shift | `team-review` → `team-rebalance` |
| Provider outage, rapid rotation | `team-rebalance --roles all` |
| Spot-check current assignments | `team-review` |

### Inputs (key parameters)

| Parameter | Required | Default | Notes |
|---|---|---|---|
| `--subscription <provider>:<tier>` | yes | — | e.g. `anthropic:max-5x` |
| `--governance-floor <0-5>` | no | 3 | Models below excluded before scoring |
| `--parallelism-cap <1-10>` | no | 5 | PM always capped at 1 |
| `--weight-overrides <json>` | no | see formula | Must sum to 1.0 ± 0.001; CLI > DEC-*-TeamWeights > defaults |
| `--threshold-overrides <json>` | no | `{"heavy_min":0.70,"medium_min":0.40}` | CLI > DEC-*-TeamWeights > defaults |
| `--landscape-artifact <ART-id>` | no | 3-level discovery | Explicit > project-tag > kit default; see `references/landscape-resolution.md` |
| `--landscape <ART-id>` | no | alias for `--landscape-artifact` | Kept for backwards compatibility |

### Tiering formula (summary)

**Defaults rebalanced 2026-04-15 after Phase 3 dogfood (see ART-20260415_1545-TeamWeaver-team-creator-dogfood-diff §7).** Capability weight raised from 0.40 to 0.60 to prevent tier inversion on same-provider candidate sets where cost and capability are anti-correlated.

| Dim | Weight | Source |
|---|---|---|
| Capability (C) | 0.60 | swe_bench + community rating, normalised |
| Cost efficiency (K) | 0.20 | 1 / (output_price × quota_burn), normalised |
| Latency fit (L) | 0.10 | output_tokens_per_sec, normalised |
| Governance (G) | 0.10 | G-score from model-recommender, normalised |

`TierScore ≥ 0.70 → heavy | 0.40–0.69 → medium | < 0.40 → light`

See `references/tiering-formula.md` for normalisation, thresholds,
and a full worked example.

### Role archetypes → tier mapping

| Role | Tier pin | Override? |
|---|---|---|
| project-manager | heavy | Never (immutable) |
| senior-architect | heavy | Medium only if no heavy clears G-floor + owner approval |
| senior-researcher | heavy | Same as senior-architect |
| junior-architect | medium | Heavy if capability gap > 15pp on SWE-bench |
| developer | medium | Heavy if `--security-critical` flag set |
| junior-researcher | medium | No override |
| junior-developer | light | Medium if no light model accessible |
| assistant | light | No override; shares junior-developer model if no light available |

See `references/role-archetypes.md` for full override-when rules.

Project-level overrides: if `context/team/role-archetypes.yaml`
exists, it is loaded before archetype mapping and validated eagerly.
See `references/role-archetypes-override.md` for schema and invariants.

### Commands

**`team-create`** runs the full derivation: queries models, applies
the tiering formula, maps archetypes, writes 8 Role + 8 Actor + 8
Binding entities, rewrites `context/team/roster.md`, and emits a
chartering DecisionRecord that explicitly supersedes the prior team
DecisionRecord. Use `--dry-run` to print the plan without writing.

**`team-review`** is read-only. Re-scores current assignments against
the latest landscape snapshot, diffs tier scores against those stored
in the governing DecisionRecord, and surfaces tier-shifts, unavailable
models, and new outperformers. No entities are written.

**`team-rebalance`** applies a review recommendation. Requires
`--confirm`. For `--roles all`, the full create logic re-runs and a
new DecisionRecord is written. For named roles, the governing
DecisionRecord's `progress_notes` are amended instead.

### Outputs

| Output | `team-create` | `team-review` | `team-rebalance` |
|---|---|---|---|
| Role / Actor / Binding entities | YES ×8 | NO | scoped roles |
| `context/team/roster.md` | YES | NO | YES (in-place) |
| DecisionRecord | YES (new) | NO | amend notes |
| Diff report | NO | YES (stdout) | NO |

## Gotchas

- **Missing landscape-summary artifact.** This skill ingests a
  pre-structured markdown artifact tagged `landscape-summary`, NOT
  raw HTML. If none exists, error: "No landscape-summary artifact
  found. Run the Haiku landscape-ingest task to produce
  ART-*-LandscapeSnapshot before team-create." Do not parse HTML.

- **Entity deactivation sequence on re-create.** When `team-create`
  is re-run: (a) resolve prior team from roster.md; (b) for each
  prior Binding call `binding-management.end_binding` with
  `reason="superseded by team-create run <timestamp>"`; (c) for each
  prior Actor whose model is NOT in the new team call
  `actor-profile.deactivate_actor`; (d) for prior Actors whose model
  IS re-assigned to the same role REUSE the existing Actor — do not
  create a duplicate; (e) then create new Bindings.

- **Formula weight persistence.** Weights live in the chartering
  DecisionRecord's `spec.inputs_snapshot` block. `team-rebalance`
  reads them via `decision-record.get_decision`. No skill-local
  config file for weights.

- **Tier-collapse (< 3 tiers accessible).** Promote the two highest-
  scoring light models to medium. Never fail; degrade gracefully.

- **PM clone cap.** Parallelism cap never applies to project-manager;
  PM is always exactly 1 (per DEC-20260414_0900).

- **Snapshot staleness.** Artifact older than 90 days → warn and
  proceed. Surface the artifact date in the DecisionRecord.

- **Provider-neutrality.** Never hardcode any model name, provider
  name, or tier label anywhere in this skill. All identifiers flow
  in from `model-recommender`.

- **Override audit trails (OpenWeave layers 1–4).** Each override
  layer produces a distinct audit entry in the chartering team
  DecisionRecord's `inputs_snapshot`:
  - Layer 1 (landscape): `landscape_artifact_source` records
    `"explicit"` | `"project-tag"` | `"kit-default"` and the
    resolved ART-id. The project-tagged artifact (ART-*) is itself
    the audit record for who created the custom landscape.
  - Layer 2 (weights): `weights_source` + `weights` block records
    which DEC-*-TeamWeights (if any) was applied, or `"cli"` /
    `"skill-default"`. The DEC-*-TeamWeights record carries its own
    rationale and deciders.
  - Layer 3 (thresholds): `tier_thresholds_source` + `tier_thresholds`
    block; co-located with Layer 2 in the same DEC-*-TeamWeights
    record (causally coupled — cannot drift).
  - Layer 4 (role pins): `archetype_override_file`, `archetype_override_semantics`,
    and `archetype_overrides` list each overridden role with its new
    pin. The `context/team/role-archetypes.yaml` file itself is
    the persistent record.
  Any run with overrides: query the single chartering team
  DecisionRecord to fully reconstruct the run's configuration.

- **Canonical schema fields (processkit v0.16.0).** This skill
  emits five fields introduced in v0.16.0: Role fields
  `primary_contact` (bool), `clone_cap` (int),
  `cap_escalation` (string); Actor fields `is_template` (bool),
  `templated_from` (string, nullable). Seed Actors are always
  `is_template: true`; rebalance-spawned clones are
  `is_template: false` with `templated_from` pointing at the seed.

## Agent-driven discovery

Agents can resolve user intent into the correct override layer using
the trigger phrases below. These are additive to the existing
`team-creator` triggers above — they extend the skill-finder entries,
not replace them.

| Layer | Trigger phrases | Routed action |
|---|---|---|
| Layer 1 — landscape | "use our own model list", "we have a custom provider set", "this project uses different models", "update the landscape for this project" | Create project-tagged landscape artifact (`landscape-summary-project` tag + `project_id`); pass `--landscape-artifact` to `team-create` |
| Layer 2 — weights | "we value latency more", "cost matters more here", "prioritise governance", "adjust the scoring weights", "latency is critical for this project" | Write a `DEC-*-TeamWeights` record (tag: `team-weights-override`) via `decision-record-write`; re-run `team-create` to pick it up |
| Layer 3 — thresholds | "the tier cutoffs don't fit our model set", "too many models landing in medium", "adjust the heavy threshold", "nothing is reaching heavy tier" | Amend or create `DEC-*-TeamWeights` with a `tier_thresholds` block; validate against rules in `references/team-weights-decision-schema.md` |
| Layer 4 — pins | "pin senior-architect to medium for this project", "we want all researchers on heavy", "remap role classes", "change the role tier assignments" | Create or update `context/team/role-archetypes.yaml`; re-run `team-create` (eager validation fires on startup) |

Agents MUST consult skill-finder before routing any of these requests.
The trigger phrases are designed to match natural-language owner requests
without the owner naming the override layer explicitly.

## No-skill-inflation rationale

Option C chosen over extending `agent-management` (heaviest skill;
scope bleed) or `model-recommender` (individual model profiling vs.
team orchestration). Pure orchestration consumer: 5 existing skills
composed, 3 commands added, 0 new primitives or MCP tools.

## Full reference

### CLI contract — `team-create`

```
team-create
  --subscription <provider>:<tier>     # e.g. anthropic:max-5x
  --providers <list|"any">             # comma-separated; "any" = all accessible
  --parallelism-cap <int>              # max clones per role, default 5
  --governance-floor <0-5>             # G-score floor, default 3
  [--weight-overrides <json>]          # CLI > DEC-*-TeamWeights > defaults
  [--threshold-overrides <json>]       # CLI > DEC-*-TeamWeights > defaults
  [--landscape-artifact <ART-id>]      # explicit landscape; skips discovery
  [--landscape <ART-id>]               # alias for --landscape-artifact
  [--allow-degraded]                   # include models whose availability
                                       # is not "operational"
  [--security-critical]                # promotes developer archetype to heavy
  [--dry-run]                          # print plan; write nothing
```

Full step-by-step process (8 steps: resolve landscape → query models →
score+classify → map archetypes → deactivate prior team → write
entities → write roster → write chartering DecisionRecord) lives in
`commands/team-create.md`. Read that file before changing the
sequence; the order of (a) eager `role-archetypes.yaml` validation
before mapping, (b) `binding-management.end_binding` before
`actor-profile.deactivate_actor`, and (c) DecisionRecord written
LAST so its ID can be embedded in roster.md is load-bearing.

### CLI contract — `team-review`

```
team-review
  [--landscape-artifact <ART-id>]      # default: same 3-level resolution
                                       # as team-create
  [--threshold <float>]                # tier-shift sensitivity, default 0.05
```

Read-only. Re-scores the current team's models against the latest
landscape snapshot, diffs each TierScore against the value stored in
the governing chartering DecisionRecord's
`spec.inputs_snapshot.tier_scores` block, and prints a tier-shift
table to stdout. No entities written; no DecisionRecord amended.

### CLI contract — `team-rebalance`

```
team-rebalance
  --roles <list|"all">                 # comma-separated archetype names
  --confirm                            # required; no-op without it
  --reason <string>                    # free-text rationale; persisted
  [--landscape-artifact <ART-id>]
  [--weight-overrides <json>]
  [--threshold-overrides <json>]
```

For `--roles all`, full `team-create` logic re-runs and a new
chartering DecisionRecord supersedes the current one. For named
roles, only the affected Bindings/Actors are rotated and the
governing DecisionRecord's `progress_notes` are amended with the
`--reason` string and the new tier-score for each rotated role.

### Skill composition map

| Phase | MCP / skill call | Source skill |
|---|---|---|
| Step 2 | `check_availability`, `query_models`, `get_pricing` | model-recommender |
| Step 5 (a) | `end_binding` (per prior Binding) | binding-management |
| Step 5 (b) | `deactivate_actor` (per non-reused prior Actor) | actor-profile |
| Step 6 (Role) | `create_role` × 8 | role-management |
| Step 6 (Actor) | `create_actor` × 8 (or reuse) | actor-profile |
| Step 6 (Binding) | `create_binding` × 8 | binding-management |
| Step 8 | `record_decision` (chartering DEC) | decision-record |
| Rebalance read | `get_decision` (governing DEC for stored weights) | decision-record |

`team-creator` itself ships **no MCP server**; every write goes via the
upstream skills above. This is what keeps `provides.primitives: []`
and `provides.mcp_tools: []` in the frontmatter honest.

### OpenWeave 4-layer override model (data sources)

| Layer | What | Where it lives | Audit field in chartering DEC |
|---|---|---|---|
| 1 — landscape | The candidate model set | ART-*-LandscapeSnapshot artifact (tagged `landscape-summary` or `landscape-summary-project`) | `landscape_artifact`, `landscape_artifact_source`, `landscape_artifact_date` |
| 2 — weights | `{C, K, L, G}` for the TierScore formula | DEC-*-TeamWeights (tag `team-weights-override`) `spec.weight_overrides` | `weights`, `weights_source` |
| 3 — thresholds | `{heavy_min, medium_min}` | Same DEC-*-TeamWeights `spec.tier_thresholds` (causally coupled to layer 2) | `tier_thresholds`, `tier_thresholds_source` |
| 4 — archetype pins | Role → tier mapping overrides | `context/team/role-archetypes.yaml` (delta or replace mode) | `archetype_override_file`, `archetype_override_semantics`, `archetype_overrides` |

CLI flags ALWAYS win over DEC-stored values, which always win over
skill defaults. Eager validation for layer 4 fires BEFORE archetype
mapping; layers 2/3 validation is invoked from
`references/team-weights-decision-schema.md`. Validation failures at
any layer are hard errors — the run aborts before any entity write.

### Tiering formula — canonical definition

```
norm(x)        = clamp((x - min) / (max - min), 0.01, 1.00)
TierScore(m)   = C·norm(C_raw(m))
               + K·norm(K_raw(m))
               + L·norm(L_raw(m))
               + G·norm(G_raw(m))

C_raw(m)       = (swe_bench_verified + swe_bench_pro + community_rating) / 3
K_raw(m)       = 1 / (output_price_per_1M × quota_burn_vs_flagship)
L_raw(m)       = output_tokens_per_sec
G_raw(m)       = G_score / 5             # normalise 0-5 → 0-1

Defaults       C=0.60  K=0.20  L=0.10  G=0.10   (sum = 1.0 ± 0.001)
Tier cuts      heavy ≥ 0.70   medium ≥ 0.40   light < 0.40
```

Worked example and edge cases in `references/tiering-formula.md`.

### Role archetype pin table (kit defaults)

| Archetype | Pin | `primary_contact` | `clone_cap` | `cap_escalation` | Override-when |
|---|---|---|---|---|---|
| project-manager | heavy | true | 1 | owner | Never (immutable; PM clone cap is hard-coded 1) |
| senior-architect | heavy | false | 5 | owner | Medium only if no heavy clears G-floor + owner approval |
| senior-researcher | heavy | false | 5 | owner | Same as senior-architect |
| junior-architect | medium | false | 5 | owner | Heavy if capability gap > 15pp on SWE-bench |
| developer | medium | false | 5 | owner | Heavy if `--security-critical` flag set |
| junior-researcher | medium | false | 5 | owner | No override |
| junior-developer | light | false | 5 | owner | Medium if no light model accessible |
| assistant | light | false | 5 | owner | No override; shares junior-developer model if no light available |

Schema fields (`primary_contact`, `clone_cap`, `cap_escalation`,
`is_template`, `templated_from`) were introduced in processkit
v0.16.0 — older entity files predate them. See
`references/role-archetypes.md` for the full responsibilities lists
that flow into `create_role.responsibilities=[…]`.

### chartering DecisionRecord — `inputs_snapshot` schema

```yaml
spec:
  inputs_snapshot:
    subscription: <provider>:<tier>
    governance_floor: <0-5>
    parallelism_cap: <int>
    weights:                  {C: <float>, K: <float>, L: <float>, G: <float>}
    weights_source:           cli | dec-team-weights | skill-default
    tier_thresholds:          {heavy_min: <float>, medium_min: <float>}
    tier_thresholds_source:   cli | dec-team-weights | skill-default
    landscape_artifact:       ART-<id>
    landscape_artifact_date:  <ISO date>
    landscape_artifact_source: explicit | project-tag | kit-default
    tier_scores:              {<model-id>: <float>, ...}
    weight_overrides_applied: <bool>
    archetype_override_file:  present | absent
    archetype_override_semantics: delta | replace | null
    archetype_overrides:      [{role, kit_default_pin, override_pin, rationale}]
```

`team-rebalance` and `team-review` both read this block via
`decision-record.get_decision`. It is the single source of truth for a
team's configuration — there is no skill-local config file for
weights, thresholds, or pins. `inputs_snapshot.tier_scores` is what
`team-review`'s diff is computed against.

### Referenced source files

| Path | Purpose |
|---|---|
| `commands/team-create.md` | Step-by-step command process (8 steps) |
| `commands/team-review.md` | Read-only diff process |
| `commands/team-rebalance.md` | Targeted rotation process |
| `references/landscape-resolution.md` | 3-level landscape discovery rules |
| `references/tiering-formula.md` | Normalisation, thresholds, worked example |
| `references/role-archetypes.md` | Kit-default pins + responsibilities |
| `references/role-archetypes-override.md` | YAML schema + validation invariants |
| `references/team-weights-decision-schema.md` | DEC-*-TeamWeights schema + threshold validation rules |

### Extension points

- **New role archetype.** Add to `references/role-archetypes.md`,
  extend the override schema in `references/role-archetypes-override.md`
  (PM-immutability invariant must continue to apply), and add a row to
  the pin table above. The 8-archetype assumption is hard-coded in
  `team-create` step 6 — bump it carefully.
- **New scoring dimension.** Extend the weight set (currently
  `{C, K, L, G}`); update `weights` schema in `inputs_snapshot`,
  the worked example in `references/tiering-formula.md`, and the
  validation rule that weights sum to 1.0 ± 0.001.
- **New override layer.** Add a layer-N row to the OpenWeave table,
  define an audit field in `inputs_snapshot`, and document the
  precedence rule (CLI > DEC > defaults) for the new layer.

### Corner cases

- **Tier collapse (< 3 distinct tiers).** Promote the two
  highest-scoring light models to medium. Never abort — degrade
  gracefully. Recorded in DEC `inputs_snapshot.notes`.
- **Snapshot staleness > 90 days.** Warn and continue; surface the
  artifact date in the DecisionRecord rather than hiding it.
- **PM clone-cap.** `--parallelism-cap` is silently overridden to 1
  for `project-manager` regardless of CLI / archetype overrides
  (DEC-20260414_0900). Layer-4 overrides cannot raise it.
- **Reused Actor identification.** The seed Actor for an unchanged
  role is identified by `is_template: true`, NOT by name or ID
  prefix — this is the authoritative test (template clones drift but
  the seed never does).
- **Empty preferred-providers tie-break.** Two models within 0.05
  TierScore and no preferred provider supplied: pick the higher raw
  Capability score, then alphabetical model-id as final tie-break.
- **Provider neutrality.** No model name, provider name, or tier
  label may be hardcoded anywhere in this skill — all identifiers
  flow in from `model-recommender`. Hardcoding is caught by
  release-audit's `skill_structure` check on the next release.
