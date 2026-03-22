# API & Integration Skills

Skills for API design, protocol patterns, and system integration.

---

### api-design

> REST API design including resource naming, HTTP methods, status codes, pagination, versioning, and OpenAPI specs. Use when designing APIs, reviewing API contracts, or writing OpenAPI/Swagger documentation.

**Triggers:** Designing a new REST API or extending an existing one, reviewing API contracts, writing OpenAPI/Swagger docs, choosing pagination or versioning strategies, defining error response formats.
**Tools:** `Bash` `Read` `Write`
**References:** `rest-conventions.md`, `openapi-patterns.md`

Key capabilities:

- Resource naming conventions (plural nouns, kebab-case, shallow nesting)
- HTTP method-to-CRUD mapping with correct status codes
- Cursor-based and offset-based pagination patterns
- Filtering and sorting via query parameters
- URI path versioning and header versioning strategies
- Consistent error response envelope format
- Rate limiting headers (X-RateLimit-Limit, Remaining, Reset)
- HATEOAS links for discoverable APIs
- OpenAPI 3.1 spec authoring with reusable components
- API design review checklist (8-point verification)

??? example "Example usage"
    Design a REST API for managing orders in an e-commerce system. The agent
    designs endpoints (POST, GET, PATCH, DELETE for /v1/orders and sub-resources),
    writes an OpenAPI 3.1 spec with shared schemas for Order, LineItem, Payment,
    and PaginatedResponse, and includes error envelope and rate limit headers.

---

### graphql-patterns

> GraphQL schema design, resolver patterns, N+1 prevention with DataLoader, and federation. Use when designing GraphQL APIs, implementing resolvers, or optimizing GraphQL performance.

**Triggers:** Designing a GraphQL schema, implementing resolvers, diagnosing N+1 query problems, adding pagination, setting up federation, evolving a schema without breaking clients.
**Tools:** `Bash` `Read` `Write`
**References:** None

Key capabilities:

- Schema design from the client perspective (types, queries, mutations, subscriptions)
- Resolver patterns (root, field, default) with thin resolver architecture
- N+1 problem diagnosis and DataLoader batching/caching solution
- Relay Connection Spec cursor-based pagination
- Domain errors as union types for type-safe error handling
- Apollo Federation with @key directives and subgraph composition
- Schema evolution rules (safe additions, deprecation with @deprecated, breaking change avoidance)
- Automatic persisted queries (APQ) for bandwidth and security

??? example "Example usage"
    A list query fetching 50 projects takes 3 seconds. The agent identifies
    N+1 queries (1 for projects + 50 for owner + 50 for taskCount), implements
    DataLoader for both fields, and drops response time to 120ms.

---

### grpc-protobuf

> Protocol Buffers schema design and gRPC service patterns including streaming, error handling, and backward compatibility. Use when designing gRPC services, writing .proto files, or implementing gRPC clients/servers.

**Triggers:** Designing .proto files, implementing gRPC services (unary, streaming), choosing communication patterns, handling errors with gRPC status codes, ensuring backward compatibility, adding interceptors.
**Tools:** `Bash(protoc:*)` `Bash(grpcurl:*)` `Read` `Write`
**References:** `proto-conventions.md`

Key capabilities:

- Proto3 schema design with proper packaging, enums (UNSPECIFIED zero value), and timestamps
- Four gRPC service patterns: unary, server streaming, client streaming, bidirectional
- Dedicated Request/Response wrapper messages per RPC
- Error handling with gRPC status codes (INVALID_ARGUMENT, NOT_FOUND, UNAVAILABLE, etc.)
- Rich error details using google.rpc.Status with BadRequest/ErrorInfo
- Backward compatibility rules and reserved field management
- Interceptor chains for auth, logging, metrics, and validation
- Tooling guidance: buf for linting/breaking change detection, grpcurl for invocation

??? example "Example usage"
    A price field needs to change from int32 to int64 but clients already use it.
    The agent adds a new price_cents (int64) field with a new number, deprecates
    the old field, populates both during migration, and runs buf breaking to confirm
    no violations.

---

### webhook-integration

> Webhook design and consumption including signature verification, idempotency, retry handling, and security. Use when implementing webhooks, designing event notification systems, or debugging webhook deliveries.

**Triggers:** Designing a webhook system for event notifications, implementing a webhook consumer, adding HMAC-SHA256 signature verification, debugging failed deliveries or duplicate processing, setting up dead letter queues.
**Tools:** `Bash(curl:*)` `Read` `Write`
**References:** None

Key capabilities:

- Payload design with unique event IDs, dotted type names, and stable envelope format
- HMAC-SHA256 signature verification with constant-time comparison and replay prevention
- Idempotency via event ID deduplication with TTL-bounded storage
- Retry handling with exponential backoff (sender) and async processing (consumer)
- Dead letter queues for exhausted retries with replay tooling
- Out-of-order event handling with version/sequence numbers
- Local testing with ngrok/cloudflared and payload inspection tools
- Security hardening: TLS-only, IP allowlisting, payload size limits, vault-stored secrets

??? example "Example usage"
    A webhook consumer processes some events twice and misses others. The agent
    finds missing idempotency checks (retries reprocessed) and synchronous heavy
    processing causing sender timeouts. Adds event ID dedup with a DB unique
    constraint, moves processing to a background queue, and returns 202 immediately.
    Success rate rises from 74% to 99.8%.
