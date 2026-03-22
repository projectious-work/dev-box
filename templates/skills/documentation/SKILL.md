---
name: documentation
description: Guides writing effective documentation — READMEs, API docs, inline comments. Use when creating or improving project documentation.
---

# Documentation

## When to Use

When the user asks to "write docs", "document this", "add a README", "improve comments", or when code lacks explanation for non-obvious behavior.

## Instructions

1. **Identify the audience:** developers, end-users, or future maintainers?
2. **README structure** (for projects):
   - What it is (one sentence)
   - Quick start (install + first command)
   - Usage examples
   - Configuration reference
   - Contributing guide (if open source)
3. **API documentation:**
   - Every public function gets a doc comment
   - Include: what it does, parameters, return value, errors/panics
   - Add a usage example for non-obvious APIs
4. **Inline comments — use sparingly:**
   - Comment WHY, not WHAT (the code shows what)
   - Document non-obvious business rules
   - Explain workarounds with links to issues
   - Never comment out code — delete it (git remembers)
5. **Keep docs close to code:** doc comments > wiki pages > external docs

## Examples

**User:** "Document this function"
**Agent:** Adds a doc comment explaining purpose, parameters, return type, and a short example:
```rust
/// Calculates the shipping cost based on weight and destination zone.
///
/// Returns `None` if the destination zone is not serviceable.
/// Weights over 30kg use freight pricing instead.
```
