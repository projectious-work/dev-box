# Model Profiles

> Validated: 2026-Q1. Check Artificial Analysis (artificialanalysis.ai) and
> LMSYS Chatbot Arena for live benchmark updates before making high-stakes
> decisions. Scores here are calibrations, not direct benchmark readings.

Each profile shows the six top-level dimension scores (1–5) plus sub-dimension
notes, a narrative, and best-for / avoid-for guidance.

---

## Claude Opus 4.6

**Provider:** Anthropic · **Tier:** Frontier flagship · **Context:** 200K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓▓▓░░░░░░  2/5  Limited
  Breadth      ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Reliability  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Governance   ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
```

**Sub-dimension notes:**
- Reasoning: Top-tier GPQA Diamond, MATH, complex multi-step chains;
  outstanding on ambiguous problems and novel domains
- Engineering: #1–2 on SWE-bench Verified; excels at architectural decisions,
  cross-file refactoring, subtle bug diagnosis, agentic multi-tool tasks
- Speed: Slowest and most expensive Claude model; not suitable for real-time UX
  or high-volume generation
- Breadth: 200K context; strong vision; no native audio/video
- Reliability: Lowest hallucination rate in the Claude family; excellent at
  instruction following even with complex, multi-constraint prompts
- Governance: No training on user data by default; SOC 2 Type II, HIPAA-eligible;
  zero data retention option; EU region available

**Best for:** Hardest engineering problems, novel algorithm design, complex
architectural decisions, high-stakes code review, subtle debugging across large
codebases, tasks where errors are very costly, ambiguous/vague prompts that need
interpretation.

**Avoid for:** High-volume generation, real-time applications (<500ms TTFT),
cost-sensitive workflows at scale.

**Typical use in task routing:** Deep-Think cluster (R+E dominant), any cluster
where reliability tolerance is zero.

---

## Claude Sonnet 4.6

**Provider:** Anthropic · **Tier:** Frontier mid-size · **Context:** 200K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
```

**Sub-dimension notes:**
- Reasoning: Top-tier across most domains; slightly below Opus on the hardest
  novel-reasoning tasks; excellent on scientific and domain-expert questions
- Engineering: Exceptional; competitive with Opus on most real-world coding tasks;
  excels at agentic tool-use chains, Claude Code's native model
- Speed: Mid-range frontier latency (~1–2s TTFT); mid-range cost; suitable for
  interactive use, not bulk generation
- Breadth: 200K context; strong vision; no native audio/video
- Reliability: Low hallucination; strong instruction following; consistent across
  long contexts
- Governance: Same Anthropic data policies as Opus; no training on data by default

**Best for:** Everyday engineering tasks, code review, refactoring, agentic
workflows, tasks touching sensitive or enterprise data, interactive developer
tooling (this is Claude Code's default model).

**Avoid for:** Extreme cost sensitivity at very high volume, real-time <100ms UX,
native audio/video processing.

**Typical use in task routing:** Production-Coder cluster (E dominant), mixed
Reasoning+Engineering tasks where Opus is over-spec.

---

## Claude Haiku 4.5

**Provider:** Anthropic · **Tier:** Frontier fast/small · **Context:** 200K tokens

```
  Reasoning    ▓▓▓▓▓▓░░░░  3/5  Moderate
  Engineering  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Speed        ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
```

**Sub-dimension notes:**
- Reasoning: Good for routine tasks; struggles with novel multi-step reasoning
  chains; not appropriate for hard math or scientific problems
- Engineering: Good at straightforward code generation and simple bug fixes;
  not suitable for complex architectural work or novel algorithms
- Speed: Fastest and cheapest Claude model; sub-200ms TTFT achievable; excellent
  for streaming and real-time UX
- Breadth: 200K context but capability degrades faster than Sonnet/Opus on
  complex long-doc tasks; basic vision support
- Governance: Inherits Anthropic's full data-privacy stance

**Best for:** High-volume generation, formatting and transformation tasks, simple
test scaffolding, interactive chat, latency-sensitive applications, cost-sensitive
pipelines, classification and routing.

**Avoid for:** Complex architecture, novel algorithm design, ambiguous prompts
requiring deep interpretation, tasks where errors are costly.

**Typical use in task routing:** High-Volume cluster (S dominant), bulk
generation, preprocessing steps.

---

## GPT-4o

**Provider:** OpenAI · **Tier:** Frontier multi-modal · **Context:** 128K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Engineering  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓░░░░░░  2/5  Limited
```

**Sub-dimension notes:**
- Reasoning: Strong across domains; competitive with Claude Sonnet on most tasks
- Engineering: Strong coding; good SWE-bench scores; solid tool use
- Speed: Mid-range; similar to Sonnet pricing/latency tier
- Breadth: Exceptional; native voice mode (audio in/out), vision, 128K context,
  broad multilingual support, structured outputs; best multimodal model at this tier
- Reliability: Strong; some degradation in very long context windows
- Governance: Data may be used for model improvement without enterprise agreement;
  US-centric datacenters; weaker data sovereignty guarantees; G:2 unless on
  Enterprise/Azure OpenAI with DPA

**Best for:** Multimodal tasks (vision + language + audio in the same pipeline),
voice applications, broad consumer-facing use cases, non-sensitive data workflows.

**Avoid for:** Privacy-sensitive or regulated data without explicit enterprise DPA;
EU data sovereignty requirements without Azure OpenAI EU region configuration.

**Typical use in task routing:** Multimodal cluster (B dominant), mixed tasks
where audio/video are involved.

---

## o3

**Provider:** OpenAI · **Tier:** Frontier reasoning flagship · **Context:** 128K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓░░░░░░░░  1/5  Minimal
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓░░░░░░  2/5  Limited
```

**Sub-dimension notes:**
- Reasoning: SOTA on GPQA Diamond, MATH/AIME 2025, ARC-AGI 2; the go-to model
  for competition-level math, hard science, and formal reasoning
- Engineering: Top SWE-bench Verified scores; excellent at complex multi-file
  reasoning and cross-dependency analysis
- Speed: Extended thinking mode makes responses take seconds to minutes; very
  expensive per token; rate limits are aggressive; not suitable for interactive use
- Breadth: 128K context; vision; no native audio
- Governance: Inherits OpenAI's data practices (G:2)

**Best for:** Competition-level math and science, formal proofs, hardest
algorithmic problems, situations where Opus still gets it wrong, one-off
high-stakes analysis where cost is not a constraint.

**Avoid for:** Interactive use, high-volume tasks, anything time-sensitive, any
task that doesn't truly need SOTA reasoning (cost and latency are prohibitive).

**Typical use in task routing:** Deep-Think cluster when Opus is insufficient;
used sparingly as the "last resort" for the hardest cluster items.

---

## o4-mini

**Provider:** OpenAI · **Tier:** Frontier reasoning mid-size · **Context:** 128K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Engineering  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓░░░░░░  2/5  Limited
```

**Sub-dimension notes:**
- Reasoning: Near o3 on many benchmarks at a fraction of the cost; good balance
  of reasoning depth and latency
- Engineering: Strong coding; good for tasks that need o3-class reasoning but at
  interactive speed
- Speed: Faster than o3; still slower than non-thinking-mode models
- Governance: Same as o3 (OpenAI data practices)

**Best for:** Tasks that need extended reasoning but can't afford o3's latency
or cost; a good middle ground between Sonnet and o3.

**Typical use in task routing:** Deep-Think cluster when Opus is unavailable and
o3 is too slow.

---

## Gemini 2.5 Pro

**Provider:** Google DeepMind · **Tier:** Frontier flagship · **Context:** 1M+ tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Engineering  ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Reliability  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Governance   ▓▓▓▓░░░░░░  2/5  Limited
```

**Sub-dimension notes:**
- Reasoning: Top-tier; competitive with o3 and Claude Opus on hard reasoning;
  excellent on scientific and mathematical tasks
- Engineering: Excellent; top SWE-bench scores; strong agentic capabilities
- Breadth: Best-in-class context window (1M+ tokens); native multimodal across
  text, vision, audio, video simultaneously; broadest modality coverage
- Governance: Google data practices apply; data may be used for improvement
  without Workspace/enterprise agreement; US-centric by default; G:2 without
  explicit Cloud DPA and region lockdown

**Best for:** Tasks requiring huge context windows (entire repositories, long
documents, hours of video), multimodal pipelines combining vision + audio +
text, tasks where breadth is the primary constraint.

**Avoid for:** Privacy-sensitive or regulated data without explicit Cloud DPA and
EU region configuration.

**Typical use in task routing:** Long-Context cluster (B dominant), any task
where the context requirement exceeds Claude's 200K window.

---

## Gemini Flash 2.0

**Provider:** Google DeepMind · **Tier:** Frontier fast · **Context:** 1M+ tokens

```
  Reasoning    ▓▓▓▓▓▓░░░░  3/5  Moderate
  Engineering  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Speed        ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Breadth      ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Reliability  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Governance   ▓▓▓▓░░░░░░  2/5  Limited
```

**Sub-dimension notes:**
- Speed: Extremely fast and cheap; one of the lowest latency frontier models
- Breadth: Inherits Gemini's 1M context and full multimodal stack at Flash speed
- Reliability: More prone to instruction-following gaps than Pro tier; suitable
  for tasks where output is reviewed before use

**Best for:** High-volume multimodal tasks, fast classification, streaming
pipelines, tasks needing huge context at low cost (e.g., summarizing a 500K
token codebase).

**Avoid for:** Tasks requiring precise instruction following, complex reasoning,
or high-stakes output without human review.

**Typical use in task routing:** High-Volume cluster (S dominant) when multimodal
or very long context is also needed.

---

## Llama 3.3 70B / 3.1 405B (self-hosted)

**Provider:** Meta (open weights) · **Tier:** Open-source · **Context:** 128K tokens

```
  Reasoning    ▓▓▓▓▓▓░░░░  3/5  Moderate    (70B) / 4/5 Strong (405B)
  Engineering  ▓▓▓▓▓▓░░░░  3/5  Moderate    (70B) / 4/5 Strong (405B)
  Speed        ▓▓▓▓▓▓▓▓░░  4/5  Strong      (hardware-dependent)
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Governance   ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
```

**Sub-dimension notes:**
- Reasoning: 70B is adequate for routine tasks; 405B approaches frontier quality
  on many benchmarks; both are weaker than Claude/GPT4o on the hardest tasks
- Speed: Excellent throughput when self-hosted on appropriate GPU hardware;
  API throughput varies by provider
- Breadth: 128K context; Llama-Vision variants add image support but multimodal
  is limited compared to frontier models
- Governance: Exceptional; open weights; fully self-hostable; zero data egress;
  air-gap deployable; no external API calls required; the only option for true
  data sovereignty in regulated environments

**Best for:** Privacy-First cluster; tasks involving PII, PHI, secrets, or
confidential business logic where data must not leave your infrastructure;
regulated industries (healthcare, finance, legal, government); EU-GDPR
environments requiring data residency.

**Avoid for:** Tasks requiring cutting-edge reasoning or coding capability;
multimodal pipelines; cases where the self-hosting infrastructure cost exceeds
the privacy requirement.

**Typical use in task routing:** Privacy-First (G dominant) cluster; the default
recommendation whenever G:5 is required.

---

## Mistral Large 3

**Provider:** Mistral AI (France) · **Tier:** Mid-tier frontier · **Context:** 128K tokens

```
  Reasoning    ▓▓▓▓▓▓░░░░  3/5  Moderate
  Engineering  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Speed        ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Governance   ▓▓▓▓▓▓▓▓░░  4/5  Strong
```

**Sub-dimension notes:**
- Engineering: Strong coder; good at standard software tasks; competitive at
  this price point
- Speed: Faster and cheaper than frontier flagship models; good for interactive
  use at scale
- Governance: EU-headquartered; GDPR-native; strong enterprise data isolation
  in Le Chat Enterprise; self-hostable via weights for G:5 requirements;
  G:4 as API (EU datacenter, data isolation agreement available)

**Best for:** EU organizations needing GDPR compliance without full self-hosting;
mid-tier engineering tasks at lower cost than Sonnet; Privacy-First cluster
fallback when Llama self-hosting is not feasible.

**Avoid for:** Cutting-edge reasoning, multimodal, or tasks requiring SOTA coding.

**Typical use in task routing:** Privacy-First cluster fallback; EU-compliant
alternative to OpenAI/Google for non-SOTA tasks.

---

## DeepSeek V3

**Provider:** DeepSeek (China) · **Tier:** Frontier-class, low cost · **Context:** 64K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Engineering  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Speed        ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Governance   ▓▓░░░░░░░░  1/5  Minimal
```

**GOVERNANCE WARNING — G:1**

DeepSeek is operated by a Chinese company subject to Chinese law, including
data access obligations to Chinese authorities. **Never use for:**
- PII, PHI, or any personal data
- Confidential business information or unreleased IP
- Security-sensitive code, credentials, or secrets
- Regulated data (HIPAA, GDPR, SOC 2, FedRAMP)
- Government, defense, or critical infrastructure work

**Best for:** Non-sensitive tasks where cost is the dominant constraint; open-
source project work; tasks involving only public information.

**Typical use in task routing:** Only route non-sensitive, public-domain tasks.
Always flag to the user if you route here and ask them to confirm the data
sensitivity before proceeding.

---

## DeepSeek R1

**Provider:** DeepSeek (China) · **Tier:** Frontier reasoning, low cost · **Context:** 64K tokens

```
  Reasoning    ▓▓▓▓▓▓▓▓▓▓  5/5  Exceptional
  Engineering  ▓▓▓▓▓▓▓▓░░  4/5  Strong
  Speed        ▓▓▓▓▓▓░░░░  3/5  Moderate
  Breadth      ▓▓▓▓▓▓░░░░  3/5  Moderate
  Reliability  ▓▓▓▓▓▓░░░░  3/5  Moderate
  Governance   ▓▓░░░░░░░░  1/5  Minimal
```

**GOVERNANCE WARNING — G:1** (same as V3 above)

**Best for:** Hard reasoning and math tasks on non-sensitive, public data where
o3 or Opus are cost-prohibitive.

**Typical use in task routing:** Deep-Think cluster only for fully public,
non-sensitive work; always with explicit user acknowledgment of G:1 risk.

---

## Live data sources

When profiles are >6 months old or a new model has released, consult:

- **Artificial Analysis** — artificialanalysis.ai/models — intelligence index,
  latency, cost, context window comparisons updated frequently
- **LMSYS Chatbot Arena** — chat.lmsys.org — human preference Elo ratings,
  head-to-head win rates
- **SWE-bench Leaderboard** — swebench.com — engineering (E) dimension ground truth
- **BFCL Leaderboard** — gorilla.cs.berkeley.edu — tool use / function calling
- **Scale AI HELM** — crfm.stanford.edu/helm — multi-dimension academic benchmark
