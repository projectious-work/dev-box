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
    layer: null
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
          args: "--subscription --providers --parallelism-cap \
--governance-floor [--weight-overrides] [--dry-run]"
          description: >
            Full team derivation. Writes 8 Role + 8 Actor + 8
            Binding entities, roster.md, and a chartering
            DecisionRecord.
        - name: team-review
          args: "[--landscape <artifact-id>] [--threshold <float>]"
          description: >
            Read-only health-check. Diffs current assignments
            against the latest landscape snapshot. No writes.
        - name: team-rebalance
          args: "--roles <list> --confirm --reason <string>"
          description: >
            Apply a team-review recommendation. Ends old Bindings,
            creates new ones, amends roster.md and DecisionRecord.
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
