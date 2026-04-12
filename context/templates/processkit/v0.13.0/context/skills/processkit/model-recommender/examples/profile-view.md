# Example: Profile View Output

**Input:** "Compare Claude Sonnet 4.6 vs Gemini 2.5 Pro for my use case — I
need to process large codebases and explain architecture, but we work with
internal proprietary code."

---

## Model Profiles

```
Model: Claude Sonnet 4.6
Provider: Anthropic · Tier: Frontier mid-size · Context: 200K tokens
────────────────────────────────────────────────────────────
  Reasoning    ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
────────────────────────────────────────────────────────────
Best for:   Complex coding, code review, refactoring, agentic workflows,
            tasks touching sensitive or proprietary code
Avoid for:  Native audio/video, context requirements above 200K tokens

Model: Gemini 2.5 Pro
Provider: Google DeepMind · Tier: Frontier flagship · Context: 1M+ tokens
────────────────────────────────────────────────────────────
  Reasoning    ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓░░░░░░  2/5  Limited
────────────────────────────────────────────────────────────
Best for:   Huge context sweeps (entire repos), multimodal, tasks on
            public or non-sensitive data where Breadth is the constraint
Avoid for:  Proprietary code, internal systems, regulated data (G:2)
```

## Head-to-Head Verdict

**For your use case: use Claude Sonnet 4.6.**

Gemini 2.5 Pro's 1M+ context window is genuinely better for sweeping entire
large codebases in a single pass, and its Reasoning scores match or beat
Sonnet. In a world where data classification didn't matter, Gemini would be
the pick for "explain this large codebase."

But your code is proprietary. Gemini 2.5 Pro scores G:2 — by default, Google
may use interactions to improve its models, and the data is processed on US
infrastructure without regional guarantees. For internal proprietary code,
that's a disqualifying constraint under the governance ceiling rule.

Claude Sonnet 4.6 scores G:5: Anthropic does not train on your data by
default, enterprise DPA is available, and EU region is an option. At 200K
tokens, it handles most architectural analysis tasks — typical service-level
file sets fit well within that window. For codebases that genuinely exceed
200K tokens, use a chunking strategy with Sonnet rather than routing to Gemini.

**Recommendation:** Claude Sonnet 4.6 for all proprietary codebase work.
If context exceeds 200K tokens, escalate to Claude Opus 4.6 (200K, G:5) or
chunk with Sonnet. Do not route proprietary code to Gemini without a signed
Cloud DPA with explicit training opt-out and EU region lock.
```
