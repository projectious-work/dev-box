# model-recommender MCP server

Structured query engine for the model roster. Provides capability scoring,
sub-dimension drill-down, comparison, user access configuration, and live
provider status checks.

## Tools

| Tool | Purpose |
|---|---|
| `list_models(apply_user_filter?)` | List all models with top-level scores; respects user config by default |
| `get_profile(model_id, scope?)` | Full or scoped profile; scope: "summary", "full", "R", "E", "S", "B", "L", "G" |
| `compare_models(model_ids, scope?)` | Side-by-side comparison; scope "top_level" or dimension letter for sub-dim drill-down |
| `query_models(R?, E?, S?, B?, L?, G?, sub_R?, sub_E?, sub_B?, apply_user_filter?, limit?)` | Find models meeting minimum scores, ranked by fit |
| `get_pricing(model_ids?, sort_by?)` | Per-token pricing + value score (capability per dollar); sort by output cost or value |
| `check_availability(model_ids?)` | Fetch provider status pages; returns operational / degraded / major_outage / unknown |
| `get_config()` | Show current user configuration (access list, governance floor, budget, preferences) |
| `set_config(available_models?, blocked_models?, require_governance_min?, budget_tier?, preferred_providers?)` | Update user configuration |

## Storage

| File | Purpose |
|---|---|
| `model_scores.json` | Source of truth: all model profiles with sub-dimension scores, pricing, metadata |
| `user_config.json` | User's access list, blocked models, governance floor, budget tier, preferred providers |

## Running

```bash
uv run context/skills/model-recommender/mcp/server.py
```

## Key design decisions

**Governance ceiling is enforced by `query_models` and `_apply_user_filter`.**
A model that fails `require_governance_min` is excluded from all query results
even if it scores perfectly on other dimensions. This is intentional.

**`check_availability` is best-effort.** It fetches Atlassian statuspage.io
`/api/v2/status.json` endpoints. Rate limits, quota exhaustion, and subscription
expiry cannot be detected from status pages — those must be checked manually
via the provider's API dashboard.

**`apply_user_filter=False`** bypasses user_config filtering. Use this when you
want to see the full roster regardless of what the user has configured.

**Sub-dimension requirements** in `query_models` are supported for R, E, and B
(the most commonly filtered). S, L, G sub-dims can be filtered post-query using
`get_profile` if needed.

## Adding new models

1. Add an entry to `model_scores.json` following the existing schema.
2. Score all 6 dimensions and all sub-dimensions.
3. Add `pricing` fields (see schema in model_scores.json).
4. Bump `_meta.validated` to the current quarter.
5. Update `references/model-profiles.md` with the narrative profile.
6. Update the summary table in `SKILL.md` Overview.
