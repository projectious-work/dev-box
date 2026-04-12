---
name: metrics-management
description: |
  Manage Metric entities — quantified observations the project cares about (velocity, error rates, lead time, NPS). Use when defining or recording a quantitative measure — team velocity, error rate, lead time, customer satisfaction score.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-metrics-management
    version: "1.0.0"
    created: 2026-04-06T00:00:00Z
    category: devops
    layer: 4
    uses:
      - skill: event-log
        purpose: Log events to keep the audit trail accurate after every write.
    provides:
      primitives: [Metric]
      templates: [metric]
---

# Metrics Management

## Intro

A Metric is a named, quantified observation — something the project measures
and cares about. Examples: team velocity per sprint, p99 latency, release
frequency, error rate, customer NPS.

## Overview

### Metric vs observation

The Metric primitive defines *what is measured*. Individual readings of the
metric are LogEntries (`event_type: metric.recorded`) or external time-series
data. processkit stores the definition, not the time series.

### Shape

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Metric
metadata:
  id: METRIC-velocity
  created: 2026-04-06T00:00:00Z
spec:
  name: velocity
  description: "Story points completed per sprint."
  unit: points
  direction: higher-is-better
  measurement: "Sum of spec.estimate.value for WorkItems with spec.state=done in the sprint's SCOPE."
  target: 40
  tolerance: "±20%"
  cadence: sprint
  owner: ACTOR-tech-lead
---
```

### Recording observations

Individual measurements are LogEntries:

```yaml
kind: LogEntry
spec:
  event_type: metric.recorded
  timestamp: 2026-04-14T17:00:00Z
  actor: ACTOR-claude
  subject: METRIC-velocity
  subject_kind: Metric
  summary: "Sprint 42 velocity: 38 points"
  details:
    value: 38
    scope: SCOPE-sprint-42
```

The index MCP server (Phase 3) joins Metric definitions with their LogEntry
observations to produce time series.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Defining a Metric without a measurement source.** "We want to
  improve velocity" is not a Metric; "Story points completed per
  sprint, measured from `query_workitems(state=done, scope=sprint-N)`"
  is. Without a source, the Metric can't be measured and becomes
  decoration.
- **Setting target values without a baseline.** "Reduce error rate
  by 50%" is meaningless if you don't know the current rate.
  Always record the baseline at Metric creation time, and update
  it explicitly when you reset goals.
- **Confusing leading vs lagging indicators.** A leading indicator
  predicts future outcomes (PR review turnaround time); a lagging
  indicator measures past outcomes (incidents per quarter). Don't
  optimize a lagging indicator if a leading one is available —
  feedback loops are slower.
- **Metrics without owner or cadence.** "Who looks at this and how
  often" is part of the Metric definition, not an afterthought.
  Unowned metrics rot.
- **Storing measurement values inline in the Metric entity.**
  Metric entities define WHAT is measured, not the measurements
  themselves. Time-series values belong in an external store
  (Prometheus, BigQuery, …); the Metric entity references the
  store, not the data.
- **Vanity metrics that don't drive decisions.** "Total LOC", "PR
  count", "tickets closed" sound impressive but don't predict
  anything actionable. If knowing the number wouldn't change what
  you do next, it's vanity — drop it.
- **Hand-waving "trending up" without numbers.** When asked about
  a Metric's status, fetch the actual value from the source the
  Metric points at; don't synthesize a trend from memory.

## Full reference

### Fields

| Field         | Type     | Notes                                               |
|---------------|----------|-----------------------------------------------------|
| `name`        | string   | kebab-case identifier                               |
| `description` | string   | What this metric measures                           |
| `unit`        | string   | `points` / `ms` / `%` / `count` / etc.              |
| `direction`   | enum     | `higher-is-better` / `lower-is-better` / `target`   |
| `measurement` | string   | How to compute the value                            |
| `target`      | any      | Desired value (number or string)                    |
| `tolerance`   | string   | Acceptable deviation from target                    |
| `cadence`     | string   | How often it's measured                             |
| `owner`       | string   | Actor ID responsible for the metric                 |
| `active`      | bool     | `false` = no longer tracked                         |

### Vanity metrics

Avoid metrics that always trend up just by activity (commits per week, lines
of code) — they reward effort, not outcome. Favor metrics that can get
worse when the project is unhealthy: lead time, defect rate, customer
satisfaction, mean-time-to-recovery.

### Metrics vs goals

A goal is a desired future state recorded in a Scope's `spec.goals`. A Metric
is the quantitative thing the goal is measured by. One goal may reference
multiple metrics; one metric may inform multiple goals.
