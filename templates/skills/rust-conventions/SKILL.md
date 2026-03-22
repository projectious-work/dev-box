---
name: rust-conventions
description: Rust patterns and conventions — error handling, module structure, clippy compliance. Use when writing or reviewing Rust code.
---

# Rust Conventions

## When to Use

When the user is working with Rust code and asks about patterns, error handling, module organization, or says "how should I structure this in Rust?".

## Instructions

1. **Error handling:**
   - Use `anyhow::Result` for application code, `thiserror` for libraries
   - Add `.context("description")` to all `?` operations
   - Never `.unwrap()` in production code — use `.expect("reason")` only for invariants
2. **Module structure:**
   - One module per file, re-export public items in `mod.rs` or parent
   - Keep `main.rs` / `lib.rs` thin — delegate to modules
   - Group by domain (commands/, config/, output/) not by type
3. **Naming:**
   - Types: `PascalCase`, functions: `snake_case`, constants: `SCREAMING_SNAKE`
   - Builder pattern for complex construction
   - `new()` for simple constructors, `from_*()` for conversions
4. **Clippy compliance:**
   - Always build with `cargo clippy -- -D warnings`
   - Fix all warnings — don't `#[allow]` without a comment explaining why
   - Prefer `&str` over `&String`, `&[T]` over `&Vec<T>`
5. **Testing:**
   - Unit tests in `#[cfg(test)] mod tests` at bottom of each file
   - Integration tests in `tests/` directory
   - Use `assert_eq!` with descriptive messages

## Examples

**User:** "Add error handling to this function"
**Agent:** Replaces `.unwrap()` calls with `?` and `.context()`, changes return type to `anyhow::Result<T>`, adds meaningful error messages that help diagnose failures.
