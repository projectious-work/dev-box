---
name: rag-engineering
description: |
  Retrieval-Augmented Generation pipelines — ingestion, chunking, embedding, vector stores, retrieval, evaluation. Use when building a RAG pipeline, choosing chunking strategies or embedding models, debugging retrieval quality or hallucinations, evaluating an existing RAG system, or scaling/migrating vector stores.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-rag-engineering
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# RAG Engineering

## Intro

A RAG pipeline has two phases: indexing (offline) and
retrieval+generation (online). Decisions cascade — bad chunking
ruins retrieval no matter how good your embeddings are, and bad
retrieval ruins generation no matter how strong your LLM is. Start
from the data and work forward, and measure with a golden eval set
from day one.

## Overview

### Architecture

```
Indexing:  Documents -> Parse -> Chunk -> Embed -> Vector DB
Query:     Question  -> Embed -> Retrieve top-k -> Prompt -> LLM
```

### Document ingestion

Parse source documents into clean text with metadata preserved.

- **PDF** — `pymupdf4llm` or `unstructured`. Preserve headings,
  tables, page numbers. OCR fallback for scanned docs via
  `tesseract`.
- **HTML** — strip boilerplate with `trafilatura` or
  `readability-lxml`; keep headings, lists, code blocks.
- **Code** — tree-sitter for AST-aware parsing. Keep function/class
  boundaries intact. Preserve file paths and language metadata.
- **Always capture** source URL/path, document title, section
  heading, page number, last-modified date. Metadata enables
  filtering at retrieval time.

### Chunking strategies

Pick based on document structure and query patterns. See
`references/chunking-strategies.md` for code and trade-offs.

- **Fixed-size with overlap** — simple baseline; 256–1024 tokens,
  10–20% overlap.
- **Sentence-based** — group 3–5 sentences; preserves boundaries.
- **Semantic chunking** — split where embedding similarity drops;
  best for heterogeneous docs but slow.
- **Recursive character** — LangChain default; splits on `\n\n`,
  then `\n`, then space.
- **Document-structure-aware** — split on headings, code blocks,
  sections; best when structure is reliable.

Rule of thumb: chunk size should match expected answer granularity.
FAQ -> small chunks. Long technical explanations -> larger chunks.

### Embedding models

Pick on domain match, dimensionality, speed, and cost.

| Model | Dims | Context | Notes |
|---|---|---|---|
| `text-embedding-3-small` | 1536 | 8191 | Good default, cheap |
| `text-embedding-3-large` | 3072 | 8191 | Better quality, 2x cost |
| `nomic-embed-text` | 768 | 8192 | Open source, strong MTEB |
| `bge-large-en-v1.5` | 1024 | 512 | Open source, good for code |
| `voyage-code-2` | 1536 | 16000 | Best for code retrieval |

Always benchmark on **your** queries before committing. MTEB
leaderboard scores don't predict domain-specific performance. For
significant gains on a tight domain, fine-tune with synthetic
query/passage pairs and contrastive loss — even 1000 pairs helps.

### Vector stores

- **FAISS** — in-memory, fastest for <10M vectors, no filtering;
  prototyping or read-heavy workloads.
- **pgvector** — Postgres extension; use when you already have
  Postgres and need joins/filtering. HNSW index for performance.
- **Chroma** — embedded DB, easy local dev; <1M vectors practical.
- **Qdrant** — Rust-based, strong filtering, good hybrid search;
  self-host or cloud; production-ready.
- **Pinecone** — fully managed, scales well; expensive at scale.
- **Weaviate** — built-in vectorization, multimodal, feature-rich.

For most projects: start with Chroma locally, move to Qdrant or
pgvector for production.

### Retrieval strategies

- **Dense** — embed query, find nearest neighbors. Default.
- **Sparse (BM25)** — term matching. Better for exact terms,
  acronyms, IDs.
- **Hybrid** — combine dense + sparse with reciprocal rank fusion.
  Almost always better than either alone.
- **Reranking** — retrieve top-50 with a fast method, rerank to
  top-5 with a cross-encoder (`bge-reranker-v2-m3`, Cohere Rerank).
- **MMR (Maximal Marginal Relevance)** — diversify results;
  lambda 0.5–0.7.
- **Parent-document retrieval** — embed small chunks, return the
  parent section; combines precision with context.
- **Multi-query** — LLM generates 3–5 query variants, retrieve for
  each, deduplicate. Helps with ambiguous queries.

See `references/retrieval-patterns.md` for the architecture
diagram and per-pattern code.

### Prompt construction

The retrieval-to-generation handoff is where many RAG systems
fail.

- **Context ordering** — most relevant chunks first; LLMs attend
  more to the start and end of context.
- **Context budget** — reserve 30–40% of context window for
  retrieved content.
- **Citations** — number each chunk `[1]`, `[2]`; instruct the LLM
  to cite; verify citations in post-processing.
- **Deduplication** — remove near-duplicates (cosine > 0.95)
  before inserting.
- **No-context handling** — if retrieval scores are below threshold,
  tell the LLM explicitly that no relevant info was found rather
  than stuffing bad context.

### Evaluation

Key metrics: **context precision** (are retrieved chunks
relevant?), **context recall** (are all needed chunks retrieved?),
**faithfulness** (is the answer grounded in context?), **answer
relevance** (does it address the question?).

Use the RAGAS framework. Build a golden dataset of 50–100
question/ideal-context/reference-answer triples from your real
documents. See `references/evaluation.md` for metric details and
debugging workflow.

Common failure modes:

- **Hallucination** — low faithfulness. Better retrieval, explicit
  grounding instructions.
- **Incomplete answers** — low context recall. Smaller chunks,
  multi-query.
- **Wrong documents** — low context precision. Hybrid search,
  metadata filtering, reranking.
- **Right docs, bad answer** — model issue. Better prompt,
  stronger model.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Tuning the prompt to fix a retrieval problem.** If the retrieved chunks are wrong, rewriting the generation prompt will not fix the underlying retrieval failure — it just masks symptoms. Isolate which stage is failing (retrieval vs. generation) before making changes. Never tune the prompt to compensate for poor retrieval.
- **Chunks too large relative to query granularity.** A 2,000-token chunk contains many facts. The embedding represents the average of all of them, so it will be a mediocre match for any specific fact inside the chunk. Match chunk size to the granularity of expected queries — FAQ answers need small chunks; technical explanations need larger ones.
- **Pure dense retrieval for catalogs with exact identifiers.** Dense semantic search fails on product SKUs, version numbers, proper nouns, and acronyms. These need exact term matching via sparse (BM25) retrieval. Hybrid dense + sparse is the correct default for production.
- **Stuffing low-scoring retrieved chunks into the context.** When no relevant content exists, returning the top-k chunks regardless of relevance score causes hallucinations — the model will try to construct an answer from irrelevant content. Set a similarity threshold; if no chunk passes it, tell the LLM explicitly that no relevant information was found.
- **No faithfulness evaluation before production.** A RAG system that produces fluent, confident answers that contradict the retrieved context is worse than one that says "I don't know." Measure faithfulness (answer grounded in context?) from the first prototype, not as an afterthought.
- **Re-indexing for every iteration when debugging chunking.** Rebuilding the full vector index to test every chunking change is expensive and slow. Maintain a small representative test corpus that re-indexes in seconds; use the full corpus only for final validation.
- **Building without a golden eval set.** Without a ground-truth set of queries and expected context passages, "does the retrieval work?" is a feeling, not a measurement. Build the eval set from real user questions before the first production deployment and run it after every change.

## Full reference

### Hybrid search and RRF

Reciprocal Rank Fusion is the standard combiner for hybrid
retrieval — no score-weight tuning needed:

```
RRF(d) = sum( 1 / (k + rank_i(d)) ) for each retrieval method i
                                                 (k typically 60)
```

Hybrid + RRF outperforms either dense or sparse alone on nearly
every benchmark and should be the default for production. Vector
stores with native hybrid support: Qdrant, Weaviate, Elasticsearch.
For others, run both searches and fuse in application code.

### Reranking

Bi-encoders (embedding models) score query and document
independently. Cross-encoders score `(query, document)` jointly —
much more accurate but too slow for full-corpus search.

```
Stage 1: Retrieve top-50 with fast method (dense, hybrid)
Stage 2: Rerank to top-5 with cross-encoder
```

Models: `bge-reranker-v2-m3` (open, strong), Cohere Rerank (API,
easy), `ms-marco-MiniLM-L-12-v2` (fast, lighter). Latency is
typically 100–500ms for 50 candidates.

### MMR

```
MMR = argmax[ lambda * sim(q, d) - (1 - lambda) * max(sim(d, d_selected)) ]
```

Greedy: pick the highest-scoring candidate, repeat. Lambda 1.0 =
pure relevance, 0.0 = pure diversity. Start at 0.6. Lower lambda
when chunks are highly redundant.

### Parent-document retrieval

Embed at 256 tokens (precise matching) but store a mapping to a
1024-token parent (richer context). Match on child embeddings,
return the parent. Small chunks find precise matches; large chunks
give the LLM enough context to answer well.

### Multi-query retrieval

Use a small/fast LLM (Haiku, GPT-4o-mini) to rewrite the query into
3–5 variants, retrieve top-10 for each, deduplicate by document ID,
fuse with RRF. Adds ~200–500ms of latency. Best for ambiguous or
broad queries.

### Metadata filtering

Pre-filter candidates before or during vector search to reduce
search space and improve precision.

```python
results = vector_store.similarity_search(
    query_embedding, k=10,
    filter={"source": "api-docs", "version": ">=3.0"},
)
```

Useful filters: document type, date range, language, author,
product version, access control. Pre-filter > post-filter when the
index supports it.

### Chunking guidelines

1. **Always measure.** Chunk strategy changes retrieval quality —
   re-run eval after every change.
2. **Embed metadata in the chunk.** Prepend
   `"Title: {title}\nSection: {heading}\n\n"` so retrieval matches
   queries that reference titles or sections.
3. **Track provenance.** Source file, page or line number, heading
   path with each chunk — essential for citations and debugging.
4. **Small chunks for retrieval, large for generation.** Use
   parent-document retrieval.
5. **Deduplicate.** Cosine > 0.95 = duplicate. Common with
   overlapping chunks and repeated boilerplate.

### RAGAS targets

| Metric | Target |
|---|---|
| Context precision | > 0.8 |
| Context recall | > 0.7 |
| Faithfulness | > 0.9 (non-negotiable for production) |
| Answer relevancy | > 0.8 |

### Failure-mode debugging

| Symptom | Likely cause | Fix |
|---|---|---|
| Wrong docs retrieved | Vocabulary mismatch | Add hybrid search |
| Right doc exists, not retrieved | Chunk too large, answer buried | Smaller chunks, parent-doc retrieval |
| Top-1 right, rest noise | No reranking | Add cross-encoder reranker |
| Same info repeated | Overlapping chunks, no dedup | MMR or dedup pass |
| Answer contradicts context | Hallucination / model prior | Stronger grounding prompt, better model |
| Vague generic answer | Context not used effectively | Reorder context, improve template |
| Refuses despite good context | Over-cautious system prompt | Relax prompt, check conflicts |
| Answers part of question | Low context recall | Multi-query, smaller chunks |

Golden rule: never tune the prompt to fix a retrieval problem, and
never tune retrieval to fix a prompt problem. Isolate the variable.

### Worked scenarios

**Basic RAG over internal docs.** Recursive chunking at 512
tokens, `text-embedding-3-small`, Chroma. Dense retrieval, top-5.
Build an eval set from real user questions. Iterate from this
baseline.

**RAG misses relevant info.** Diagnose context recall — what was
retrieved vs. what should have been? Add hybrid search (BM25 +
dense), add cross-encoder reranking, try smaller chunk sizes. Test
each change against the eval set.

**Benchmarking a RAG system.** Build a golden dataset: 50
questions, manually identified ideal context passages, reference
answers. Run RAGAS. Identify the weakest metric and focus there.
Set up automated eval in CI to catch regressions.
