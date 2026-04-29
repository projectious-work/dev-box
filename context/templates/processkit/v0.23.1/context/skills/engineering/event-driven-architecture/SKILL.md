---
name: event-driven-architecture
description: |
  Event-driven systems — pub/sub, event sourcing, CQRS, sagas, broker selection, reliability. Use when designing or migrating to an event-driven system, choosing a message broker, implementing event sourcing or CQRS, designing sagas, or reviewing async architectures for reliability and ordering.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-event-driven-architecture
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Event-Driven Architecture

## Intro

Event-driven design decouples producers from consumers via a message
broker. Done well it scales independently, audits naturally, and
absorbs bursts; done badly it creates implicit, untraceable spaghetti.
Pick it only when synchronous request-response would not be simpler
and sufficient.

## Overview

### When event-driven fits

Good fit:

- Loose coupling needed between services
- Complex workflows spanning multiple services
- Real-time processing requirements
- Independent scaling of producers and consumers
- Audit-trail requirements (event sourcing)

Poor fit:

- Simple CRUD with strong consistency needs
- Low throughput where synchronous calls are perfectly fine
- Small monolith with no distribution requirement

The honest test: "Would synchronous request-response be simpler and
sufficient?" If yes, do not introduce events for their own sake.

### Messaging patterns

The deeper diagrams live in `references/messaging-patterns.md`.

| Pattern | Use case | Guarantee |
|---|---|---|
| Pub/Sub | Fan-out notifications, event broadcasting | At-least-once per subscriber |
| Point-to-Point | Task distribution, work queues | At-least-once with ack |
| Request-Reply | Async RPC, query with callback | Correlation-based |
| Competing Consumers | Load balancing across workers | At-least-once, single delivery in group |

### Choosing a broker

- **Apache Kafka** — log-based, very high throughput, replay
  capability. Best for event sourcing, stream processing, high-volume
  pipelines. Operationally heavy; prefer managed (Confluent, MSK).
- **RabbitMQ** — mature, flexible routing via exchanges and bindings.
  Best for task queues, complex topologies, RPC-style patterns.
  Lower throughput than Kafka but richer routing.
- **NATS** — lightweight, low latency, simple ops. Best for
  microservice communication, IoT, edge. JetStream adds persistence.
- **Amazon SQS/SNS** — fully managed, zero ops. SQS for queues, SNS
  for pub/sub. Limited features but reliable.
- **Redis Streams** — good for lightweight streaming when Redis is
  already in the stack. Not a Kafka replacement at scale.

Decision factors: throughput, ordering guarantees, replay, ops
budget, existing infrastructure.

### Event sourcing and CQRS

**Event sourcing** stores state as an append-only sequence of
immutable events (`OrderPlaced`, `ItemAdded`, `OrderShipped`).
Current state is derived by replaying events; snapshots avoid
replaying full history. Never delete events — emit compensating
events (`OrderCancelled`) instead.

**CQRS** separates the write model (commands, validation, business
rules) from read models (denormalized projections optimized per query
shape). Read models lag writes; design UIs to handle eventual
consistency. CQRS without event sourcing is valid and simpler — only
add event sourcing when you genuinely need full audit or temporal
queries.

### Sagas for distributed transactions

For workflows spanning multiple services, use sagas instead of 2PC:

- **Orchestration** — a central coordinator tells each service what
  to do and handles compensation on failure. Easier to understand and
  debug; the flow is in one place. Risk: orchestrator becomes a god
  service.
- **Choreography** — services react to each other's events with no
  central coordinator. Better decoupling. Risk: hard to track the
  overall flow without distributed tracing.

Choose orchestration when the workflow logic is complex, visibility
matters, and there are fewer than ~5-6 steps. Choose choreography
when services are truly independent, the flow is simple, and teams
own their services independently.

### Reliability essentials

- **Idempotent consumers** — every consumer must handle duplicates
  safely. Use idempotency keys (event ID + consumer ID) with a
  dedup store.
- **Dead letter queues** — route messages that fail processing after
  N retries to a DLQ. Monitor depth. Build tooling to inspect and
  replay.
- **Outbox pattern** — write events to an outbox table in the same
  DB transaction as the state change, then publish asynchronously.
  Prevents lost events when the broker is down.
- **Schema evolution** — use a schema registry. Make changes
  additive: add fields, never remove or rename. Version event types.
- **Ordering** — Kafka guarantees order within a partition, RabbitMQ
  does not guarantee cross-queue order. Design consumers to tolerate
  out-of-order delivery where possible.

### Example workflows

- **Order processing pipeline:** Kafka backbone, event sourcing on
  the order aggregate. Orchestration saga for fulfillment with
  compensating actions (refund on payment failure, release inventory
  on shipment failure). Avro schemas in a registry.
- **Sync to async migration:** map the call graph; identify
  fire-and-forget interactions vs queries. Pub/sub for notifications,
  request-reply for queries. Start with the outbox pattern to avoid
  dual-write issues. RabbitMQ if routing is rich.
- **Debugging duplicates / out-of-order:** check consumer
  idempotency keys. Verify Kafka partition key strategy (related
  events on the same partition?). Inspect consumer group rebalancing
  and commit/ack strategy. Drain the DLQ for poison messages.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Non-idempotent consumers.** Webhook delivery and message broker delivery are both at-least-once. A consumer that processes a payment or sends an email without checking whether it already processed this event ID will charge customers twice or send duplicate notifications when the message is redelivered. Every consumer must check a deduplicated event ID before processing and be safe to run twice on the same event.
- **No dead letter queue.** A consumer that fails repeatedly on a poison message (malformed payload, an invariant the code cannot handle) will retry indefinitely, blocking the queue for all subsequent messages. Route messages that exceed the retry limit to a DLQ and alert on its depth — otherwise one bad message stops the entire consumer group.
- **Dual write — writing to the database and the broker without an outbox.** If the broker publish succeeds but the database commit fails (or vice versa), producers and consumers have diverged state. Use the transactional outbox pattern: write the event to a database table in the same transaction as the state change, then publish from the outbox asynchronously.
- **Publishing events before the state is persisted.** If a service publishes `OrderShipped` and then fails to commit the database update, the event has leaked into the world without a corresponding state change. Consumers will react to a shipment that the system does not record. Commit state first; publish only after a successful commit.
- **Removing or renaming fields in published event schemas.** A consumer running an old version that reads a field that was renamed will silently process empty data or crash. Published event schemas are a contract — make changes additive only: add new optional fields, never remove or rename existing ones. Use a schema registry to enforce this.
- **Choreographed sagas with no distributed tracing.** A saga implemented as 6 services reacting to each other's events produces a workflow that is invisible unless you can correlate trace IDs across events. Debugging "why did the payment never succeed" means manually correlating log entries across 6 services and 20 minutes of event history. Wire distributed tracing before deploying choreography-based sagas.
- **Choosing event-driven architecture when synchronous request-response is simpler.** Events introduce eventual consistency, ordering ambiguity, and significant operational complexity (brokers, schemas, DLQs, monitoring). The threshold question is: "Would synchronous request-response be simpler and sufficient?" If yes, use it. Events are not a default for microservices — they are a tool for specific problems.

## Full reference

### Pub/Sub topology

```
                    +-> [Subscriber A]
[Producer] -> [Topic]
                    +-> [Subscriber B]
                    +-> [Subscriber C]
```

Each subscriber maintains its own offset. At-least-once per
subscriber. Slow subscribers can cause backpressure or lag — monitor
consumer lag per subscription. Broker support: Kafka topics, RabbitMQ
fanout exchanges, NATS subjects, SNS topics.

### Point-to-Point work queue

```
[Producer] -> [Queue] -> [Consumer A]  (msg 1, 3, 5)
                      -> [Consumer B]  (msg 2, 4, 6)
```

Exactly one consumer processes each message. At-least-once with
acknowledgment — unacked messages return to the queue. Ordering is
not guaranteed across consumers; use a single consumer or partition
keys when order matters.

### Request-Reply

```
[Requester] --request--> [Queue A] --> [Responder]
[Requester] <--reply---- [Queue B] <-- [Responder]
            (correlation ID matches request to reply)
```

Correlation ID links request to reply. Always set a timeout for
missing replies. Use only when the decoupling benefit is real;
otherwise prefer a synchronous call.

### Dead letter queue

```
[Queue] -> [Consumer] --fail (retry 1)--> [Consumer]
                      --fail (retry 2)--> [Consumer]
                      --fail (retry 3)--> [DLQ]
                                            |
                                    [Inspect / replay]
```

Configure max retries (typically 3-5) with exponential backoff. Alert
on DLQ depth > 0. Build inspection and replay tooling. Broker
support: RabbitMQ dead-letter exchanges, SQS DLQ, Kafka requires
custom retry topics.

### Event sourcing example stream

Order #42:

```
1: OrderPlaced   { items: [...], total: 99.00 }
2: PaymentTaken  { method: "card", ref: "abc" }
3: ItemRemoved   { item: "widget", new_total: 79.00 }
4: OrderShipped  { tracking: "XYZ123" }
```

Replay events 1-4 to reconstruct current state. Snapshot every N
events (commonly 100-500) to avoid full replay. Add fields freely;
never remove fields from published events. The event store is not a
query database — use CQRS projections for read patterns.

### CQRS topology

```
[Client]
   |
   +--command--> [Command Handler] -> [Write Model / Event Store]
   |                                       |
   |                                 (events projected)
   |                                       |
   +--query---> [Query Handler] -> [Read Model DB]
```

Multiple read models are common — one for list views, one for
search, one for analytics — each rebuilt from events and optimized
for its query shape. Typical projection lag is milliseconds to
seconds.

### Anti-patterns

- **Events for everything** — synchronous calls are fine when
  request-response is the natural shape.
- **Non-idempotent consumers** — duplicate delivery is inevitable.
- **No DLQ** — poison messages block the queue forever.
- **Dual writes** — writing to DB and broker without an outbox
  loses events on failure.
- **Removing or renaming event fields** — breaks every consumer
  pinned to old schema.
- **Querying the event store** for read patterns — that is what CQRS
  projections exist for.
- **Choreographed sagas with no tracing** — debugging becomes
  archaeology.
- **Orchestrator god service** — when one orchestrator owns every
  workflow, it becomes the new monolith.
