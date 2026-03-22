# Observability Skills

Skills for logging, monitoring, tracing, and alerting in production systems.

---

### logging-strategy

> Structured logging strategy including log levels, correlation IDs, context propagation, and PII avoidance. Use when designing logging, reviewing log statements, or setting up log aggregation.

**Triggers:** Designing a logging approach, reviewing existing log statements, setting up log aggregation (ELK, Loki, CloudWatch), adding correlation IDs, deciding what to log and what to avoid.
**Tools:** `Bash` `Read` `Write`
**References:** `structured-logging.md`

Key capabilities:

- Six log levels with clear usage guidance (TRACE through FATAL) and rules for choosing between them
- Structured logging (JSON/key-value) over unstructured text for machine parseability
- Correlation IDs: generate request_id at system boundary, propagate via X-Request-ID header, include in every log line
- W3C Trace Context propagation with trace_id and span_id for distributed systems
- MDC (Mapped Diagnostic Context) for transparent ID propagation
- What to log: request summaries, state transitions, decision points, errors, performance data, lifecycle events, retries, external calls
- What NOT to log: PII, secrets, sensitive business data, high-cardinality user input
- Log aggregation: centralized systems, JSON ingestion, retention policies (hot/warm/cold), rotation by size or time
- Performance considerations: lazy evaluation, avoid logging in tight loops, sampling, async appenders
- Anti-pattern detection: log-and-throw, everything-at-INFO, string concatenation, missing context, swallowed exceptions

??? example "Example usage"
    A REST API in Python has no structured logging. The agent recommends replacing
    the stdlib logging formatter with structlog, configures a processor chain
    adding timestamp, level, service, and request_id with JSON output, adds
    middleware to generate and propagate request_id, and sets log level via
    environment variable.

---

### metrics-monitoring

> Application metrics and monitoring using RED/USE methods, Prometheus patterns, and SLO-based alerting. Use when instrumenting applications, designing dashboards, or setting up monitoring.

**Triggers:** Instrumenting an application with metrics, designing monitoring dashboards, setting up alerting, choosing metric types, defining SLIs/SLOs, applying observability methodology (RED, USE, golden signals).
**Tools:** `Bash` `Read` `Write`
**References:** `metric-types.md`

Key capabilities:

- Four Golden Signals (Google SRE): latency, traffic, errors, saturation
- RED method for request-driven services: rate, errors, duration (p50/p95/p99)
- USE method for infrastructure resources: utilization, saturation, errors
- Prometheus metric types: counter, gauge, histogram, summary with naming conventions
- Dashboard design: golden signals first, percentile latency, layered dashboards, deployment markers
- Alerting thresholds: symptom-based over cause-based, SLO burn-rate alerting, baseline-derived thresholds, multi-window alerting
- SLI/SLO/SLA framework: quantitative indicators, target objectives, error budgets, budget-based feature freezes
- Instrumentation checklist: endpoint RED metrics, dependency metrics, queue depth, connection pools, business metrics, runtime metrics

??? example "Example usage"
    Define SLOs for an e-commerce platform. The agent proposes three SLIs:
    availability (non-5xx > 99.95%), latency (p99 checkout < 1s at 99.9%),
    correctness (order-inventory match > 99.99%). Sets 30-day rolling windows,
    calculates error budgets (22 min/month for availability), and recommends
    multi-window burn-rate alerts.

---

### distributed-tracing

> Distributed tracing with OpenTelemetry including spans, traces, context propagation, and sampling strategies. Use when instrumenting distributed systems, debugging request flows, or setting up tracing infrastructure.

**Triggers:** Instrumenting a distributed system with tracing, debugging requests spanning multiple services, setting up OpenTelemetry, choosing sampling strategies, understanding latency across service boundaries.
**Tools:** `Bash` `Read` `Write`
**References:** None

Key capabilities:

- Core concepts: traces, spans (with parent-child tree structure), context propagation, baggage
- OpenTelemetry architecture: SDK, API, Exporter, Collector, auto-instrumentation
- Instrumentation strategy: auto-instrumentation first, then manual spans for business logic
- Span naming (`component.operation`), attributes (HTTP, DB, business context), and error recording
- Context propagation: W3C Trace Context (traceparent/tracestate), B3, message queues, async boundaries
- Head-based sampling: probabilistic and rate-limiting approaches
- Tail-based sampling via OTel Collector: error-based, latency-based, and policy-based rules
- Trace analysis for debugging: critical path identification, gap detection, fast-vs-slow comparison, waterfall analysis
- Common pitfalls: missing propagation, over-instrumentation, sensitive data in attributes, no production sampling, broken async traces

??? example "Example usage"
    A checkout endpoint sometimes takes 10 seconds but usually 200ms. The agent
    searches for slow traces, examines the waterfall, and identifies 9 seconds
    spent calling the inventory service which makes sequential DB queries per item.
    Slow traces correlate with large carts (>20 items). Recommends batching the
    DB query and adding a cart.item_count span attribute.

---

### alerting-oncall

> Alert design and on-call practices including severity levels, runbooks, SLO-based alerting, and escalation policies. Use when designing alerts, writing runbooks, or improving on-call processes.

**Triggers:** Designing alerts for a service, writing runbooks, reducing alert fatigue, setting up escalation policies, implementing SLO-based alerting, improving on-call processes.
**Tools:** None
**References:** None

Key capabilities:

- Alert severity levels: P1 (immediate, page), P2 (30 min, page), P3 (4 hours, ticket), P4 (1 business day, ticket)
- Page vs ticket decision framework based on user impact and intervention urgency
- SLO-based burn-rate alerting with multi-window thresholds (14.4x/5min+1hr for P1, 6x/30min+6hr for P2, 3x/2hr+24hr for P3)
- Runbook template: what it means, likely causes, diagnosis steps, mitigation, escalation
- Alert fatigue prevention: track volume (<2 pages/shift target), weekly review, merge related alerts, auto-resolve transients
- Escalation policies: primary to secondary (10 min P1, 30 min P2), to engineering lead (1hr P1, 4hr P2), incident declaration
- On-call handoff practices: consistent timing, written summaries, acknowledgment, shadow rotations for new members
- Incident communication: dedicated channel, initial summary, 15-30 min updates, status page, post-incident review within 48 hours

??? example "Example usage"
    A team gets paged 15 times a week, mostly false alarms. The agent audits 30
    days of alerts, categorizes by action taken (mitigated/auto-resolved/no-action),
    identifies the top 3 noisy alerts, raises thresholds or converts them to
    tickets, implements alert grouping, and sets a target of <2 pages per on-call
    shift with weekly alert review.
