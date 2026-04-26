# Team roster — v0.19.0

This file is the narrative companion to the entities under
`context/team-members/`, `context/roles/`, `context/models/`, and
`context/bindings/`. Load it at session start whenever a task needs
routing.

**Governing decisions:**
- `DEC-20260422_0233-SpryTulip` — TeamMember model + file-based tiered
  memory + A2A Agent Cards + team-manager skill (replaces actor-profile).
- `DEC-20260422_0234-BraveFalcon` — 51-role curated catalog with
  pure-ordinal seniority ladder.
- `DEC-20260422_0234-LoyalComet` — first-class Model artifacts,
  T-shirt capacity tiers, named efforts, `model-assignment` bindings,
  8-layer resolution precedence.

Earlier roster decisions
(`DEC-20260414_0900-TeamRoster-permanent-ai-team-composition`,
`DEC-20260417_1800-CapabilityProfileRouting-three-layer-model-selection`)
are **superseded** by the v0.19.0 trio above.

## Architecture (v0.19.0)

The team is now a thin overlay on three primitives:

1. **Roles** (`context/roles/`) — 51 curated organisational labels,
   grouped by function. Roles are descriptive routing labels, not
   restrictive access control. **No seniority in slugs.**
2. **Seniority** — pure ordinal attribute, ladder
   `junior → specialist → expert → senior → principal`. Carried by
   TeamMembers (default) or per-dispatch.
3. **Models** (`context/models/`) — 34 first-class Model artifacts,
   one per `(provider, family)` with versions nested. Each model
   declares an `equivalent_tier` in the T-shirt ladder
   (`xs / s / m / l / xl / xxl`).
4. **Bindings** (`context/bindings/`) — `model-assignment` bindings
   wire `(role, seniority)` → `(model, effort)`. The default binding
   pack ships 30 seeds at
   `context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml`.

Routing is a single MCP call: `model-recommender.resolve_model(role,
seniority, team_member?, scope?, task_hints?)` runs the 8-layer
precedence ladder and returns ranked candidates with rationale.

## TeamMembers — persistent identities

Persistent participants live as directory trees under
`context/team-members/<slug>/`:

```
context/team-members/<slug>/
  team-member.md     # entity (frontmatter + spec)
  persona.md         # static identity prompt
  card.json          # A2A v0.3 Agent Card (signature placeholder until crypto wired)
  knowledge/         # semantic — promoted facts
  journal/           # episodic — append-only daily log
  skills/            # procedural — recurring workflows
  relations/         # per-counterpart interaction notes
  lessons/           # A-MemGuard dual memory — distilled corrections
  private/           # developer-local; gitignored
```

Memory file frontmatter convention: `tier`, `source`, `sensitivity`,
`confidence`, `importance`, `created`, `last_reinforced`, `scope`.
Default consolidation cadence: per-task + daily journal + weekly
importance-triggered promotion.

### Current TeamMembers

| Slug | Type | Display name | Default role | Default seniority | Notes |
|---|---|---|---|---|---|
| `thrifty-otter` | human | Bernhard | `ROLE-ceo` | principal | Project owner. Migrated from `ACTOR-20260421_0144-ThriftyOtter-owner`. |

Add additional TeamMembers via
`team-manager.create_team_member(name, type, slug, default_role, default_seniority, …)`.
For `type=ai-agent`, the `name` must be drawn from the international
name pool at
`context/skills/processkit/team-manager/data/name-pool.yaml`
(team-manager enforces this with the `team.name.off_pool` warning).

### Ephemeral consultants

Ad-hoc worker invocations are **not** persisted. Dispatch via
`(role, seniority)` directly; the resolver picks the model. No
team-member entity is created for one-off work.

## Role catalog (51) — function groups

| Group | Count | Examples |
|---|---|---|
| engineering-software | 2 | `software-engineer`, `embedded-engineer` |
| platform-infra | 5 | `devops-engineer`, `site-reliability-engineer`, `cloud-engineer`, `database-engineer`, `observability-engineer` |
| data-ml | 3 | `data-scientist`, `machine-learning-engineer`, `ai-research-scientist` |
| research-rnd | 1 | `research-scientist` |
| security | 2 | `security-operations-engineer`, `security-architect` |
| qa | 1 | `qa-engineer` |
| architecture | 4 | `solutions-architect`, `enterprise-architect`, `cloud-architect`, `data-architect` |
| devrel-docs | 2 | `technical-writer`, `community-manager` |
| product-program | 5 | `product-manager`, `product-owner`, `program-manager`, `scrum-master`, `chief-of-staff` |
| design-ux | 2 | `product-designer`, `ux-designer` |
| marketing | 5 | `product-marketing-manager`, `seo-specialist`, `brand-manager`, `pr-manager`, `content-marketer` |
| sales-customer | 3 | `account-executive`, `sales-engineer`, `customer-success-manager` |
| people-hr | 2 | `recruiter`, `learning-development-manager` |
| finance | 3 | `financial-analyst`, `controller`, `treasury-analyst` |
| legal-compliance | 3 | `general-counsel`, `compliance-officer`, `data-protection-officer` |
| operations | 2 | `business-operations-analyst`, `coo` |
| executive | 5 | `ceo`, `cto`, `cpo`, `cfo`, `cmo` |
| support | 1 | `assistant` |

Full role definitions live one-per-file under `context/roles/`.

## Model capacity tiers

Provider-neutral T-shirt ladder, extensible in both directions:

| Tier | Typical use | Example models (current roster) |
|---|---|---|
| `xs` | (reserved for ultra-light future models) | — |
| `s` | minimal cost / reflexive responses | `mistral-small` |
| `m` | routine cheap work | `claude-haiku`, `gemini-2-5-flash`, `phi`, `llama-3-70b` |
| `l` | mid-capacity utility | `gemini-3-flash`, `mistral-large` |
| `xl` | strong day-to-day flagship | `gpt-4o`, `o4-mini`, `codestral`, `qwen2-5-coder-32b` |
| `xxl` | frontier / deep-reasoning | `claude-opus`, `claude-sonnet`, `gpt-5`, `gpt-5-pro`, `o3`, `gemini-2-5-pro`, `grok-4`, `deepseek-r`, `qwen3-235b` |

Each Model's `equivalent_tier` is declared in its artifact and used by
the resolver for abstract matching.

## Effort levels

`[none, low, medium, high, extra-high, max]`. Anthropic's `xhigh`
corresponds to processkit `extra-high` (aliased at the provider-adapter
boundary). `max` resolves to the model's `max_thinking_budget` field.

## Routing — how to dispatch

### Programmatic (MCP)

```
model-recommender.resolve_model(
    role="ROLE-software-engineer",
    seniority="senior",
    team_member="TEAMMEMBER-thrifty-otter",   # optional
    scope="processkit",                         # optional
    task_hints={"requires_tool_use": True},     # optional
)
→ {candidates: [{model_id, version_id, effort, rank, source_layer, rationale}, ...]}
```

### Interactive

```
/pk-explain-routing software-engineer senior
```

Renders the 8-layer trace, dropped candidates, final ranking, and
warnings.

### 8-layer precedence (high → low)

1. Task-pinned override (`task_hints.model_pin`).
2. TeamMember preference bindings.
3. Project veto bindings (hard filter).
4. Capability filter (modalities vs `task_hints.requires_*`).
5. Role + seniority bindings.
6. Role default bindings (no seniority).
7. Project bias bindings (`cost_bias`, `provider_preference`) —
   reorder, don't filter.
8. Shim fallback (`role.spec.default_model`); emits warning.

Tie-breakers within a layer: project-preferred provider, lower cost,
more recent GA version, higher reliability.

## Token-budget orientation

Driven by the project owner's subscription mix. Orientation targets
(not hard limits), expressed against the new tier ladder rather than
provider-specific names:

| Tier | Target share of session effort |
|---|---|
| `xxl` (Opus / GPT-5 / Gemini 2.5 Pro / Grok 4 / o3) | ≈ 5 % |
| `xl` + `l` (Sonnet / GPT-4o / Mistral Large / etc.) | ≈ 85 % |
| `m` + `s` (Haiku / Flash / Phi / Llama / etc.) | ≈ 10 % |

When session share deviates more than ±10 pp from a target, surface to
the owner with the driving task mix. Owner decides whether to rebalance.

## Team-manager workflows

| Task | MCP tool / command |
|---|---|
| Add a persistent team-member | `team-manager.create_team_member` |
| Reserve a name from the pool | `team-manager.reserve_name` |
| Check team consistency | `team-manager.check_all_consistency` (or `pk-doctor --category=team_consistency`) |
| Export a team-member | `team-manager.export_team_member` |
| Import a team-member from another project | `team-manager.import_team_member` |

## Adding bindings (override defaults)

Per-project overrides for routing:

```
binding-management.create_binding(
    type="model-assignment",
    subject="ROLE-product-manager",
    target="MODEL-anthropic-claude-opus",
    conditions={
        "seniority": "senior",
        "rank": 1,
        "effort_floor": "medium",
        "effort_ceiling": "extra-high",
        "rationale": "Project-specific: PM does deep planning here, prefers Opus over default Sonnet"
    }
)
```

Lower-numbered ranks win; layer 5 (role+seniority) overrides layer 6
(role default) overrides layer 8 (shim).

## Refreshing the model roster

When a new model version ships:

1. `/pk-model-refresh` updates the catalog.
2. Edit the Model artifact under `context/models/<provider>-<family>.md`
   to add a new `versions[]` entry; bump `equivalent_tier` if capacity
   shifted.
3. If a new family unlocks rerankings in the default binding pack,
   update
   `context/skills/processkit/model-recommender/default-bindings/MANIFEST.yaml`
   and re-materialise to `context/bindings/`.
4. Record a DecisionRecord (`/pk-dec`) only if the change is
   cross-cutting.
