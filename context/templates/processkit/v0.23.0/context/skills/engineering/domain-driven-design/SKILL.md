---
name: domain-driven-design
description: |
  Domain-Driven Design strategic and tactical patterns — bounded contexts, aggregates, value objects, context mapping. Use when modeling a complex business domain, defining microservice boundaries, designing aggregates, establishing a ubiquitous language, or refactoring a big ball of mud into well-defined contexts.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-domain-driven-design
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Domain-Driven Design

## Intro

DDD has two halves: strategic design (bounded contexts, context maps,
ubiquitous language) and tactical design (aggregates, entities, value
objects, domain events, repositories). Use it when the business logic
is the differentiator and the cost of getting the model wrong is
high.

## Overview

### Ubiquitous language

Before writing any code, build a shared vocabulary with domain
experts. Identify key terms and write down precise definitions.
Reject technical jargon in the model — use `Order`, `Shipment`,
`PolicyRenewal`, not `EntityManager` or `ServiceHandler`. If two
teams use the same word differently, that is a strong signal of two
distinct bounded contexts. Maintain a glossary in the project docs;
class names, method names, and variable names should all reflect the
language.

### Strategic design — bounded contexts

A bounded context is a boundary within which a single domain model
and ubiquitous language apply. To find them:

- Map the business capabilities (ordering, billing, shipping,
  inventory).
- Look for linguistic boundaries — where the same word means
  different things (`Product` in catalog vs warehouse).
- Align with team ownership — one team per context is ideal.
- Start coarse, split later. Merging contexts is harder than
  splitting.

Each bounded context has its own domain model (its own `Customer`
class, not shared), its own data store or schema, and communicates
with others through well-defined interfaces.

### Context mapping

Define how bounded contexts relate. The deeper reference lives in
`references/ddd-building-blocks.md`.

| Pattern | Relationship | Use when |
|---|---|---|
| Shared Kernel | Two contexts share a small common model | Tightly coupled teams co-evolving a shared piece |
| Customer-Supplier | Upstream supplies, downstream consumes | Clear dependency, upstream accommodates downstream |
| Conformist | Downstream conforms to upstream's model | You cannot influence upstream (third-party API) |
| Anti-Corruption Layer | Downstream translates upstream's model | Protecting your model from a legacy or external system |
| Open Host Service | Upstream provides a well-defined protocol | Many consumers, upstream publishes a stable API |
| Published Language | Shared interchange format (JSON schema, Avro) | Cross-context communication needs a neutral format |
| Separate Ways | No integration | Contexts have no meaningful interaction |

Default recommendation: use an Anti-Corruption Layer when integrating
with any external or legacy system. It prevents foreign concepts from
leaking into your domain.

### Tactical design — aggregates

An aggregate is a cluster of domain objects treated as a single unit
for data changes. Rules:

- **One aggregate root** — external references point only at the
  root.
- **Protect invariants** — the aggregate enforces business rules
  within its boundary.
- **Small aggregates** — prefer small aggregates that reference each
  other by ID, not by object reference.
- **One transaction modifies one aggregate** — cross-aggregate
  consistency is eventual.
- **Identity** — aggregate roots have a globally unique ID; internal
  entities have locally unique IDs.

The classic mistake is making aggregates too large. If `Order`
contains `Customer` contains `Address`, any address change locks the
entire order. Reference `customerId` instead.

### Entities, value objects, and domain events

**Entities** have identity and a lifecycle. Two entities with
identical attributes but different IDs are different. Implement
equality by ID.

**Value objects** are defined entirely by their attributes and are
immutable. Two value objects with the same attributes are equal.
Prefer `EmailAddress` over `String`, `Money` over `float`.

**Domain events** record something meaningful that has happened.
Named in past tense (`OrderPlaced`, `PaymentReceived`,
`InventoryReserved`), immutable, carry enough data for consumers to
react without querying back, and published only after the state
change is persisted.

### Repositories, services, factories

- **Repositories** provide collection-like access to aggregates. One
  per aggregate root. Interface in the domain layer, implementation
  in infrastructure. Methods speak domain (`find_pending_by_customer`),
  not SQL.
- **Domain services** hold logic that does not naturally belong to a
  single entity (e.g. `TransferService.transfer(from, to, amount)`).
  Stateless. Use sparingly — most logic should live on aggregates.
- **Factories** encapsulate complex creation when validation,
  defaults, or multi-step initialization are involved.

### Example workflows

- **E-commerce domain:** identify contexts (Catalog, Ordering,
  Payment, Shipping, Inventory). Aggregates: `Product` in Catalog,
  `Order` with `OrderLine` in Ordering, `Shipment` in Shipping. Value
  objects: `Money`, `Address`, `SKU`. Ordering integrates with
  Inventory through events (reserve stock); Payment uses an ACL to
  the gateway.
- **Reviewing a god aggregate:** a `Customer` containing `Orders`,
  `Addresses`, `PaymentMethods`, and `Preferences` locks the world on
  any address change. Split into `Customer` (profile), `Order`
  (references customerId), `PaymentMethod` (references customerId),
  each its own aggregate.
- **Splitting a monolith:** map existing code to business
  capabilities via event storming. Identify contexts by linguistic
  boundaries and team ownership. Propose a context map with ACLs at
  legacy boundaries and pub/sub between contexts. Extract the most
  independent context first.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Anemic domain model — entities as data bags with all logic in services.** When `Order` is a plain struct with getters and setters and all business logic lives in `OrderService`, you get the complexity of objects without the benefit of encapsulation. The aggregate cannot enforce its own invariants. Move business behavior onto entities and aggregates so they protect their own rules.
- **God aggregate that locks everything.** An `Order` aggregate that contains the full `Customer`, all their `Addresses`, their `PaymentMethods`, and their `Preferences` means any address change locks the entire customer's orders. Cross-aggregate references should be by ID, not by object reference. One transaction modifies one aggregate.
- **Defining bounded contexts prematurely before understanding the domain.** Starting with microservices boundaries drawn on a whiteboard before any code is written produces contexts that split along technical lines (auth service, data service) rather than domain lines. Work in a modular monolith first, discover actual linguistic and team-ownership boundaries through usage, then extract.
- **Shared database tables across bounded contexts.** Two bounded contexts reading and writing the same `users` table are not actually bounded — they are tightly coupled through the database schema. Every schema change must coordinate across contexts. Each bounded context must own its data store or, at minimum, its own schema within a shared database.
- **Publishing domain events before persisting the state change.** Publishing an `OrderSubmitted` event before the database write commits means the event is out in the world but the state change failed — consumers react to an event whose cause doesn't exist. Use the outbox pattern: write event and state change in the same transaction; publish from the outbox asynchronously.
- **Using technical event names instead of domain language.** `UserEntityUpdatedEvent` describes an implementation detail. `CustomerEmailVerified` describes something that happened in the domain. Events must be named in past tense using ubiquitous language so they are meaningful to domain experts, not just developers.
- **Treating value objects as mutable.** A `Money` object that can have its `amount` mutated after construction is not a value object — it is a mutable entity without identity. Value objects must be immutable: to "change" a value, create a new instance. Immutability makes equality-by-value safe and enables value objects to be shared.

## Full reference

### Aggregate example

```python
class Order:  # Aggregate root
    def __init__(self, order_id, customer_id):
        self.id = order_id
        self.customer_id = customer_id  # reference by ID, not object
        self.lines = []                  # internal entities
        self.status = "draft"

    def add_line(self, product_id, quantity, unit_price):
        if self.status != "draft":
            raise DomainError("Cannot modify a submitted order")
        self.lines.append(OrderLine(product_id, quantity, unit_price))

    def submit(self):
        if not self.lines:
            raise DomainError("Cannot submit empty order")
        self.status = "submitted"
        return OrderSubmitted(self.id, self.total())
```

External code should reference only `Order` and call `add_line` /
`submit`; nothing reaches into `OrderLine` directly. The root
enforces every invariant in the cluster.

### Value object example

```python
@dataclass(frozen=True)
class Money:
    amount: Decimal
    currency: str

    def __post_init__(self):
        if self.amount < 0:
            raise ValueError("Amount cannot be negative")
        if self.currency not in VALID_CURRENCIES:
            raise ValueError(f"Invalid currency: {self.currency}")

    def add(self, other: "Money") -> "Money":
        if self.currency != other.currency:
            raise DomainError("Cannot add different currencies")
        return Money(self.amount + other.amount, self.currency)
```

Self-validating, immutable, equal-by-attributes. To "change" a value,
construct a new instance.

### Domain event example

```python
@dataclass(frozen=True)
class OrderSubmitted:
    order_id: str
    customer_id: str
    total: Money
    items: list[OrderLineSnapshot]
    occurred_at: datetime
```

Past tense. Immutable. Carries the data consumers need so they do
not have to call back to the producing service. Published only after
the state change is persisted.

### Repository example

```python
# Domain layer — interface
class OrderRepository(Protocol):
    def find_by_id(self, order_id: OrderId) -> Order | None: ...
    def save(self, order: Order) -> None: ...
    def find_pending_by_customer(
        self, customer_id: CustomerId
    ) -> list[Order]: ...

# Infrastructure layer — implementation
class PostgresOrderRepository:
    def find_by_id(self, order_id: OrderId) -> Order | None:
        row = self.db.query("SELECT ... WHERE id = %s", order_id)
        return self._to_aggregate(row) if row else None
```

The interface speaks domain language and returns fully reconstituted
aggregates. The infrastructure implementation hides SQL and ORM
details.

### Anti-patterns

- **Anemic domain model** — entities are data bags with
  getters/setters and all logic lives in services. Move behavior onto
  entities and aggregates.
- **God aggregate** — one massive aggregate that locks everything.
  Split into smaller aggregates connected by ID references.
- **Leaking context** — using another context's types directly.
  Introduce an ACL or map to local types.
- **Premature contexts** — defining bounded contexts before
  understanding the domain. Start with a modular monolith and
  discover boundaries through usage.
- **Shared database** — multiple contexts reading and writing the
  same tables. Each context owns its own store or at least its own
  schema.
- **Technical event names** (`OrderUpdatedEvent`) instead of domain
  names (`OrderSubmitted`).
- **Publishing events before persisting state** — leads to
  inconsistency on failure. Use the outbox pattern.
- **Comparing entities by attributes** instead of ID.
- **Exposing setters** — use intention-revealing methods like
  `change_email`, not `set_email`.

### Pattern selection notes

- **Shared Kernel** is high-coupling. Use only within a single team.
- **Anti-Corruption Layer** is the default for external integrations
  — it isolates your model from foreign concepts.
- **Open Host Service + Published Language** is best when many
  consumers depend on a stable upstream API.
