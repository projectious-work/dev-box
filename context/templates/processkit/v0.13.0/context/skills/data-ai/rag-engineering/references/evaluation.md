# RAG Evaluation Reference

## Core Metrics

RAG evaluation separates **retrieval quality** from **generation quality**. Both must be measured independently to diagnose problems.

### Retrieval Metrics

**Context Precision** — Of the retrieved chunks, what fraction is relevant?
- Low precision = stuffing irrelevant context into the prompt, wasting tokens, confusing the LLM.
- Measure: For each retrieved chunk, label relevant/irrelevant. Precision = relevant / total retrieved.

**Context Recall** — Of all chunks needed to answer the question, what fraction was retrieved?
- Low recall = missing information, incomplete or wrong answers.
- Measure: Identify all gold-standard chunks for a question. Recall = retrieved-relevant / total-relevant.

### Generation Metrics

**Faithfulness** — Is every claim in the answer grounded in the retrieved context?
- The most critical RAG metric. Low faithfulness = hallucination.
- Measure: Decompose answer into atomic claims. For each claim, verify it is supported by the provided context.

**Answer Relevance** — Does the answer address the question asked?
- Low relevance = the model went off-topic or answered a different question.
- Measure: Generate N questions from the answer, compute similarity to the original question.

**Answer Correctness** — Is the answer factually correct compared to a reference answer?
- Combines faithfulness with factual accuracy. Requires ground-truth answers.
- Measure: F1 overlap between generated answer and reference answer, or LLM-as-judge comparison.

## RAGAS Framework

RAGAS automates RAG evaluation using LLM-as-judge for metrics that are hard to compute deterministically.

```python
from ragas import evaluate
from ragas.metrics import (
    context_precision,
    context_recall,
    faithfulness,
    answer_relevancy,
)
from datasets import Dataset

eval_data = Dataset.from_dict({
    "question": questions,
    "answer": generated_answers,
    "contexts": retrieved_contexts,      # list of list of strings
    "ground_truth": reference_answers,
})

results = evaluate(
    eval_data,
    metrics=[
        context_precision,
        context_recall,
        faithfulness,
        answer_relevancy,
    ],
)
print(results)  # per-metric scores + overall
```

**Interpreting RAGAS scores:** All metrics are 0-1. Target benchmarks:
- Context precision: > 0.8
- Context recall: > 0.7
- Faithfulness: > 0.9 (this is non-negotiable for production)
- Answer relevancy: > 0.8

## Building Evaluation Datasets

A golden eval set is the single most valuable asset for RAG development.

**Step 1: Sample questions.** Collect 50-100 real user questions. If none exist, generate synthetic questions from your documents using an LLM, then filter for realism.

**Step 2: Identify gold contexts.** For each question, manually find the 1-3 chunks that contain the answer. Record their IDs or text.

**Step 3: Write reference answers.** Write concise, factual answers grounded only in the gold contexts.

**Step 4: Maintain the dataset.** Add failure cases as you find them. Re-run eval after every pipeline change.

```python
# Minimal eval dataset structure
eval_set = [
    {
        "question": "What authentication methods does the API support?",
        "gold_contexts": ["chunk_42", "chunk_43"],
        "reference_answer": "The API supports OAuth 2.0, API keys, and JWT tokens.",
    },
    # ... 49-99 more
]
```

**Synthetic data generation:** Use an LLM to read a chunk and generate a question that the chunk answers. Filter aggressively: discard questions that are too easy, too vague, or answerable without the chunk.

## Common Failure Modes

### Retrieval Failures

| Symptom | Likely Cause | Diagnosis | Fix |
|---------|-------------|-----------|-----|
| Wrong documents retrieved | Query-document vocabulary mismatch | Check if BM25 retrieves better results | Add hybrid search |
| Relevant doc exists but not retrieved | Chunk too large, answer buried | Inspect chunk containing the answer | Reduce chunk size, try parent-doc retrieval |
| Top-1 is correct, rest are noise | No reranking | Check score distribution of top-k | Add cross-encoder reranker |
| Same info retrieved multiple times | Overlapping chunks, no dedup | Inspect retrieved chunks for similarity | Add MMR or deduplication |
| Retrieval works for some topics, not others | Uneven corpus coverage | Compare recall across topic categories | Add missing documents, adjust metadata filters |

### Generation Failures

| Symptom | Likely Cause | Diagnosis | Fix |
|---------|-------------|-----------|-----|
| Answer contradicts context | Hallucination / model prior | Check faithfulness score | Stronger grounding prompt, better model |
| Answer is vague or generic | Context not used effectively | Compare answer with/without context | Improve prompt template, reorder context |
| Answer cites wrong source | Citation mapping broken | Verify chunk numbering in prompt | Fix citation template, add post-processing check |
| Refuses to answer despite good context | Over-cautious system prompt | Test with relaxed prompt | Adjust system prompt, check for conflicting instructions |
| Answers only part of the question | Low context recall | Check if all needed chunks retrieved | Multi-query retrieval, smaller chunks |

## Evaluation Pipeline in Practice

```
+------------------+     +------------------+     +------------------+
|  Golden Dataset  | --> |  Run RAG Pipeline| --> |  Compute Metrics |
|  (questions +    |     |  (retrieve +     |     |  (RAGAS or       |
|   gold contexts  |     |   generate)      |     |   custom)        |
|   + ref answers) |     |                  |     |                  |
+------------------+     +------------------+     +--------+---------+
                                                           |
                                                  +--------v---------+
                                                  |  Dashboard /     |
                                                  |  CI Gate         |
                                                  +------------------+
```

**CI integration:** Run eval on every pipeline change (chunking, embedding, prompt). Fail the build if faithfulness drops below 0.9 or context recall drops below 0.6.

**A/B testing:** For subjective quality, run blind comparisons. Show two answers (system A vs B) to reviewers without labels. 50 comparisons is usually enough to detect meaningful differences.

## Debugging Workflow

When RAG output quality degrades:

1. **Check retrieval first.** Print retrieved chunks for failing questions. Is the right information there?
2. **If retrieval is good, check generation.** Same chunks, different prompt or model -- does the answer improve?
3. **If retrieval is bad, check the index.** Is the source document ingested? Is the chunk containing the answer present? Is the embedding reasonable?
4. **Isolate the variable.** Change one thing at a time. Measure after each change. Log everything.

Never tune the prompt to fix a retrieval problem, and never tune retrieval to fix a prompt problem.
