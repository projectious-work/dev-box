---
name: testing-strategy
description: |
  Testing approach — unit vs integration vs E2E, coverage goals, organization. Use when deciding what to test, choosing between unit/integration/E2E tests, planning a test strategy for new functionality, or setting coverage goals.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-testing-strategy
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
    layer: 3
---

# Testing Strategy

## Intro

Pick the cheapest test that gives you confidence the code is
correct. Many fast unit tests, fewer integration tests, and a thin
layer of end-to-end tests for the critical paths — that is the
testing pyramid, and it still works.

## Overview

### Match the test type to the code

- **Pure logic** (calculations, transformations) → unit tests.
- **Integration points** (database, HTTP API, file I/O) →
  integration tests.
- **User workflows** (end-to-end paths) → E2E tests.

### Apply the testing pyramid

- Many unit tests — fast, isolated, cheap to run on every commit.
- Fewer integration tests — slower, exercise the boundary between
  your code and external systems.
- Minimal E2E tests — slowest and most flaky, reserved for the
  critical happy paths a user must be able to complete.

### Naming and shape

Name tests `test_<function>_<scenario>_<expected_result>` so
failures self-document. Each test follows the AAA pattern:
**Arrange** the inputs, **Act** by calling the unit, **Assert**
the expected outcome. One logical assertion per test where
practical.

### Prioritize what you test

- Error paths and edge cases over happy paths
- Public API over internal implementation details
- Complex branching logic over trivial getters/setters

### Coverage as a smoke alarm

Aim for ~80% coverage on business logic. Do not chase 100%
everywhere — coverage is a smoke alarm, not a goal. The last 20%
is usually trivial code, generated code, or defensive paths that
cost more to test than they save.

### Example

User says: "I added a new `calculate_discount()` function, what
tests do I need?" The agent recommends:

- `test_calculate_discount_percentage_applied_correctly`
- `test_calculate_discount_zero_amount_returns_zero`
- `test_calculate_discount_negative_amount_returns_error`
- `test_calculate_discount_exceeds_max_caps_at_limit`

One happy path, one boundary, one error case, one limit case.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Testing implementation details (private methods, internal state).** Tests that verify internals couple the test suite to the implementation — any structural change breaks tests even when behavior is correct. Test through the public API at the appropriate level.
- **Slow unit tests that hit disk or network.** A unit test that opens a file, makes an HTTP call, or connects to a database is an integration test with a misleading name. It will be slow and flaky in CI. Mock or stub at the boundary; keep unit tests pure.
- **E2E tests for every code path.** E2E tests are expensive to write, slow to run, and flaky to maintain. They should cover the critical user journeys that must work — not every conditional branch. Those belong in unit and integration tests.
- **Coverage goals as the only quality metric.** 100% line coverage with assertions that never fail (e.g. `assert True`) provides zero confidence. Coverage is a smoke alarm — it tells you what was executed, not whether the behavior was verified.
- **No test for the unhappy path.** Error cases and edge cases are where bugs live. Happy-path-only tests give false confidence. For every behavior, consider: what happens with empty input, invalid input, a missing dependency, a limit exceeded?
- **Deviating from the testing pyramid without understanding the trade-offs.** If the test suite is inverted (mostly E2E) or trophy-shaped (mostly integration), that may be appropriate for the architecture — but it should be a deliberate decision with documented trade-offs, not an accident.
- **Tests that pass when the production code is deleted.** A test suite that stays green after the module is removed was not testing anything. Run a mutation test or delete the module temporarily to verify the suite would actually catch a regression.

## Full reference

### Choosing the right level

| Question | Right test level |
|---|---|
| Does this pure function compute the right value? | Unit |
| Does this query return the right rows from the DB? | Integration |
| Does this HTTP endpoint persist and respond correctly? | Integration |
| Can a user complete checkout end-to-end? | E2E |
| Does this private helper handle edge case X? | Test through the public API, not directly |

### When to break the pyramid

- **Inverted pyramid (lots of E2E):** the system has weak
  internal seams; integration tests would be more valuable but
  the architecture makes them hard. Refactor for testability.
- **Trophy shape (heavy integration):** typical for thin services
  where most logic is glue between external systems; the unit
  layer is genuinely small.
- **Diamond (heavy integration, light unit and E2E):** common for
  microservices that mostly orchestrate.

The pyramid is a default, not a law.

### Anti-patterns

- Testing implementation details (e.g., asserting on private
  method calls) — couples tests to refactoring
- Slow unit tests because they hit the disk or network
- E2E tests that exercise every code path instead of critical
  flows
- Coverage goals as the only quality metric
- Tests that pass when the production code is deleted (they
  weren't testing anything)
- Skipping tests for "trivial" code that turns out to be wrong
