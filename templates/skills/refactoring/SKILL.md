---
name: refactoring
description: Guides safe refactoring with proven patterns. Use when restructuring code without changing behavior — extract function, rename, simplify.
---

# Refactoring

## When to Use

When the user says "refactor this", "clean this up", "this is messy", "simplify", or when code smells are identified during review.

## Instructions

1. **Before refactoring:**
   - Confirm existing tests pass (or write them first)
   - Identify the specific smell: duplication, long function, unclear naming, deep nesting
   - Plan the refactoring as a series of small steps
2. **Apply the appropriate pattern:**
   - **Extract function:** Pull out a block with a descriptive name
   - **Rename:** Make names reveal intent (variables, functions, types)
   - **Simplify conditionals:** Replace nested if/else with early returns or guard clauses
   - **Remove duplication:** Extract shared logic into a common function
   - **Inline:** Remove unnecessary indirection (single-use helper functions)
   - **Split module:** Break large files along responsibility boundaries
3. **After each step:**
   - Verify tests still pass
   - Commit the refactoring separately from behavior changes
4. **Never mix refactoring with feature changes** in the same commit

## Examples

**User:** "This function is too long, clean it up"
**Agent:** Identifies three responsibilities in the function, extracts each into a named helper, keeping the original function as a high-level orchestrator. Runs tests after each extraction.

**User:** "Simplify these nested ifs"
**Agent:** Converts nested conditions to early-return guard clauses, reducing indentation depth from 4 to 1.
