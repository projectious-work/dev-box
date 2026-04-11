---
name: nosql-patterns
description: |
  NoSQL patterns for document, key-value, wide-column, and graph stores. Access-pattern-driven design and consistency models. Use when choosing between NoSQL store types, designing a document or single-table model, picking a consistency level, or migrating between relational and NoSQL.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-nosql-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# NoSQL Patterns

## Intro

NoSQL is a family of stores, not a single technology — document,
key-value, wide-column, and graph databases each solve different
problems. The unifying design rule is the inverse of relational
modeling: start from the queries you must answer, then shape the
data to serve them.

## Overview

### Choose the right store type

Match the database type to the access pattern:

| Type | Best for | Examples |
|------|----------|---------|
| Document | Variable-schema entities, nested data, content management | MongoDB, Couchbase, Firestore |
| Key-Value | Caching, sessions, feature flags, counters | Redis, DynamoDB (simple), Memcached |
| Wide-Column | Time-series, IoT telemetry, high-write analytics | Cassandra, ScyllaDB, HBase |
| Graph | Relationship-heavy queries: social, recommendations, fraud | Neo4j, Neptune, ArangoDB |

**Default to PostgreSQL** unless the use case clearly benefits from a
specialized store. Signs that NoSQL is warranted:

- Schema varies significantly per record (document store).
- Sub-millisecond latency for simple lookups (key-value).
- Write volume exceeds what a single relational node can handle
  (wide-column).
- Queries traverse many relationships at variable depth (graph).

### Design for access patterns, not entities

Relational modeling normalizes around entities. NoSQL inverts this:

1. List every read and write the application performs.
2. For each read, define the exact key or query that retrieves it.
3. Shape documents, rows, or keys so each read is one lookup or one
   bounded scan.
4. Accept duplication as a feature — the cost of a denormalized read
   is paid by writes, which you can control.

### Consistency models

| Model | Guarantee | Use when |
|-------|-----------|----------|
| Strong | Read always returns latest write | Financial, inventory |
| Eventual | Reads may be briefly stale | Feeds, analytics, caches |
| Causal | Respects happens-before ordering | Chat, collaborative edit |
| Read-your-writes | Writer sees own updates immediately | Profile updates, forms |

CAP forces a choice during a network partition: availability (AP) or
consistency (CP). Most NoSQL stores default to AP with tunable
consistency. Use strong consistency where stale reads cause harm,
eventual elsewhere, and make writes idempotent so retries are safe.

### Example: caching layer in Redis

A cache-aside pattern with TTL-based expiration. Keys follow
`resource:id:variant` naming, individual resources use `SET` with
`EX`, paginated lists go in sorted sets, and writes invalidate via
`DEL`. A circuit breaker keeps the API alive when Redis is
unreachable.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Modeling a document store with a relational schema.** Designing documents as flat, normalized records and then joining them in application code recreates the worst of both worlds — no join support from the database and no embedding benefit from the document model. If entities are always queried together, embed them; if they are queried independently, separate them.
- **Unbounded arrays inside documents.** Storing all comments, tags, or events inside a parent document's array field means the document grows without limit over time, eventually exceeding document size limits and causing reads to return megabytes of data for a simple lookup. Apply a growth bound: cap the array, move to a sub-collection, or use a separate collection for high-cardinality child entities.
- **Hot partition keys in distributed stores.** Choosing a partition key like `date` or `country_code` concentrates write traffic on one shard when writes cluster by time or geography. Distribute load with a high-cardinality key — user ID, entity UUID — or composite key with a random suffix. A hot partition is a throughput ceiling that is expensive to undo after data is written.
- **Requesting strong consistency by default everywhere.** Strong consistency requires coordination across replicas, adding latency and reducing availability. Most reads tolerate eventual consistency — a profile page, a product listing — and should use it. Reserve strong consistency for operations where stale reads would produce incorrect behavior (e.g., inventory decrement, financial balances).
- **Using Pub/Sub or a message queue as a durable event store.** Message brokers designed for fan-out have limited retention and no replay-from-offset semantics in the same sense as an append-only log. Replaying events after a consumer bug, or querying historical event sequences, requires a separate event store. Use a dedicated event store when durability and replay are requirements.
- **Forgetting that NoSQL writes require idempotency.** Retries under network failures can result in duplicate writes to a document store that has no unique constraint by default. Design write operations to be idempotent by using a client-generated idempotency key, a conditional write (update-if-version), or an upsert pattern so re-running the operation produces the same state.
- **Choosing a NoSQL store without measuring the relational database's limitation.** The assumption "SQL won't scale" is often made without load testing the relational option with proper indexing and connection pooling. Most applications under 10M rows and moderate write rates perform well on a relational database. Choose NoSQL for a specific, measured reason — not as a default for new projects.

## Full reference

### Document stores (MongoDB)

**Embed vs reference:**

- **Embed** when child data is always read with the parent, child
  count is bounded, and the child is not shared across parents.
- **Reference** when the child is shared, the count is unbounded, or
  the child is updated independently.

```javascript
// Embedded: order with line items (always read together, bounded)
{
  _id: ObjectId("..."),
  customer_id: "cust_123",
  status: "shipped",
  items: [
    { product_id: "prod_456", name: "Widget", qty: 2, price: 9.99 },
    { product_id: "prod_789", name: "Gadget", qty: 1, price: 24.99 }
  ],
  created_at: ISODate("2025-06-15T10:30:00Z")
}

// Referenced: user with posts (unbounded, queried independently)
{ _id: "user_123", name: "Alice", email: "alice@example.com" }
{ _id: "post_001", author_id: "user_123", title: "...", body: "..." }
```

**Bucket pattern** for time-series data — group measurements into
hourly (or similar) documents to keep document count manageable:

```javascript
{
  sensor_id: "temp-01",
  bucket: ISODate("2025-06-15T10:00:00Z"),
  measurements: [
    { ts: ISODate("2025-06-15T10:00:12Z"), value: 23.5 },
    { ts: ISODate("2025-06-15T10:01:45Z"), value: 23.7 }
  ],
  count: 2
}
```

### Key-value stores (Redis)

Use colons as separators for hierarchical keys so prefixes group
naturally:

```
user:123:profile        -> JSON blob
user:123:sessions       -> Set of session IDs
session:abc-def-ghi     -> JSON blob with expiry
rate_limit:api:user:123 -> Counter with TTL
```

Common structures:

- `SET`/`GET` with TTL for caching and sessions.
- `INCR` with `EXPIRE` for rate limiting.
- Sorted sets (`ZADD`/`ZRANGEBYSCORE`) for leaderboards and
  time-ordered feeds.
- Streams (`XADD`/`XREAD`) for durable event processing.
- Pub/Sub for fire-and-forget notifications (no persistence — use
  Streams when durability matters).

### DynamoDB single-table design

Store multiple entity types in one table using composite keys
(`PK` partition + `SK` sort):

```
PK                  SK                      Attributes
USER#123            PROFILE                 {name, email, plan}
USER#123            ORDER#2025-06-15#001    {status, total, items}
USER#123            ORDER#2025-06-15#002    {status, total, items}
PRODUCT#456         METADATA                {name, price, category}
PRODUCT#456         REVIEW#USER#123         {rating, comment}
```

Query patterns served:

- Get user profile: `PK = USER#123, SK = PROFILE`.
- List user orders: `PK = USER#123, SK begins_with ORDER#`.
- Orders in date range: `PK = USER#123,
  SK between ORDER#2025-06-01 and ORDER#2025-06-30`.

Add Global Secondary Indexes (GSIs) for access patterns that need a
different key structure — for example, a GSI keyed on activity type
to support admin dashboards, with a TTL attribute to age out old
records.

### Wide-column stores (Cassandra-style)

Model one table per query. The partition key determines locality;
the clustering key determines on-disk order within a partition.
Avoid hot partitions by including a high-cardinality component in
the partition key (often a time bucket). Wide-column stores reward
write-optimized denormalization — duplicate data into multiple
tables instead of joining at read time.

### Graph stores

Use a graph database when traversal depth is variable (friends-of-
friends, fraud rings, dependency chains) and traversal patterns
dominate. For simple hierarchies, recursive CTEs in PostgreSQL are
usually enough; reach for Neo4j or Neptune when you need shortest
paths, weighted traversals, or pattern matching across many hops.

### Anti-patterns

- **Modeling NoSQL like a relational schema** — normalized documents
  with cross-collection joins defeat the point.
- **Unbounded arrays inside a document** — they grow until the
  document hits a size limit; bucket or split.
- **Hot partitions in DynamoDB/Cassandra** — concentrating writes on
  one partition key throttles the whole table.
- **Strong consistency by default everywhere** — costs latency and
  availability without measurable benefit for most reads.
- **Pub/Sub used for durable events** — messages are dropped if no
  subscriber is connected; use Streams.
- **Forgetting idempotency** — retries during failover will produce
  duplicates without dedupe keys.
