---
name: refactoring
description: |
  Systematic refactoring with Fowler's catalog, GoF patterns, and code smell detection. Use when restructuring code without changing behavior — cleaning up smells, applying patterns, reducing duplication, or preparing a module for an upcoming feature.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-refactoring
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
    layer: 3
---

# Refactoring

## Intro

Refactoring is changing the structure of code without changing its
observable behavior. Do it in small, test-protected steps, one
transformation at a time, and never mix it with feature changes in
the same commit.

## Overview

### Verify the safety net first

Before any refactoring, run the existing tests and confirm they
pass. If no tests cover the code you are about to change, write
characterization tests first — tests that capture current behavior,
not desired behavior. Plan the refactoring as a sequence of atomic
transformations where each step keeps the suite green, and commit
after each step. The golden rule: never mix refactoring with feature
changes in the same commit.

### Identify code smells

Scan systematically for the well-known smells. The full catalog
lives in `references/code-smells.md`; the most common offenders:

| Smell | What to look for |
|---|---|
| Long Method | Function > 20 lines or doing more than one thing |
| Large Class | Class with > 5 responsibilities or > 300 lines |
| Long Parameter List | Function with > 3 parameters |
| Feature Envy | Method uses another object's data more than its own |
| Data Clumps | Same group of variables appears together repeatedly |
| Primitive Obsession | Strings/ints where a domain type belongs |
| Shotgun Surgery | One change requires edits in many unrelated files |
| Divergent Change | One class changed for multiple unrelated reasons |
| Duplicate Code | Same logic in two or more places |
| Dead Code | Unreachable or unused code |

### Apply Fowler's refactoring catalog

Match smells to specific refactorings:

- **Composing methods** — Extract Function, Inline Function, Extract
  Variable, Replace Temp with Query.
- **Moving features** — Move Function/Field, Extract Class, Inline
  Class.
- **Simplifying conditionals** — Replace Nested Conditional with
  Guard Clauses, Decompose Conditional, Replace Conditional with
  Polymorphism, Introduce Null Object.
- **Organizing data** — Introduce Parameter Object, Replace Magic
  Number with Constant, Encapsulate Collection.
- **Generalization** — Pull Up / Push Down Method, Extract
  Interface/Trait, Replace Inheritance with Composition.

### Apply GoF patterns when they simplify

Only introduce a design pattern when it solves a concrete problem,
never speculatively. The full catalog is in
`references/gof-patterns.md`. Most commonly useful: Strategy,
Observer, Factory Method, Builder, Adapter, Decorator, Command,
State, Template Method, Iterator. Warning signs of overuse: adding
a pattern "just in case", pattern adds more code than it removes,
only one implementation exists, the team cannot explain why.

### Example

User says "this function is too long". The agent identifies three
distinct responsibilities, extracts each into a named helper
keeping the original as a high-level orchestrator, runs tests after
each extraction, and commits each extraction separately.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Mixing refactoring with feature changes in the same commit.** When a commit changes both structure and behavior, reviewers cannot verify that behavior was preserved, and rollback becomes surgical. Keep refactoring commits separate; the commit message should start with `refactor:`.
- **Starting without a test safety net.** If the code has no tests, changing its structure with no verification is guessing. Write characterization tests first — tests that capture current behavior, not desired behavior — then refactor.
- **Big-bang rewrite instead of incremental transformation.** Full rewrites take longer, introduce more bugs, and stall new features during the rewrite. Refactor incrementally: one smell, one transformation, one commit, tests green throughout.
- **Removing code assumed to be dead without verifying.** Dynamic dispatch, reflection, plugin systems, and config-driven loads can call code that static analysis says is unreachable. Search for all invocation paths before deleting anything.
- **Speculative generality.** Adding extension points, plugin hooks, or configuration flags "just in case" is not refactoring — it is adding untested code for hypothetical future requirements. Refactor to the current need, not the imagined future.
- **Pattern cargo-culting.** Introducing a design pattern because it sounds clever adds indirection and complexity without value. Apply a pattern only when it solves a concrete, present problem — and only after the "rule of three" (three similar occurrences) is met.
- **Skipping the refactor step after green.** The refactor step is where the design payoff from TDD-style cycles lives. Skipping it means the test suite is green but the technical debt is accumulating.

## Full reference

### Modern considerations

For async/concurrent code, extract async boundaries clearly (sync
core, async shell), replace callback chains with async/await where
possible, and isolate side effects at the edges. For
functional-style code, replace mutable loops with map/filter/reduce
pipelines, extract pure functions from impure ones, and use
algebraic types (enums/unions) over class hierarchies for closed
sets.

### Smell-to-pattern map

| Smell / situation | Candidate pattern |
|---|---|
| Complex conditional logic on types | Strategy, State, polymorphism |
| Growing switch/match statements | Factory Method + polymorphism |
| Duplicated algorithm with variations | Template Method |
| Complex object construction | Builder |
| Third-party integration coupling | Adapter |
| Need to add behavior to existing code | Decorator |
| Multiple objects reacting to events | Observer |
| Complex state-dependent behavior | State |
| Request needs undo/queue/log | Command |
| Many interacting objects | Mediator |

### Selecting a pattern (rule of three)

Wait until you see a need in three places before abstracting. One
occurrence is a fact, two is a coincidence, three is a pattern.
Premature abstraction is harder to undo than late abstraction
because the wrong shape constrains future changes.

### Anti-patterns

- Mixing refactoring with feature work in the same commit
- Refactoring without a test safety net
- Speculative generality — adding hooks "just in case"
- Pattern cargo-culting — applying GoF because it sounds clever
- Big-bang rewrites instead of incremental transformation
- Removing dead code without checking it is truly unreachable
  (search for dynamic dispatch, reflection, config-driven loads)

### Example: extracting test seams

To make code more testable, identify concrete dependencies (file
I/O, HTTP, database), extract interfaces/traits at those
boundaries (Adapter pattern), move domain logic into pure
functions, and introduce dependency injection so tests can supply
fakes. This is refactoring in service of testability, not feature
work, and should ship in its own commit series.
