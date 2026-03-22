# Data & Analytics Skills

Skills for data science, data engineering, and analytics workflows.

---

### data-science

> Data analysis workflow from import through modeling and communication. Covers tidy data, EDA, statistical reasoning, and visualization. Use when analyzing datasets, building statistical models, exploring data, or communicating findings.

**Triggers:** Analyzing datasets, exploring data, building statistical models, creating visualizations, cleaning messy data, communicating findings
**Tools:** `Bash(python:*) Bash(jupyter:*) Read Write`
**References:** `tidy-data-principles.md`, `statistical-methods.md`, `visualization-guidelines.md`

Key capabilities:

- Import and clean data: inspect shape/dtypes/nulls, handle missing data explicitly, parse dates, validate assumptions
- Reshape data to tidy form (one variable per column, one observation per row) using melt/pivot
- Conduct exploratory data analysis: univariate distributions, bivariate relationships, outlier detection, groupby aggregations
- Apply statistical reasoning: state the question first, check assumptions, report effect sizes alongside p-values, use confidence intervals
- Perform feature selection: remove zero-variance features, handle multicollinearity, use domain knowledge then data-driven methods
- Follow model selection workflow: start simple (baseline), add complexity only when justified, use cross-validation, document decisions
- Visualize with best practices: titles, axis labels, colorblind-friendly palettes, annotations, publication-quality export
- Communicate results: lead with findings, plain language, show uncertainty, include actionable "so what"

??? example "Example usage"
    "I have a CSV of customer transactions. Help me understand churn patterns." -- Loads the CSV, prints shape/dtypes/nulls, creates tidy time-series per customer, runs EDA with churn-rate distributions and cohort analysis, tests whether usage frequency differs between churned/retained groups (t-test with effect size), and produces annotated visualizations summarizing the key drivers.

---

### data-pipeline

> Data pipeline patterns including ETL/ELT, batch vs streaming, idempotency, and orchestration. Use when designing data pipelines, reviewing data workflows, or troubleshooting data processing.

**Triggers:** Designing data pipelines, choosing batch vs. streaming, implementing data ingestion or transformation, setting up orchestration, debugging pipeline failures
**Tools:** `Bash Read Write`
**References:** None

Key capabilities:

- Choose between ETL and ELT based on target system capabilities and governance requirements
- Decide batch vs. streaming: start with batch unless explicit real-time requirement, micro-batch as middle ground
- Ensure idempotency: upserts, partition overwrite on rerun, deduplication by natural key hash
- Implement data quality checks at each stage: row counts, null checks, value range validation, fail fast on violations
- Manage schemas with registries, backward compatibility, versioning, and field documentation
- Design backfill strategies with date-range parameters, partition overwrite, and progress tracking
- Set up orchestration (Airflow, Prefect, Dagster) with explicit DAG dependencies, retries with exponential backoff, and SLA tagging
- Monitor pipelines with alerts on failure/SLA breach, metadata logging, and dead letter queues for failed records

??? example "Example usage"
    "Design a pipeline to load daily sales data from an API into our warehouse" -- Designs an ELT pipeline: extract (API call with pagination, save raw JSON to cloud storage), load (bulk insert into staging), transform (SQL in warehouse to clean, deduplicate, join). Adds date-range parameters for backfills, idempotent loads via partition overwrite, row-count checks, and an Airflow DAG with retries.

---

### data-visualization

> Data visualization best practices including chart selection, color accessibility, and dashboard design. Use when creating charts, designing dashboards, or reviewing data presentations.

**Triggers:** Creating charts, designing dashboards, choosing visualization types, improving chart readability, reviewing data presentations
**Tools:** `Bash(python:*) Read Write`
**References:** `chart-selection.md`

Key capabilities:

- Select chart types by data relationship: bar for comparison, line for trend, histogram for distribution, scatter for relationship
- Apply color and accessibility: colorblind-friendly palettes (viridis, cividis), sequential/diverging/categorical schemes, 7-color maximum
- Annotate insights: max/min values, threshold lines, trend-change events, direct labels instead of legends
- Design dashboard layouts: most important metric top-left, consistent grid, grouped by domain, filters at top, 6-8 visualizations max
- Tell stories with data: lead with conclusion, context-finding-implication structure, progressive disclosure, highlight the relevant
- Choose static vs. interactive: matplotlib/seaborn for reports, plotly/Altair/D3 for dashboards with tooltips and zoom
- Avoid common mistakes: truncated y-axis on bars, dual y-axes, overplotting, missing units, default titles, excess decimal places

??? example "Example usage"
    "This dashboard has 15 charts and stakeholders say it is overwhelming" -- Audits for redundancy, groups remaining charts by business domain, moves detail charts to drill-down pages, keeps 6 key metrics on the main view, and adds a summary card row at the top with KPIs and sparklines.

---

### feature-engineering

> Feature engineering for ML including encoding, imputation, scaling, selection, and time-series features. Use when preparing data for ML models, selecting features, or engineering new features from raw data.

**Triggers:** Preparing data for ML models, encoding categorical variables, handling missing data, creating new features, selecting features, building time-series features
**Tools:** `Bash(python:*) Read Write`
**References:** None

Key capabilities:

- Encode categorical variables: one-hot (< 15 values), ordinal (natural order), target encoding (high cardinality, cross-validated), frequency encoding
- Handle missing data: understand missingness mechanism (MCAR/MAR/MNAR), median/mode imputation, binary indicator columns, KNN/MICE for correlated features
- Scale features: StandardScaler for linear models, MinMaxScaler for neural networks, RobustScaler for outlier-heavy data; tree models need no scaling
- Select features: filter methods (correlation, mutual information), wrapper methods (RFE), embedded methods (L1, tree importance, permutation importance)
- Engineer time-series features: lag values, rolling statistics, seasonal extraction (hour/day/month with sin/cos encoding), difference features, expanding statistics
- Create text features: TF-IDF, count vectorizer, pre-trained embeddings, extracted features (length, sentiment, readability)
- Build interaction and derived features: multiplication, ratios, polynomial terms, domain-driven binning, log transforms
- Prevent data leakage: fit transformers on training data only, respect temporal ordering, use sklearn Pipeline

??? example "Example usage"
    "I have a dataset with user_id, city, purchase_amount, and timestamp. How should I engineer features?" -- Target-encodes city with cross-validated means, extracts day_of_week/hour/is_weekend from timestamp, creates lag features for previous purchase amounts per user, adds rolling 7-day and 30-day purchase means, and creates a days_since_last_purchase feature, all wrapped in a sklearn Pipeline.

---

### data-quality

> Data quality framework covering completeness, accuracy, consistency, validation rules, and data contracts. Use when implementing data validation, setting up data quality checks, or defining data contracts.

**Triggers:** Implementing data validation, setting up quality checks for pipelines, defining data contracts between teams, investigating data anomalies
**Tools:** `Bash Read Write`
**References:** None

Key capabilities:

- Evaluate data against six dimensions: completeness, accuracy, consistency, timeliness, uniqueness, validity
- Define validation rules: schema validation, range checks, format checks, referential integrity, business rules, freshness checks -- categorized by severity (error vs. warning)
- Detect anomalies: track metrics over time (row counts, null rates, distinct values), alert on threshold deviations, monitor distribution shifts and schema changes
- Define data contracts: formal agreements on schema, SLAs, quality thresholds, ownership -- versioned and machine-readable (JSON Schema, protobuf, YAML)
- Implement Great Expectations patterns: expectation suites per table, core expectations (row count, not null, unique, in set), pipeline integration
- Set up data observability: metadata instrumentation, lineage graphs, health dashboards, automated root cause analysis
- Place quality checks at pipeline boundaries with stored results, configurable thresholds, and the five critical checks (row count, null rate, duplicate rate, freshness, schema match)

??? example "Example usage"
    "Set up data quality checks for our customer table" -- Implements checks across all six dimensions: completeness (null rate for email, name, created_at), uniqueness (customer_id has no duplicates), validity (email matches regex, status in allowed enum), consistency (country matches postal code format), timeliness (most recent created_at within 24 hours), and accuracy (spot check against CRM export).
