# Messaging Patterns Reference

Common messaging patterns with diagrams, trade-offs, and implementation notes.

## 1. Publish/Subscribe (Fan-Out)

Producer publishes to a topic; all subscribers receive a copy.

```
                    +-> [Subscriber A]
[Producer] -> [Topic]
                    +-> [Subscriber B]
                    +-> [Subscriber C]
```

- **Use when:** Multiple services need to react to the same event independently.
- **Guarantee:** At-least-once per subscriber. Each subscriber maintains its own offset/cursor.
- **Broker support:** Kafka topics, RabbitMQ fanout exchanges, NATS subjects, SNS topics.
- **Pitfall:** Subscriber count affects broker load. Slow subscribers can cause backpressure or lag.

## 2. Point-to-Point (Work Queue)

Producer sends to a queue; exactly one consumer processes each message.

```
[Producer] -> [Queue] -> [Consumer A]  (gets msg 1, 3, 5)
                      -> [Consumer B]  (gets msg 2, 4, 6)
```

- **Use when:** Task distribution where each task must be processed once (email sending, image resize).
- **Guarantee:** At-least-once with acknowledgment. Unacked messages return to the queue.
- **Broker support:** RabbitMQ queues, SQS, NATS queue groups, Kafka consumer groups on a single topic.
- **Pitfall:** Message ordering is not guaranteed across consumers. Use single consumer or partition keys if order matters.

## 3. Request-Reply (Async RPC)

Requester sends a message with a reply-to address; responder sends the result back.

```
[Requester] --request--> [Queue A] --> [Responder]
[Requester] <--reply---- [Queue B] <-- [Responder]
            (correlation ID matches request to reply)
```

- **Use when:** You need async request-response semantics (query another service without blocking the thread).
- **Guarantee:** Correlation ID links request to reply. Set a timeout for missing replies.
- **Broker support:** RabbitMQ reply-to header, NATS request-reply (built-in), Kafka with reply topics.
- **Pitfall:** Adds complexity over synchronous calls. Use only when the decoupling benefit is real.

## 4. Competing Consumers

Multiple consumers in a group share the load from a single queue or topic partition.

```
                        +-> [Consumer 1] (group G)
[Producer] -> [Topic] --+-> [Consumer 2] (group G)
                        +-> [Consumer 3] (group G)
              (each message goes to exactly one consumer in group G)
```

- **Use when:** Scaling processing horizontally. Each message is handled by one worker.
- **Guarantee:** At-least-once within the group. Rebalancing on consumer join/leave.
- **Broker support:** Kafka consumer groups, RabbitMQ competing consumers, NATS queue groups.
- **Pitfall:** Rebalancing can cause duplicate processing. Design idempotent consumers.

## 5. Dead Letter Queue (DLQ)

Messages that fail processing after retry limits are routed to a separate queue for inspection.

```
[Queue] -> [Consumer] --fail (retry 1)--> [Consumer]
                      --fail (retry 2)--> [Consumer]
                      --fail (retry 3)--> [DLQ]
                                            |
                                    [Manual inspection / replay]
```

- **Use when:** Always. Every production queue should have a DLQ.
- **Implementation:** Configure max retries (typically 3-5) with exponential backoff. Route to DLQ on exhaustion.
- **Monitoring:** Alert on DLQ depth > 0. Build tooling to inspect message content and replay to the original queue.
- **Broker support:** RabbitMQ dead-letter exchanges, SQS DLQ, Kafka requires custom implementation (retry topics).

## 6. Event Sourcing

Store every state change as an immutable event. Derive current state by replaying.

```
Command: "Place Order #42"
    |
    v
[Aggregate] -> validates -> emits [OrderPlaced { id:42, items:[...], total:99.00 }]
    |
    v
[Event Store]  (append-only log)
    |
    v
[Read Model Projector] -> updates [Orders DB] (materialized view)
```

Event stream for Order #42:
```
1: OrderPlaced   { items: [...], total: 99.00 }
2: PaymentTaken  { method: "card", ref: "abc" }
3: ItemRemoved   { item: "widget", new_total: 79.00 }
4: OrderShipped  { tracking: "XYZ123" }
```

- **State reconstruction:** Replay events 1-4 to get current order state.
- **Snapshots:** After N events, save a snapshot to avoid full replay.
- **Schema evolution:** Add fields freely. Never remove fields from published events. Use a schema registry.
- **Pitfall:** Event store is not a database — do not query it for read patterns. Use CQRS projections instead.

## 7. CQRS (Command Query Responsibility Segregation)

Separate write path (commands) from read path (queries) with independent models.

```
[Client]
   |
   +--command--> [Command Handler] -> [Write Model/Event Store]
   |                                        |
   |                                  (events projected)
   |                                        |
   +--query---> [Query Handler] -> [Read Model DB]
                                   (denormalized views optimized for reads)
```

- **Write model:** Enforces invariants, emits events. Can be event-sourced or traditional.
- **Read model:** Denormalized projections optimized for specific query patterns. Rebuilt from events.
- **Consistency:** Read models are eventually consistent. Typical lag: milliseconds to seconds.
- **Multiple read models:** One for list views, one for search, one for analytics — each optimized.
- **Pitfall:** Adds complexity. Only use when read and write patterns genuinely differ in structure or scale.

---

## Pattern Selection Guide

| Need | Pattern |
|------|---------|
| Notify multiple services of an event | Pub/Sub |
| Distribute work across workers | Point-to-Point + Competing Consumers |
| Async call expecting a response | Request-Reply |
| Full audit trail of state changes | Event Sourcing |
| Different read/write scaling or models | CQRS |
| Handle processing failures gracefully | Dead Letter Queue |
| Decouple services in a complex workflow | Pub/Sub + Saga (orchestration or choreography) |
