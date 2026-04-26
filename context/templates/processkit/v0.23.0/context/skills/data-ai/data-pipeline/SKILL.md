---
name: data-pipeline
description: |
  Data pipeline patterns — ETL/ELT, batch vs streaming, idempotency, orchestration. Use when designing a data pipeline, choosing between batch and streaming, implementing ingestion or transformation, setting up orchestration, or debugging pipeline failures.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-data-pipeline
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# Data Pipeline

## Intro

A reliable data pipeline is idempotent, observable, and stage-validated.
Default to ELT into a modern warehouse, default to batch over streaming,
and design every step to be safely re-runnable from day one.

## Overview

### ETL vs ELT

ETL (Extract, Transform, Load) transforms before loading; use it when the
target has limited compute or PII must be filtered before storage. ELT
(Extract, Load, Transform) loads raw data first and transforms in the
warehouse; prefer it for analytics on BigQuery, Snowflake, or Redshift
because it preserves raw data and allows re-transformation. Never split
transformation logic between pipeline and warehouse without documenting
which side owns the source of truth.

### Batch vs streaming

Start with batch unless there is an explicit real-time requirement. Batch
is simpler to build, debug, and retry. Streaming is required for
real-time dashboards, alerting, and event-driven systems but adds
ordering, late-arrival, and exactly-once complexity. Micro-batch (Spark
Structured Streaming, 1–5 minute intervals) bridges the gap when full
streaming is overkill.

### Idempotency

Every step must be safely re-runnable without duplicating data. Use
upserts (`INSERT ON CONFLICT UPDATE`) or merge statements for loading.
Partition output by date or run id and overwrite entire partitions on
rerun. Include a deduplication step that hashes natural-key columns
before loading. Running a job twice should produce the same result as
running it once.

### Data quality at every stage

Validate after extraction, after transformation, and after loading.
Compare source and destination row counts within tolerance. Check for
nulls in required columns and duplicates in unique keys. Validate value
ranges (dates in expected window, amounts non-negative). Fail fast on
violations rather than propagating bad data downstream, and log row
counts, null rates, and duplicate rates for every run.

### Orchestration

Use a workflow orchestrator (Airflow, Prefect, Dagster) for scheduling
and dependency management. Define DAGs with explicit task dependencies,
configure retries with exponential backoff (3 retries at 1/5/15 minute
delays), and prefer sensors or triggers over polling. Tag tasks with
owners and SLA expectations. Keep task definitions thin — business
logic lives in importable modules so it can be unit-tested.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Insert-only loads with no deduplication.** Running a pipeline twice — due to a retry, a bug fix re-run, or an overlapping schedule — produces duplicate rows if the load step is not idempotent. Design loads to upsert or deduplicate on a natural key, or use insert-if-not-exists semantics so re-runs are safe by default.
- **Transformation logic split between the pipeline and the warehouse.** When some transforms happen in pipeline code and others in warehouse SQL views, neither place is the full truth. A change in one layer silently breaks the other. Put all business logic in one layer — ideally SQL in the warehouse where it's versioned and queryable — and keep the pipeline as pure extraction and loading.
- **Adding streaming "for future-proofing" when batch suffices.** Streaming infrastructure (brokers, consumers, exactly-once semantics, late-arrival handling) adds substantial operational complexity. Unless the business requires sub-minute latency, start with scheduled batch and migrate to streaming only when latency is a demonstrated problem.
- **Quality checks only at the end of the pipeline.** Validating data only after full transformation means a bad upstream record causes the entire pipeline run to fail or produce corrupt output. Add checks immediately after ingestion on raw data so bad records are caught and quarantined before they propagate.
- **No date-range parameter for backfills.** A pipeline that hardcodes "process yesterday's data" cannot be used to reprocess a historical window without code changes. Parameterize the date range from day one so backfills are operational procedures, not engineering tasks.
- **DAGs hiding business logic in orchestration code.** Using the DAG framework (Airflow operators, etc.) to implement transformation logic couples business rules to infrastructure. Keep DAGs as wiring — trigger, sequence, and retry — and keep transformation logic in testable Python or SQL that runs outside the orchestrator.
- **Silently dropping malformed records.** A `try/except` that logs a warning and continues means corrupt or schema-violating records disappear without anyone being paged. Always count rejects, surface the count as a metric, and alert when it exceeds a threshold.

## Full reference

### Schema management

Use a schema registry (Avro, Protobuf, JSON Schema) for event-driven
pipelines. Enforce backward compatibility: new fields are optional,
existing fields are never removed or renamed. Version schemas
explicitly (`v1`, `v2`) with documented migration paths. Document each
field with name, type, description, example value, and nullability.
Test schema changes against consumers before deploying to production.

### Backfill strategies

Design pipelines to accept a date-range parameter from the start. Use
partition overwrite for backfills: reprocess and replace entire
partitions rather than mutating in place. Run backfills at lower
priority so they do not starve production workloads. Test on a small
date range before scaling up, and track which partitions have been
reprocessed.

### Monitoring and dead letter queues

Alert on pipeline failure, SLA breach, and unexpected row-count
changes (> 20% deviation). Log start time, end time, rows processed,
and rows rejected. Use dead letter queues for records that fail
processing — DLQ entries must include the original payload, error
message, timestamp, and source topic or table. Review and reprocess
DLQ records regularly; they represent data loss until resolved.

### Worked examples

- **Daily sales API to warehouse:** ELT in three stages — extract
  (paginated API call, raw JSON to cloud storage), load (bulk insert
  into a staging table), transform (SQL in the warehouse to clean,
  deduplicate, and join with dimensions). Date-range parameter for
  backfills, partition-overwrite loads, source/staging row-count
  checks, Airflow DAG with retries.
- **Duplicate records:** Diagnose the idempotency gap — insert-only
  load is the usual culprit. Add a natural-key hash column,
  switch to `INSERT ON CONFLICT`, add a pre-load dedupe step, and
  add a quality check comparing distinct key counts source vs target.
- **Streaming vs batch for events:** If events only feed daily
  reports, recommend batch with hourly micro-batches. If real-time
  alerting is needed, recommend Kafka-based streaming but warn about
  exactly-once complexity. Either way, design for idempotency and
  include a DLQ for malformed events.

### Anti-patterns

- Insert-only loads with no dedupe — guaranteed duplicates on retry
- Transformation logic split across pipeline and warehouse with no
  documented owner
- Streaming chosen for "future-proofing" without a real-time use case
- Quality checks only at the end of the pipeline (bad data has
  already propagated)
- DAGs that hide business logic inside task callables, making them
  untestable in isolation
