# Test Fixtures Reference

Fixture and factory patterns for integration tests across languages.

## Python: pytest Fixtures and Factories

### Fixtures with Scope Control

```python
import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import Session

@pytest.fixture(scope="session")
def db_engine():
    """One engine per test session -- shared across all tests."""
    engine = create_engine("postgresql://test:test@localhost:5432/testdb")
    Base.metadata.create_all(engine)
    yield engine
    engine.dispose()

@pytest.fixture(scope="function")
def db_session(db_engine):
    """Fresh transaction per test -- rolls back after each."""
    conn = db_engine.connect()
    tx = conn.begin()
    session = Session(bind=conn)
    yield session
    session.close()
    tx.rollback()
    conn.close()
```

### Factory Functions

```python
def make_user(db_session, name="Alice", email=None, role="member"):
    """Create a user with sensible defaults. Override only what matters."""
    email = email or f"{name.lower()}@example.com"
    user = User(name=name, email=email, role=role)
    db_session.add(user)
    db_session.flush()  # Assign ID without committing
    return user

def test_admin_can_delete_users(db_session):
    admin = make_user(db_session, name="Admin", role="admin")
    target = make_user(db_session, name="Target")
    result = delete_user(db_session, actor=admin, target_id=target.id)
    assert result.success is True
```

### factory_boy for Complex Object Graphs

```python
import factory
from factory.alchemy import SQLAlchemyModelFactory

class UserFactory(SQLAlchemyModelFactory):
    class Meta:
        model = User
        sqlalchemy_session_persistence = "flush"

    name = factory.Faker("name")
    email = factory.LazyAttribute(lambda o: f"{o.name.lower().replace(' ', '.')}@example.com")
    role = "member"

class OrderFactory(SQLAlchemyModelFactory):
    class Meta:
        model = Order
        sqlalchemy_session_persistence = "flush"

    user = factory.SubFactory(UserFactory)
    total = factory.Faker("pydecimal", left_digits=3, right_digits=2, positive=True)

# Usage: order = OrderFactory(user__role="admin")
```

## JavaScript/TypeScript: beforeEach and Helpers

### Setup/Teardown Pattern

```typescript
import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

beforeEach(async () => {
  // Truncate all tables in correct order (respecting foreign keys)
  await prisma.$executeRaw`TRUNCATE TABLE orders, users RESTART IDENTITY CASCADE`;
});

afterAll(async () => {
  await prisma.$disconnect();
});

function createUser(overrides: Partial<User> = {}): Promise<User> {
  return prisma.user.create({
    data: {
      name: "Test User",
      email: `user-${Date.now()}@example.com`,
      role: "member",
      ...overrides,
    },
  });
}
```

### Builder Pattern for Complex Fixtures

```typescript
class OrderBuilder {
  private data: Partial<OrderInput> = {
    status: "pending",
    items: [],
  };

  withUser(userId: string) { this.data.userId = userId; return this; }
  withItem(name: string, qty: number) {
    this.data.items!.push({ name, qty });
    return this;
  }
  withStatus(s: string) { this.data.status = s; return this; }

  async build(prisma: PrismaClient) {
    return prisma.order.create({ data: this.data as OrderInput });
  }
}

// Usage:
const order = await new OrderBuilder()
  .withUser(user.id)
  .withItem("widget", 3)
  .withStatus("confirmed")
  .build(prisma);
```

## Rust: Test Helpers and Fixtures

### Helper Functions with Builders

```rust
pub struct TestUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

impl TestUser {
    pub fn builder() -> TestUserBuilder {
        TestUserBuilder::default()
    }
}

#[derive(Default)]
pub struct TestUserBuilder {
    name: Option<String>,
    role: Option<String>,
}

impl TestUserBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub async fn insert(self, pool: &PgPool) -> TestUser {
        let name = self.name.unwrap_or_else(|| "Alice".to_string());
        let email = format!("{}@example.com", name.to_lowercase());
        let row = sqlx::query_as!(
            TestUser,
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email",
            name, email
        )
        .fetch_one(pool)
        .await
        .expect("failed to insert test user");
        row
    }
}
```

### Transaction-Based Test Isolation

```rust
use sqlx::PgPool;

/// Each test gets a transaction that rolls back automatically.
pub async fn test_tx(pool: &PgPool) -> sqlx::Transaction<'_, sqlx::Postgres> {
    pool.begin().await.expect("failed to begin transaction")
}

#[sqlx::test]
async fn test_user_creation(pool: PgPool) {
    let mut tx = test_tx(&pool).await;
    let user = TestUser::builder()
        .name("Bob")
        .insert_tx(&mut tx)
        .await;
    assert_eq!(user.name, "Bob");
    // tx drops here -> automatic rollback
}
```

## Database Seeding Strategies

### Minimal Seed (Recommended)

Create only the data each test needs. Use factory/builder functions with sensible defaults. This keeps tests independent and easy to understand.

### Shared Baseline Seed

Load a common dataset once per suite (reference data, lookup tables). Individual tests add their own specific records on top. Good for read-heavy test suites.

```sql
-- seed.sql: loaded once per suite
INSERT INTO countries (code, name) VALUES ('DE', 'Germany'), ('US', 'United States');
INSERT INTO roles (name) VALUES ('admin'), ('member'), ('viewer');
```

### Snapshot Seed

Dump a known-good database state to a SQL file or Docker image. Restore before each suite. Fastest for large datasets but harder to maintain.

## Cleanup Approaches Summary

| Approach | Speed | Isolation | Multi-Connection |
|----------|-------|-----------|------------------|
| Transaction rollback | Fast | Per-test | No (single conn only) |
| TRUNCATE + CASCADE | Medium | Per-test | Yes |
| DROP + CREATE schema | Slow | Per-suite | Yes |
| Disposable container | Slow | Per-suite | Yes |

Choose transaction rollback when possible. Fall back to truncation for tests that need multiple connections (e.g., testing concurrent access). Use disposable containers for full isolation in CI.
