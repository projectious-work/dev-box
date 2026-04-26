# Dimension Specifications

Full sub-dimension detail, benchmark mappings, and scoring guidelines for
the six model-recommender dimensions.

---

## 1. Reasoning (R)

What it measures: The model's capacity for hard thinking — multi-step logical
chains, mathematical derivations, scientific reasoning, and novel problem-solving
in domains the model hasn't seen before.

### Sub-dimensions

| Sub-dimension | What it covers | Key benchmark |
|---|---|---|
| Mathematical reasoning | Arithmetic, algebra, calculus, combinatorics, proofs | MATH (Hendrycks), AIME 2024/2025 |
| Scientific / domain-expert reasoning | Physics, chemistry, biology at PhD level; unsearchable expert questions | GPQA Diamond |
| Abstract / novel reasoning | Pattern recognition, analogy, ARC-style novel tasks | ARC-AGI 2 |
| Multi-step logical chains | Chain-of-thought depth; maintaining coherence over many reasoning steps | BBH (BIG-Bench Hard), MT-Bench |
| Debugging reasoning chains | Identifying where a chain of logic breaks; root-cause analysis | Derived from SWE-bench + qualitative |

### Scoring guidelines

| Score | Observable behavior |
|---|---|
| 5 | Solves AIME problems; exceeds human expert on GPQA Diamond; handles novel tasks without prior exposure; maintains coherence across 10+ step chains |
| 4 | Solves most competition math (AMC-level); strong on GPQA; occasional gaps on the hardest novel tasks |
| 3 | Handles routine reasoning (algebra, basic logic); struggles with multi-step novel problems; adequate for most business reasoning |
| 2 | Manages simple reasoning; fails on competition math or expert science; use only for pre-structured reasoning tasks |
| 1 | Unreliable on any non-trivial reasoning; prone to hallucinated logic steps |

### Task signals that require high R

- "Design an algorithm that..." (novel)
- "Prove that..." or "Derive..."
- "Find the optimal..." (combinatorial/NP-class)
- "Why is this failing?" (debugging chain)
- Mathematical or scientific domain expertise

---

## 2. Engineering (E)

What it measures: Software engineering work — code generation, bug fixing,
tool use, agentic multi-step tasks, and navigation of large codebases.

### Sub-dimensions

| Sub-dimension | What it covers | Key benchmark |
|---|---|---|
| Function-level code generation | Write a function given a spec and unit tests | HumanEval (pass@1), MBPP |
| Repository-scale task completion | Navigate a real codebase, locate bugs, produce patches | SWE-bench Verified (pass@1) |
| Tool use / function calling | Choose the right tool, parameterize correctly, handle errors | BFCL v4 (Berkeley Function Calling) |
| Agentic planning and self-correction | Multi-step plans; recovering from tool errors; re-planning | BFCL v4 holistic agentic eval |
| Large codebase navigation | Cross-file reasoning, dependency tracing, refactoring across modules | Derived from SWE-bench + qualitative |

### Scoring guidelines

| Score | Observable behavior |
|---|---|
| 5 | Top SWE-bench Verified scores (>50%); handles multi-file refactoring; reliable tool-use chains; recovers gracefully from tool errors |
| 4 | Strong HumanEval and SWE-bench scores; handles most real-world coding tasks; occasional gaps on complex multi-file work |
| 3 | Solid on straightforward code; struggles with cross-file reasoning or subtle bugs; adequate for routine implementation |
| 2 | Can produce simple code; unreliable on bug fixing or anything requiring codebase context; needs heavy supervision |
| 1 | Code output is frequently incorrect or incomplete; not suitable for production coding |

### Task signals that require high E

- "Implement...", "Fix...", "Refactor..."
- "Review this PR" or "code review"
- "Write tests for...", "TDD"
- "Debug...", "why is this failing"
- "Use these tools to..." (agentic)
- "Migrate the codebase to..."

---

## 3. Speed (S)

What it measures: Response latency, throughput, and cost efficiency — how fast
and cheap the model is to run at scale.

### Sub-dimensions

| Sub-dimension | What it covers | Target for interactive UX |
|---|---|---|
| Time to First Token (TTFT) | Time from request send to first token in response | <500ms for interactive; <200ms for real-time |
| Inter-Token Latency (ITL) | Time between successive tokens while streaming | <50ms for smooth streaming |
| Tokens per second (TPS) | Output throughput; determines how quickly long responses complete | >100 TPS for responsive feel |
| Cost per 1M input tokens | API pricing for input tokens | Varies by tier: $0.25–$15 per 1M |
| Cost per 1M output tokens | API pricing for output tokens | Typically 3–5× input price |
| Rate limits / quota | Requests per minute, tokens per minute, daily quotas | Varies by tier and contract |

### Scoring guidelines

| Score | Typical characteristics |
|---|---|
| 5 | TTFT <200ms; TPS >200; cost <$1/1M output tokens; high rate limits |
| 4 | TTFT <500ms; TPS >100; cost <$5/1M output; reasonable rate limits |
| 3 | TTFT 500ms–2s; TPS 50–100; cost $5–$15/1M output; moderate rate limits |
| 2 | TTFT 2–10s; TPS <50; cost $15–$60/1M output; tight rate limits |
| 1 | TTFT >10s (extended thinking); cost >$60/1M output; aggressive rate limits |

### Task signals that require high S

- "Generate [large number] of..."
- "Process every file in..."
- "Bulk transform...", "batch this"
- Real-time UX (chat UI, voice, streaming)
- Cost budget is a stated constraint
- High-frequency polling or pipeline tasks

---

## 4. Breadth (B)

What it measures: Coverage of modalities, context window size, and language
support — how wide the model's perceptual and memory reach is.

### Sub-dimensions

| Sub-dimension | What it covers | Notes |
|---|---|---|
| Context window | Maximum tokens (input + output) | 4K–1M+; effective use often less than max |
| Long-doc faithfulness | Accuracy when relevant content is deep in a long context | "Lost in the middle" degradation |
| Vision (images/screenshots) | Understand images, diagrams, screenshots, charts | Most frontier models; varies in quality |
| Audio | Native speech-to-text/text-to-speech; audio reasoning | Only GPT-4o, Gemini natively |
| Video | Understanding video frames, temporal reasoning | Only Gemini natively at scale |
| Multilingual | Quality in non-English languages | Frontier models: strong; mid-tier: variable |
| Structured output | Reliable JSON, XML, schema-constrained output; function calling | Most frontier models; reliability varies |

### Scoring guidelines

| Score | Typical characteristics |
|---|---|
| 5 | Context >500K tokens with high faithfulness; vision + audio + video natively; 50+ languages; reliable structured output |
| 4 | Context 128–200K with good faithfulness; vision; one or two additional modalities; 30+ languages |
| 3 | Context 64–128K; vision only or limited; English-dominant; structured output works but can fail |
| 2 | Context 32–64K; limited vision; primarily English; structured output unreliable |
| 1 | Context <32K; no vision; English only; structured output not reliable |

### Task signals that require high B

- "Read the entire codebase and..."
- "Analyze this image / screenshot / diagram"
- "Process this audio recording"
- "Watch this video and..."
- "In [non-English language]..."
- Document count or size that exceeds ~100K tokens

---

## 5. Reliability (L)

What it measures: Instruction fidelity, factual accuracy, consistency, and
alignment — how trustworthy the model's output is.

### Sub-dimensions

| Sub-dimension | What it covers | Key benchmark |
|---|---|---|
| Instruction following | Does the model do exactly what was asked, including format and constraints? | IFEval, MT-Bench |
| Hallucination rate | How often does the model state false facts confidently? | TruthfulQA, HaluEval |
| Multi-turn consistency | Does the model stay coherent across a long conversation? | MT-Bench multi-turn |
| Safety / harmlessness | Does the model refuse harmful requests and avoid toxic output? | RealToxicityPrompts, custom red-teaming |
| Format adherence | Does the model produce the requested output format reliably? | Derived from IFEval + qualitative |

### Scoring guidelines

| Score | Observable behavior |
|---|---|
| 5 | Near-perfect instruction following on complex multi-constraint prompts; very low hallucination; consistent over 50+ turns; strong safety |
| 4 | Good instruction following with occasional misses on complex constraints; low hallucination; consistent over most conversations |
| 3 | Adequate instruction following; moderate hallucination risk on factual claims; some drift in long conversations |
| 2 | Frequent instruction following failures; hallucination is a real risk; output format unreliable |
| 1 | Cannot be trusted to follow instructions reliably; high hallucination risk |

### Task signals that require high L

- "Exactly...", "must include...", "must not include..."
- "Cite your sources", "don't hallucinate"
- "The output must follow this schema exactly"
- Customer-facing or production-ready output
- Long multi-turn sessions where state must be maintained
- Safety-critical applications

---

## 6. Governance (G)

What it measures: Data privacy, sovereignty, compliance posture, and
auditability — whether you can trust the model with sensitive data.

### Sub-dimensions

| Sub-dimension | What it covers | Key standards |
|---|---|---|
| Data retention / training policy | Is data used to train future models? Is it retained? | Provider ToS, DPA |
| Data sovereignty / region | Where is data processed and stored? | GDPR, CCPA, local reqs |
| Self-hostable / open weights | Can you run it on your own infrastructure? | Apache 2.0, Meta license |
| Compliance certifications | What has the provider been audited for? | SOC 2 Type II, HIPAA BAA, ISO 27001 |
| Prompt injection resistance | Resistance to adversarial prompt injection attacks | Custom red-team; model card |
| Audit logging | Can you get a log of all model interactions for compliance? | Provider audit log feature |

### Scoring guidelines

| Score | Characteristics |
|---|---|
| 5 | No training on data by default; self-hostable OR fully private deployment; SOC 2 + HIPAA BAA available; data stays in your region; full audit logging; open weights available |
| 4 | No training on data with enterprise agreement; EU region available; SOC 2; HIPAA BAA available; strong audit logging |
| 3 | No training on data with explicit opt-out or enterprise agreement; compliance certs available but may need procurement; limited audit logging |
| 2 | Data used for improvement without enterprise agreement; limited region options; compliance posture not documented; limited audit logging |
| 1 | Data subject to foreign jurisdiction with legal data access obligations; no compliance certs; no audit logging; do not use for any sensitive data |

### Task signals that require high G

- Any mention of PII (names, emails, addresses, SSNs, etc.)
- PHI (medical records, diagnoses, prescriptions)
- Financial data (account numbers, transactions)
- Credentials, API keys, secrets
- Confidential business information, unreleased IP
- Legal documents
- Government or defense contexts
- "GDPR", "HIPAA", "SOC 2", "regulated", "compliance"
- "internal only", "confidential", "do not share"

### The governance ceiling rule

**A model's Governance score is a ceiling, not a floor.** If G:2 is too low
for the task's data classification, the model is disqualified regardless of
its scores in all other dimensions. Route to the next model in the roster
that meets the required G score, even if it scores lower elsewhere.

Example: A task requires G:5 and the best engineering model is Claude Sonnet
(G:5, E:5). If Sonnet is unavailable, the fallback is Llama 3.3 self-hosted
(G:5, E:3) — not GPT-4o (G:2, E:4). Governance trumps engineering capability.
