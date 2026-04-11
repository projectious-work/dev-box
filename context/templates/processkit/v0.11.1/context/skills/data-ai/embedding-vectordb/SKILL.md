---
name: embedding-vectordb
description: |
  Vector embeddings and vector DB patterns — model choice, similarity metrics, index tuning. Use when choosing an embedding model, picking or migrating between vector databases, optimizing semantic search quality or latency, or building a hybrid (dense + sparse) retrieval pipeline.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-embedding-vectordb
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# Embedding and Vector Database Patterns

## Intro

Semantic search lives or dies on three choices: the embedding model,
the vector store, and the index parameters. Default to cosine
similarity, hybrid retrieval (dense + BM25), and HNSW indexes — then
benchmark on your own eval set before tuning.

## Overview

### Embedding model selection

Pick on domain match, dimensionality, context window, latency, and
cost. MTEB scores are general — benchmark on YOUR data. Smaller
models (MiniLM) suit real-time; larger ones (e5-mistral) suit batch.
Open-source is free to run but needs GPU infrastructure.

Commercial:

| Model | Dims | Max tokens | Strengths |
|---|---|---|---|
| OpenAI text-embedding-3-small | 1536 | 8191 | Good default, low cost |
| OpenAI text-embedding-3-large | 3072 | 8191 | Higher quality, Matryoshka |
| Cohere embed-v3 | 1024 | 512 | Multilingual, search/classify |
| Voyage voyage-3 | 1024 | 32000 | Long context, strong on code |

Open-source:

| Model | Dims | Max tokens | Strengths |
|---|---|---|---|
| nomic-embed-text-v1.5 | 768 | 8192 | Strong MTEB, Matryoshka |
| bge-large-en-v1.5 | 1024 | 512 | English, well-tested |
| e5-mistral-7b-instruct | 4096 | 32768 | Best open quality, high compute |
| all-MiniLM-L6-v2 | 384 | 256 | Tiny, fast, prototyping |

### Dimensionality and Matryoshka

Matryoshka embeddings (OpenAI text-embedding-3, nomic-embed) allow
truncating dimensions without retraining: 3072 → 1024 or 512 trades
modest quality loss for faster search. Test at each dimension on
your eval set before committing. Rule of thumb: 256–512 dims is
enough for most retrieval; 1024+ for fine-grained similarity.

### Similarity metrics

| Metric | Range | Best for |
|---|---|---|
| Cosine similarity | [-1, 1] | Normalized embeddings (most common) |
| Dot product | (-inf, inf) | When magnitude matters |
| Euclidean (L2) | [0, inf) | Spatial clustering |

Default is cosine — most embedding models are trained with it. For
unit-length embeddings, cosine equals dot product.

### Vector database selection

| Database | Architecture | Best for | Scaling |
|---|---|---|---|
| FAISS | In-memory library | Prototyping, < 10M | Single machine |
| pgvector | Postgres extension | Postgres shops, joins + filtering | Vertical |
| Chroma | Embedded DB | Local dev, quick experiments | Single, < 1M |
| Qdrant | Rust client-server | Production, advanced filtering | Horizontal |
| Weaviate | Go, built-in vectorizers | Multimodal, auto-vectorization | Horizontal |
| Pinecone | Managed SaaS | Zero-ops, serverless | Fully managed |
| Milvus | Distributed cloud-native | Billions of vectors | Horizontal |

Decision flow: prototyping → Chroma or FAISS. Already on Postgres →
pgvector. Production with advanced filtering → Qdrant or Weaviate.
Zero ops → Pinecone. Billions of vectors → Milvus.

### Index types

**HNSW** is the default for most vector DBs. Key params: `M`
(connections per node, default 16), `ef_construction` (build quality,
default 200), `ef_search` (query quality, default 100). Higher M and
ef = better recall, more memory, slower build. Start with defaults;
tune `ef_search` up for recall, down for latency.

**IVF** clusters vectors and searches only relevant clusters. Params:
`nlist` ≈ sqrt(N), `nprobe` ≈ nlist/10 to nlist/5. Faster than HNSW
above ~100M vectors.

**PQ (Product Quantization)** compresses vectors with slight quality
loss. Use when the dataset will not fit in memory. Often combined as
IVF-PQ.

### Hybrid search

Combine dense (semantic) and sparse (keyword/BM25) retrieval: dense
catches "automobile" matching "car," sparse catches exact matches
(product IDs, acronyms, proper nouns). Fuse with Reciprocal Rank
Fusion: `RRF = sum(1 / (k + rank_i))` with k typically 60. Hybrid
almost always outperforms either method alone. Qdrant and Weaviate
have built-in hybrid; for others, run BM25
(Elasticsearch/OpenSearch) and vector search separately and fuse.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Choosing an embedding model based solely on MTEB leaderboard scores.** MTEB benchmarks general-purpose retrieval; your domain may have different vocabulary, query patterns, or document lengths. Always benchmark candidate models on a small eval set from your own data before committing to one.
- **Using L2 (Euclidean) distance with embeddings trained for cosine similarity.** Most embedding models are trained with cosine similarity as the loss. Using L2 distance with them produces incorrect similarity rankings. Check the model card for the intended distance metric before configuring the vector store index.
- **Embedding raw documents and chunking after the fact.** An embedding represents the full input as a single vector. Embedding a 10,000-word document produces one vector that represents everything and nothing specifically. Chunk first, then embed each chunk.
- **Pure dense retrieval for catalogs with exact identifiers.** Dense (semantic) retrieval fails on product SKUs, proper nouns, acronyms, and precise version numbers — it is too fuzzy. Combine with sparse (BM25 / keyword) retrieval using reciprocal rank fusion for these use cases.
- **No eval set to detect retrieval regressions.** Without a golden set of queries with expected results, there is no way to know whether a change to chunking, embedding model, or index parameters improved or degraded retrieval quality. Build an eval set before the first production deployment.
- **Single embedding model for both query and document without asymmetric fine-tuning.** Queries and documents have different linguistic patterns. Some models (e.g., Voyage, `e5-instruct`) require instruction prefixes like `"Represent this document:"` for passages and `"Represent this query:"` for queries. Using the wrong prefix produces poorer retrieval.
- **Sharing one DataLoader (or equivalent batch object) instance across search requests.** Results from a cached retrieval object may belong to a different request's context. Always create per-request retrieval objects when batching or caching at the database layer.

## Full reference

### Metadata filtering and multi-tenancy

Store metadata alongside vectors: source, date, category, tenant_id.
Pre-filter (filter before vector search) is more efficient than
post-filter. For multi-tenancy, use a metadata filter on tenant_id
or separate collections per tenant. Index metadata fields used in
filters for performance.

### Embedding pipeline best practices

- **Batch processing** — embed in batches of 100–500 for API
  efficiency
- **Caching** — cache by content hash to skip re-embedding unchanged
  content
- **Normalization** — normalize to unit length when using cosine
- **Chunking alignment** — chunk text before embedding, never embed
  then chunk
- **Monitoring** — track embedding latency, cache hit rate, and
  index size over time

### Worked examples

- **Semantic search for a docs site:** `text-embedding-3-small` for
  cost-effective embedding; pgvector if already on Postgres,
  otherwise Qdrant. Hybrid search (BM25 + dense). Chunk by section
  headers at 512 tokens. Build a 50-query eval set with expected
  results to tune retrieval.
- **Slow vector search at 5M vectors (> 500ms):** If flat index,
  switch to HNSW. If HNSW, lower `ef_search` (recall vs speed).
  Consider Matryoshka 3072 → 1024. Enable metadata pre-filtering to
  shrink the search space. Benchmark each change against the eval
  set.
- **Migrating Chroma → Qdrant:** Export vectors and metadata.
  Create Qdrant collection with matching distance metric and
  dimensions. Batch-upload. Verify record count. Run the eval set
  against both for recall and latency. Configure Qdrant replication
  for HA.

### Anti-patterns

- Picking a model from MTEB without benchmarking on your domain
- Using L2 distance with embeddings trained for cosine
- Embedding raw documents and chunking after the fact
- Pure dense retrieval for catalogs full of SKUs and proper nouns
- Production vector store with no eval set to detect regressions
