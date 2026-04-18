# role-archetypes.yaml Override — Reference

## Purpose

A project may supply a `context/team/role-archetypes.yaml` file to
remap tier pins and clone caps for some or all of the 8 processkit
role archetypes. `team-create` loads this file before archetype
mapping (Step 4) and validates it eagerly — before model scoring —
so violations fail fast.

## File location

```
context/team/role-archetypes.yaml
```

The file must be committed to the project repository. If absent,
kit defaults from `references/role-archetypes.md` apply unchanged.

## Full schema

```yaml
version: "1"                     # schema version; currently must be "1"
override_semantics: delta         # "delta" | "replace"  (default: delta)

roles:
  <archetype-name>:               # must match one of the 8 kit archetype names
    tier_pin: heavy|medium|light  # REQUIRED per entry
    rationale: |                  # REQUIRED — non-empty; must be project-specific
      <why this project overrides the kit default for this role>
    clone_cap_override: <int|null>  # null = inherit project --parallelism-cap
    override_when: []             # optional; inherits kit rules if omitted
```

Valid `<archetype-name>` values (exactly these 8):
`project-manager`, `senior-architect`, `senior-researcher`,
`junior-architect`, `developer`, `junior-researcher`,
`junior-developer`, `assistant`

## Override semantics

**`delta` (default):** Only the roles listed are overridden. All
other archetypes inherit kit defaults unchanged. Use this when a
project needs to change one or two roles — no need to specify the
full set.

**`replace`:** The file is the complete archetype table. All 8 role
archetypes must appear; missing roles are a hard validation error.
Required only when a project needs to restructure the full set. The
kit's `references/role-archetypes.md` is ignored entirely in
`replace` mode.

## Validation invariants (eager — checked at team-create startup)

The following are hard errors, checked before model scoring begins:

| Invariant | Error message |
|---|---|
| `project-manager.tier_pin != "heavy"` | "PM tier_pin must remain heavy — immutable per DEC-20260414_0900" |
| `project-manager.clone_cap_override > 1` | "PM clone_cap_override must be null or 1 — immutable per DEC-20260414_0900" |
| Any `rationale` is empty or absent | "Role <name>: rationale is required in role-archetypes.yaml override" |
| `replace` mode with fewer than 8 archetypes | "replace mode requires all 8 archetypes; missing: <list>" |
| `clone_cap_override > --parallelism-cap` | "Role <name>: clone_cap_override <N> exceeds project parallelism-cap <M>" |

The following is a warning (logged to DecisionRecord, not an error):

| Condition | Warning |
|---|---|
| Two or more roles share `tier_pin: heavy` but the accessible heavy-tier set has fewer models than heavy-pinned roles | "Heavy tier has <N> models but <M> roles are pinned heavy — some roles will share a model" |

## Examples

### Delta override (one role remapped)

```yaml
version: "1"
override_semantics: delta
roles:
  senior-architect:
    tier_pin: medium
    rationale: |
      This edge-deployment project has no accessible heavy-tier model
      with G-score ≥ 4 under the target subscription. Owner approved
      downgrade to medium per project charter 2026-Q2.
    clone_cap_override: null
```

### Replace override (full archetype table)

```yaml
version: "1"
override_semantics: replace
roles:
  project-manager:
    tier_pin: heavy       # immutable — must remain heavy
    rationale: "Kit default retained — PM routing quality requirement unchanged."
    clone_cap_override: 1
  senior-architect:
    tier_pin: medium
    rationale: "Research lab does not need frontier models for architecture review."
    clone_cap_override: null
  senior-researcher:
    tier_pin: heavy
    rationale: "Multi-source synthesis is the lab's primary output — max capability."
    clone_cap_override: null
  junior-architect:
    tier_pin: medium
    rationale: "Module-scoped design; medium capability is sufficient."
    clone_cap_override: null
  developer:
    tier_pin: medium
    rationale: "Implementation against detailed specs; medium capability is sufficient."
    clone_cap_override: null
  junior-researcher:
    tier_pin: medium
    rationale: "Bounded research scope; medium capability matches task complexity."
    clone_cap_override: null
  junior-developer:
    tier_pin: light
    rationale: "Kit default retained — single-file edits do not require upgrade."
    clone_cap_override: null
  assistant:
    tier_pin: light
    rationale: "Kit default retained — admin tasks do not require capability upgrade."
    clone_cap_override: null
```

## Audit trail

When `context/team/role-archetypes.yaml` is present, `team-create`
records in the chartering DecisionRecord:

```yaml
inputs_snapshot:
  archetype_override_file: "present"      # or "absent"
  archetype_override_semantics: delta     # or "replace"
  archetype_overrides:
    - role: senior-architect
      kit_default_pin: heavy
      override_pin: medium
      rationale: "<first sentence of rationale>"
    # ... one entry per overridden role
```

This ensures every run's configuration is fully reconstructable from
its chartering DecisionRecord alone.
