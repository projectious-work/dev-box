---
name: testing-strategy
description: Advises on testing approach — when to unit test vs integration test, coverage goals, and test organization. Use when writing tests or planning test strategy.
---

# Testing Strategy

## When to Use

When the user asks "should I test this?", "how should I test this?", "what tests do I need?", or is writing new functionality and needs testing guidance.

## Instructions

1. Identify the type of code being tested:
   - **Pure logic** (calculations, transformations) → Unit tests
   - **Integration points** (database, API, file I/O) → Integration tests
   - **User workflows** (end-to-end paths) → E2E tests
2. Apply the testing pyramid:
   - Many unit tests (fast, isolated, cheap)
   - Fewer integration tests (slower, test boundaries)
   - Minimal E2E tests (slowest, test critical paths only)
3. Test naming convention: `test_<function>_<scenario>_<expected_result>`
4. Each test should have: Arrange, Act, Assert (AAA pattern)
5. Prioritize testing:
   - Error paths and edge cases over happy paths
   - Public API over internal implementation
   - Complex logic over trivial getters/setters
6. Coverage goal: aim for 80% on business logic, don't chase 100% everywhere

## Examples

**User:** "I added a new `calculate_discount()` function, what tests do I need?"
**Agent:**
- Unit test: `test_calculate_discount_percentage_applied_correctly`
- Unit test: `test_calculate_discount_zero_amount_returns_zero`
- Unit test: `test_calculate_discount_negative_amount_returns_error`
- Unit test: `test_calculate_discount_exceeds_max_caps_at_limit`
