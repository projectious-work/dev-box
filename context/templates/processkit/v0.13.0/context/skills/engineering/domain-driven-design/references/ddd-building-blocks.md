# DDD Building Blocks Reference

Definitions, rules, code examples, and common mistakes for each tactical building block.

## Aggregate

A cluster of associated objects treated as a unit for data changes, with a single root entity.

**Rules:**
- External objects reference only the aggregate root, never internal entities
- The root enforces all invariants across the cluster
- One transaction = one aggregate. Cross-aggregate updates are eventually consistent.
- Delete the root = delete the entire aggregate

**Example:**
```python
class Order:  # Aggregate root
    def __init__(self, order_id, customer_id):
        self.id = order_id
        self.customer_id = customer_id  # reference by ID, not object
        self.lines = []  # internal entities
        self.status = "draft"

    def add_line(self, product_id, quantity, unit_price):
        if self.status != "draft":
            raise DomainError("Cannot modify a submitted order")
        line = OrderLine(product_id, quantity, unit_price)
        self.lines.append(line)

    def submit(self):
        if not self.lines:
            raise DomainError("Cannot submit empty order")
        self.status = "submitted"
        return OrderSubmitted(self.id, self.total())
```

**Common mistakes:** Making aggregates too large (include only what is needed to enforce invariants). Modifying multiple aggregates in one transaction (use domain events instead).

## Entity

An object defined by its identity, not its attributes. Has a lifecycle.

**Rules:**
- Equality is based on ID, not attribute values
- Has a unique identifier within its context
- Can change state over time (mutable)
- Encapsulates behavior related to its state

**Example:**
```python
class User:
    def __init__(self, user_id, name, email):
        self.id = user_id
        self.name = name
        self.email = email

    def change_email(self, new_email):
        if not Email.is_valid(new_email):
            raise DomainError("Invalid email")
        self.email = Email(new_email)

    def __eq__(self, other):
        return isinstance(other, User) and self.id == other.id
```

**Common mistakes:** Comparing entities by attributes instead of ID. Exposing setters without validation (use intention-revealing methods like `change_email` not `set_email`).

## Value Object

An object defined entirely by its attributes. Immutable and interchangeable.

**Rules:**
- No identity — two value objects with the same attributes are equal
- Immutable — create a new instance to "change" a value
- Self-validating — invalid states cannot be constructed
- Prefer value objects over primitive types

**Example:**
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

**Common mistakes:** Using primitives instead of value objects (`str` for email, `float` for money). Making value objects mutable.

## Domain Event

A record of something significant that happened in the domain. Past tense.

**Rules:**
- Immutable once created
- Named in past tense reflecting domain language
- Contains enough data for consumers to react without querying back
- Published after the state change is persisted (not before)

**Example:**
```python
@dataclass(frozen=True)
class OrderSubmitted:
    order_id: str
    customer_id: str
    total: Money
    items: list[OrderLineSnapshot]
    occurred_at: datetime
```

**Common mistakes:** Using technical names (`OrderUpdatedEvent`) instead of domain names (`OrderSubmitted`). Publishing events before persisting state (leads to inconsistency on failure). Including too little data (forces consumers to call back).

## Repository

Provides collection-like access to aggregates. Abstracts persistence.

**Rules:**
- One repository per aggregate root
- Interface defined in the domain layer, implementation in infrastructure
- Methods use domain language, not SQL/ORM language
- Returns fully reconstituted aggregates, not partial data

**Example:**
```python
# Domain layer — interface
class OrderRepository(Protocol):
    def find_by_id(self, order_id: OrderId) -> Order | None: ...
    def save(self, order: Order) -> None: ...
    def find_pending_by_customer(self, customer_id: CustomerId) -> list[Order]: ...

# Infrastructure layer — implementation
class PostgresOrderRepository:
    def find_by_id(self, order_id: OrderId) -> Order | None:
        row = self.db.query("SELECT ... WHERE id = %s", order_id)
        return self._to_aggregate(row) if row else None
```

**Common mistakes:** Exposing ORM query builders. Having repositories for non-root entities. Returning DTOs instead of aggregates.

## Domain Service

Stateless operations that involve multiple aggregates or do not belong to a single entity.

**Rules:**
- Stateless — all state lives in entities and value objects
- Named after a domain operation, not a technical pattern
- Use sparingly — first try to place logic on an aggregate

**Example:**
```python
class TransferService:
    def transfer(self, from_acct: Account, to_acct: Account, amount: Money):
        from_acct.debit(amount)
        to_acct.credit(amount)
        return FundsTransferred(from_acct.id, to_acct.id, amount)
```

**Common mistakes:** Putting all logic in services (anemic model). Making services stateful. Using services for logic that belongs on a single aggregate.

---

## Context Mapping Patterns

How bounded contexts relate to each other:

```
[Catalog Context]                    [Ordering Context]
    |                                       |
    +-- Open Host Service (product API) --> ACL (translates Product to OrderItem)
    |
    +-- Published Language (JSON schema for product events)

[Payment Context]                    [External Payment Gateway]
    |                                       |
    +------------ ACL ----------------------+
    (translates gateway responses to domain PaymentResult)

[Shipping Context] <-- Customer/Supplier -- [Ordering Context]
    (Ordering tells Shipping what to ship; Shipping accommodates)
```

- **Shared Kernel:** Both contexts share a small, co-evolved module. High coupling — use only within the same team.
- **Anti-Corruption Layer:** A translation layer that protects your model from external models. Default choice for external integrations.
- **Open Host Service + Published Language:** Upstream provides a stable API with documented schema. Best for many-consumer scenarios.
