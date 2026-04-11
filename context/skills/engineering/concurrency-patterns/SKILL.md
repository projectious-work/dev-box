---
name: concurrency-patterns
description: |
  Concurrency and parallelism patterns — async/await, threads, actors, channels, deadlock prevention. Use when choosing a concurrency model, designing concurrent or parallel pipelines, debugging deadlocks or race conditions, or reasoning about shared state and backpressure.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-concurrency-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Concurrency Patterns

## Intro

Concurrency is about structuring a program to handle multiple tasks;
parallelism is about running them on multiple cores simultaneously.
Pick the model that matches the bottleneck (I/O vs CPU), prefer
message passing over shared state, and design for backpressure from
day one.

## Overview

### Concurrency vs parallelism

Concurrency is a structuring technique — interleaving tasks so the
program makes progress on many things at once, even on a single core.
Parallelism is true simultaneous execution across multiple cores.
Choose based on the bottleneck:

- **I/O-bound** (network, disk, database): use concurrency
  (async/await or green threads). One thread can multiplex thousands
  of connections.
- **CPU-bound** (computation, data processing): use parallelism (OS
  threads or processes) so all cores are busy.
- **Mixed**: combine both — async for I/O at the edges, a thread or
  process pool for the CPU-heavy core.

### Choosing a concurrency model

| Model | Best for | Watch out for |
|---|---|---|
| **async/await** | High-concurrency I/O (web servers, API clients) | Don't block the event loop with CPU work |
| **OS threads** | CPU-bound parallelism, simple shared state | Per-thread stack overhead, limited scalability |
| **Green threads / goroutines** | Massive concurrency with simple code (Go, Erlang) | Scheduler quirks, blocking FFI calls |
| **Actors** | Distributed systems, fault isolation | Ordering guarantees, debugging |
| **Channels / message passing** | Decoupled producer-consumer pipelines | Backpressure, channel capacity |
| **Thread pool + work queue** | Bounded parallelism for batch jobs | Queue sizing, graceful shutdown |

### Managing shared state

Prefer message passing. When shared state is unavoidable, pick the
right primitive:

- **Mutex** — one writer or one reader at a time. Use for general
  shared mutable state.
- **RwLock** — many concurrent readers or one exclusive writer. Use
  when reads vastly outnumber writes (configuration, lookup tables).
- **Atomics** — lock-free updates for simple values like counters and
  flags. Cheapest option when it fits.
- **Concurrent data structures** — ConcurrentHashMap, DashMap, sync
  collections — let the library handle synchronization for you.

Hold locks for the shortest possible duration. Never perform I/O or
allocations while holding a lock. Prefer scoped lock guards
(`Arc<Mutex<T>>`, `synchronized` blocks) over manual lock/unlock.

### Backpressure

When producers outpace consumers, unbounded queues cause memory
exhaustion. Design backpressure in from the start:

- **Bounded channels** — block or reject the producer when full.
- **Rate limiting** — control the inbound rate at the edge.
- **Load shedding** — drop low-priority work under pressure.
- **Reactive streams** — propagate demand signals upstream so the
  producer slows down naturally.

```go
// Go: bounded channel provides natural backpressure
jobs := make(chan Job, 100)  // blocks producer when 100 items queued
```

### Example workflows

- **I/O-bound web scraper:** async/await with a semaphore capping
  concurrency (e.g. 50 simultaneous requests). `tokio::spawn` plus
  `tokio::sync::Semaphore` in Rust, or `asyncio.gather` with
  `asyncio.Semaphore` in Python.
- **Bulk record processing:** fan-out / fan-in pipeline. A reader
  streams records into a bounded channel, N workers process in
  parallel (typically `NumCPU()`), a collector aggregates results.
  Channel buffer roughly `2 * N` for smooth flow.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Blocking the event loop with CPU-bound work in async code.** An async function that runs a tight computation loop, calls a synchronous file I/O function, or invokes a C extension that holds the GIL will block the entire event loop — other coroutines cannot run until it returns. Offload CPU-bound or blocking work to a thread pool (`asyncio.to_thread`, `tokio::task::spawn_blocking`) so the event loop stays responsive.
- **Unbounded channels or queues.** A channel with no capacity limit grows without bound when the producer outpaces the consumer, eventually exhausting memory. Always size queues explicitly and design what happens when the queue is full — block the producer, drop low-priority items, or apply backpressure upstream.
- **Holding a lock across an await point or blocking I/O call.** Acquiring a mutex and then awaiting a network call while holding it blocks all other waiters for the entire I/O duration and can deadlock if a waiter also tries to acquire the lock. Release the lock before any await; restructure to hold it only for the actual state mutation.
- **Introducing shared mutable state as the first solution.** Shared state protected by a mutex is correct only if the lock is always acquired, never held too long, and acquired in consistent order. Start from message passing — channels, actors, or immutable data passed between workers — and add shared state only when message passing is genuinely awkward.
- **Manual lock/unlock instead of scoped guards.** A lock acquired with a manual `lock()` call that is only released in a `finally` block is released incorrectly on early returns, panics, or exceptions that bypass the finally. Use scoped guards (RAII in Rust, `with` in Python, `synchronized` in Java) so the lock is always released when the scope exits.
- **Using parallelism for I/O-bound tasks and concurrency for CPU-bound tasks.** Spawning 100 OS threads for 100 simultaneous database queries wastes memory and scheduler overhead — a single async event loop handles this with far less overhead. Conversely, running CPU-heavy image processing inside an async event loop starves other coroutines. Match the tool to the bottleneck: async/coroutines for I/O, threads/processes for CPU.
- **No graceful shutdown for worker pools.** A thread or goroutine pool that is not given a shutdown signal when the application exits leaves background workers running indefinitely or drops in-flight work. Use a stop channel, context cancellation, or a sentinel value so all workers drain and exit cleanly on shutdown.

## Full reference

### Pattern catalog

#### Producer-Consumer

One or more producers generate work items, one or more consumers
process them, connected by a shared queue. Use to decouple data
generation from processing, smooth bursts, or bridge different
production and consumption rates.

```python
import queue, threading

q = queue.Queue(maxsize=100)  # bounded for backpressure

def producer():
    for item in generate_items():
        q.put(item)  # blocks when full
    q.put(None)  # sentinel

def consumer():
    while (item := q.get()) is not None:
        process(item)
```

#### Reader-Writer

Many readers concurrently or one exclusive writer. Use for read-heavy
workloads with infrequent updates (configuration, caches, lookup
tables). Watch for **writer starvation** — most implementations have
a fair scheduling policy to prevent it.

#### Fork-Join

Split a task into independent subtasks, run in parallel, combine
results. Use for embarrassingly parallel problems (map-reduce,
parallel sort, image processing).

```go
func processChunks(data []Item) []Result {
    chunks := split(data, runtime.NumCPU())
    results := make(chan []Result, len(chunks))
    for _, chunk := range chunks {
        go func(c []Item) { results <- processChunk(c) }(chunk)
    }
    var combined []Result
    for range chunks {
        combined = append(combined, <-results...)
    }
    return combined
}
```

#### Pipeline

Chain of stages connected by channels. Each stage runs concurrently,
processing items as they flow through. Use for multi-step data
transformations where each step can be parallelized independently.

#### Barrier

A synchronization point where all threads must arrive before any can
proceed. Use for phased computation: simulations, iterative
algorithms, where every worker must finish phase N before any starts
phase N+1.

#### Semaphore

Limits concurrent access to a resource or section of code. Use for
connection pools, rate limiting, and bounded parallelism.

```python
import asyncio
sem = asyncio.Semaphore(10)  # max 10 concurrent requests

async def fetch(url):
    async with sem:
        return await http_client.get(url)
```

#### Future / Promise

A placeholder for a value that will be available later. Compose
asynchronous operations and combine results from multiple concurrent
tasks. `tokio::join!` in Rust, `Promise.all` in JavaScript.

#### Actor model

Isolated actors communicate exclusively through asynchronous
messages. Each actor owns private state and processes one message at
a time from its mailbox. Properties:

- **No shared state** — all communication is via messages
- **Location transparency** — actors can be local or remote
- **Fault isolation** — one actor crashing does not affect others
- **Supervision trees** (Erlang/Akka) — parent actors restart failed
  children

Best fit: distributed systems, fault-tolerant services, complex state
machines with many independent entities (game servers, IoT device
managers).

The full pattern catalog with extended examples lives in
`references/patterns-catalog.md`.

### Deadlock prevention

Coffman's four conditions: mutual exclusion, hold-and-wait, no
preemption, circular wait. Break any one to prevent deadlock:

- **Lock ordering** — always acquire locks in a consistent global
  order.
- **Try-lock with timeout** — `try_lock()` with a deadline, then back
  off and retry.
- **Reduce lock scope** — restructure to avoid holding multiple locks
  simultaneously.
- **Lock-free algorithms** — atomics and CAS (compare-and-swap) for
  simple operations.

Detection in development: Rust `parking_lot` deadlock detection,
Go `GODEBUG=mutexprofile`, Java `jstack` thread dumps.

### Race condition detection

Race conditions occur when correctness depends on the timing of
uncontrolled events. Tools:

- **Rust** — compile-time prevention via the ownership system; use
  `cargo miri` for unsafe code
- **Go** — `go run -race` / `go test -race` (ThreadSanitizer)
- **C/C++** — ThreadSanitizer (`-fsanitize=thread`), Helgrind
- **Java** — FindBugs/SpotBugs, ThreadSanitizer integrations

Patterns that indicate races:

- Check-then-act without holding a lock (TOCTOU)
- Publishing a partially initialized object
- Updating a shared collection without synchronization
- Non-atomic read-modify-write sequences

### Anti-patterns

- **Blocking the event loop** with CPU work in async code — offload
  to a thread pool (`asyncio.to_thread`, `tokio::task::spawn_blocking`).
- **Unbounded channels** — always size for backpressure.
- **Holding a lock across an await or I/O call** — releases other
  workers from making progress and invites deadlock.
- **Sharing mutable state by default** — start from message passing
  and only add shared state when measured.
- **Manual lock/unlock** instead of scoped guards — leaks on early
  return or panic.

### Pattern selection guide

| Situation | Pattern |
|---|---|
| Decouple fast producer from slow consumer | Producer-Consumer |
| Many readers, few writers | Reader-Writer |
| Split work, combine results | Fork-Join |
| Multi-stage data transformation | Pipeline |
| Synchronize phases of computation | Barrier |
| Limit concurrent access to a resource | Semaphore |
| Compose multiple async operations | Future/Promise |
| Independent entities with private state | Actor model |
| Bounded parallel processing of a queue | Thread pool + work queue |
