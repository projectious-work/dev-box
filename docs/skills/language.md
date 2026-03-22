# Language Skills

Language-specific conventions, patterns, and best practices.

---

### python-best-practices

> Python conventions and patterns -- typing, testing, project layout, tooling. Use when writing or reviewing Python code.

**Triggers:** When the user is working with Python code and asks about conventions, project structure, typing, testing, or says "how should I structure this Python project?".
**Tools:** None
**References:** None

Key capabilities:

- Project layout: `src/` layout with `pyproject.toml`, use `uv` for dependency management
- Type hints on all public function signatures with `from __future__ import annotations`
- Testing with pytest: fixtures, parametrize, test naming `test_<function>_<scenario>_<expected>`
- Code style: `ruff format` and `ruff check`, prefer dataclasses/Pydantic over dicts, pathlib over os.path
- Error handling: raise specific exceptions, custom exception classes, never bare `except:`

??? example "Example usage"
    User asks to set up a new Python project. The agent creates `pyproject.toml` with project metadata and dependencies, `src/` layout, `tests/` directory, `ruff` config, and basic `__init__.py`.

---

### rust-conventions

> Rust patterns and conventions -- error handling, module structure, clippy compliance. Use when writing or reviewing Rust code.

**Triggers:** When the user is working with Rust code and asks about patterns, error handling, module organization, or says "how should I structure this in Rust?".
**Tools:** None
**References:** None

Key capabilities:

- Error handling: `anyhow::Result` for apps, `thiserror` for libraries, `.context()` on all `?` operations
- Module structure: one module per file, thin `main.rs`/`lib.rs`, group by domain
- Naming: `PascalCase` types, `snake_case` functions, `SCREAMING_SNAKE` constants, builder pattern
- Clippy compliance: build with `cargo clippy -- -D warnings`, prefer `&str` over `&String`
- Testing: unit tests in `#[cfg(test)] mod tests`, integration tests in `tests/`, descriptive `assert_eq!` messages

??? example "Example usage"
    User asks to add error handling to a function. The agent replaces `.unwrap()` calls with `?` and `.context()`, changes return type to `anyhow::Result<T>`, and adds meaningful error messages that help diagnose failures.

---

### typescript-patterns

> TypeScript project patterns -- strict mode, type safety, project setup. Use when writing or reviewing TypeScript code.

**Triggers:** When the user is working with TypeScript and asks about project setup, type patterns, strict mode, or says "how should I type this?".
**Tools:** None
**References:** None

Key capabilities:

- Project setup: strict mode in `tsconfig.json`, `noUncheckedIndexedAccess`, committed lockfile
- Type safety: avoid `any`, use `unknown` with type guards, explicit return types on public functions
- Discriminated unions for state modeling, `as const` for literal types, `satisfies` operator
- Use `zod` or similar for runtime validation of external data
- Error handling: custom error classes, Result types in library code, validate all external inputs
- Testing with vitest or jest, type-level testing with `expectTypeOf`

??? example "Example usage"
    User asks "How should I handle API responses?" The agent defines a response type with zod schema, validates the response at the boundary, and uses discriminated unions for success/error handling downstream.

---

### go-conventions

> Go idioms and conventions including error handling, interfaces, goroutine patterns, and testing. Use when writing Go code, reviewing Go projects, or designing Go package layouts.

**Triggers:** When the user is working with Go code and asks about idiomatic patterns, error handling, concurrency, package organization, or testing strategies.
**Tools:** `Bash(go:*)`, `Read`, `Write`
**References:** `references/go-patterns.md`

Key capabilities:

- Error handling: return errors as last value, wrap with `fmt.Errorf` and `%w`, use `errors.Is`/`errors.As`
- Interfaces: accept interfaces, return concrete types, keep interfaces small (1-3 methods), define at consumption site
- Goroutine patterns: `context.Context` as first parameter, `errgroup.Group` for fan-out/fan-in, clear lifecycle ownership
- Package layout: organize by domain, avoid `util`/`common` packages, `internal/` for private packages
- Testing: table-driven tests, `testify/assert`, `httptest`, `t.Helper()`, `go test -race`
- Go proverbs: share memory by communicating, clear is better than clever, make the zero value useful
- Code style: follow `gofmt` unconditionally, group imports, exported names get doc comments

??? example "Example usage"
    User needs a worker pool that processes jobs from a channel. The agent creates a pool using `errgroup.Group` with configurable workers, each reading from a shared job channel. Uses `context.Context` for cancellation and returns the first error encountered, with graceful shutdown draining remaining jobs.

---

### java-patterns

> Modern Java 17+ patterns including records, sealed classes, Stream API, and Spring Boot conventions. Use when writing Java code, reviewing Java projects, or modernizing legacy Java.

**Triggers:** When the user is working with Java code and asks about modern language features, Spring Boot conventions, Stream API patterns, or says "how should I modernize this Java code?".
**Tools:** `Bash(mvn:*)`, `Bash(gradle:*)`, `Read`, `Write`
**References:** None

Key capabilities:

- Modern Java 17+ features: records, sealed classes, pattern matching with `instanceof`, switch expressions, text blocks
- Records and sealed classes for algebraic data types with exhaustive switch handling
- Stream API: `filter` -> `map` -> `collect` pipelines, `groupingBy`, `flatMap`, proper Optional usage
- Spring Boot: constructor injection, thin controllers, `@Transactional` at service layer, `@ConfigurationProperties`, `@RestControllerAdvice`
- Dependency injection: prefer constructor injection, use interfaces for contracts, avoid circular dependencies
- Testing with JUnit 5 and Mockito: `@ParameterizedTest`, AssertJ assertions, test slices (`@WebMvcTest`, `@DataJpaTest`)
- Code organization: package by feature, single responsibility, prefer composition over inheritance

??? example "Example usage"
    User says "Convert this class with getters/setters to modern Java." The agent replaces the POJO with a `record`, removes boilerplate methods, adds a compact constructor for validation, and updates all call sites to use the record's accessor methods.

---

### sql-style-guide

> SQL formatting and naming conventions for tables, columns, queries, migrations, and constraints. Use when writing SQL, reviewing database code, or establishing SQL style guidelines.

**Triggers:** When the user is writing SQL queries, designing schemas, creating migrations, or asks "how should I format this SQL?" or "what naming convention should I use for tables?".
**Tools:** None
**References:** None

Key capabilities:

- Table and column naming: `snake_case`, singular table names, `is_`/`has_` for booleans, `_at` for timestamps
- Keyword capitalization: SQL keywords in UPPERCASE, identifiers in lowercase
- Query formatting: one clause per line, leading commas, explicit `JOIN` syntax, meaningful table aliases
- Comment conventions: `--` for single-line, explain WHY not WHAT
- Migration file naming: sequential timestamps, one structural change per migration, include both `up` and `down`
- Constraint naming: `pk_`, `fk_`, `uq_`, `ck_`, `ix_` prefixes with table and column names
- Query best practices: `WHERE EXISTS` over `WHERE IN`, CTEs for complex queries, avoid `SELECT *`

??? example "Example usage"
    User asks to create a schema for a task management app. The agent designs tables with singular names (`task`, `project`, `user`), snake_case columns, explicit constraint names, timestamp columns with `_at` suffix, and boolean columns with `is_` prefix.

---

### latex-authoring

> Comprehensive LaTeX document authoring with LuaLaTeX, modern packages, math, TikZ, and bibliography management. Use when writing or editing LaTeX documents.

**Triggers:** When the user asks to write or edit LaTeX documents, set up document classes and preambles, create math equations or TikZ diagrams, or manage bibliographies.
**Tools:** None
**References:** `references/packages.md`, `references/math-reference.md`, `references/tikz-reference.md`

Key capabilities:

- Document classes: `article`, `book`, `report`, `beamer`, `standalone`, and when to use each
- LuaLaTeX vs pdfLaTeX: prefer LuaLaTeX for new projects (Unicode, system fonts, Lua scripting)
- Essential packages: `geometry`, `fontspec`, `amsmath`, `siunitx`, `tabularray`, `tikz`, `biblatex`, `tcolorbox`
- Document structure: one sentence per line, split with `\input{}`, preamble in separate file
- Bibliography with BibLaTeX and Biber backend
- Math typesetting: inline, display, multi-line environments, custom commands, SI units with `siunitx`
- TikZ for programmatic vector graphics with common libraries
- Common mistakes to avoid: `$$...$$` for display math, missing `\label` after `\caption`

??? example "Example usage"
    User asks to set up a LaTeX paper with LuaLaTeX. The agent creates a `main.tex` with `\documentclass{article}`, a `preamble.tex` loading geometry, fontspec, amsmath, biblatex, and siunitx, sets up section structure with `\input{}`, and provides a `latexmk` build command.
