---
name: model-recommender
description: >
  Recommend the right AI model for a task by scoring candidates across
  six dimensions (Reasoning, Engineering, Speed, Breadth, Reliability,
  Governance) and displaying a spider-chart profile.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-model-recommender
    version: "1.2.0"
    created: 2026-04-09T00:00:00Z
    category: processkit
    layer: null
    uses: []
    provides:
      primitives: []
      mcp_tools: [list_models, get_profile, compare_models, query_models, get_pricing, check_availability, get_config, set_config]
      assets: []
      processes: [model-profiling, task-routing, roster-refresh, setup-questionnaire]
    commands:
      - name: model-recommender-profile
        args: "model-id [scope]"
        description: "Show the spider-chart capability profile for the named model"
      - name: model-recommender-route
        args: "task-description"
        description: "Route a task or task plan to the optimal AI model"
      - name: model-recommender-setup
        args: ""
        description: "Run the guided questionnaire to configure model access and preferences"
      - name: model-recommender-refresh
        args: ""
        description: "Run Workflow C to research and refresh the model roster from live benchmarks"
---

# Model Recommender

## Intro

This skill scores AI models across six capability dimensions, two gate
dimensions (Availability and User Access), and per-token pricing — then
helps you pick the right model. It has three workflows: **Profile View**
(spider-chart for one or more models, with optional sub-dimension drill-down),
**Task Router** (cluster a task plan and route each cluster to the optimal
model), and **Roster Refresh** (live internet research to update benchmark
scores and discover new models). Gates always apply before capability scoring:
a model the user can't access or that is currently down is excluded regardless
of how well it scores.

## Overview

### The six dimensions

Every model and every task is evaluated against the same six axes. Each
scores 1–5.

| Dim | Symbol | What it measures | Sub-dimensions |
|-----|--------|-----------------|----------------|
| **Reasoning** | R | Hard thinking: math, science, logic, novel problem-solving | Mathematical reasoning, scientific/domain-expert reasoning, abstract/novel reasoning, multi-step logical chains, debugging chains |
| **Engineering** | E | Software work: code, tools, agents, codebases | Function-level code generation, repo-scale task completion, tool use / function calling (BFCL), agentic planning and self-correction, large-codebase navigation |
| **Speed** | S | Response latency, throughput, and cost efficiency | Time to First Token (TTFT), Inter-Token Latency (ITL), tokens/sec, cost per 1M input tokens, cost per 1M output tokens, rate limits |
| **Breadth** | B | Context window, modalities, language coverage | Context window size, long-doc faithfulness, vision, audio, video, structured output (JSON/function calling), multilingual |
| **Reliability** | L | Instruction fidelity, factual accuracy, consistency | Instruction following, hallucination rate, multi-turn consistency, safety/harmlessness, format adherence |
| **Governance** | G | Privacy, sovereignty, compliance, auditability | Data retention policy, data sovereignty / region, self-hostable / open weights, compliance certs (SOC 2, HIPAA, GDPR), prompt injection resistance |

**Score rubric:**

| Score | Label | Meaning |
|-------|-------|---------|
| 5 | Exceptional | Best-in-class or near-best; make this a primary reason to choose the model |
| 4 | Strong | Above average; reliable strength, not a risk |
| 3 | Moderate | Adequate; not a differentiator; works for routine tasks |
| 2 | Limited | Can do it, but expect trade-offs; consider alternatives |
| 1 | Minimal | Poor fit; the model is not designed for this; choose differently |

---

### Gate dimensions (applied before capability scoring)

Gates are binary — they disqualify a model entirely, not partially. Apply
them first; only models that pass both gates are scored on the six dimensions.

**Gate 1 — User Access:** Does the user have an active subscription, API key,
and sufficient quota for this model? Ask at the start of any routing session
if unknown. Maintain user access state in `mcp/user_config.json` via
`set_config(available_models=[...])`. The MCP `query_models` and `list_models`
tools respect this automatically.

**Gate 2 — Availability:** Is the provider's API currently operational? Check
with `check_availability()` for live status. Three sub-states:
- `operational` — no known issues
- `degraded` — elevated error rate or latency; usable but risky for production
- `major_outage` — do not route here; escalate to fallback

**Additional flags to surface when relevant:**
- Rate limit exhausted — user has hit their per-minute or daily cap; fallback required
- Quota / budget exceeded — subscription limit reached; model is effectively unavailable
- Subscription expired — no access until renewed

When a model's gate status is unknown, ask the user rather than assuming it passes.

---

### Cost and value

Cost is a sub-dimension of Speed (S.cost_efficiency) in the spider chart but
is also surfaced explicitly because raw per-token prices and derived value
matter independently of speed.

**Per-token pricing:** stored in `mcp/model_scores.json` under each model's
`pricing` field. Use `get_pricing()` to view and sort. Self-hosted models
(Llama) have null API pricing — infra cost applies instead.

**Value score:** `(R + E + L) / 3 / output_cost_per_1M × 10`. Higher is
better. Surfaces models with frontier-class capability at low cost. DeepSeek
models score highest on value but are excluded for G-sensitive work.

| Model | Input /1M | Output /1M | Value score | G |
|-------|-----------|------------|-------------|---|
| Gemini Flash 2.0 | $0.08 | $0.30 | ~43 | 2 |
| DeepSeek V3 | $0.14 | $0.28 | ~50 | 1 |
| Claude Haiku 4.5 | $0.25 | $1.25 | ~11 | 5 |
| DeepSeek R1 | $0.55 | $2.19 | ~15 | 1 |
| o4-mini | $1.10 | $4.40 | ~5 | 2 |
| Mistral Large 3 | $2.00 | $6.00 | ~3 | 4 |
| Claude Sonnet 4.6 | $3.00 | $15.00 | ~1.5 | 5 |
| Gemini 2.5 Pro | $1.25 | $10.00 | ~2 | 2 |
| GPT-4o | $2.50 | $10.00 | ~1.4 | 2 |
| o3 | $10.00 | $40.00 | ~0.5 | 2 |
| Claude Opus 4.6 | $15.00 | $75.00 | ~0.3 | 5 |
| Llama 3.3 70B | self-hosted | self-hosted | — | 5 |

When the user asks "what's the best value for money" or has a per-hour budget,
use `get_pricing(sort_by="value_score")` and filter by accessible models.

---

### MCP tools

The `mcp/server.py` provides structured queries over the roster. Use these
instead of reading markdown files when the user asks for comparisons,
filtered lists, or pricing analysis.

| Tool | When to use |
|------|-------------|
| `list_models()` | First step when access config is unknown; shows what's usable |
| `query_models(R=4, G=5)` | "Find models with strong reasoning and full privacy" |
| `get_profile("claude-sonnet-4.6", scope="E")` | Sub-dimension drill-down on Engineering |
| `compare_models(["claude-sonnet-4.6", "gemini-2.5-pro"], scope="B")` | Side-by-side Breadth sub-dims |
| `get_pricing(sort_by="value_score")` | Value-for-money ranking |
| `check_availability()` | Live status before routing a plan |
| `get_config()` / `set_config(...)` | Show or update user access list |

---

### Workflow A — Profile View

Use when the user wants to understand a specific model or compare two.

**Output format (render one block per model):**

```
Model: Claude Sonnet 4.6
Provider: Anthropic · Tier: Frontier mid-size · Updated: 2026-Q1
────────────────────────────────────────────────
  Reasoning    ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
────────────────────────────────────────────────
Best for:   Complex coding, code review, refactoring, agentic workflows,
            tasks touching sensitive data or enterprise privacy requirements
Avoid for:  Extreme cost sensitivity at very high volume, real-time <100ms
            UX, native audio/video processing
```

Bar widths: 5→▓▓▓▓▓▓▓▓▓▓, 4→▓▓▓▓▓▓▓▓░░, 3→▓▓▓▓▓▓░░░░, 2→▓▓▓▓░░░░░░, 1→▓▓░░░░░░░░

For head-to-head comparison of two models, render both blocks, then add a
one-paragraph verdict: which model wins overall and for what use cases.

**Sub-dimension drill-down:** When the user wants detail on a specific
dimension (e.g., "which of these two is better at tool use?"), call
`compare_models([...], scope="E")` and render the sub-dimension table:

```
Engineering sub-dimensions: Claude Sonnet 4.6 vs GPT-4o
─────────────────────────────────────────────────────
Sub-dimension          Sonnet 4.6   GPT-4o
Function codegen            5          4
Repo-scale tasks            5          4
Tool use / BFCL             5          4
Agentic planning            5          3
Codebase navigation         5          4
─────────────────────────────────────────────────────
Top-level E score           5          4
```

Trigger: "drill down on [dimension]", "compare at the detail level",
"which is better at [sub-dimension]", "I need strong tool use specifically".

---

### Workflow B — Task Router

Use when the user has a plan, task list, or backlog and wants to know
which model to use for which work. This is the Dispatch matching mode:
tasks are missions, models are heroes.

**Steps:**

1. **Parse** the task list. Accept any format: markdown bullets, numbered
   list, WorkItems, or a free-form plan.

2. **Score each task** against the six dimensions: which dimensions does
   this task *require*? Use the Task Scoring Quick-Reference below.

3. **Cluster** tasks by their dominant dimension profile. Tasks with the
   same top-2 dimensions belong in the same cluster. Typical clusters:
   - Deep-Think (R+E dominant) — complex architecture, novel algorithms
   - Production-Coder (E dominant) — routine implementation, bug fixes
   - High-Volume (S dominant) — repetitive generation, bulk transforms
   - Long-Context (B dominant) — large codebase sweeps, doc analysis
   - Privacy-First (G dominant) — PII, PHI, regulated data, secrets

4. **Recommend** a model per cluster from the user's accessible models.
   Call `query_models(R=..., E=..., G=..., apply_user_filter=True)` to
   find the best available match. State the primary recommendation and one fallback.

5. **Theoretical-best hint.** After recommending from the user's accessible
   models, run the same query with `apply_user_filter=False`. If the
   theoretical-best model differs from the recommended one, show the hint:

   ```
   ⚡ Theoretical best (not in your access): Claude Opus 4.6
      Gap: R:5 E:5 vs your best R:4 E:5 — 1 point on Reasoning matters for
      this cluster (novel algorithm design). Consider adding Opus access if
      the task justifies it → anthropic.com/api
   ```

   Omit the hint if the user's accessible model already matches the theoretical
   best, or if the gap is only 1 point on a non-dominant dimension.

6. **Output** the routing table:

```
Task Routing Analysis
══════════════════════════════════════════════

Cluster 1 — Deep-Think (Reasoning + Engineering)   [N tasks]
  Profile:    R:5  E:5  S:2  B:3  L:4  G:4
  Your model: Claude Sonnet 4.6  (fallback: Qwen 2.5 Coder 32B self-hosted)
  ⚡ Theoretical best: Claude Opus 4.6 — R gap: 4→5; worth it for novel algorithms
  Tasks:
    • Design consensus algorithm for distributed cache
    • Refactor auth middleware to zero-trust model

Cluster 2 — Production-Coder (Engineering)          [N tasks]
  Profile:    R:3  E:5  S:3  B:3  L:4  G:4
  Your model: Claude Sonnet 4.6  (no gap — this is the theoretical best)
  Tasks:
    • Implement JWT refresh token rotation
    • Add pagination to /api/v2/users endpoint

Cluster 3 — High-Volume (Speed)                     [N tasks]
  Profile:    R:2  E:3  S:5  B:3  L:3  G:3
  Your model: Claude Haiku 4.5  (fallback: Gemini Flash 2.0)
  ⚡ Theoretical best: Gemini Flash 2.0 — slightly cheaper at this volume;
     only matters if processing >10M tokens/month
  Tasks:
    • Generate unit test stubs for all 200 endpoints
    • Reformat 5,000 changelog entries to new template

Cluster 4 — Privacy-First (Governance)              [N tasks]
  Profile:    R:3  E:3  S:3  B:3  L:4  G:5
  Your model: Llama 3.3 70B self-hosted  (no gap — self-hosted is optimal)
  Tasks:
    • Process patient consent records
    • Summarize HIPAA audit logs

══════════════════════════════════════════════
Total: N tasks across M clusters.
Suggested sequence: Cluster 4 → Cluster 1 → Cluster 2 → Cluster 3.
```

---

---

### Workflow D — Setup Questionnaire

Use when the user says "set up my models", "configure my access", "which models do
I have access to", "help me set up the recommender", "onboard me", or "I only have
access to X". Replaces manual JSON editing with a guided conversation.

**Steps:**

1. **Show current state.** Call `get_config()`. If `available_models` is non-empty,
   say "You currently have [N] models configured — I'll update your settings."
   If empty, say "Your model access isn't set up yet. Let me walk you through it."

2. **Ask about provider access.** One question, accept a free-form answer:
   > "Which AI providers do you have active API access to?
   > Options: Anthropic, OpenAI, Google (Gemini), Meta/self-hosted (Llama),
   > Mistral, xAI (Grok), Cohere, Alibaba/Qwen, MiniMax, DeepSeek, Microsoft (Phi),
   > or other."

3. **For each confirmed provider, ask about model tiers:**
   - Anthropic: "Do you have Haiku only, Haiku + Sonnet, or full access (all three)?"
   - OpenAI: "Standard (GPT-4o), reasoning tier (o3/o4-mini), or both?"
   - Google: "Gemini 2.5 Pro, Flash, or both?"
   - Open-source (Llama/Qwen/Phi/Gemma): "Self-hosted or via a third-party API
     (Together, Groq, etc.)?" — this determines the effective G score.
   - Others: accept the provider-level answer.

4. **Data sensitivity floor.** One question:
   > "What's the highest sensitivity of data you typically work with in this project?
   > (a) Public only — open-source code, public information
   > (b) Internal / proprietary — code or business data that isn't public
   > (c) Personal data — names, emails, addresses, any PII
   > (d) Regulated — HIPAA (medical), financial, legal, or government data"
   Map: (a)→G:0, (b)→G:3, (c)→G:4, (d)→G:5.

5. **Budget preference.** One question:
   > "Budget preference?
   > (a) Cost-first — use the cheapest model that meets the requirement
   > (b) Balanced — trade off cost and quality
   > (c) Quality-first — use the best model regardless of cost"
   Map: (a)→low, (b)→medium, (c)→high. Set `budget_tier`.

6. **Exclusions.** One question:
   > "Any models to always exclude? (e.g., 'no DeepSeek', 'nothing from China',
   > 'only Anthropic')"
   Parse response and add to `blocked_models`.

7. **Summarise and confirm.** Display a brief summary:
   ```
   Here's your configuration:
     Accessible models: [list]
     Governance floor:  G:4 (personal data)
     Budget:            balanced
     Blocked:           deepseek-v3, deepseek-r1
   Apply this? (yes / adjust)
   ```

8. **Apply.** On confirmation, call `set_config(...)` with all fields.
   Then call `list_models()` and show the effective roster so the user can
   verify it looks right.

---

### Workflow C — Roster Refresh

Use when the user says "update the model scores", "check for new models",
"are there newer benchmarks", or "refresh the roster". This is the only
workflow that requires internet access.

**Steps:**

1. **Check validated date.** Read `_meta.validated` from `model_scores.json`.
   If less than 3 months old, ask the user if they want to proceed anyway.

2. **Discover new models.** Web-search for "new LLM models [current year]" and
   "AI model releases [last 3 months]". For each new model found:
   - Note it as a candidate addition (do NOT auto-write to model_scores.json)
   - Report it to the user with a one-line capability summary
   - Ask: "Do you want me to add [model] to the roster?"
   - Only add after explicit confirmation; adds are additive (never overwrite existing entries)

3. **Refresh benchmark scores.** Fetch current leaderboard pages listed in
   `_meta.leaderboards`. For each model already in the roster:
   - Check if its SWE-bench rank has changed significantly
   - Check if its LMSYS Arena Elo has moved by >50 points
   - Check Artificial Analysis for latency/cost updates
   - Propose score changes as a diff — do NOT auto-apply

4. **Check for new benchmarks.** Search for benchmarks that have gained
   traction since the last validated date (e.g., new agentic evals, new
   multimodal evals). If found, assess whether they warrant a new sub-dimension.
   Report to user; do NOT add sub-dimensions without discussion.

5. **Check pricing.** Fetch the provider pricing pages in `_meta.pricing_pages`.
   Compare to stored `pricing` fields. Report any price changes.

6. **Present proposed changes.** List:
   - New models to add (awaiting confirmation)
   - Score adjustments (with before/after and source)
   - Price updates (before/after)
   - New benchmark candidates

7. **Apply only confirmed changes.** Update `model_scores.json` for approved
   changes. Bump `_meta.validated` to the current quarter.

**Constraint:** refresh is additive. Never delete models from the roster
during refresh — that is a separate cleanup task for the user to do manually.

### Task Scoring Quick-Reference

| Task contains… | Raise these dimensions |
|---|---|
| "design", "architect", "algorithm", "prove", "derive", "optimize (complexity)" | R |
| "implement", "fix", "debug", "refactor", "review code", "write tests", "agentic" | E |
| "generate N items", "bulk", "batch", "fast", "cheap", "thousands of" | S |
| "entire codebase", "all files", "image", "screenshot", "audio", "video", "long doc" | B |
| "exactly", "format must be", "strict schema", "no hallucination", "cite sources" | L |
| PII, PHI, credentials, regulated, GDPR, HIPAA, internal/confidential | G |

A task can score high on multiple dimensions. When in doubt, assign the
top-2 and cluster on those.

---

### Model roster summary

The roster contains **34 models** across 12 providers (25 validated, 9 estimated
for post-Aug-2025 releases). Estimated models are marked `_estimated: true` in
`mcp/model_scores.json` and should be validated via Workflow C before using in
production routing.

**Live table:** `list_models()` — returns the roster filtered by your user config.  
**Full table with context windows:** see `references/roster-quick-ref.md`.  
**Narrative profiles:** see `references/model-profiles.md`.

**Providers covered:** Anthropic, OpenAI, Google (API + open weights),
xAI (Grok), Meta (Llama), Alibaba/Qwen (open), Microsoft/Phi (open),
MiniMax, Mistral (all tiers + Codestral + Deep Think), Cohere, DeepSeek.

**⚠️ G:1 warning:** DeepSeek APIs, Alibaba Cloud API, and MiniMax API operate
under Chinese jurisdiction. Never route sensitive, regulated, or personal data
to these APIs. Self-hosting their open weights raises governance to G:3–4.

---

### Commands

- `/model-recommender-profile model-id [scope]` — show the spider-chart capability profile for a model
- `/model-recommender-route task-description` — route a task or task plan to the optimal AI model
- `/model-recommender-setup` — run the guided questionnaire to configure model access and preferences
- `/model-recommender-refresh` — run Workflow C to research and refresh the model roster from live benchmarks

## Gotchas

- **Skipping the gate check before routing.** Always check User Access and
  Availability before scoring capability. A model scoring E:5 G:5 is useless
  if the user's API key is expired or the provider is in a major outage.
  Call `check_availability()` and `get_config()` at the start of any
  Task Router session, or ask the user explicitly if MCP is unavailable.

- **Assuming access from context.** If the user mentions "Claude" it does
  not mean they have Opus, Sonnet, and Haiku — they may only have one tier.
  Ask or call `get_config()` before recommending a specific tier.

- **Recommending a model the user has rate-limited.** Rate limits and quota
  exhaustion are not visible on status pages — they are per-account states.
  If the user says "it keeps failing", check for rate-limit errors before
  re-routing to the same model. Suggest the next model in the fallback chain.

- **Conflating low cost with high value.** DeepSeek models have the highest
  raw value score but score G:1, which disqualifies them for most enterprise
  work. Always surface the governance warning alongside the value score when
  a low-cost model has a G:1 or G:2 rating. Cost optimization within a G
  floor is the correct framing, not raw cost minimization.

- **Not telling the user the actual price.** When recommending a model for
  a bulk task ("generate 10,000 descriptions"), compute an estimate:
  (estimated tokens × price per 1M / 1,000,000). Even a rough estimate
  ($0.50 vs. $8.00) changes the decision. Use `get_pricing()` for current rates.

- **Treating the roster as current truth.** Model scores change with every
  release. The profiles in `references/model-profiles.md` have a
  "validated" date. If the user's model is newer or the date is >6 months
  old, caveat your recommendation and point to Artificial Analysis or
  LMSYS Arena for live data.

- **Recommending a model the user can't access.** Before recommending o3
  or Opus 4.6, ask (or check context) whether the user has access and
  budget. A perfect score on paper is useless if the quota is exhausted or
  the model is not provisioned. Offer a concrete fallback every time.

- **Scoring the task instead of the task cluster.** When routing a plan,
  assess what the *cluster as a whole* needs, not individual task quirks.
  One unusual subtask should not pull an entire cluster to a different model.

- **Ignoring the Governance dimension for "internal" work.** Many teams
  assume internal code is not sensitive. But source code with credentials,
  unreleased algorithms, or regulated business logic can be just as
  sensitive as PII. When in doubt, ask whether the organization has an AI
  data classification policy before routing to non-sovereign models.

- **Conflating Speed with Engineering.** A fast model (S:5) is not
  necessarily a good coder (E:5). Haiku and Flash are excellent at
  high-volume, low-complexity code tasks but will struggle with novel
  architecture or debugging subtle async races. Always check both dimensions
  before routing engineering work to a speed-optimized model.

- **Presenting scores as objective benchmarks.** The 1-5 scores in this
  skill are informed calibrations, not direct benchmark readings. When the
  user needs precision (e.g., choosing between two models scoring 4 vs. 4
  on the same dimension), surface the underlying benchmarks
  (SWE-bench Verified, GPQA Diamond, BFCL v4) and link to current
  leaderboards. The skill's scores are a starting point, not the final word.

- **Over-routing to expensive models.** The task router should push work
  toward the cheapest model that meets the required profile, not the highest-
  scoring model globally. Opus 4.6 is not the right answer for a task that
  only needs E:3 S:4 — that is Haiku or Sonnet territory.

- **Skipping the sequence suggestion.** After clustering, always add a
  recommended execution sequence. Some clusters are prerequisites for others
  (e.g., design decisions made with Opus should precede implementation with
  Sonnet). The sequence is often more valuable than the per-cluster picks.

- **Calling `set_config` without user confirmation.** `set_config` overwrites `user_config.json` in full — it is marked `destructiveHint: true`. Always show the proposed configuration to the user and wait for explicit approval before calling `set_config`. Never chain `set_config` into a batch of other calls.

- **Treating `check_availability` failures as errors.** `check_availability` makes live HTTP requests to provider status pages. Network failures, rate limits, and subscription expiry are not detectable from status pages. Treat any failure or non-operational status as `unknown`, report it to the user, and do not retry more than once without explicit user awareness.

## Full reference

### Sub-dimension detail

Full sub-dimension specs, benchmark mappings, and scoring guidelines are
in `references/dimension-specs.md`. For structured sub-dimension comparison,
use `compare_models(model_ids, scope="R")` (or E/S/B/L/G).

### Complete model profiles

Narrative descriptions, per-sub-dimension scores, and best-for/avoid-for
lists for all twelve models are in `references/model-profiles.md`.
Structured scores are in `mcp/model_scores.json` — the MCP server is the
preferred interface for queries; the markdown file is the human-readable view.

### Pricing and value analysis

- `get_pricing(sort_by="value_score")` — ranked by capability per dollar
- `get_pricing(sort_by="output_per_1m")` — ranked cheapest output first
- Provider pricing pages are in `mcp/model_scores.json` under `_meta.pricing_pages`
- For bulk cost estimates: tokens × (output_price / 1,000,000)
- Self-hosted models (Llama): no API cost, but require GPU infra (~$1–3/hr/A100)

### Availability and user access

- `check_availability()` — fetches live status from provider status pages
- `get_config()` / `set_config(...)` — manage user access list and governance floor
- Provider status pages: `mcp/model_scores.json` under `_meta.status_pages`
- Rate limits and quota: must be checked manually via the provider's API dashboard
  (Anthropic: console.anthropic.com, OpenAI: platform.openai.com, etc.)

### Dimension trade-off map

Common trade-offs to surface when the user is torn between two options:

| Trade-off | Typical tension | Resolution heuristic |
|---|---|---|
| Reasoning vs. Speed | o3 vs. Haiku | Choose by task complexity: reasoning chains >5 steps → o3 tier; routine → Haiku |
| Breadth vs. Governance | Gemini 2.5 Pro vs. Llama | If context >200K AND data is non-sensitive → Gemini; otherwise → Llama + chunking |
| Engineering vs. Governance | GPT-4o vs. Mistral | If task is routine coding AND data is non-sensitive → GPT-4o; regulated → Mistral or Llama |
| Speed vs. Reliability | Flash vs. Sonnet | For customer-facing output → Sonnet; for internal draft generation → Flash |
| Cost vs. Capability | DeepSeek vs. Claude | If data is non-sensitive AND cost is paramount → DeepSeek; otherwise avoid |

### How scores were derived

Scores are calibrated from:
- Public benchmarks: SWE-bench Verified (E), GPQA Diamond (R), MATH/AIME (R),
  BFCL v4 (E+B), LMSYS Arena Elo (L), Artificial Analysis Intelligence Index (R+E)
- Provider documentation and model cards for Speed and Governance dimensions
- Practitioner reports and Artificial Analysis latency/cost benchmarks

### Keeping profiles current

When a major new model releases or an existing model is updated:
1. Add or update its entry in `references/model-profiles.md`
2. Update the summary table in SKILL.md Overview
3. Bump the validated date in model-profiles.md
4. If scores change significantly, note what changed and why

### Anti-patterns

- **Single-dimension routing.** Never pick a model on one dimension alone.
  A model that excels at reasoning but scores L:2 on reliability will
  produce confident, wrong answers. Always check the dimensions that matter
  for the task's tolerance for error.
- **Assuming "best overall model" = right model.** The highest-ranked model
  on LMSYS Arena is not always the right pick. A task requiring G:5 rules
  out the entire LMSYS top tier (all score G:2 or lower). Optimize for the
  user's actual constraints, not global rankings.
- **Ignoring fallbacks.** Always provide a fallback. The recommended model
  may be unavailable, rate-limited, or over budget mid-session. A router
  with no fallback fails the user at the worst moment.
- **Not re-routing when a cluster grows.** If a new task is added to the
  plan that doesn't fit the cluster's profile, surface it rather than
  silently absorbing it. A Governance-sensitive task silently absorbed into
  a DeepSeek-routed cluster is a data breach waiting to happen.
