---
name: load-testing
description: |
  Load testing methodology — test types, scenario design, capacity planning, CI integration. Use when planning or running a load test, choosing a load testing tool, designing scenarios, analyzing results, setting up performance testing in CI, or estimating capacity from test data.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-load-testing
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Load Testing

## Intro

Load tests answer specific questions: does it work, can it handle
expected traffic, where does it break, can it survive a spike, is
there long-term degradation. Always start with a smoke test, model
real user behavior, and capture latency percentiles — averages hide
problems.

## Overview

### Test types

Each type answers a different question:

| Type | VUs / Load | Duration | Question It Answers |
|---|---|---|---|
| **Smoke** | 1-5 VUs | 1-2 min | Does it work at all under minimal load? |
| **Load** | Expected traffic | 10-30 min | Can it handle normal production traffic? |
| **Stress** | 2-5x expected | 10-30 min | Where does it break? What fails first? |
| **Spike** | Sudden burst (10x) | 5-10 min | Can it handle sudden traffic surges? |
| **Soak / Endurance** | Normal traffic | 2-12 hours | Are there memory leaks or degradation over time? |

Always smoke first. A failing smoke test means you have a setup
problem, not a capacity problem.

### Realistic scenarios

A useful load test simulates real user behavior, not raw request
throughput:

1. Identify key user journeys (login -> browse -> search -> checkout)
2. Use realistic data — randomize IDs, search terms, and product IDs
   from a dataset
3. Include think time (1-5 s pauses between actions)
4. Model traffic distribution (e.g. 60% browse, 25% search, 10% add
   to cart, 5% checkout)
5. Include authentication: token refresh, session management
6. Account for downstream dependencies (payment providers, etc.)

```javascript
// k6: realistic scenario with stages
import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
  stages: [
    { duration: '2m', target: 50 },   // ramp up
    { duration: '5m', target: 50 },   // hold steady
    { duration: '2m', target: 200 },  // stress
    { duration: '5m', target: 200 },  // hold stress
    { duration: '2m', target: 0 },    // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1500'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const res = http.get('https://api.example.com/products');
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(Math.random() * 3 + 1);  // 1-4s think time
}
```

### What to capture

**Latency**: p50, p95, p99, and max — averages hide problems. Track
per endpoint, not just aggregate. **Throughput**: successful
requests/sec. **Error rate**: 5xx, timeouts, connection refused.
**Resource utilization** from monitoring: CPU per service, memory RSS
and heap, connection pool usage, queue depth, network I/O, open
connections.

### Tool overview

| Tool | Language | Strengths |
|---|---|---|
| **k6** | JavaScript | Developer-friendly scripting, good CLI output, Grafana integration |
| **Locust** | Python | Easy to write complex scenarios, distributed mode, web UI |
| **Gatling** | Scala/Java | High performance, detailed HTML reports, CI-friendly |
| **wrk** | Lua (config) | Extremely high throughput for simple URL hammering |
| **hey** | Go | Simple CLI for quick benchmarks, no scripting needed |
| **vegeta** | Go | Constant-rate load generation, good for precise RPS targets |

Use k6 or Locust for realistic scenario testing. Use wrk or hey for
quick single-endpoint benchmarks.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Running the load test from a single client machine that becomes the bottleneck.** A client generating 10,000 concurrent connections can exhaust its own ephemeral ports (typically 28,000), CPU, or memory before the server does, producing results that reflect the client's limits, not the server's. Distribute load generation across multiple machines or use a cloud-native load testing service for high VU counts.
- **Hammering a single endpoint and declaring the system load-tested.** A test that hits `GET /products` 1000 times per second does not exercise authentication, write paths, search, or checkout — the code paths most likely to have bottlenecks. Model real user journeys with realistic traffic distribution across endpoints; include writes, auth flows, and the paths that touch the most downstream dependencies.
- **Ignoring warm-up — measuring cold-start metrics as if they are steady-state.** The first 30–60 seconds of a load test often exercises JVM JIT compilation, connection pool warm-up, cold caches, and application initialization. These latencies are real but not representative of steady-state performance. Either discard the warm-up period from analysis or run a dedicated warm-up stage before measuring.
- **Comparing results across test runs without controlling for environment state.** A latency improvement between Tuesday and Wednesday's runs might reflect cache state, different co-tenant load on a shared cloud host, or CPU frequency scaling — not your change. Run comparison tests back-to-back in the same environment with controlled initial state; treat cross-day comparisons as indicative, not conclusive.
- **Reporting average latency instead of percentiles.** Average latency hides the long tail. A system with p50=50ms and p99=5000ms has an "average" of roughly 100ms that sounds acceptable — while 1% of users wait 5 seconds. Always report p50, p95, and p99. Use p99 as the primary SLA metric because it captures the worst experience of real users.
- **Disabling auth, rate limiting, or caching to simplify the test.** Bypassing middleware to make the test simpler also removes the overhead that exists in production, producing results that cannot be extrapolated to real behavior. Test through the full production stack — with auth, rate limiting, and caching as configured — unless the specific goal is to isolate the application tier in isolation.
- **Running load tests against production.** A load test can overwhelm a production system, spike database connections, corrupt cache state, or produce real charges (payment processors, SMS gateways). Load tests must target a dedicated environment with production-representative data volume but no real users or financial consequences.

## Full reference

### Identifying bottlenecks from results

Read results systematically:

- **Latency climbs linearly with load** — resource saturation (CPU,
  DB connections, thread pool)
- **Latency spikes periodically** — GC pauses, cron jobs, log
  rotation, cache expiration
- **Errors start at a specific VU count** — resource exhaustion (file
  descriptors, connection pool, memory)
- **Throughput plateaus while latency grows** — system at capacity;
  find the saturated resource
- **Errors only under concurrency** — race conditions, deadlocks,
  connection pool starvation

Always correlate load test metrics with infrastructure monitoring
(Grafana, Datadog, CloudWatch) to pinpoint the saturated component.

### Capacity planning

Use load test data to estimate scaling needs:

1. Find the **throughput ceiling**: the RPS where p99 latency exceeds
   your SLA
2. Calculate **headroom**: production should run at 50-70% of the
   ceiling for safety
3. **Scale estimation**: if 1 instance handles 500 RPS at ceiling and
   you need 2000 RPS, plan for 2000 / 500 * 1.5 = 6 instances
4. Verify with a load test against the scaled setup

### CI integration

Add performance gates to prevent regressions:

```yaml
# GitHub Actions example with k6
- name: Run load test
  run: |
    k6 run --out json=results.json load-test.js
- name: Check thresholds
  run: |
    # k6 exits non-zero if thresholds fail
    # Thresholds are defined in the test script
```

Guidelines for CI load tests:

- Run smoke or light load tests on every PR (1-2 minutes)
- Run full load tests nightly or before release
- Use **relative thresholds** (compare to baseline) over absolute
  values — environments drift
- Test against a dedicated environment, never production
- Store results historically to detect gradual degradation

### Common mistakes

- Running load tests from a single client box that itself becomes the
  bottleneck (CPU, sockets, ephemeral ports)
- Forgetting to disable rate limiting or auth caching during the test
- Hammering one URL and declaring victory — that does not exercise
  realistic code paths
- Comparing runs across days without accounting for noisy neighbors,
  CPU frequency scaling, or cache state
- Ignoring warm-up — the first 30 seconds of any test usually
  measures cold caches and JIT compilation, not steady state
