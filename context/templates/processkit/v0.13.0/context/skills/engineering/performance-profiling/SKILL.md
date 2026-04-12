---
name: performance-profiling
description: |
  Performance analysis and profiling for CPU, memory, and I/O — flame graphs, benchmarks, regression detection. Use when identifying performance bottlenecks, profiling CPU/memory/I/O, interpreting flame graphs, setting up benchmarking or regression detection, or optimizing a slow function or endpoint.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-performance-profiling
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Performance Profiling

## Intro

Intuition about performance is unreliable — always profile before
optimizing. Work the cycle: identify the symptom, measure the
baseline, profile the right resource, optimize the top bottleneck,
verify the improvement. One change at a time.

## Overview

### Performance analysis cycle

Always work the full cycle: **Identify -> Measure -> Profile ->
Optimize -> Verify**.

1. **Identify** the concern: slow response, high memory, excessive
   CPU, I/O stalls
2. **Measure** the baseline with concrete numbers (latency p50/p95/p99,
   throughput, RSS)
3. **Profile** with the right tool for the resource type — see
   `references/profiling-tools.md`
4. **Optimize** the top bottleneck only; do not scatter-shot
5. **Verify** with the same measurement from step 2

Never skip measurement. Without a number you cannot tell whether the
change helped or hurt.

### CPU profiling

Identify what the application spends CPU time on:

1. Use a **sampling** profiler (not instrumenting) to minimize
   overhead
2. Collect a representative workload, not a trivial test case
3. Generate a flame graph for visual analysis
4. Look for **wide stacks** (hot functions), **deep stacks** (excess
   abstraction), and unexpected functions (regex compilation,
   serialization, logging)

Track **CPU time** per operation, not wall-clock (which includes I/O
waits).

### Memory profiling

Identify allocation pressure and leaks:

1. Track **peak RSS** and **allocation rate**, not just current usage
2. For leaks, take heap snapshots at intervals and diff them
3. Look for unbounded collections, caches without eviction, event
   listener accumulation, and reference cycles in GC languages
4. Distinguish a leak (unbounded growth) from high-but-stable usage,
   which may be acceptable

### I/O profiling

Identify disk, network, or database bottlenecks:

1. Compare wall-clock time with CPU time — a large gap is I/O wait
2. For databases: enable slow query logging, look for N+1, missing
   indexes
3. For network: check connection pooling, DNS, TLS handshake overhead
4. For disk: check synchronous writes, excessive fsync, unnecessary
   file ops

### Quick checklist before profiling

1. What is the **user-visible symptom**? (slow page, high bill, OOM)
2. What **resource** is constrained? (CPU, memory, disk, network, DB)
3. **Constant or intermittent**? (steady-state vs spike-triggered)
4. **What changed recently?** Regressions usually correlate with
   recent deploys
5. What is the **acceptable target**? (p99 < 200 ms, RSS < 512 MB —
   you need a goal to know when you are done)

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Optimizing before establishing a baseline measurement.** Without a concrete before number (p50/p95/p99, RSS, throughput), there is no way to tell whether the change helped, hurt, or made no difference. Establish the baseline first — then optimize — then verify with the same measurement.
- **Profiling under trivial or synthetic load.** A profiler run against a tight loop with no I/O tells you nothing about production behavior. Collect a workload representative of real traffic — replay production logs, replicate actual access patterns, or drive realistic concurrency before reading the profiler output.
- **Chasing secondary bottlenecks before exhausting the primary one.** After a profiling pass, fix the top bottleneck first and re-profile. Fixing the second-biggest bottleneck while the primary one still saturates the resource often produces no measurable improvement.
- **Reporting wall-clock time when the constrained resource is CPU.** A function that takes 200ms wall-clock while waiting for a database query uses almost no CPU. Profiling the wrong resource leads to wrong conclusions. Distinguish CPU time, wall-clock, I/O wait, and memory allocation — they require different tools and different fixes.
- **Declaring success after a microbenchmark without re-running under realistic load.** Verifying a change with a tight loop under no concurrency and then shipping it is not a verification. Re-run the same realistic workload used for the baseline before declaring the goal met.
- **Fixing a performance regression without adding a regression test.** A regression fixed once will recur unless a benchmark or load-test encodes the boundary. After every performance fix, add a test that would catch reversion to the slow path.
- **Running high-rate sampling profilers against production.** Profilers at 100% sampling rate or instrumenting every function call introduce measurable overhead that can itself degrade the system under investigation. Use a low-overhead sampling approach in production; reserve high-rate profiling for staging.

## Full reference

### Profiling tools by language

`references/profiling-tools.md` has the full table. Quick picks:

- **Python**: `py-spy` (sampling CPU, attach to running process,
  flame graphs); `memray` (allocations, native + Python frames);
  `scalene` (line-level CPU + memory + GPU); `cProfile` for built-in
  function-level
- **Node.js**: `clinic.js` (Doctor / Bubbleprof / Flame); `0x` (flame
  graphs); Chrome DevTools for heap snapshots
- **Rust**: `cargo flamegraph` (CPU); `criterion` (statistical
  benchmarks); `dhat` (heap); `perf` for hardware counters
- **Go**: `pprof` for CPU/heap/goroutine; `trace` for scheduler and
  GC events; `benchstat` for statistical comparison
- **JVM**: JFR (Flight Recorder) — built-in and production-safe;
  `async-profiler` for sampling CPU/alloc/lock with no safepoint
  bias; `jmap` + MAT for heap dumps

### Benchmarking and regression detection

1. Isolate the code under test — benchmark the function, not the
   setup
2. Run enough iterations for statistical significance (mean, stddev,
   percentiles)
3. Control external factors: disable turbo boost, pin CPU frequency,
   close other processes
4. For regression detection, store baseline results in version
   control and compare on each run

```bash
# Rust criterion
cargo bench -- --save-baseline main
# After changes:
cargo bench -- --baseline main
```

### Reading flame graphs

- **X-axis** is alphabetical (not time); width = proportion of
  samples
- **Y-axis** is stack depth, read bottom-up (caller -> callee)
- **Wide boxes at the top** are hot leaf functions — optimize first
- **Wide boxes at the bottom** are hot callers — consider algorithmic
  changes
- Look for unexpected frames: GC pauses, lock contention, allocator
  overhead
- Use **differential** flame graphs (red/blue) to compare before/after

Common patterns: a single tall tower means one deep call chain
dominates (look for unnecessary abstraction); a wide plateau at the
top means a leaf function consuming most CPU; many thin towers with
the same top function is a memoization candidate; GC/malloc frames
indicate allocation pressure; lock/mutex frames mean contention;
syscall frames (`read`, `write`, `poll`) mean the workload is I/O
bound.

### Common optimization patterns

After profiling reveals the bottleneck, apply the right fix:

- **Algorithmic** — O(n²) -> O(n log n). Always check this first.
- **Batching** — combine many small operations into fewer large ones
- **Caching** — reuse computed results, but measure the hit rate
- **Pooling** — reuse expensive resources (connections, threads,
  buffers)
- **Lazy evaluation** — defer work until actually needed
- **Data layout** — struct-of-arrays vs array-of-structs for cache
  friendliness

### General tips

- Always profile a **realistic workload**, not a microbenchmark
- Profile in an environment **close to production** — same data
  size, same concurrency
- Sampling profilers (`py-spy`, `async-profiler`, `perf`) are
  preferred over instrumenting profilers in production
- Collect profiles for **at least 30 seconds** for statistical
  significance
- Compare **before and after** with the same workload to validate
  improvements
