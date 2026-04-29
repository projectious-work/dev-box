---
name: software-modularization
description: >
  Design and evaluate module and package boundaries for maintainability,
  low coupling, and AI agent context efficiency.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-software-modularization
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: engineering
---

# Software Modularization

## Intro

Modularization is the practice of decomposing software into units with
clear boundaries, stable interfaces, and minimal cross-cutting dependencies.
Good module design is what makes a codebase navigable by humans — and
loadable by AI agents without blowing the context window.

## Overview

### The two goals

Modularization serves two audiences who want the same thing for different
reasons:

- **Humans** — small modules are understandable, testable, and deployable
  independently. The cognitive load of a 500-line module is manageable;
  a 50,000-line monolith is not.
- **AI agents** — the agent's context window is finite. A module that an
  agent can fully load and reason about without truncation produces better
  results than one that must be partially loaded. Target modules that fit
  in ~8,000 tokens of context (roughly 500–800 lines of code + docs).

### Module boundary heuristics

Use these signals to find the right cut:

**High cohesion** — everything in the module exists for one reason.
If you need two sentences to describe what the module does, it probably
does two things.

**Low coupling** — the module's public API is small and stable. Count
the imports from other internal modules: more than 5–7 direct internal
dependencies is a warning sign.

**Independent deployability** — could you version, test, and deploy this
module on its own? If yes, the boundary is probably right. If it always
requires co-deployment with other modules, the boundary may be wrong.

**Domain alignment** — module boundaries should follow business domain
concepts, not technical layers. `user-auth` is a better boundary than
`controllers` or `services`. Layer-based decomposition produces modules
that change together for every feature, defeating the independence goal.

### Decomposition strategies

**Vertical slicing (by feature/domain)**
Each module owns the full stack for a domain: API, business logic, data
access. Used in DDD and microservices. Produces high cohesion, low coupling
across domains, and clear ownership.

```
src/
  auth/         ← owns everything about authentication
  billing/      ← owns everything about billing
  notifications/ ← owns everything about notifications
```

**Horizontal slicing (by technical layer)**
Modules are organized by their role in the architecture: API handlers,
domain logic, infrastructure adapters. Common in traditional layered
architectures. Produces coupling across features; harder to decompose later.

**Strangler fig (for existing monoliths)**
Do not decompose the whole monolith at once. Identify the highest-value
boundary, extract it, prove the pattern works, then repeat. One module
at a time. Each extraction should leave the monolith working.

### API surface design

The public API of a module is its contract with the rest of the system.
Design it deliberately:

- **Explicit exports** — prefer explicit export lists (`__all__` in Python,
  `pub use` in Rust, `index.ts` barrel in TypeScript) over implicit "everything
  is public by default."
- **Stable over convenient** — the API should reflect what the module
  guarantees to support, not every function that happens to be useful.
  Internal utilities leak into the API accidentally; fight this actively.
- **Narrow for agents** — a module an agent uses should expose the minimum
  necessary interface. An agent that can only call `create_user()`,
  `get_user()`, and `delete_user()` makes fewer mistakes than one that
  can call any of 47 internal functions.
- **Versioned changes** — if a module is consumed by other teams or
  external code, semver its API changes. A breaking change to an internal
  module is a refactor; a breaking change to a public API is a contract
  breach.

### Dependency hygiene

- **No circular dependencies** — if module A imports from module B and
  module B imports from module A, they are the same module. Extract the
  shared concern into a third module C that neither A nor B modifies.
- **Dependency direction** — dependencies should flow in one direction:
  domain ← application ← infrastructure (the dependency rule / clean
  architecture). Infrastructure modules should never be imported by domain
  modules.
- **Vendored vs. pinned** — external dependencies should be pinned to a
  specific version in a lockfile. "Latest" is not a version.
- **Audit surface** — fewer direct dependencies means fewer potential
  vulnerabilities. Each dependency is a trust decision; treat it as one.

## Gotchas

- **Decomposing by technical layer instead of domain.** A `controllers/`
  module that contains every HTTP handler, a `services/` module that contains
  every business rule, and a `repositories/` module that contains every
  database query means every feature change touches three modules. Domain
  decomposition (auth, billing, notifications) produces modules that change
  for one reason only.
- **Making the module boundary too fine.** A module per class or per function
  is not a module — it is a namespace. Modules should contain related
  concepts that evolve together. If you have 47 modules each containing one
  function, the coupling has moved from the code to the import graph, which
  is harder to see and harder to fix.
- **Ignoring context window size when setting module boundaries.** A module
  that an agent must load in pieces (because it exceeds the context window)
  will produce inconsistent reasoning across the pieces. If a module exceeds
  ~800 lines of code, check whether it can be decomposed. The agent-context
  constraint is a useful forcing function — a module too large for an agent
  is usually too large for a human too.
- **Leaking internal types through the public API.** If the public API of
  a module returns internal structs, database models, or implementation
  details, callers couple to the implementation, not the interface. A change
  to the internal type propagates to all callers. Map to API types at the
  boundary.
- **Creating a new module for every decomposition instead of using the
  strangler fig pattern.** Decomposing a monolith all at once produces a
  partially-decomposed system that works neither as a monolith nor as
  modules. Extract one boundary, prove it, then continue. The monolith
  should remain shippable throughout the decomposition.
- **Circular dependencies allowed to persist because "it's internal."**
  Circular dependencies are not a code style issue — they prevent
  independent testing, independent deployment, and independent reasoning by
  an agent. Detect them with a linter (e.g., `pylint --disable=all
  --enable=cyclic-import`, `madge` for JS/TS, `cargo` for Rust) and treat
  them as build failures.
- **Not updating AGENTS.md / context budget when modules are added.**
  A new module means new files an agent might need to load. Update the
  project's `AGENTS.md` lazy-load guidance when significant new modules are
  added so the agent knows where to find things without doing filesystem
  walks.

## Full reference

### Module health checklist

Before finalizing a module boundary:

- [ ] Can the module be described in one sentence?
- [ ] Does the public API surface fewer than 10 exported symbols?
- [ ] Are there fewer than 7 direct internal dependencies?
- [ ] Does the module fit in ~800 lines / ~8,000 tokens?
- [ ] Can it be tested independently without loading other modules?
- [ ] Is there a clear owner (team, person, or agent role)?
- [ ] Do all imports flow in the correct direction (no circular deps)?

### Detecting coupling problems

| Signal | Likely cause | Fix |
|---|---|---|
| Every feature change touches 5+ modules | Layer-based decomposition | Redecompose by domain |
| Module A and B always deployed together | Implicit coupling, missing abstraction | Extract shared interface |
| One module imports from 10+ others | "God module" antipattern | Split by responsibility |
| Tests require loading the whole system | No true isolation at module boundary | Introduce interface/port |
| Module exceeds 1,000 lines | Too broad scope | Decompose by subdomain |

### Context-window-aware design

For AI agent consumers, consider documenting module load cost in AGENTS.md:

```markdown
## Module load guide

| Module | Lines | Load tokens (est.) | When to load |
|---|---|---|---|
| auth/ | 320 | ~4,000 | auth-related tasks only |
| billing/ | 180 | ~2,200 | billing and subscription tasks |
| core/ | 90 | ~1,100 | always (foundation types) |
```

This lets agents lazy-load correctly without filesystem walks.
