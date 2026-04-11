---
name: data-quality
description: |
  Data quality framework — completeness, accuracy, consistency, validation, and contracts. Use when implementing data validation, setting up quality checks for a pipeline, defining data contracts between teams, or investigating data anomalies.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-data-quality
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# Data Quality

## Intro

Data quality is measurable across six dimensions and enforced with
explicit validation rules at every pipeline boundary. Codify producer
and consumer expectations as data contracts and run them as automated
checks rather than relying on manual review.

## Overview

### The six dimensions

Evaluate data against:

- **Completeness** — null rate per column, row count vs expected
- **Accuracy** — spot checks against a source of truth or reference
- **Consistency** — cross-field validation (end >= start, city matches
  zip)
- **Timeliness** — freshness since last update, SLA adherence
- **Uniqueness** — distinct count of natural keys vs total rows
- **Validity** — regex matches, range checks, enum membership

### Validation rules

Schema validation enforces column names, types, and nullability. Range
checks bound numerics (age 0–150, price >= 0). Format checks enforce
ISO 8601 dates, email regexes, normalized phone numbers. Referential
integrity confirms foreign keys resolve. Business rules express domain
truths (order total = sum of line items). Freshness checks confirm the
latest record is within the expected window. Categorize each rule by
severity: `error` blocks the pipeline, `warning` logs and continues.

### Anomaly detection

Track key metrics over time: row counts, null rates, distinct value
counts, mean and median. Alert when metrics deviate beyond a threshold
(row count drops > 20% vs previous run). Use z-score for normally
distributed metrics, IQR for skewed ones. Monitor distribution shifts
and schema changes (new columns, type changes, dropped columns). Watch
for sudden spikes in null rates or default values.

### Data contracts

A data contract is a formal agreement between producer and consumer
specifying schema, SLAs, quality thresholds, and ownership. Make
contracts machine-readable (JSON Schema, protobuf, YAML) and enforce
them automatically. Version them — breaking changes require a new
version with a migration period. Include who to contact when violated
and the escalation path. Review quarterly or when business
requirements change.

### Implementing checks

Place checks at pipeline boundaries: after extraction, after
transformation, after loading. Compare source to destination row
counts. Store results in a quality metadata table (timestamp, check
name, result, details). Start with the five critical checks — row
count, null rate on key columns, duplicate rate, freshness, schema
match — and add more as issues arise. Keep thresholds in config files,
not hardcoded.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Checks only at the end of the pipeline.** Validating data after full transformation means a bad upstream record travels the entire pipeline before failing, corrupting intermediate outputs along the way. Add checks at ingestion on raw data and at each transformation boundary so failures are caught and quarantined early.
- **Hardcoded thresholds disconnected from historical baselines.** A null rate threshold of 5% chosen arbitrarily will either fire constantly on a column that naturally has 4% nulls, or miss a regression on a column that usually has 0%. Derive thresholds from historical distributions — mean ± N standard deviations — and update them as the data evolves.
- **Soft warnings that nobody monitors.** A check that logs a warning but does not block the pipeline or alert anyone is not a data quality gate — it is documentation that the data is wrong. Quality checks must have a defined consequence: block the run, quarantine the records, or page on-call. Unmonitored warnings become noise within a week.
- **Data contracts in a wiki instead of enforceable code.** A contract documented in Confluence can drift silently from the actual schema for months before anyone notices. Enforce contracts as code — schema validation in tests or a contract-testing framework — so that upstream changes that violate the contract fail the pipeline immediately.
- **Schema drift detection without alerting the producer.** Detecting that an upstream schema changed is only half the job. If the alert goes only to the pipeline owner, the producer never learns they broke a downstream consumer. Schema drift alerts should notify both the pipeline owner and the upstream team so the source of the change is aware.
- **Validating only schema, not values.** A column typed as INTEGER passes schema validation even when it contains -1 where only positive values are valid, or 0 where null is the intended sentinel. Value-level checks (range, enum membership, referential integrity) catch a different class of errors than schema checks and are equally important.
- **No lineage tracking for quarantined records.** When records fail quality checks and are quarantined, there must be a way to trace which source records were rejected, why, and what downstream impact the rejection had. Without lineage, quarantined records are invisible holes in the data that are nearly impossible to audit or recover.

## Full reference

### Great Expectations patterns

Organize expectations into suites — one per table or pipeline stage.
Always include:

- `expect_table_row_count_to_be_between` — catch empty or exploding
  tables
- `expect_column_values_to_not_be_null` — for required columns
- `expect_column_values_to_be_unique` — for natural keys
- `expect_column_values_to_be_in_set` — for enum columns

Run validation as a pipeline step and fail on critical expectation
failures. Store validation results for trend analysis and audit
trails.

### Data observability

Instrument pipelines with metadata: row counts, schema snapshots,
execution time. Build a lineage graph so you know where each dataset
comes from and what depends on it. Dashboard freshness, volume trends,
and per-table error rates. Alert on stale data, volume anomalies,
schema drift, and quality-rule failures. When quality drops, automate
root-cause traversal back through lineage to find the source.

### Worked examples

- **Customer table checks:** Completeness on email, name, created_at;
  uniqueness on customer_id; validity on email regex and status enum;
  consistency between country and postal-code format; timeliness on
  most-recent created_at within 24 hours; accuracy via spot check
  against a CRM export. Store results with severity labels.
- **Daily report numbers look wrong but pipeline succeeded:** Add
  anomaly detection that tracks row counts and key aggregates and
  alerts on > 2σ deviation from the 30-day rolling average. Add
  cross-field consistency checks and a freshness check before the
  report runs.
- **Events-to-analytics contract:** Versioned YAML specifying schema,
  null rate < 1% for user_id, row count > 1000/day, freshness SLA
  06:00 UTC, ownership and escalation contacts. Automated validation
  on each delivery, notifications to both teams on violations.

### Anti-patterns

- Quality checks only at the end of the pipeline
- Hardcoded thresholds that nobody can tune without a deploy
- "Soft" warnings that nobody monitors — equivalent to silence
- Contracts that live in a wiki page instead of as enforceable code
- Schema drift detection without an alert path to the producer
