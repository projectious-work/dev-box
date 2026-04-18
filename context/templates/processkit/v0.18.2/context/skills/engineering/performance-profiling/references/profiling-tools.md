# Profiling Tools by Language

Quick reference for choosing the right profiler for each language and resource type.

## Python

| Tool | Type | Use Case |
|---|---|---|
| `cProfile` | CPU (instrumenting) | Built-in, low-overhead function-level profiling |
| `py-spy` | CPU (sampling) | Low-overhead, attaches to running process, flame graph output |
| `memray` | Memory | Tracks every allocation, native + Python frames, flame graphs |
| `scalene` | CPU + Memory + GPU | Line-level profiling, separates Python vs C time |
| `line_profiler` | CPU (line-level) | `@profile` decorator for targeted line-by-line timing |

```bash
# py-spy: profile a running process and generate flame graph
py-spy record -o profile.svg --pid 12345
# memray: track allocations in a script
memray run my_script.py && memray flamegraph memray-output.bin
```

## Node.js / JavaScript

| Tool | Type | Use Case |
|---|---|---|
| `--prof` / `--prof-process` | CPU | Built-in V8 profiler, generates tick files |
| `clinic.js` | CPU + Event Loop | Doctor (overview), Bubbleprof (async), Flame (CPU) |
| `0x` | CPU | Flame graph generation from V8 profiler data |
| Chrome DevTools | CPU + Memory | Heap snapshots, allocation timeline, CPU profiles |

```bash
# clinic.js: diagnose an HTTP server
npx clinic doctor -- node server.js
# 0x: generate flame graph
npx 0x -- node app.js
```

## Rust

| Tool | Type | Use Case |
|---|---|---|
| `cargo flamegraph` | CPU | Generates flame graphs via perf/dtrace |
| `criterion` | Benchmarking | Statistical benchmarks with regression detection |
| `dhat` | Memory (heap) | Heap profiling via Valgrind's DHAT |
| `perf` | CPU + Cache | Linux perf events, hardware counters |
| `tracing` + `tracy` | Timeline | Spans and events with GUI visualization |

```bash
# Flame graph for a binary
cargo flamegraph --bin my_app -- --my-args
# Criterion benchmark
cargo bench -- --save-baseline before
# After changes:
cargo bench -- --baseline before
```

## Go

| Tool | Type | Use Case |
|---|---|---|
| `pprof` (CPU) | CPU | Built-in, HTTP endpoint or `runtime/pprof` |
| `pprof` (heap) | Memory | Allocation profiling with in-use vs alloc views |
| `pprof` (goroutine) | Concurrency | Goroutine stack dumps, detect leaks |
| `trace` | Timeline | Goroutine scheduling, GC events, syscalls |
| `benchstat` | Benchmarking | Compare benchmark results with statistical tests |

```go
// Enable pprof HTTP endpoint
import _ "net/http/pprof"
go func() { http.ListenAndServe(":6060", nil) }()
```

```bash
# Collect and visualize CPU profile
go tool pprof -http=:8080 http://localhost:6060/debug/pprof/profile?seconds=30
```

## Java / JVM

| Tool | Type | Use Case |
|---|---|---|
| JFR (Flight Recorder) | CPU + Memory + I/O | Built-in, low overhead, production-safe |
| `async-profiler` | CPU + Alloc + Lock | Sampling profiler, flame graph output, no safepoint bias |
| VisualVM | CPU + Memory | GUI-based profiling and heap analysis |
| `jmap` + MAT | Memory | Heap dumps and leak analysis |

```bash
# JFR: record for 60 seconds
jcmd <pid> JFR.start duration=60s filename=profile.jfr
# async-profiler: CPU flame graph
./asprof -d 30 -f flamegraph.html <pid>
```

## Flame Graph Interpretation Guide

Reading a flame graph effectively:

1. **Start at the top**: the widest boxes at the top are where CPU time is actually spent
2. **Read bottom-up**: each row is a caller of the row above it
3. **Width = proportion**: a box taking 30% of the width consumed 30% of samples
4. **Color is arbitrary**: default coloring is random — use differential flame graphs for comparisons
5. **Search function**: most viewers support Ctrl+F to highlight a specific function across all stacks

### Common Bottleneck Patterns in Flame Graphs

- **Single tall tower**: one deep call chain dominates — look for unnecessary abstraction layers
- **Wide plateau at top**: a leaf function consuming most CPU — optimize or cache its result
- **Many thin towers with same top**: same function called from many paths — candidate for memoization
- **GC/malloc frames**: excessive allocation pressure — reduce allocations or pool objects
- **Lock/mutex frames**: contention — reduce critical section size or use lock-free structures
- **Syscall frames** (read/write/poll): I/O bound — batch operations or use async I/O

## General Tips

- Always profile a **realistic workload**, not a microbenchmark
- Profile in an environment **close to production** (same data size, same concurrency)
- **Sampling profilers** (py-spy, async-profiler, perf) are preferred over instrumenting profilers for production
- Collect profiles for **at least 30 seconds** to get statistically meaningful data
- Compare **before and after** with the same workload to validate improvements
