# Architecture Skills

Skills for software architecture, design patterns, and system design.

---

### software-architecture

> Analyzes codebases for architectural patterns and quality. Use when designing systems, creating ADRs, reviewing structure, or generating architecture diagrams.

**Triggers:** Designing systems, creating ADRs, reviewing code structure, generating C4 diagrams, applying SOLID/DRY/KISS principles
**Tools:** None
**References:** `patterns.md`

Key capabilities:

- Analyze existing architecture by mapping module organization, dependency directions, and identifying violations (circular deps, layer skipping, leaky abstractions)
- Suggest architecture patterns matched to project type (layered, hexagonal, modular monolith, microservices, pipe-and-filter, event-driven)
- Create Architecture Decision Records (ADRs) with context, decision, and consequences
- Review code for architectural violations: god modules, tight coupling, missing boundaries
- Generate C4 diagrams (System Context, Container, Component) using Mermaid syntax

??? example "Example usage"
    "Review this project's architecture" -- Maps the dependency graph, identifies that controllers directly import database models (layer violation), suggests introducing a service layer with repository traits, and provides a C4 Level 3 component diagram of the proposed structure.

---

### event-driven-architecture

> Event-driven system design including event sourcing, CQRS, pub/sub, saga patterns, and message broker selection. Use when designing event-driven systems, implementing messaging, or reviewing async architectures.

**Triggers:** Designing event-driven systems, choosing message brokers, implementing event sourcing or CQRS, designing saga patterns, debugging async architecture issues
**Tools:** None
**References:** `messaging-patterns.md`

Key capabilities:

- Evaluate whether event-driven design is a good fit for the system at hand
- Choose messaging patterns: pub/sub, point-to-point, request-reply, competing consumers
- Select message brokers (Kafka, RabbitMQ, NATS, SQS/SNS, Redis Streams) based on throughput, ordering, replay, and operational requirements
- Implement event sourcing with append-only event streams, snapshots, and schema evolution
- Design CQRS with separate write and read models, handling eventual consistency
- Implement saga patterns (orchestration vs. choreography) for distributed transactions
- Ensure reliability with idempotent consumers, dead letter queues, outbox pattern, and schema registries

??? example "Example usage"
    "Design an order processing system" -- Analyzes the workflow (order placed, payment processed, inventory reserved, shipment created), recommends Kafka for event backbone with event sourcing on the order aggregate, designs an orchestration saga with compensating actions, and defines event schemas with Avro and a schema registry.

---

### domain-driven-design

> Domain-Driven Design strategic and tactical patterns. Bounded contexts, aggregates, value objects, and context mapping. Use when modeling complex domains, designing microservice boundaries, or reviewing domain models.

**Triggers:** Modeling complex business domains, defining microservice boundaries, reviewing domain models, creating ubiquitous language, refactoring monoliths
**Tools:** None
**References:** `ddd-building-blocks.md`

Key capabilities:

- Establish ubiquitous language with precise domain term definitions aligned between code and domain experts
- Strategic design: identify bounded contexts by mapping business capabilities and linguistic boundaries
- Context mapping with patterns: Shared Kernel, Customer-Supplier, Conformist, Anti-Corruption Layer, Open Host Service, Published Language
- Tactical design: aggregate design with small aggregates, single root, transactional boundaries, and ID-based references
- Model entities (identity-based), value objects (attribute-based, immutable), and domain events (past-tense, immutable facts)
- Design repositories, domain services, and factories following DDD principles
- Identify anti-patterns: anemic domain model, god aggregate, leaking context, shared database

??? example "Example usage"
    "Design a domain model for an online store" -- Identifies bounded contexts (Catalog, Ordering, Payment, Shipping, Inventory), defines aggregates (Product, Order with OrderLine, Shipment), uses value objects for Money, Address, and SKU, and maps context relationships with ACLs at integration boundaries.

---

### system-design

> System design methodology from requirements through capacity estimation to component design and trade-offs. Use when designing distributed systems, evaluating architectures, or preparing system design discussions.

**Triggers:** Designing distributed systems from scratch, evaluating scalability and reliability, performing capacity estimation, discussing architectural trade-offs
**Tools:** None
**References:** `estimation-cheatsheet.md`

Key capabilities:

- Gather functional and non-functional requirements (scale, latency, availability, consistency, durability, cost)
- Back-of-envelope capacity estimation: users to QPS, storage, bandwidth, and compute
- High-level design with 5-10 major components following data flow from client inward
- Component deep dives: API design, data model, scaling strategy, failure handling, caching
- Trade-off analysis: consistency vs. availability, latency vs. throughput, simplicity vs. scalability
- Apply scalability patterns: horizontal scaling, sharding, read replicas, caching layers, async processing, rate limiting, circuit breakers

??? example "Example usage"
    "Design a URL shortener" -- Functional: create short URL, redirect, analytics. Estimates ~40 writes/s, ~4000 reads/s (100:1 ratio), ~10TB storage over 5 years. Designs a stateless API service with Redis cache for hot URLs, sharded database for URL mapping, Base62 encoding, CDN for redirect caching, and async event stream for analytics.
