# AI & Machine Learning Skills

Skills for AI/ML development, RAG pipelines, prompt engineering, and model evaluation.

---

### ai-fundamentals

> Core ML/AI concepts including model types, training pipelines, evaluation metrics, and neural network architectures. Use when explaining AI concepts, choosing model approaches, designing ML solutions, or reviewing AI-related code.

**Triggers:** Explaining ML/AI concepts, choosing model architectures, designing training pipelines, selecting evaluation metrics, debugging model performance (overfitting, leakage, class imbalance)
**Tools:** None
**References:** `ml-concepts.md`, `math-foundations.md`

Key capabilities:

- Classify problems by learning paradigm: supervised, unsupervised, reinforcement, self-supervised
- Match model types to problems: linear models for baselines, tree-based (XGBoost/LightGBM) for tabular data, neural networks for unstructured data, probabilistic models for uncertainty
- Design correct training pipelines: data prep, train/val/test split before preprocessing, feature engineering, training, hyperparameter tuning, regularization, final evaluation
- Select evaluation metrics by task: F1/AUC-ROC for classification, RMSE/MAE for regression, NDCG/MAP for ranking, BLEU/ROUGE for generation
- Understand neural network architectures: MLP, CNN, RNN/LSTM, Transformer, GAN, VAE, diffusion models
- Explain modern LLM concepts: attention, tokenization, pre-training + fine-tuning, RLHF, prompting strategies, scaling laws
- Identify common pitfalls: data leakage, overfitting, underfitting, class imbalance, distribution shift, metric mismatch

??? example "Example usage"
    "Choose an approach for tabular customer churn prediction" -- With 50K labeled rows of structured data, recommends gradient-boosted trees (XGBoost/LightGBM) with stratified k-fold cross-validation for the imbalanced target. Reports F1 and AUC-ROC. Baselines with logistic regression first, only considers neural approaches if tree models plateau.

---

### rag-engineering

> Retrieval-Augmented Generation pipeline design including document ingestion, chunking, embedding, vector stores, retrieval strategies, and evaluation. Use when building RAG systems, optimizing retrieval quality, or debugging RAG pipelines.

**Triggers:** Building RAG pipelines, choosing chunking/embedding/vector store strategies, debugging poor retrieval quality or hallucinations, evaluating RAG systems
**Tools:** `Bash Read Write`
**References:** `chunking-strategies.md`, `retrieval-patterns.md`, `evaluation.md`

Key capabilities:

- Design end-to-end RAG architecture: indexing (parse, chunk, embed, store) and query (embed, retrieve, construct prompt, generate)
- Ingest documents from PDF, HTML, and code with metadata extraction (source, title, section, page, date)
- Choose chunking strategies: fixed-size with overlap, sentence-based, semantic chunking, recursive character, document-structure-aware
- Select embedding models by domain, dimensionality, context window, and cost (OpenAI, nomic, bge, voyage)
- Choose vector stores: FAISS for prototyping, Chroma for local dev, pgvector for Postgres shops, Qdrant for production, Pinecone for zero ops
- Implement retrieval strategies: dense, sparse (BM25), hybrid search with RRF, reranking with cross-encoders, MMR for diversity, parent-document retrieval, multi-query
- Construct effective prompts: context ordering, window budgeting, citation numbering, chunk deduplication, low-similarity handling
- Evaluate with RAGAS metrics: context precision, context recall, faithfulness, answer relevance -- using golden datasets of 50-100 triples

??? example "Example usage"
    "RAG answers miss relevant information" -- Diagnoses by checking context recall against what should have been retrieved. Tries hybrid search (BM25 + dense), adds reranking with a cross-encoder, experiments with smaller chunk sizes, and tests each change against the eval set.

---

### prompt-engineering

> Prompt design patterns for LLMs including few-shot, chain-of-thought, structured output, and injection defense. Use when crafting prompts, optimizing LLM outputs, or building prompt-based features.

**Triggers:** Crafting or refining LLM prompts, improving output quality and consistency, designing system prompts, implementing structured output, defending against prompt injection
**Tools:** None
**References:** `techniques-catalog.md`

Key capabilities:

- Structure prompts with role/context, task, constraints, examples, and input -- using delimiters for separation
- Apply core techniques: zero-shot, few-shot (2-5 diverse examples), chain-of-thought, self-consistency, structured output with JSON schema
- Design system prompts: persona definition, hard constraints, output format, domain knowledge -- versioned and tested
- Tune temperature and sampling: 0.0-0.3 for factual/code, 0.5-0.8 for creative, top-p as alternative, appropriate max tokens and stop sequences
- Defend against prompt injection: input sanitization, delimited input sections, output validation, privilege separation, canary tokens
- Build reusable prompt templates with variable slots and systematic iteration (test on 10-20 inputs, identify failure modes, add constraints)
- Evaluate prompts systematically: build eval sets of 20-50 pairs, score pass/fail or rubric-based, track metrics across versions

??? example "Example usage"
    "Classify support tickets into categories" -- Designs a few-shot prompt with 3-5 example tickets per category including edge cases. Uses temperature 0.0 for consistency. Requests JSON output with category and confidence. Validates output schema programmatically and measures accuracy against a labeled test set.

---

### llm-evaluation

> LLM output evaluation including automated metrics, LLM-as-judge, A/B testing, and regression testing. Use when evaluating model outputs, building eval pipelines, or comparing prompt versions.

**Triggers:** Measuring LLM output quality, comparing prompt or model versions, building automated evaluation pipelines, setting up regression testing, detecting bias
**Tools:** None
**References:** None

Key capabilities:

- Build evaluation datasets: 50-100 representative input/expected-output pairs from real usage, including edge cases, with metadata labels
- Apply automated metrics: text overlap (BLEU, ROUGE, exact match), semantic similarity (BERTScore, embedding similarity), task-specific (Pass@k for code, schema compliance)
- Implement LLM-as-judge: rubric with 3-5 criteria scored 1-5, temperature 0.0, mitigations for position bias, multi-judge averaging, calibration against human ratings
- Conduct human evaluation: 3+ raters per example, clear rubrics, blinded to version, Cohen's kappa for agreement
- Run A/B testing: same eval set, automated + LLM-as-judge scoring, distribution comparison, significance testing, regression checking
- Set up regression testing: golden test suite with expected outputs, threshold-based pass/fail in CI, trend alerting
- Detect bias and safety issues: demographically diverse inputs, stereotyping/toxicity checks, red-teaming, refusal rate monitoring
- Evaluate RAG pipelines with RAGAS: context precision, context recall, faithfulness, answer relevance

??? example "Example usage"
    "Systematically evaluate our customer support chatbot" -- Builds a JSONL eval set from production logs, defines a rubric (accuracy, helpfulness, tone, escalation appropriateness), implements LLM-as-judge scoring, sets up a CI job that runs evals on every prompt change, and flags regressions beyond 5% on any metric.

---

### embedding-vectordb

> Vector embeddings and vector database patterns including model selection, similarity metrics, and index tuning. Use when building semantic search, choosing vector stores, or optimizing embedding pipelines.

**Triggers:** Choosing embedding models, selecting or migrating vector databases, optimizing semantic search, implementing hybrid search, tuning vector index parameters
**Tools:** None
**References:** None

Key capabilities:

- Select embedding models: commercial (OpenAI text-embedding-3, Cohere embed-v3, Voyage) and open-source (nomic, bge, e5-mistral, MiniLM) with trade-offs on quality, dimensions, context window, latency, and cost
- Use Matryoshka embeddings for dimension reduction (3072 to 1024 or 512) without retraining
- Choose similarity metrics: cosine similarity (default), dot product (when magnitude matters), Euclidean (spatial clustering)
- Select vector databases: FAISS (prototyping), pgvector (Postgres), Chroma (local dev), Qdrant (production), Weaviate (multimodal), Pinecone (managed), Milvus (billions of vectors)
- Tune index types: HNSW (default, tune M/ef_construction/ef_search), IVF (large datasets, tune nlist/nprobe), PQ (memory-constrained)
- Implement hybrid search: dense + sparse (BM25) with Reciprocal Rank Fusion
- Configure metadata filtering and multi-tenancy with pre-filter strategy
- Optimize embedding pipelines: batch processing, content-hash caching, normalization, chunk-before-embed ordering, monitoring

??? example "Example usage"
    "Add semantic search to a documentation site" -- Recommends text-embedding-3-small for embedding, pgvector if Postgres is available (otherwise Qdrant). Implements hybrid search with BM25 for exact terms and dense retrieval for semantic matches. Chunks docs by section headers at 512 tokens and sets up a 50-query eval set to tune retrieval.

---

### ml-pipeline

> ML pipeline design including data versioning, experiment tracking, model deployment, and drift monitoring. Use when building ML pipelines, setting up MLOps, or reviewing ML infrastructure.

**Triggers:** Building ML pipelines from data to deployment, setting up experiment tracking, choosing deployment patterns, implementing drift monitoring, designing ML CI/CD
**Tools:** `Bash Read Write`
**References:** `pipeline-stages.md`

Key capabilities:

- Design reproducible pipeline architecture: data collection through versioning, feature engineering, training, evaluation, registry, deployment, monitoring
- Version data with DVC, Delta Lake, or immutable datasets in object storage with naming conventions
- Set up feature stores (Feast, Tecton, Hopsworks) for consistent online/offline feature access, or skip for simple projects
- Track experiments with MLflow, W&B, or Neptune: log git hash, dataset version, hyperparameters, metrics per epoch, model artifacts, environment
- Manage model registry: versioned models with staging/production/archived lifecycle, model cards, promotion gates
- Choose deployment patterns: shadow (validation), canary (gradual rollout), A/B test (business metrics), blue-green (instant rollback), feature flag (kill switch)
- Monitor for drift: data drift (KS test, PSI), concept drift (prediction distribution shift, business metric tracking), operational metrics (latency, error rate, resource utilization)
- Implement ML CI/CD: unit tests and smoke training on CI, full evaluation + bias checks + shadow deployment + canary rollout on CD

??? example "Example usage"
    "Debugging model degradation in production" -- Checks data drift dashboard for feature distribution changes, looks for upstream pipeline failures, compares recent feature distributions to training data using PSI. If drift detected, identifies which features drifted and traces to root cause. If no drift, checks for concept drift by comparing predictions on recent labeled data. Recommends retraining on recent data if drift is confirmed.
