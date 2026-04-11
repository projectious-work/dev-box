---
name: graphql-patterns
description: |
  GraphQL schema design, resolver patterns, DataLoader, pagination, federation, and schema evolution. Use when designing a GraphQL schema, implementing resolvers, diagnosing N+1 problems, adding pagination, setting up federation, or evolving a schema without breaking clients.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-graphql-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# GraphQL Patterns

## Intro

GraphQL schemas are designed from the client's perspective, not the
database. Resolvers stay thin, DataLoader prevents N+1 fan-out, lists
use Relay-style cursor pagination, and schemas evolve only by adding
fields and deprecating the old ones.

## Overview

### Schema design

Model the domain as the client wants to consume it. Types are nouns
(`User`, `Order`); queries are reads (`user(id: ID!)`,
`orders(filter: OrderFilter)`); mutations are verb-object writes
(`createOrder`, `cancelOrder`); subscriptions are real-time streams
(`orderStatusChanged(orderId: ID!)`).

Default fields to non-nullable (`String!`) and only mark them
nullable when null is a meaningful state — the inverse of most REST
APIs. Define custom scalars for domain values like `DateTime` and
`Email`. Always use `input` types for mutation arguments so they can
evolve independently of output types.

### Resolver patterns

- **Root resolvers** handle top-level Query and Mutation fields.
- **Field resolvers** handle nested fields that come from a different
  source than the parent.
- **Default resolvers** return `parent[fieldName]` automatically — do
  not write trivial passthrough resolvers.

Resolvers stay thin: they call into a service or domain layer. Never
embed business logic, validation, or SQL inside a resolver.

### DataLoader and N+1

A list query that triggers one extra query per item is the N+1
problem. Solve it with DataLoader, which batches and caches reads
within a single request:

```javascript
const orderLoader = new DataLoader(async (userIds) => {
  const orders = await db.orders.findByUserIds(userIds);
  return userIds.map(id => orders.filter(o => o.userId === id));
});
// In User.orders resolver: (user) => orderLoader.load(user.id)
```

Two non-negotiable rules:

1. Create a new DataLoader instance **per request**. Sharing across
   requests leaks data between users.
2. The batch function must return results in the **same order** as
   the input keys, with `null` for missing entries.

### Pagination (Relay Connection spec)

Use cursor-based connections for every list field:

```graphql
type OrderConnection {
  edges: [OrderEdge!]!
  pageInfo: PageInfo!
}
type OrderEdge   { cursor: String!  node: Order! }
type PageInfo    { hasNextPage: Boolean!  hasPreviousPage: Boolean!  endCursor: String }
```

Cursors are opaque base64 tokens. Default `first` to 20, enforce a
hard maximum of 100 to prevent unbounded queries.

### Errors

Prefer **typed union results** for expected, domain-level errors so
clients pattern-match in a type-safe way:

```graphql
union CreateOrderResult = Order | ValidationError | InsufficientStock
```

Reserve the top-level `errors` array for unexpected failures
(authentication, server errors, transport). Always include
`extensions.code` (e.g. `UNAUTHENTICATED`, `INTERNAL_SERVER_ERROR`)
so clients can branch on a stable string.

### Schema evolution

- **Adding** types, fields, and enum values is always safe.
- **Deprecating**: mark with `@deprecated(reason: "Use newField")`,
  monitor field-usage telemetry, then remove once usage is near zero.
- **Never** change a field's type, and never make a nullable field
  non-nullable — both are silently breaking.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **DataLoader shared across requests.** A DataLoader caches results for the lifetime of the instance. Sharing one instance across requests means user A's query result can be returned to user B. Always create a new DataLoader instance per request.
- **N+1 resolvers without DataLoader.** A list resolver that triggers one database query per item will fan out to N+1 queries for a list of N items. This is the most common GraphQL performance failure mode. Every nested field resolver that fetches from a database or external service needs to be batched with DataLoader.
- **Making nullable fields non-nullable in a schema update.** A nullable field made non-nullable is a breaking change for any client that previously received `null` and handled it. Schema evolution only goes in one direction: add fields, deprecate old ones, never tighten nullability.
- **Returning `Boolean` from mutations.** A mutation that returns `true/false` forces the client to re-fetch the entity to update its cache. Return the affected entity or a result union so the client has the data it needs without a round-trip.
- **No depth or complexity limits in production.** A deeply nested recursive query can cause exponential database fan-out. Without depth and complexity limits, a single malicious or buggy query can exhaust the server. Enable depth limiting and cost-based rate limiting.
- **CRUD resolvers that mirror the database schema.** GraphQL APIs should be designed from the client's consumption needs, not from the storage model. A mutation called `updateOrder(id, status, tracking_number, shipped_at, ...)` with 10 optional fields is a database operation disguised as an intent; model it as `shipOrder(id, tracking_number)` instead.
- **One mutation that does everything with optional fields.** Large mutations with many optional fields make it impossible for the client to know which fields are required for which operations and make server-side validation complex. Split into intent-specific, focused mutations.

## Full reference

### Federation

For microservice architectures using Apollo Federation:

- Each service owns a subgraph and marks shared entities with
  `@key(fields: "id")`.
- The gateway composes subgraphs into a supergraph at build time, not
  at request time, so composition errors fail fast in CI.
- A subgraph only extends fields it can resolve from its own data —
  never reach across subgraphs from a resolver.

This keeps each subgraph independently deployable and avoids the
distributed-monolith trap.

### Persisted queries

In production, prefer **automatic persisted queries (APQ)**: the
client first sends only the SHA-256 hash of the query; the server
looks it up and executes the cached query. Benefits:

- Reduced bandwidth (hash, not full query string).
- Allowlisting becomes possible — reject any query whose hash is not
  in the registry, blocking arbitrary or hostile queries.
- Easier query-level caching at the edge.

### Performance and depth limits

Reject queries that exceed a sensible depth (e.g. 10) and complexity
score. Without limits, a malicious client can request a deeply
recursive graph and exhaust the server. Most GraphQL servers ship a
depth-limit middleware — turn it on.

Rate-limit by query cost, not by request count. A single GraphQL
request can be cheap or catastrophically expensive depending on the
selection set.

### Subscriptions

Subscriptions push events over WebSockets. Use them sparingly: each
subscription holds a connection, so the server pays for every
connected client. Prefer polling for low-frequency updates and
reserve subscriptions for genuinely real-time UX (live order status,
chat, collaborative editing).

### Anti-patterns to avoid

- **CRUD resolvers that mirror database tables.** Design from the
  client's needs, not from the schema migration file.
- **One mutation that does everything.** Split into named
  intent-specific mutations (`shipOrder`, `cancelOrder`) instead of
  `updateOrder(input)` with twenty optional fields.
- **Returning `Boolean` from mutations.** Return the affected entity
  or a result union so the client can update its cache.
- **DataLoader shared across requests.** Per-request, always.
- **Throwing strings or generic `Error` from resolvers.** Use the
  result-union pattern for expected errors.
- **Letting clients send arbitrary queries in production.** Use APQ
  + allowlisting.

### When to break the rules

Internal-only APIs with a single trusted client can skip persisted
queries, depth limits, and federation. Tiny schemas (under ~20 types)
can skip DataLoader if every resolver hits an in-memory cache. The
result-union error pattern adds verbosity that may not be worth it
for a small admin UI — top-level `errors` is fine there. As always,
understand the rule first; only then break it deliberately.
