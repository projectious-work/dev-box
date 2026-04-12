---
name: tdd-workflow
description: |
  Test-driven development workflow — red-green-refactor, naming, doubles, when to apply. Use when writing code with TDD, practicing the red-green-refactor cycle, deciding between TDD and test-after, naming tests, choosing test doubles, or exploring property-based and mutation testing.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-tdd-workflow
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
    layer: 3
---

# TDD Workflow

## Intro

Test-driven development means writing a failing test before any
production code, then making it pass with the smallest possible
change, then refactoring with the test as a safety net. The cycle
is short — a few minutes — and the discipline is what produces the
design benefit, not the tests themselves.

## Overview

### Red-green-refactor

Follow the cycle strictly in order:

1. **Red** — write a failing test that describes the desired
   behavior. Run it. Confirm it fails for the right reason, not a
   typo or missing import.
2. **Green** — write the minimum code to make the test pass. Do
   not optimize, do not handle edge cases yet. The goal is a green
   bar as fast as possible.
3. **Refactor** — clean up production and test code. Remove
   duplication, improve naming, extract helpers. All tests must
   stay green throughout.

Rules of thumb: never write production code without a failing
test; each cycle should take 2–10 minutes (longer means the step
is too big); commit after each green-refactor pair so rollback is
cheap.

### When to use TDD vs test-after

**Use TDD when:**

- Implementing well-understood business rules or algorithms
- Building APIs whose contract is known upfront
- Fixing bugs (write the failing reproduction test first)
- Working with pure functions or isolated modules

**Use test-after when:**

- Prototyping or exploring an unfamiliar problem space
- Writing integration tests against third-party systems
- The design is highly uncertain and likely to change completely
- Working on UI layout where visual feedback is the primary signal

### Outside-in vs inside-out

**Outside-in (London school):** start at the outermost layer (HTTP
handler, CLI entry point), use test doubles for dependencies,
build inward defining interfaces as you go. Best for layered
architectures.

**Inside-out (Chicago school):** start at the core domain logic
with no dependencies, build outward composing tested units, use
real objects where possible. Best for algorithmic or data-heavy
code.

### Test naming

Names should describe behavior, not implementation:

- `test_empty_cart_has_zero_total` — good, describes behavior.
- `test_calculate` — bad, says nothing about expectation.
- `should_reject_negative_quantities` — good, states the rule.
- Pattern: `test_<scenario>_<expected_outcome>` or
  `should_<expected_behavior>_when_<condition>`.

### Test doubles

| Double | Purpose | When to use |
|---|---|---|
| Stub | Returns canned data | Replace a dependency that provides input |
| Mock | Verifies interactions | Assert a method was called with specific args |
| Spy | Records calls for later assertion | Need real behavior + verification |
| Fake | Working implementation (e.g., in-memory DB) | When stubs become too complex |

Prefer fakes and stubs over mocks. Excessive mocking couples tests
to implementation and makes refactoring painful.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Writing the test after the code "to satisfy TDD".** Retrofitting tests to code that already exists does not produce the design benefit TDD promises. The design benefit comes from the constraint of writing a failing test first, which forces the interface to be specified before the implementation.
- **Red-green cycles that take 30+ minutes.** A long cycle means the behavior being specified is too large. Split the failing test into smaller behaviors; each cycle should describe a single, narrow behavior change.
- **Skipping the refactor step because "tests pass".** The refactor step is where the design payoff from TDD lives. Skipping it means the code works but accumulates duplication and poor naming with each cycle.
- **Excessive mocking.** Mocks that replace internal collaborators (not external boundaries) couple tests to implementation details. When the implementation changes, the mocks break even if the behavior did not. Prefer fakes and stubs over mocks; prefer real objects over fakes when feasible.
- **Naming tests after functions rather than scenarios.** `test_calculate` tells you nothing when it fails. `test_empty_cart_has_zero_total` identifies the broken behavior immediately. Name every test with `test_<scenario>_<expected_outcome>`.
- **Chasing 100% coverage instead of meaningful behavior coverage.** The last few percentage points of coverage typically cover trivial code, generated code, or defensive paths that cost more to test than they save. Target meaningful branch coverage of business logic, not total line coverage.
- **Testing implementation details instead of observable behavior.** Tests that assert on private method calls or internal state prevent refactoring — any structural change breaks the tests even when behavior is preserved. Test through the public API.

## Full reference

### Property-based testing

Instead of testing specific examples, define properties that must
always hold:

- **Invariants:** `sort(xs).length == xs.length`
- **Round-trips:** `decode(encode(x)) == x`
- **Idempotence:** `f(f(x)) == f(x)`

Use property-based testing alongside example-based tests, not as a
replacement. Examples are good documentation; properties are good
coverage.

### Mutation testing

Mutation testing measures test quality by introducing small
changes (mutations) to production code and checking whether tests
catch them. A killed mutant means a test failed (good); a survived
mutant means a coverage gap. Tools: `mutmut` (Python),
`cargo-mutants` (Rust), `Stryker` (JS/TS). A mutation score above
~80% indicates tests verify behavior rather than just executing
code.

### Example: full red-green-refactor in Python

```python
# RED: Write failing test
def test_fizzbuzz_returns_fizz_for_multiples_of_3():
    assert fizzbuzz(3) == "Fizz"
    assert fizzbuzz(9) == "Fizz"

# Run: pytest => NameError: name 'fizzbuzz' is not defined (FAILS)

# GREEN: Minimal implementation
def fizzbuzz(n):
    if n % 3 == 0:
        return "Fizz"
    return str(n)

# Run: pytest => PASS

# REFACTOR: Add next rule via new failing test
def test_fizzbuzz_returns_buzz_for_multiples_of_5():
    assert fizzbuzz(5) == "Buzz"

# Run: pytest => AssertionError (next RED phase)
```

### Example: outside-in with stubs (TypeScript)

```typescript
// Start from the handler, stub the repository
describe("CreateOrder", () => {
  it("should persist order and return confirmation", async () => {
    const repo = { save: vi.fn().mockResolvedValue({ id: "ord-1" }) };
    const handler = new CreateOrderHandler(repo);

    const result = await handler.execute({ item: "widget", qty: 2 });

    expect(result.id).toBe("ord-1");
    expect(repo.save).toHaveBeenCalledWith(
      expect.objectContaining({ item: "widget", qty: 2 })
    );
  });
});
```

### Example: property-based test (Rust)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn reverse_twice_is_identity(ref v in prop::collection::vec(any::<i32>(), 0..100)) {
        let mut reversed = v.clone();
        reversed.reverse();
        reversed.reverse();
        assert_eq!(v, &reversed);
    }

    #[test]
    fn sort_preserves_length(ref v in prop::collection::vec(any::<i32>(), 0..100)) {
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(v.len(), sorted.len());
    }
}
```

### Anti-patterns

- Writing the test after the code "to satisfy TDD" — that is
  test-after with extra steps
- Cycles that take 30+ minutes — the step is too big
- Tests that mock the unit under test
- Naming tests after the function rather than the scenario
- Chasing 100% coverage instead of meaningful coverage of branches
- Skipping the refactor step because "tests pass"
