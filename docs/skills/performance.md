# Performance Skills

Skills for performance analysis, optimization, and load testing.

---

### performance-profiling

> Performance analysis methodology and profiling techniques for CPU, memory, and I/O. Flame graphs, benchmarking, and regression detection. Use when optimizing performance, profiling bottlenecks, or reviewing performance-critical code.

**Triggers:** When identifying bottlenecks, profiling CPU/memory/I/O, interpreting flame graphs, setting up benchmarks, or optimizing slow code paths.
**Tools:** `Bash` `Read` `Write`
**References:** `profiling-tools.md`

Key capabilities:

- Follow the full performance analysis cycle: Identify, Measure, Profile, Optimize, Verify
- CPU profiling with sampling profilers and flame graph generation
- Memory profiling to detect leaks, allocation pressure, and unbounded growth
- I/O profiling for disk, network, and database bottlenecks (N+1 queries, connection pooling, slow queries)
- Benchmarking with statistical significance and regression detection
- Flame graph interpretation: reading X-axis (alphabetical, not time), Y-axis (stack depth), and differential flame graphs
- Common optimization patterns: algorithmic improvements, batching, caching, pooling, lazy evaluation, data layout

??? example "Example usage"
    **Slow API endpoint:** Measures end-to-end latency, profiles the handler, discovers 80% of time spent in 47 sequential database queries (N+1 problem). Rewrites as a single JOIN query, reducing response time from 3 seconds to 120ms.

---

### caching-strategies

> Caching patterns including cache-aside, write-through, TTL strategies, cache invalidation, and HTTP caching. Use when designing caching layers, optimizing response times, or debugging cache-related issues.

**Triggers:** When adding caching to reduce latency, choosing caching patterns, configuring HTTP caching headers, debugging stale data or cache stampede issues, or designing invalidation strategies.
**Tools:** None
**References:** None

Key capabilities:

- Choose the right caching pattern: cache-aside, read-through, write-through, write-behind
- Design TTL strategies with jitter to prevent thundering herd on expiration
- Cache invalidation via event-driven, tag-based, or versioned key approaches
- HTTP caching configuration: Cache-Control, ETag, CDN caching with s-maxage and Surrogate-Key
- Prevent cache stampede with locking (mutex), probabilistic early expiration (XFetch), and stale-while-revalidate
- Cache warming strategies for deploys and predictable access patterns

??? example "Example usage"
    **Product page too slow:** Profiles the endpoint and finds 3 database queries per request. Implements cache-aside with Redis: product data (TTL 10min), category tree (TTL 1hr), user-specific pricing (TTL 60s, private). Adds stale-while-revalidate to HTTP headers. Response time drops to 45ms on cache hit.

---

### concurrency-patterns

> Concurrency and parallelism patterns including async/await, threads, actors, channels, and deadlock prevention. Use when designing concurrent systems, debugging race conditions, or choosing between concurrency models.

**Triggers:** When choosing between threads, async/await, or actors; designing concurrent pipelines; debugging deadlocks or race conditions; implementing producer-consumer or fan-out/fan-in patterns.
**Tools:** None
**References:** `patterns-catalog.md`

Key capabilities:

- Distinguish concurrency from parallelism and choose based on I/O-bound vs CPU-bound bottlenecks
- Choose the right model: async/await, OS threads, green threads, actors, channels, or thread pools
- Manage shared state safely with Mutex, RwLock, and atomic operations
- Prevent deadlocks via lock ordering, try-lock with timeout, reduced lock scope, and lock-free algorithms
- Detect and fix race conditions using ThreadSanitizer, cargo miri, and pattern recognition (TOCTOU, partial init)
- Implement backpressure with bounded channels, rate limiting, load shedding, and reactive streams

??? example "Example usage"
    **Go service deadlocks under load:** Enables mutex profiling with `GODEBUG=mutexprofile`, identifies two goroutines acquiring locks on `userCache` and `sessionCache` in opposite orders. Fixes by establishing consistent lock ordering and reducing the critical section.

---

### load-testing

> Load testing methodology including test types, scenario design, and capacity planning. Use when planning load tests, analyzing test results, or setting up performance testing in CI.

**Triggers:** When planning or running load tests, choosing tools, designing test scenarios, analyzing results, estimating capacity, or setting up performance testing in CI pipelines.
**Tools:** `Bash` `Read` `Write`
**References:** None

Key capabilities:

- Choose the right test type: smoke, load, stress, spike, and soak/endurance tests
- Design realistic scenarios with user journeys, think time, traffic distribution, and authentication
- Capture essential metrics: latency percentiles (p50/p95/p99), throughput (RPS), error rate, and resource utilization
- Tool selection guidance: k6, Locust, Gatling, wrk, hey, vegeta
- Identify bottlenecks from results: linear latency climb, periodic spikes, error thresholds, throughput plateaus
- Capacity planning: find throughput ceiling, calculate headroom, estimate scaling needs
- CI integration with performance gates and relative thresholds

??? example "Example usage"
    **Pre-Black Friday load test:** Designs a test plan with smoke test first, then load test at 2x normal traffic, then stress test at 5x to find the breaking point. Uses k6 with scenarios modeling the top 5 user journeys weighted by actual traffic distribution. Configures thresholds at p95 < 500ms and error rate < 0.1%.
