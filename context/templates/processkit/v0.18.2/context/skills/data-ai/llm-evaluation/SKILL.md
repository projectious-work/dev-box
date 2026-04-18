---
name: llm-evaluation
description: |
  LLM output evaluation — automated metrics, LLM-as-judge, A/B testing, regression testing. Use when measuring LLM output quality, comparing prompt or model versions, building an automated eval pipeline, setting up regression tests for prompt changes, or evaluating RAG systems and bias/safety.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-llm-evaluation
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# LLM Evaluation

## Intro

You can't improve what you don't measure. LLM evaluation combines a
small high-quality eval dataset, automated metrics where reference
answers exist, LLM-as-judge for subjective quality, and regression
tests in CI. Build the eval set first; everything else depends on it.

## Overview

### Build an evaluation dataset

Without an eval set, you're guessing. Aim for 50–100 representative
input/expected-output pairs from real usage, with 10–20% adversarial
or edge-case examples. Label each row with metadata (difficulty,
category, expected behavior) and split into a dev set (for
iteration) and a held-out test set (for final measurement).

Store as JSONL or CSV:

```jsonl
{"input": "...", "expected": "...", "category": "billing"}
```

Sources: production logs, user feedback, manually curated examples,
synthetic generation followed by human review.

### Automated metrics

Use when output has a clear reference answer. Cheap and fast, but
miss nuance — always combine with human or LLM-as-judge.

- **BLEU** — n-gram precision; good for translation, poor for
  open-ended generation.
- **ROUGE** (especially ROUGE-L) — recall-oriented; good for
  summarization.
- **Exact match** — binary pass/fail; factual extraction,
  classification.
- **BERTScore** — cosine similarity of contextual embeddings;
  better than BLEU/ROUGE for paraphrased correct answers.
- **Embedding similarity** — quick semantic similarity to a
  reference.
- **Code:** Pass@k (does generated code pass test cases?), syntax
  validity.
- **Classification:** precision/recall/F1 per class.
- **Structured output:** schema compliance rate, field accuracy.

### LLM-as-judge

Use a strong LLM to score outputs from a weaker LLM or competing
prompt versions.

Setup: define a rubric with 3–5 criteria (accuracy, helpfulness,
safety, format compliance), score each on a 1–5 scale with explicit
level definitions, run at temperature 0 for reproducibility, and
include the rubric, input, and output in the judge prompt.

```
You are evaluating the quality of an AI response.

Criteria:
- Accuracy (1-5): Is the information factually correct?
- Completeness (1-5): Does it address all parts of the question?
- Clarity (1-5): Is the response well-organized and easy to read?

Input: {input}
Response: {response}

Score each criterion and provide a brief justification.
Return as JSON:
{"accuracy": N, "completeness": N, "clarity": N, "justification": "..."}
```

Mitigate judge bias: swap order in pairwise comparisons (position
bias), run each evaluation twice for consistency, average across
multiple judge models, and calibrate against human ratings on a
small set.

### A/B testing prompts

Compare prompt versions on the same eval set. Run both, score with
automated metrics + LLM-as-judge, compare distributions (mean,
median, p25, p75), then run a paired significance test (paired
t-test or Wilcoxon signed-rank). Critically: even if version B wins
overall, check whether it regressed on cases version A handled. A
prompt leaderboard tracking versions over time pays off quickly.

### Regression testing in CI

Maintain a golden suite of critical test cases with expected
outputs. Run on every prompt or model change. Set thresholds
("accuracy >95%, format compliance 100%") and fail the build when
breached. Alert on degradation trends even within thresholds.

### RAG-specific (RAGAS)

For RAG pipelines, the four metrics are:

- **Context precision** — fraction of retrieved chunks that are
  relevant.
- **Context recall** — fraction of needed information that was
  retrieved.
- **Faithfulness** — is the answer grounded in the retrieved
  context, or hallucinated?
- **Answer relevance** — does the answer actually address the
  question?

Build a golden dataset of 50+ question/ideal-context/reference
triples and score with RAGAS to localize the failing stage
(retrieval vs. generation).

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Eval set built entirely from synthetic data.** Synthetic data captures what you think can go wrong, not what actually goes wrong in production. A chatbot eval set built from GPT-generated questions will miss the real distribution of user queries. Mix production examples into the eval set before using it to make decisions.
- **Tuning prompts against the test set.** Choosing prompts based on test-set performance makes the test set part of the development loop — it no longer measures generalization. Keep a true held-out test set; iterate on a separate dev set.
- **Single-number metric hiding subgroup regressions.** A new prompt version with an overall accuracy of 92% may be 80% on billing questions even though it is 99% on FAQ questions. Always slice metrics by category, difficulty, and edge-case subsets before declaring a winner.
- **LLM-as-judge without calibration.** Judge models have biases: position bias (preferring the first option), verbosity bias (preferring longer answers), and self-preference (preferring their own outputs). Use swap-order evaluation and calibrate the judge on a small set of human-labeled examples before trusting its scores.
- **No reproducibility on eval runs.** LLM evaluation at temperature > 0 produces different results on every run. Set temperature = 0 for evaluation and pin all model versions and prompt hashes. Without this, you can't tell whether a score change is a real improvement or noise.
- **Treating automated metrics as ground truth for open-ended tasks.** BLEU and ROUGE measure n-gram overlap, not quality. A correct answer phrased differently from the reference scores poorly; a fluent but wrong answer may score well. Use automated metrics only where reference answers exist; use LLM-as-judge for open-ended tasks.
- **Reporting only means without distributions.** A mean faithfulness score of 0.85 is compatible with half the responses being hallucinated (0.7) and half being perfect (1.0). Always report p50, p90, p99 and the fraction of responses below a minimum acceptable threshold.

## Full reference

### Human evaluation

The gold standard, but expensive — reserve for high-stakes
decisions. Use 3+ raters per example to measure inter-rater
agreement (Cohen's kappa). Provide clear rubrics; vague instructions
produce noisy ratings. Blind raters to which version they're
evaluating. 50–100 examples is usually enough for directional
calls.

### Bias and safety

- Test with demographically diverse inputs (names, locations,
  contexts).
- Probe for stereotyping, toxicity, and refusal of valid requests.
- Run red-team prompts to provoke harmful outputs.
- Track refusal rate — too high means the model is over-cautious.
- Tools: Perspective API for toxicity, custom classifiers for
  domain-specific harms.

### Choosing metrics by task

| Task | Metrics |
|---|---|
| Translation | BLEU, chrF, COMET, human eval |
| Summarization | ROUGE-L, BERTScore, faithfulness, human eval |
| Open-ended generation | LLM-as-judge, human eval |
| Classification | Precision, recall, F1, confusion matrix |
| Structured extraction | Schema compliance, field accuracy, exact match |
| Code | Pass@k, syntax validity, lint |
| RAG | Context precision/recall, faithfulness, answer relevance |
| Agents / multi-step | Task success rate, step efficiency, tool-call accuracy |

### Statistical rigor

Always report variance, not point estimates. For paired comparisons
(same eval set, two systems) use the paired t-test or Wilcoxon
signed-rank. Bootstrap confidence intervals when distributions are
skewed. Beware multiple-comparison inflation when testing many
prompt variants — apply Bonferroni or Holm-Bonferroni correction.

### Common eval anti-patterns

- **Eval set built from synthetic data only** — never reflects
  real failure modes. Mix in real production examples.
- **Single-number metric** — hides regressions on subgroups. Always
  slice.
- **Tuning prompt against the test set** — same data leakage problem
  as in classical ML. Keep a true held-out test set.
- **Trusting LLM-as-judge without calibration** — judges have
  biases. Calibrate against human ratings on a small set.
- **Reporting averages without distributions** — mean hides tails;
  always show p50, p90, p99.
- **No reproducibility** — use temperature 0 for evaluation runs and
  pin all model and prompt versions.

### Worked scenarios

**Eval pipeline for a support chatbot.** Build a JSONL eval set
from production conversation logs. Define a rubric (accuracy,
helpfulness, tone, escalation appropriateness). Implement
LLM-as-judge with a strong model. Set up CI to run evals on every
prompt change and report a scorecard. Flag regressions beyond 5% on
any metric.

**Comparing two summarization prompts.** Run both on 50 diverse
documents. Score with ROUGE-L (automated) and LLM-as-judge
(quality rubric). Build a per-document comparison table. Run a
paired significance test. Identify documents where version B
regressed and analyze why. Recommend the winner with caveats.

**Bias check for a resume screener.** Generate test resumes with
varied names (gender, ethnicity signals) but identical
qualifications. Run the prompt on all variants. Compare score
distributions across demographic groups. Flag statistically
significant differences. Recommend debiasing: strip names before
scoring, add explicit fairness instructions, or run a separate
bias-check pass.
