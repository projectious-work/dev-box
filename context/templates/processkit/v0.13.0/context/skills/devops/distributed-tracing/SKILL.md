---
name: distributed-tracing
description: |
  Distributed tracing with OpenTelemetry — spans, context propagation, sampling. Use when instrumenting a distributed system, debugging requests that span services, setting up OpenTelemetry, choosing a sampling strategy, or understanding latency across service boundaries.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-distributed-tracing
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
---

# Distributed Tracing

## Intro

Distributed tracing follows a single request through every service it
touches, exposing latency and error contributions per hop.
OpenTelemetry is the vendor-neutral standard. Start with
auto-instrumentation, propagate W3C Trace Context, and sample to
control cost.

## Overview

### Core concepts

A **trace** is the entire journey of a request, identified by a 128-bit
`trace_id`. A **span** is one unit of work inside a trace with a name,
start time, duration, status, and attributes; spans form a tree where
every non-root span has a parent. **Context** carries `trace_id`,
`span_id`, and `trace_flags` across process boundaries via HTTP
headers, message metadata, or gRPC metadata. **Baggage** is small
key-value data propagated alongside context for cross-cutting concerns
like tenant ID — keep it tiny because every outgoing request carries it.

### OpenTelemetry architecture

OpenTelemetry (OTel) is the standard. The **API** is the
vendor-neutral interface your code instruments against. The **SDK** is
the in-process library that creates and exports spans. **Exporters**
ship spans to backends (Jaeger, Zipkin, Tempo, Datadog). The
**Collector** is an optional proxy that decouples apps from backends
and is recommended for production. **Auto-instrumentation** libraries
create spans for common HTTP, gRPC, database, and messaging frameworks
with zero code changes.

### Instrumentation strategy

1. Start with auto-instrumentation — it covers infrastructure for free
2. Add manual spans for business operations (process order, validate
   payment, generate report)
3. Use `<component>.<operation>` span names: `OrderService.createOrder`
4. Add attributes for debugging context — HTTP method/url/status, DB
   system/statement/operation, and business fields like `order.id`
5. Record errors by setting span status to ERROR and attaching the
   exception as a span event
6. Keep spans focused on one logical operation; break compound work
   into child spans

### Context propagation

Context propagation is what stitches spans across service boundaries.
Use **W3C Trace Context** (the OTel default) with `traceparent` and
`tracestate` headers — example:
`traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`.
B3 propagation (Zipkin's `X-B3-*` headers) is still supported. For
message queues, propagate context in message headers or metadata, not
in the body. For async work, pass context **explicitly** when spawning
background tasks — it does not flow automatically across thread or
goroutine boundaries in most languages.

### Sampling

In high-traffic systems, tracing every request is expensive. Pick a
strategy:

- **Head-based probabilistic** — sample X% at trace root. Simple, can
  miss rare errors. Good starting point at 10-20%.
- **Head-based rate-limiting** — sample N traces per second. Prevents
  overload but biases toward high-traffic endpoints.
- **Tail-based** (requires the Collector) — decide after the trace
  completes. Always keep traces with errors or above a latency
  threshold; sample a small percentage of successes.
- **Always-on (100%)** — only for low-traffic or critical paths.

Recommended path: head-based probabilistic in apps, tail-based
error/latency policies in the Collector as traffic grows.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Missing context propagation at async boundaries.** Trace context does not flow automatically through message queues, goroutines, thread pools, or task schedulers. If a service publishes a message without embedding the `traceparent` header, the consumer starts a new disconnected trace — the cross-service journey is invisible. Always pass context explicitly when crossing async boundaries, and inject it into message headers or job metadata.
- **Over-instrumentation — a span per function call.** Adding a span to every helper function generates enormous trace data, makes waterfalls unreadable, and increases cost with no debugging value. Spans should represent meaningful operations: an HTTP call, a database query, a cache lookup, a business action. Instrument at operation boundaries, not code boundaries.
- **Sensitive data in span attributes.** Span attributes are stored in the tracing backend and may be visible to anyone with access to traces. Embedding PII (email addresses, user IDs mapped to identities), credentials, or full SQL query parameters in attributes creates a data exposure risk. Log only safe structural identifiers — numeric IDs, status codes, operation names.
- **100% sampling in production without a plan.** Emitting a span for every request in a service handling thousands of RPS will rapidly overwhelm the tracing backend and budget. Set a sampling rate before going to production — head-based probabilistic at 10-20% is a reasonable starting point; add tail-based error/latency retention in the Collector as traffic grows.
- **Treating trace_id and request_id as interchangeable.** A `trace_id` is created by the tracing SDK and follows the OpenTelemetry propagation standard. A `request_id` is typically generated at the API boundary for customer-facing correlation. They serve different purposes: `trace_id` links to the tracing backend; `request_id` appears in API responses and error messages. Propagate both and include `trace_id` in log lines so you can pivot from logs to traces.
- **Ignoring the gaps between spans.** A gap between a parent span's start and its first child's start often represents scheduler delay, queue wait time, or serialization overhead — not actual work. Reading only span durations and missing gaps will misattribute latency. Look at the full waterfall including gaps when diagnosing slowness.
- **No Collector in production.** Sending spans directly from applications to a tracing backend couples apps to the backend's endpoint, authentication, and retry behavior. A Collector decouples instrumentation from backend, enables tail-based sampling, allows backend migration without redeployment, and buffers spans during backend outages.

## Full reference

### Trace analysis for debugging

1. Start from the symptom — find traces matching the error or latency
2. Identify the **critical path**: the longest chain of sequential
   spans determines total latency
3. Look for gaps between a parent span and its children — they
   indicate queueing or scheduling delay, not work
4. Check span attributes for error messages, SQL queries, cache
   hit/miss markers
5. Compare a fast trace with a slow one to spot divergence
6. Read the waterfall to see parallel vs sequential execution and
   spot serial calls that could be parallelized

### Common pitfalls

- **Missing context propagation** — disconnected traces almost always
  mean a service is not forwarding `traceparent`
- **Over-instrumentation** — a span per function call generates noise
  and overhead; spans should be meaningful operations
- **Sensitive data in attributes** — never put PII, credentials, or
  full SQL parameters in spans
- **No sampling in production** — 100% sampling at scale will
  overwhelm your tracing backend and budget
- **Ignoring async boundaries** — traces break when context is not
  passed to background jobs, message consumers, or thread pools
- **Confusing trace_id with request_id** — propagate both; logs
  should carry trace_id so you can pivot from logs into traces

### Cost-control tactics

When tracing volume gets expensive, look at the source mix first.
Health checks are usually 30-50% of all spans and contribute zero
debugging value — exclude them via the sampler. Audit middleware that
emits multiple spans per request and collapse to one. Deploy a
Collector with tail-based sampling that keeps 100% of errors, 100%
above a latency threshold, and 5% of successful requests. This pattern
typically cuts span volume 70-80% while preserving every interesting
trace.
