# Development Skills

Skills for code quality, testing, debugging, and development workflows.

---

### code-review

> Guides structured code review with a checklist approach. Use when reviewing PRs, diffs, or code changes before merging.

**Triggers:** When the user asks to review code, a PR, a diff, or says "review this", "check my changes", or "is this ready to merge".
**Tools:** None
**References:** None

Key capabilities:

- Evaluate changes against a structured checklist: correctness, clarity, tests, security, performance, style
- Categorize findings as must fix (bugs, security), should fix (naming, missing tests), or nit (style preferences)
- Provide specific line references and concrete suggestions
- Acknowledge what was done well

??? example "Example usage"
    User says "Review my changes." The agent reads the diff and provides structured feedback: **Must fix:** `parse_input()` on line 42 doesn't handle empty strings. **Should fix:** Variable `x` on line 15 could be renamed to `retry_count`. **Nit:** Consider extracting lines 30-45 into a helper function. Overall: Good separation of concerns.

---

### testing-strategy

> Advises on testing approach -- when to unit test vs integration test, coverage goals, and test organization. Use when writing tests or planning test strategy.

**Triggers:** When the user asks "should I test this?", "how should I test this?", "what tests do I need?", or is writing new functionality and needs testing guidance.
**Tools:** None
**References:** None

Key capabilities:

- Identify code type and map to test type: pure logic (unit), integration points (integration), user workflows (E2E)
- Apply the testing pyramid: many unit tests, fewer integration tests, minimal E2E tests
- Test naming convention: `test_<function>_<scenario>_<expected_result>`
- AAA pattern: Arrange, Act, Assert
- Prioritize error paths and edge cases over happy paths, public API over internal implementation
- Coverage goal: 80% on business logic, don't chase 100% everywhere

??? example "Example usage"
    User says "I added a new `calculate_discount()` function, what tests do I need?" The agent recommends unit tests for percentage applied correctly, zero amount returns zero, negative amount returns error, and exceeds max caps at limit.

---

### refactoring

> Systematic code refactoring using Fowler's catalog, Gang of Four patterns, and code smell detection. Use when restructuring code without changing behavior.

**Triggers:** When the user asks to refactor, clean up, restructure code, fix code smells, apply design patterns, reduce duplication, or prepare code for a new feature.
**Tools:** None
**References:** `references/code-smells.md`, `references/gof-patterns.md`

Key capabilities:

- Verify safety net before refactoring: check for tests, plan small steps, commit after each step
- Identify code smells: Long Method, Large Class, Feature Envy, Data Clumps, Primitive Obsession, Shotgun Surgery, Duplicate Code, Dead Code
- Apply Fowler's refactoring patterns: Extract Function, Move Function, Replace Nested Conditional with Guard Clauses, Introduce Parameter Object, and more
- Apply GoF design patterns when they solve a concrete problem: Strategy, Observer, Factory Method, Builder, Adapter, Decorator, Command, State
- Modern refactoring for async/concurrent code and functional-style code
- Golden rule: never mix refactoring with feature changes in the same commit

??? example "Example usage"
    User says "This function is too long, clean it up." The agent identifies three responsibilities in the function, extracts each into a named helper with descriptive names, keeps the original as a high-level orchestrator, runs tests after each extraction, and commits each extraction separately.

---

### documentation

> Guides writing effective documentation -- READMEs, API docs, inline comments. Use when creating or improving project documentation.

**Triggers:** When the user asks to "write docs", "document this", "add a README", "improve comments", or when code lacks explanation for non-obvious behavior.
**Tools:** None
**References:** None

Key capabilities:

- Identify the audience: developers, end-users, or future maintainers
- README structure: what it is, quick start, usage examples, configuration, contributing guide
- API documentation: doc comments on every public function with purpose, parameters, return value, errors
- Inline comments: comment WHY not WHAT, document non-obvious business rules, explain workarounds with links to issues
- Keep docs close to code: doc comments over wiki pages over external docs

??? example "Example usage"
    User says "Document this function." The agent adds a doc comment explaining purpose, parameters, return type, and a short example showing typical usage.

---

### debugging

> Systematic debugging process -- reproduce, isolate, fix, verify. Use when tracking down bugs, unexpected behavior, or test failures.

**Triggers:** When the user reports a bug, error, unexpected behavior, or says "this doesn't work", "why is this failing?", or "help me debug".
**Tools:** None
**References:** None

Key capabilities:

- Reproduce: get exact steps, inputs, and error messages
- Read the error: parse full stack trace, focus on first or last frame
- Isolate: binary search through code, check recent changes with `git log`/`git diff`, add targeted logging
- Hypothesize and test: form a theory, test it, don't change multiple things at once
- Fix: apply the minimal change that resolves the issue
- Verify: confirm error is gone, run full test suite, test edge cases near the fix
- Document: add a test that would have caught this bug

??? example "Example usage"
    User says "My tests pass locally but fail in CI." The agent checks for environment differences: OS, dependency versions, file paths, timezone, locale settings, and race conditions in parallel tests. Compares CI logs with local output to identify the divergence point.

---

### git-workflow

> Git workflow conventions -- branch naming, commit messages, PR descriptions, merge strategies. Use when the user needs guidance on git practices.

**Triggers:** When the user asks about branch naming, commit message format, PR descriptions, merge strategy, or how to organize commits.
**Tools:** None
**References:** None

Key capabilities:

- Branch naming: `<type>/<issue>-<short-description>` (e.g., `feat/42-add-user-auth`)
- Commit messages following Conventional Commits: `<type>: <description>` in lowercase imperative mood
- PR descriptions: title under 70 characters, body with summary, test plan, breaking changes
- Merge strategy: squash merge for features, merge commit for long-lived branches, rebase to stay up to date
- General rules: commit early and often, never force-push to shared branches, keep commits atomic

??? example "Example usage"
    User asks "What should I name this branch for adding search?" The agent suggests `feat/30-add-search-functionality` (with issue number if one exists).

---

### tdd-workflow

> Test-Driven Development workflow with red-green-refactor cycle, test naming, and when to apply TDD. Use when writing tests first, practicing TDD, or choosing between TDD and test-after approaches.

**Triggers:** When the user asks to write code using TDD, practice the red-green-refactor cycle, decide between TDD and test-after, or understand test doubles and property-based testing.
**Tools:** `Bash`, `Read`, `Write`
**References:** None

Key capabilities:

- Strict red-green-refactor cycle: write failing test, write minimum code to pass, refactor while green
- When to use TDD vs test-after (well-understood rules vs prototyping)
- Outside-in (London school) vs inside-out (Chicago school) approaches
- Test naming conventions: `test_<scenario>_<expected_outcome>`
- Test doubles: stubs, mocks, spies, fakes -- prefer fakes and stubs over mocks
- Property-based testing: invariants, round-trips, idempotence
- Mutation testing to measure test quality (`mutmut`, `cargo-mutants`, `Stryker`)

??? example "Example usage"
    User wants to implement FizzBuzz using TDD. The agent writes a failing test for "Fizz" on multiples of 3, implements the minimum code to pass, then adds the next failing test for "Buzz" on multiples of 5, continuing the red-green-refactor cycle.

---

### integration-testing

> Integration and E2E testing patterns including testcontainers, database fixtures, API mocking, and CI isolation. Use when writing integration tests, setting up test infrastructure, or debugging flaky tests.

**Triggers:** When the user asks to write integration tests, set up test containers, mock external APIs, debug flaky tests, or configure CI test isolation.
**Tools:** `Bash`, `Read`, `Write`
**References:** `references/test-fixtures.md`

Key capabilities:

- Testcontainers: spin up real databases and services as disposable containers per test suite
- Database fixtures: factories, fixtures, seeds with cleanup via transaction rollback, truncate, or recreate
- API mocking with WireMock, MSW, nock, httpmock, or responses
- Snapshot testing for serialized JSON, rendered HTML, CLI output
- E2E testing patterns: Page Object Model, test user journeys
- CI test isolation and parallelism: separate jobs, unique databases per worker, per-test timeouts
- Flaky test prevention: isolate shared state, replace sleep with wait-for, randomize test order

??? example "Example usage"
    User needs to test a service that talks to PostgreSQL. The agent sets up a testcontainers fixture that spins up a Postgres container, runs migrations, and yields the engine. Each test gets an isolated database that is torn down after the suite completes.

---

### error-handling

> Error handling patterns across languages including Result types, exceptions, retry strategies, and circuit breakers. Use when designing error handling, reviewing error-prone code, or implementing resilience patterns.

**Triggers:** When the user asks to design error handling, choose between Result types and exceptions, implement retry logic or circuit breakers, or review code for missing error handling.
**Tools:** None
**References:** None

Key capabilities:

- Result/Option types across languages: Rust (`Result<T, E>`), TypeScript (discriminated unions), Python (exceptions), Java (checked vs unchecked)
- Error hierarchies in layers: domain, application, infrastructure
- User-facing vs internal errors: never leak stack traces or SQL queries to end users
- Structured error codes with namespacing: `PAYMENT.DECLINED`, `AUTH.TOKEN_EXPIRED`
- Consistent API error response shape with code, message, details, and request_id
- Retry strategies: exponential backoff with jitter, never retry 4xx client errors
- Circuit breaker pattern: closed, open, half-open states to prevent cascading failures
- Error reporting with structured logging and error tracker integration

??? example "Example usage"
    User needs error handling for a payment service in Rust. The agent defines a domain error enum with `thiserror` including variants for `InsufficientFunds`, `CardDeclined`, and `ProviderUnavailable`, with structured context in each variant and `?` propagation throughout the call chain.

---

### dependency-management

> Cross-language dependency management including lockfiles, version pinning, update strategies, and license compliance. Use when managing project dependencies, reviewing dependency changes, or setting up automated updates.

**Triggers:** When the user asks to set up lockfile strategy, choose version pinning, configure Dependabot or Renovate, manage monorepo dependencies, audit for security or license issues, or decide whether to vendor.
**Tools:** `Bash`, `Read`, `Write`
**References:** None

Key capabilities:

- Lockfile best practices: always commit for applications, regenerate with package manager
- Version pinning strategies: exact pins for apps, compatible ranges for libraries
- Automated updates with Dependabot and Renovate: configuration, grouping, auto-merge for patches
- Monorepo dependency management with workspace features
- Security scanning: `cargo audit`, `npm audit`, `pip-audit`, `govulncheck`
- License compliance: SPDX identifiers, allow-lists, flagging copyleft in proprietary projects
- Vendoring for air-gapped environments or supply-chain control
- Dependency review process for every PR that changes dependencies

??? example "Example usage"
    User wants to set up automated dependency updates. The agent creates a `.github/dependabot.yml` configuration with ecosystem, weekly schedule, PR limits, reviewers, and grouping for minor/patch updates. Adds a `deny.toml` for license and advisory checks.

---

### code-generation

> Code generation patterns including template engines, AST manipulation, scaffolding, and build-step generation. Use when creating code generators, scaffolding tools, or deciding between generation and abstraction.

**Triggers:** When building or extending a code generator, choosing between template engines, deciding whether to generate code or write an abstraction, adding codegen as a build step, or implementing AST-based transformations.
**Tools:** `Bash`, `Read`, `Write`
**References:** None

Key capabilities:

- Generate vs abstract decision framework: generate for multi-language or mechanical output, abstract when a shared library suffices
- Template engine comparison: Jinja2, Handlebars, EJS, Tera, Go templates
- Template best practices: keep logic-free, use inheritance/blocks, emit "DO NOT EDIT" headers
- AST manipulation with language-specific parsers (`syn`, `ast`, `ts-morph`)
- Macro systems: prefer declarative over procedural, document expansion
- Scaffolding tools: cookiecutter, yeoman, `cargo-generate` with compile-and-run-immediately templates
- Build-step generation: store source-of-truth in VCS, fail build on drift, pin generator version
- Keeping generated code in sync: header comments, `.generated.` filenames, single regenerate command

??? example "Example usage"
    User needs to rename `log_event()` to `emit_event()` across 200 files but find-replace hits false positives in strings and comments. The agent writes a ts-morph codemod that parses each file, finds call expressions where the callee is `log_event` (skipping strings/comments), renames to `emit_event`, and writes back preserving formatting.
