# Role Archetypes — Full Reference

## The 8 processkit role archetypes

| Role archetype | Tier pin | Rationale | Override-when | `primary_contact` | `clone_cap` | `cap_escalation` |
|---|---|---|---|---|---|---|
| **project-manager** | heavy | Routing quality compounds; every routing error propagates to all downstream work | **Never override.** PM is always heavy; always 1 clone (no parallelism). | `true` | `1` | `"owner"` |
| **senior-architect** | heavy | Cross-subsystem design; blast radius is high; wrong architecture poisons the entire codebase | Medium only if no heavy candidate clears G-floor AND owner explicitly approves in writing. | `false` | `5` | `"owner"` |
| **senior-researcher** | heavy | Multi-source synthesis; wrong synthesis poisons downstream decisions at scale | Same as senior-architect. | `false` | `5` | `"owner"` |
| **junior-architect** | medium | Single-module scope; bounded blast radius | Heavy if median capability gap between medium and heavy tiers exceeds 15pp on SWE-bench Verified for the accessible candidate set. | `false` | `5` | `"owner"` |
| **developer** | medium | Implementation against a written plan; bounded scope | Heavy for security-critical or regulated subsystems (owner sets `--security-critical` flag). | `false` | `5` | `"owner"` |
| **junior-researcher** | medium | Bounded single-topic research; output reviewed by senior-researcher | No override. | `false` | `5` | `"owner"` |
| **junior-developer** | light | Well-specified single-file edits; no architecture decisions | Medium if no light-tier candidate is accessible (escalation fallback). | `false` | `5` | `"owner"` |
| **assistant** | light | Admin, formatting, summarisation, file management | No override. If no light model is accessible, share the junior-developer model (same ACTOR entity, same Binding target). | `false` | `5` | `"owner"` |

## Override rules in detail

### Tier upgrade: light → medium (junior-developer)

Applied automatically when the accessible candidate set yields no
light-tier models (TierScore < 0.40). The skill selects the lowest-
scoring medium model for junior-developer to minimise cost.

### Tier upgrade: light → light (assistant shares junior-developer)

If no light-tier model is accessible, assistant does not get a
separate Actor. Instead, create a single Binding from
`ACTOR-<junior-developer-alias>` to `ROLE-assistant`. The DecisionRecord
notes this as a shared-model arrangement.

### Tier downgrade: heavy → medium (senior-architect, senior-researcher)

Permitted only when:
1. No heavy-tier candidate passes the G-floor gate, AND
2. The owner provides explicit written approval (supply as
   `--override-reason "<text>"` argument to `team-create`).

The downgrade is recorded in the chartering DecisionRecord under
`spec.override_exceptions`.

### Tier upgrade: medium → heavy (junior-architect)

Applied automatically when:
- The gap between the best medium-tier model's SWE-bench Verified
  score and the best heavy-tier model's score exceeds 15 percentage
  points, AND
- At least one heavy-tier model is accessible.

Rationale: at this gap size, assigning junior-architect to a medium
model means accepting materially inferior architectural output for
single-module scope work. The cost increment is justified.

### Tier upgrade: medium → heavy (developer — security flag)

When `--security-critical` is set, developer is treated as heavy-pinned
for that run. This is appropriate for regulated or security-sensitive
subsystems. Record in DecisionRecord `override_exceptions`.

## Parallelism cap

`--parallelism-cap` (default 5) applies to all roles **except**
`project-manager`, which is always capped at 1. The rationale is in
`DEC-20260414_0900`: PM routing quality requires a single authoritative
decision-maker; parallel PMs produce conflicting routing decisions.

Values > 5 require explicit owner approval supplied via
`--override-reason "parallelism > 5 approved for <reason>"` and are
recorded in the DecisionRecord.

## Tier-collapse rules

If the accessible model set produces fewer than 3 distinct tiers:

| Collapse scenario | Resolution |
|---|---|
| Only heavy + medium (no light) | Promote the 2 lowest-scoring medium models to "effective light." |
| Only heavy + light (no medium) | Promote the 2 highest-scoring light models to medium. |
| Only one tier (e.g. all medium) | Treat the top ⅓ as heavy, middle ⅓ as medium, bottom ⅓ as light by rank. |

The skill never fails on tier-collapse. It degrades gracefully and
records the collapse scenario in the DecisionRecord `inputs_snapshot`.

## Role responsibilities reference

Brief one-liners for `role-management.create_role` calls. These are
starting points; projects may extend them.

| Role | responsibilities (imperative bullets) |
|---|---|
| project-manager | Route all tasks to appropriate actors; maintain roster.md; flag team drift; resolve role conflicts |
| senior-architect | Own cross-subsystem design decisions; review all architecture PRs; define subsystem interfaces |
| senior-researcher | Synthesise multi-source research; validate research output from junior-researcher; own landscape snapshots |
| junior-architect | Design within assigned module; produce design docs for senior-architect review |
| developer | Implement tasks from design docs; write unit tests; resolve failing CI |
| junior-researcher | Research bounded single topics; produce structured summaries for senior-researcher review |
| junior-developer | Implement well-specified single-file edits; run linters; update changelogs |
| assistant | Format documents; manage file moves; write standup notes; perform admin tasks |

## Template vs clone

The 8 seed Actors emitted by `team-create` are **templates**
(`is_template: true`, `templated_from: null`). They represent the
canonical team roster — one Actor per archetype. When
`team-rebalance` spawns a new Actor to fill a role (replace or add),
that spawned Actor is a **clone**: `is_template: false` and
`templated_from: <seed-actor-id>` pointing at the template it
derives from. This distinction lets the system separate "the 8
authoritative team members" from "task-specific parallel instances"
when querying the actor index.

## Provider-neutrality invariant

No model name, provider name, or tier label (Opus/Sonnet/Haiku or
equivalents) may appear in this reference file or in any command. The
archetype → tier mapping is abstract. Model identifiers are always
resolved at runtime by `model-recommender` and stored in the
DecisionRecord. This invariant enables the same team composition
logic to operate against any provider mix.
