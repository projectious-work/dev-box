# Retrieval Patterns Reference

## Architecture Overview

```
                          +------------------+
                          |   User Query     |
                          +--------+---------+
                                   |
                    +--------------+--------------+
                    |                             |
              +-----v-----+               +------v------+
              |  Embedding |               |   BM25 /    |
              |   Model    |               |   Sparse    |
              +-----+------+               +------+------+
                    |                             |
              +-----v-----+               +------v------+
              |  Dense     |               |  Sparse     |
              |  Search    |               |  Search     |
              +-----+------+               +------+------+
                    |                             |
                    +--------------+--------------+
                                   |
                          +--------v---------+
                          | Reciprocal Rank  |
                          | Fusion (hybrid)  |
                          +--------+---------+
                                   |
                          +--------v---------+
                          |   Cross-Encoder  |
                          |    Reranker      |
                          +--------+---------+
                                   |
                          +--------v---------+
                          |   MMR / Top-K    |
                          |   Selection      |
                          +--------+---------+
                                   |
                          +--------v---------+
                          |  Prompt Builder  |
                          +------------------+
```

## Dense Retrieval

Embed the query with the same model used for indexing. Find top-k nearest neighbors by cosine similarity (or dot product if embeddings are normalized).

**Strengths:** Captures semantic meaning, handles paraphrases, works across languages.
**Weaknesses:** Misses exact keyword matches, poor with rare terms/IDs/acronyms.

```python
query_embedding = embed_model.encode(query)
results = vector_store.similarity_search(query_embedding, k=10)
```

**Tuning k:** Start with k=5 for focused answers, k=10-20 if you plan to rerank. Monitor context precision to detect diminishing returns.

## Sparse Retrieval (BM25 / TF-IDF)

Term-frequency-based matching. BM25 is the standard; it handles term saturation and document length normalization better than raw TF-IDF.

**Strengths:** Exact term matching, handles rare words, no embedding needed, fast.
**Weaknesses:** No semantic understanding, misses synonyms.

```python
from rank_bm25 import BM25Okapi

tokenized_corpus = [doc.split() for doc in documents]
bm25 = BM25Okapi(tokenized_corpus)
scores = bm25.get_scores(query.split())
top_indices = scores.argsort()[-10:][::-1]
```

**When BM25 wins:** Queries with product codes, error messages, API names, legal clause numbers, or any domain-specific identifiers that embedding models treat as noise.

## Hybrid Search

Combine dense and sparse scores. Reciprocal Rank Fusion (RRF) is the standard approach -- no tuning of score weights needed.

```
RRF(d) = sum( 1 / (k + rank_i(d)) ) for each retrieval method i    (k typically 60)
```

**In practice:** Hybrid search with RRF outperforms either dense or sparse alone on nearly every benchmark. It should be the default for production systems.

Vector stores with native hybrid: Qdrant, Weaviate, Elasticsearch. For others, run both searches and fuse in application code.

## Reranking with Cross-Encoders

Bi-encoders (embedding models) score query and document independently. Cross-encoders score the (query, document) pair jointly -- much more accurate but too slow for full-corpus search.

```
Stage 1: Retrieve top-50 with fast method (dense, hybrid)
Stage 2: Rerank to top-5 with cross-encoder
```

**Models:** `bge-reranker-v2-m3` (open, strong), Cohere Rerank (API, easy), `ms-marco-MiniLM-L-12-v2` (fast, lighter). Latency: 100-500ms for 50 candidates.

## Maximal Marginal Relevance (MMR)

Reduces redundancy by balancing relevance to the query with diversity among selected documents.

```
MMR = argmax[ lambda * sim(q, d) - (1 - lambda) * max(sim(d, d_selected)) ]
```

For each candidate, score = `lambda * relevance - (1-lambda) * max_similarity_to_selected`. Greedily pick the highest-scoring candidate. Lambda 1.0 = pure relevance, 0.0 = pure diversity. Start at 0.6. Use lower lambda when chunks are highly redundant.

## Metadata Filtering

Pre-filter candidates before or during vector search. Reduces search space and improves precision.

```python
results = vector_store.similarity_search(
    query_embedding, k=10,
    filter={"source": "api-docs", "version": ">=3.0"}
)
```

**Useful filters:** document type, date range, language, author, product version, access control. Design your metadata schema at ingestion time.

**Pre-filter vs. post-filter:** Pre-filtering is more efficient but requires index support. Post-filtering is simpler but wastes retrieval budget on irrelevant documents.

## Parent-Document Retrieval

Index small child chunks (256 tokens) for precise matching, but store a mapping to the parent document (1024 tokens). At retrieval time, match on child embeddings, return the parent. Small chunks match queries precisely; large chunks give the LLM enough context to generate good answers.

## Multi-Query Retrieval

Use an LLM to generate query variations, retrieve for each, and deduplicate results.

```
Original: "How do I handle errors in async Python?"
Variant 1: "Python asyncio exception handling best practices"
Variant 2: "try except in async await Python"
Variant 3: "error handling patterns for coroutines"
```

Retrieve top-10 for each variant, deduplicate by document ID, take union. Fuse rankings with RRF.

**When to use:** Ambiguous or broad queries, when users don't know the right terminology. Adds one LLM call of latency (~200-500ms with a fast model).

**Cost control:** Use a small/fast model for query generation (e.g., GPT-4o-mini, Haiku). The quality bar for query variants is low.
