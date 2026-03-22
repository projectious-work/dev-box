---
name: typescript-patterns
description: TypeScript project patterns — strict mode, type safety, project setup. Use when writing or reviewing TypeScript code.
---

# TypeScript Patterns

## When to Use

When the user is working with TypeScript and asks about project setup, type patterns, strict mode, or says "how should I type this?" or "set up a TypeScript project".

## Instructions

1. **Project setup:**
   - Enable strict mode in `tsconfig.json`: `"strict": true`
   - Use `"noUncheckedIndexedAccess": true` for safer array/object access
   - Prefer `pnpm` or `npm` with lockfile committed
2. **Type safety:**
   - Avoid `any` — use `unknown` and narrow with type guards
   - Define explicit return types on public functions
   - Use discriminated unions for state modeling:
     ```typescript
     type Result<T> = { ok: true; value: T } | { ok: false; error: string }
     ```
   - Prefer `interface` for object shapes, `type` for unions and intersections
3. **Common patterns:**
   - Use `as const` for literal types and exhaustive switches
   - Use `satisfies` operator for type checking without widening
   - Prefer `Map<K,V>` over `Record<string, V>` for dynamic keys
   - Use `zod` or similar for runtime validation of external data
4. **Error handling:**
   - Define custom error classes extending `Error`
   - Use Result types instead of throwing in library code
   - Always validate external inputs (API responses, user input)
5. **Testing:** Use vitest or jest with `ts-jest`. Test types with `expectTypeOf`.

## Examples

**User:** "How should I handle API responses?"
**Agent:** Defines a response type with zod schema, validates the response at the boundary, and uses discriminated unions for success/error handling downstream.
