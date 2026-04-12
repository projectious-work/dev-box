---
name: rust-conventions
description: |
  Rust conventions — error handling with anyhow/thiserror, module layout, clippy. Use when writing or reviewing Rust code, organizing modules, or deciding on error handling, naming, and testing conventions.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-rust-conventions
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Rust Conventions

## Intro

Idiomatic Rust uses `anyhow` for applications and `thiserror` for
libraries, pays its `cargo clippy -- -D warnings` bill on every
build, and organizes modules by domain. No `.unwrap()` in
production code.

## Overview

### Error handling

Use `anyhow::Result` for application code where callers just want a
good error message, and `thiserror` for libraries where callers
need to pattern-match on error variants. Add `.context("...")` to
every `?` so the final error message is a breadcrumb trail. Never
`.unwrap()` in production code. `.expect("reason")` is acceptable
only for invariants that cannot fail — the reason documents why.

### Module structure

One module per file. Re-export public items in `mod.rs` (or the
parent module) so callers don't depend on file layout. Keep
`main.rs` and `lib.rs` thin — they should wire things up and
delegate to modules. Group by domain (`commands/`, `config/`,
`output/`), not by type (`traits/`, `structs/`).

### Naming

- Types: `PascalCase`
- Functions and methods: `snake_case`
- Constants and statics: `SCREAMING_SNAKE_CASE`
- `new()` for simple constructors; `from_*()` for conversions;
  builder pattern for complex construction.

### Clippy compliance

Build with `cargo clippy -- -D warnings` in CI. Fix every warning
rather than suppressing it. When you do need `#[allow(...)]`, add
a comment explaining why. Prefer `&str` over `&String` and `&[T]`
over `&Vec<T>` in function parameters.

### Testing

Unit tests live in a `#[cfg(test)] mod tests` block at the bottom
of each source file. Integration tests go under `tests/`. Use
`assert_eq!` with a third argument for the failure message when
the context isn't obvious from the values alone.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **`.unwrap()` in production code.** `.unwrap()` panics on `None` or `Err`, producing an unrecoverable crash with no context. Use `?` to propagate the error or `.expect("reason")` only when the reason documents an invariant that cannot fail in production — and even then, think twice.
- **`.expect("")` with an empty message.** An empty `.expect("")` panics with no information about why the invariant should hold. The message should explain the assumption: `.expect("config always has a default value")`.
- **Returning `Box<dyn Error>` from library APIs.** A library that returns `Box<dyn Error>` forces callers to pattern-match on dynamic dispatch and prevents them from handling specific error variants. Use `thiserror`-derived enums so callers get typed error variants.
- **Using `anyhow::Error` at a library boundary.** `anyhow` is for applications where you want a good error message, not for library code where callers need to match on error type. Use `thiserror` in libraries, `anyhow` in binaries.
- **Holding a lock across an `.await` point.** A `MutexGuard` (from `std::sync`) held across an `await` causes the future to be `!Send`, breaking async runtimes. Use `tokio::sync::Mutex` for async code, or acquire and release the guard before the await.
- **Suppressing clippy warnings with `#[allow(...)]` without a comment.** Clippy warnings are almost always correct. When you do need to suppress one, the `#[allow(...)]` must be accompanied by a comment explaining why the warning doesn't apply in this case — otherwise future readers have no basis for trusting the suppression.
- **`&String` or `&Vec<T>` in function parameters.** `&str` is strictly more general than `&String` (any `&str` can be passed where `&String` is accepted, but not vice versa). Similarly `&[T]` is more general than `&Vec<T>`. Prefer the slice types in function signatures.

## Full reference

### Error handling patterns

```rust
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading config from {}", path.display()))?;
    toml::from_str(&raw)
        .with_context(|| format!("parsing config at {}", path.display()))
}
```

For libraries, define a typed error enum:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("reading {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("parsing {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
}
```

### Ownership and borrowing rules of thumb

- Prefer borrowed slices (`&str`, `&[T]`) in parameters; own in
  return types unless a lifetime is part of the contract.
- Avoid `Rc`/`Arc` until a measured need appears — usually a
  refactor can eliminate shared ownership.
- Reach for `Clone` before `Rc` when the clone is cheap and the
  code stays simpler.
- Use `Cow<'_, str>` for APIs that usually borrow but sometimes
  need to own.

### Anti-patterns

- `.unwrap()` or `.expect("")` (empty message) in non-test code.
- Returning `Box<dyn Error>` from libraries — use `thiserror`.
- Catch-all `anyhow::Error` at library boundaries.
- `#[allow(...)]` without a comment explaining why.
- Deep module trees re-exporting everything — flatten the public
  surface.
- Borrowing `&String` or `&Vec<T>` in function signatures.
- Holding locks across `.await` points.

### Testing conventions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_handles_kilobytes() {
        assert_eq!(parse_size("2KB").unwrap(), 2048);
    }
}
```

Put integration tests under `tests/`, one file per feature area.
Use `#[should_panic(expected = "...")]` for panic assertions and
`assert!(matches!(...))` for enum variants.
