# ML Pipeline Stages Reference

Each stage with tools, best practices, and common pitfalls.

## 1. Data Collection and Ingestion

Gather raw data from sources into a central location.

**Tools:** Apache Airflow (orchestration), Apache Kafka (streaming ingestion), AWS Glue / GCP Dataflow (managed ETL), custom scripts for APIs and scraping.

**Best practices:**
- Log data source, collection timestamp, and schema version with every batch
- Validate data on ingestion: schema checks, null rate checks, row count sanity
- Store raw data immutably — never modify ingested data in place
- Separate raw zone (as-collected) from processed zone (cleaned, transformed)

**Common pitfalls:**
- Silently dropping malformed records without logging. Always count and log rejects.
- Assuming data sources are stable. Upstream schemas change without notice.
- Not handling late-arriving data in streaming pipelines.

## 2. Data Versioning and Lineage

Track what data was used to produce what artifact.

**Tools:** DVC (file-level versioning), Delta Lake (table-level versioning), LakeFS (Git-like branching for data lakes), Pachyderm (container-based data versioning).

**Best practices:**
- Every dataset has a version identifier tied to its content (hash-based or sequential)
- Training runs reference exact dataset versions in their metadata
- Data lineage: trace any model prediction back to the source data that trained it
- Automate: data version should be set by the pipeline, not by a human

**Common pitfalls:**
- Overwriting datasets in place ("just re-run the ETL"). Always create new versions.
- Versioning only the final dataset, not intermediate transformations.
- No lineage tracking — impossible to debug a bad model months later.

## 3. Feature Engineering

Transform raw data into model-ready features.

**Tools:** Feast (feature store), Tecton (managed feature platform), pandas/polars (batch), Apache Flink (streaming features), dbt (SQL-based transformations).

**Best practices:**
- Feature definitions are code, checked into version control
- Same feature computation logic for training (offline) and serving (online) — prevent training-serving skew
- Unit test feature transformations: given input X, expect output Y
- Document each feature: definition, source, expected distribution, known issues

**Common pitfalls:**
- Training-serving skew: features computed differently offline vs. online. Use a feature store or shared transformation code.
- Data leakage: using future information as a feature. Always respect temporal boundaries.
- Over-engineering features before having a baseline model.

## 4. Model Training

Train models with tracked experiments and reproducible configurations.

**Tools:** MLflow (tracking), W&B (tracking + viz), PyTorch / TensorFlow / scikit-learn (frameworks), Ray Train (distributed training), Optuna / Ray Tune (hyperparameter search).

**Best practices:**
- Every training run logs: code version, data version, hyperparameters, metrics, artifacts
- Use config files (YAML/TOML) for hyperparameters, not hardcoded values
- Train on a fixed random seed for reproducibility
- Start simple: baseline model first, then iterate
- Use early stopping to prevent overfitting and save compute

**Common pitfalls:**
- Not logging experiments — losing track of what configuration produced which results.
- Tuning hyperparameters on the test set (data leakage). Use train/validation/test split.
- Training on all available data without holding out a test set.

## 5. Model Evaluation

Assess model quality before promotion.

**Tools:** MLflow (metric comparison), Great Expectations (data validation), custom eval scripts, RAGAS (for RAG pipelines).

**Best practices:**
- Evaluate on a held-out test set that was never used during training or tuning
- Report multiple metrics: accuracy is rarely sufficient alone (precision, recall, F1, AUC)
- Evaluate across subgroups (demographics, data slices) to detect bias
- Compare against the current production model, not just a random baseline
- Define promotion criteria: "model must beat production by >1% on F1 and not regress on any subgroup by >2%"

**Common pitfalls:**
- Using a single aggregate metric that hides subgroup failures.
- Evaluating on data that overlaps with training data.
- Not comparing against the current production model.

## 6. Model Registry

Catalog and manage model lifecycle.

**Tools:** MLflow Model Registry, Vertex AI Model Registry, SageMaker Model Registry, custom (database + object storage).

**Best practices:**
- Register model with: name, version, metrics, training data version, model card
- Lifecycle stages: `development` -> `staging` -> `production` -> `archived`
- Promotion requires passing automated evaluation gates
- One production model per model name; rollback = promote previous version

**Common pitfalls:**
- No registry — models live on someone's laptop or in an undocumented S3 path.
- No model card — six months later, nobody knows what data or metrics apply.
- Manual promotion without automated quality gates.

## 7. Model Deployment and Serving

Get the model into production where it can make predictions.

**Tools:** TorchServe, TF Serving, Triton Inference Server, BentoML, FastAPI (custom), Seldon Core (Kubernetes), SageMaker Endpoints.

**Best practices:**
- Shadow deploy before serving real traffic
- Canary deploy with automated rollback on metric degradation
- Version the serving API: `/v1/predict`, `/v2/predict`
- Input validation on every request: reject malformed or out-of-range inputs
- Set timeouts and fallback behavior (return default prediction or error)

**Common pitfalls:**
- Deploying directly to 100% traffic without shadow or canary phase.
- No input validation — garbage in, garbage out, and hard to debug.
- No rollback plan — if the new model is bad, how do you revert?

## 8. Monitoring and Retraining

Detect degradation and trigger model updates.

**Tools:** Evidently AI (drift detection), Prometheus + Grafana (operational metrics), WhyLabs (ML monitoring platform), custom dashboards.

**Best practices:**
- Monitor input feature distributions daily (KS test, PSI, JS divergence)
- Monitor prediction distribution (is the model suddenly predicting one class 90% of the time?)
- Track business metrics alongside model metrics (correlation between model accuracy and revenue)
- Define retraining triggers: scheduled (every N days), drift-based (PSI > threshold), performance-based (metric below threshold)
- Automate the retraining pipeline: trigger -> retrain -> evaluate -> promote (with human approval gate)

**Common pitfalls:**
- No monitoring — model degrades silently for months.
- Monitoring only operational metrics (latency, errors) without model quality metrics.
- Retraining too frequently (expensive, unstable) or too rarely (stale model).

---

## Pipeline Orchestration Tools Comparison

| Tool | Type | Best For |
|------|------|----------|
| Apache Airflow | General DAG orchestration | Complex workflows, many integrations |
| Kubeflow Pipelines | Kubernetes-native ML pipelines | Teams already on Kubernetes |
| Vertex AI Pipelines | Managed (GCP) | GCP-native teams, minimal ops |
| SageMaker Pipelines | Managed (AWS) | AWS-native teams, minimal ops |
| Prefect / Dagster | Modern Python orchestration | Python-first teams, better DX than Airflow |
| ZenML | ML-specific orchestration | ML teams wanting pipeline abstraction |
