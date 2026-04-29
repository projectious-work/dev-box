---
name: integration-testing
description: |
  Integration and E2E testing patterns — testcontainers, fixtures, API mocking, snapshots, and CI isolation. Use when writing integration tests against databases, APIs, or queues; setting up testcontainers or fixture infrastructure; mocking external APIs; or debugging flaky tests.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-integration-testing
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Integration Testing

## Intro

Integration tests verify that your code works against real
collaborators — databases, APIs, queues — not just mocks. The trick
is keeping them fast, isolated, and reproducible. Disposable
containers, per-test fixtures, and strict API mocks are the
primary tools.

## Overview

### Testcontainers

Spin up real services as disposable containers for the duration of
the test suite:

- Each suite gets its own container — no shared state.
- Use fixed image tags (not `latest`) for reproducibility.
- Set health checks or wait strategies so tests start only after
  the service is ready.
- Tear down after the suite (most libraries handle this via RAII
  or `afterAll`).

Supported in Python (`testcontainers`), Rust (`testcontainers`),
JS/TS (`testcontainers`), Java (`testcontainers`), and Go
(`testcontainers-go`).

### Database fixtures

Three core strategies:

- **Factories** — generate objects with sensible defaults; override
  only what matters per test.
- **Fixtures** — predefined datasets loaded before tests; good for
  read-heavy tests.
- **Seeds** — baseline data loaded once for the suite; reset per
  test via transactions.

Cleanup approaches (pick one per project):

| Approach | Speed | Isolation | Multi-Connection |
|----------|-------|-----------|------------------|
| Transaction rollback | Fast | Per-test | No (single conn) |
| TRUNCATE + CASCADE | Medium | Per-test | Yes |
| DROP + CREATE schema | Slow | Per-suite | Yes |
| Disposable container | Slow | Per-suite | Yes |

Default to transaction rollback. Fall back to truncation when you
need multiple connections (e.g. concurrent access tests). Use
disposable containers for full isolation in CI.

### API mocking

Mock external HTTP APIs so integration tests run without network
dependencies:

| Tool | Language | Approach |
|------|----------|----------|
| **WireMock** | Java, standalone | Local HTTP server matching by URL/body |
| **MSW** | JS/TS | Network-level interception via Service Workers / node interceptors |
| **nock** | Node.js | Patches `http`/`https` modules |
| **httpmock** | Rust | Local mock server with registered expectations |
| **responses** | Python | Decorators that intercept `requests` calls |

Best practices:

- Assert on what the code sent (method, headers, body), not just
  the response.
- Use strict mode — fail on any unexpected request.
- Record real API responses once and replay in CI (contract
  tests).

### Snapshot and E2E

Snapshot tests capture function or component output and compare
against a stored baseline. Good for serialized JSON, rendered HTML,
CLI output, and error messages. Update intentionally
(`--update-snapshots` / `-u`) and review diffs in PRs. Store
snapshots next to tests in version control. Normalize volatile
data (timestamps, random IDs) before snapshotting.

For E2E:

- **Page Object Model** — encapsulate page interactions; tests
  read like user stories.
- **Arrange-Act-Assert** — set up state, perform the action,
  verify the outcome.
- Test user journeys, not individual pages. One E2E test should
  cover a meaningful flow.
- Use dedicated test accounts and isolated environments. Never
  test against production data.

### CI isolation and flaky tests

- Run integration tests in a separate CI job with longer timeouts.
- Use testcontainers or Docker Compose for backing services.
- Give each parallel worker a unique database/schema.
- Set per-test timeouts (not just per-job) to catch hangs.
- Cache container images in CI to speed up startup.

Common flaky-test causes and fixes:

- **Shared state** — per-test databases or transaction rollback.
- **Timing/race conditions** — replace `sleep` with explicit
  wait-for conditions.
- **Non-deterministic ordering** — randomize test order in CI to
  surface hidden dependencies.
- **Network flakiness** — mock all external APIs.
- **Resource exhaustion** — limit parallelism, close connections.

A flaky test that is ignored is worse than no test at all. Tag
known flaky tests and track them.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Sharing a single database across parallel test workers.** Parallel workers reading and writing to the same schema produces ordering-dependent failures and data contamination. Give each worker a unique database or schema, or use per-test transaction rollback.
- **Using `sleep()` to wait for async behavior.** Fixed delays are both fragile (fail on slow CI runners) and wasteful (over-wait on fast machines). Replace with explicit wait-for conditions that poll until the expected state is reached or a timeout fires.
- **Mocking the unit under test instead of its boundary.** If the code being tested is itself replaced by a mock, the test verifies nothing. Mocks belong at the boundary — the external API, database, or queue — not inside the module under test.
- **Asserting only on response shape, never on the request the code sent.** Testing that "we got a 200" but not verifying the request body, headers, or method means outbound contract violations go undetected. Assert on what the code sent, not just what it received.
- **Snapshotting timestamps or random IDs without normalization.** Snapshots that contain `created_at: 2026-04-08T10:23:11Z` or `id: "f3a2..."` fail on every run. Normalize or mask volatile fields before snapshotting.
- **Marking flaky tests as skipped without assigning an owner.** A skipped test is a test that never catches anything. Tag the test with the owner's handle and a follow-up date; skipped tests must be fixed before a release.
- **Using `latest` image tags for testcontainers.** When an upstream container image is updated, CI breaks unpredictably. Pin testcontainer images to fixed tags (`postgres:16-alpine`, not `postgres:latest`) for reproducible CI runs.

## Full reference

### Testcontainers with Postgres (Python)

```python
import pytest
from testcontainers.postgres import PostgresContainer
from sqlalchemy import create_engine, text

@pytest.fixture(scope="module")
def db_engine():
    with PostgresContainer("postgres:16-alpine") as pg:
        engine = create_engine(pg.get_connection_url())
        with engine.begin() as conn:
            conn.execute(text("CREATE TABLE users (id SERIAL, name TEXT)"))
        yield engine

def test_insert_and_query_user(db_engine):
    with db_engine.begin() as conn:
        conn.execute(text("INSERT INTO users (name) VALUES (:n)"), {"n": "Alice"})
        result = conn.execute(text("SELECT name FROM users")).fetchone()
    assert result[0] == "Alice"
```

### API mocking with MSW (TypeScript)

```typescript
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { fetchWeather } from "../src/weather";

const server = setupServer(
  http.get("https://api.weather.example/current", ({ request }) => {
    const url = new URL(request.url);
    if (url.searchParams.get("city") === "Berlin") {
      return HttpResponse.json({ temp: 18, condition: "cloudy" });
    }
    return new HttpResponse(null, { status: 404 });
  })
);

beforeAll(() => server.listen({ onUnhandledRequest: "error" }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test("returns weather for known city", async () => {
  const weather = await fetchWeather("Berlin");
  expect(weather.temp).toBe(18);
});
```

### httpmock (Rust)

```rust
use httpmock::prelude::*;
use reqwest;

#[tokio::test]
async fn test_client_sends_auth_header() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/data")
            .header("Authorization", "Bearer test-token");
        then.status(200)
            .json_body_obj(&serde_json::json!({"items": [1, 2, 3]}));
    });

    let client = ApiClient::new(&server.base_url(), "test-token");
    let result = client.fetch_data().await.unwrap();

    assert_eq!(result.items, vec![1, 2, 3]);
    mock.assert();
}
```

### Fixture patterns

For deeper fixture and factory patterns across Python, TypeScript,
and Rust, see `references/test-fixtures.md`. Key takeaways:

- **pytest fixtures** — use `scope="session"` for an engine and
  `scope="function"` with transaction rollback for a per-test
  session. `factory_boy` handles complex object graphs via
  `SubFactory`.
- **TypeScript** — pair `beforeEach` truncation with builder
  functions or a `Builder` class for complex fixtures. Order
  truncations to respect foreign keys.
- **Rust** — implement a `TestUserBuilder` with `.insert(pool)` or
  `.insert_tx(&mut tx)` methods. `#[sqlx::test]` plus
  per-test transactions gives automatic rollback on drop.

### Database seeding strategies

- **Minimal seed** (recommended) — each test creates only the data
  it needs through factories. Tests stay independent and easy to
  read.
- **Shared baseline seed** — load reference data (countries,
  roles) once per suite; tests add their own records on top. Good
  for read-heavy suites.
- **Snapshot seed** — dump a known-good database state to a SQL
  file or container image and restore before each suite. Fastest
  for large datasets but harder to maintain.

### Anti-patterns

- Sharing a single database across parallel test workers
- Using `sleep()` to wait for async behavior
- Mocking the unit under test instead of its boundary
- Asserting only on response shape, never on the request the code
  sent
- Snapshotting timestamps or random IDs without normalization
- Marking flaky tests as skipped without an owner or follow-up
