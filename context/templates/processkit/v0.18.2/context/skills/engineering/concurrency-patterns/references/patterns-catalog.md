# Concurrency Patterns Catalog

Detailed reference for common concurrency and parallelism patterns.

## Producer-Consumer

**Description**: One or more producers generate work items, one or more consumers process them, connected by a shared queue.

**When to use**: Decoupling data generation from processing; smoothing out bursts; different production and consumption rates.

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

threading.Thread(target=producer).start()
threading.Thread(target=consumer).start()
```

## Reader-Writer

**Description**: Multiple readers can access a resource concurrently, but writers require exclusive access.

**When to use**: Read-heavy workloads with infrequent updates (configuration, caches, lookup tables).

```rust
use std::sync::RwLock;

let config = RwLock::new(load_config());

// Many readers concurrently:
let guard = config.read().unwrap();
let value = guard.get("key");

// Exclusive writer:
let mut guard = config.write().unwrap();
guard.insert("key", new_value);
```

Watch for **writer starvation** — if readers never release, writers wait forever. Most implementations have a fair scheduling policy to prevent this.

## Fork-Join

**Description**: Split a task into independent subtasks, process in parallel, combine results.

**When to use**: Embarrassingly parallel problems (map-reduce, parallel sort, image processing).

```go
func processChunks(data []Item) []Result {
    chunks := split(data, runtime.NumCPU())
    results := make(chan []Result, len(chunks))

    for _, chunk := range chunks {
        go func(c []Item) {
            results <- processChunk(c)
        }(chunk)
    }

    var combined []Result
    for range chunks {
        combined = append(combined, <-results...)
    }
    return combined
}
```

## Pipeline

**Description**: Chain of processing stages connected by channels. Each stage runs concurrently, processing items as they flow through.

**When to use**: Multi-step data transformation where each step can be parallelized independently.

```go
func pipeline() {
    raw := produce()           // stage 1: read data
    parsed := parse(raw)       // stage 2: parse concurrently
    validated := validate(parsed) // stage 3: validate concurrently
    consume(validated)         // stage 4: write results
}

// Each stage reads from input channel, writes to output channel
func parse(in <-chan RawData) <-chan ParsedData {
    out := make(chan ParsedData)
    go func() {
        defer close(out)
        for raw := range in {
            out <- doParse(raw)
        }
    }()
    return out
}
```

## Barrier

**Description**: A synchronization point where all threads must arrive before any can proceed.

**When to use**: Phased computation where all workers must complete phase N before starting phase N+1 (simulations, iterative algorithms).

```python
import threading

barrier = threading.Barrier(num_workers)

def worker(worker_id):
    for phase in range(num_phases):
        compute_phase(worker_id, phase)
        barrier.wait()  # all workers synchronize here
```

## Semaphore

**Description**: Limits the number of concurrent accesses to a resource or section of code.

**When to use**: Controlling concurrency level (connection pools, rate limiting, bounded parallelism).

```python
import asyncio

sem = asyncio.Semaphore(10)  # max 10 concurrent requests

async def fetch(url):
    async with sem:
        return await http_client.get(url)

# Launch 1000 tasks, but only 10 run concurrently
results = await asyncio.gather(*[fetch(url) for url in urls])
```

## Future / Promise

**Description**: A placeholder for a value that will be available later. Represents an asynchronous computation.

**When to use**: Composing asynchronous operations, especially when you need to combine results from multiple concurrent tasks.

```rust
// Rust: futures with tokio
let user_future = fetch_user(user_id);
let orders_future = fetch_orders(user_id);

// Run both concurrently, await both
let (user, orders) = tokio::join!(user_future, orders_future);
```

```javascript
// JavaScript: Promise.all for concurrent execution
const [user, orders, preferences] = await Promise.all([
  fetchUser(userId),
  fetchOrders(userId),
  fetchPreferences(userId),
]);
```

## Actor Model

**Description**: Isolated actors communicate exclusively through asynchronous messages. Each actor has private state and processes one message at a time.

**When to use**: Distributed systems, fault-tolerant services, complex state machines with many independent entities (game servers, IoT device managers).

Conceptual model: each actor owns a mailbox (channel receiver) and processes messages sequentially in a loop, sending replies or messages to other actors.

Properties:
- **No shared state**: all communication is via messages
- **Location transparency**: actors can be local or remote
- **Fault isolation**: one actor crashing does not affect others
- **Supervision trees** (Erlang/Akka): parent actors restart failed children

## Pattern Selection Guide

| Situation | Pattern |
|---|---|
| Decouple fast producer from slow consumer | Producer-Consumer |
| Many readers, few writers | Reader-Writer |
| Split work, combine results | Fork-Join |
| Multi-stage data transformation | Pipeline |
| Synchronize phases of computation | Barrier |
| Limit concurrent access to a resource | Semaphore |
| Compose multiple async operations | Future/Promise |
| Independent entities with private state | Actor Model |
| Bounded parallel processing of a queue | Thread Pool + Work Queue |
