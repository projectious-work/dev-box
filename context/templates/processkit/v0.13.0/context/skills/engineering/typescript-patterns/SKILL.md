---
name: typescript-patterns
description: |
  TypeScript patterns — strict mode, discriminated unions, runtime validation, Result types. Use when writing or reviewing TypeScript, setting up a project's tsconfig, or designing type-safe data flows.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-typescript-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# TypeScript Patterns

## Intro

TypeScript earns its keep in strict mode with no `any`.
Discriminated unions model state, `zod` validates external data at
the boundary, and Result types keep library code from throwing on
expected failures.

## Overview

### Project setup

Enable strict mode in `tsconfig.json`:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true
  }
}
```

`strict` turns on a family of checks including `noImplicitAny`,
`strictNullChecks`, and `strictFunctionTypes`.
`noUncheckedIndexedAccess` makes array and object index access
return `T | undefined`, which catches a whole class of bugs.
Commit your lockfile whether you use `pnpm`, `npm`, or `yarn`.

### Type safety

- Avoid `any`. Use `unknown` and narrow with type guards or `zod`.
- Give public functions explicit return types so inference
  changes can't silently reshape the API.
- Model state with discriminated unions:

```ts
type Result<T> =
  | { ok: true; value: T }
  | { ok: false; error: string }
```

- Prefer `interface` for object shapes (they can be extended and
  merged) and `type` for unions and intersections.

### Common patterns

- `as const` for literal-type locking and exhaustive switches.
- `satisfies` for checking a value against a type without
  widening it — you keep the narrow literal types for inference
  downstream.
- `Map<K, V>` over `Record<string, V>` when keys are dynamic and
  iteration matters.
- `zod` (or similar) for runtime validation of anything crossing
  a process boundary: API responses, user input, environment
  variables.

### Error handling

- Define custom error classes extending `Error` and set the
  `name` property so logs identify them.
- In library code, prefer Result types over throwing for
  *expected* failure modes. Throw for programmer errors and
  unrecoverable state.
- Validate external inputs at the boundary — once a value is
  inside your type system, the types should be trustworthy.

### Testing

Use `vitest` or `jest` for runtime tests. For type tests, use
`expectTypeOf` (vitest) or similar to assert that types behave
correctly — catching regressions in generic helpers before they
reach callers.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Using `any` instead of `unknown` for untyped boundaries.** `any` disables type checking for the variable and everything it touches downstream. `unknown` forces you to narrow before use — it's the type-safe alternative for values whose shape is genuinely unknown.
- **Disabling strict mode "temporarily" in a new project.** A TypeScript project without `strict: true` is a JavaScript project with extra syntax. Enabling strict mode later is painful because the whole codebase has accumulated implicit nulls and anys. Enable it on day one.
- **Type assertions (`x as Foo`) as a refactoring shortcut.** `as` bypasses the type checker. It is appropriate for narrow cases like DOM element access or JSON parsing, not as a way to silence type errors without fixing them. Fix the real type mismatch.
- **Skipping runtime validation at process boundaries.** TypeScript's types exist only at compile time. A REST API response, environment variable, or user input that passes type-checking at compile time can still have the wrong shape at runtime. Validate external data with `zod` or equivalent at every boundary.
- **Using `Record<string, V>` when a `Map<K, V>` is semantically correct.** `Record` is for objects with a known finite set of keys. Dynamic keys, iteration in insertion order, presence checks, and non-string keys all call for `Map`. The wrong choice leads to surprising behavior with `hasOwnProperty` and `for...in`.
- **`Optional` field (`?:`) when the property is actually required.** Marking a property optional (`email?: string`) says "this property may be absent". If the property is always present but sometimes has no value, use `email: string | null` instead — the distinction affects how callers narrow the type.
- **No `never` check in `switch` exhaustiveness.** Without an explicit `const _exhaustive: never = s` in the `default` branch, adding a new variant to a discriminated union compiles without error, producing a silent runtime gap. Always add the exhaustiveness check.

## Full reference

### Discriminated unions with exhaustive checks

```ts
type Shape =
  | { kind: "circle"; radius: number }
  | { kind: "square"; side: number }
  | { kind: "rect"; width: number; height: number }

function area(s: Shape): number {
  switch (s.kind) {
    case "circle": return Math.PI * s.radius ** 2
    case "square": return s.side ** 2
    case "rect":   return s.width * s.height
    default: {
      const _exhaustive: never = s
      return _exhaustive
    }
  }
}
```

The `never` assignment makes adding a new variant a compile error
at every consumer. This is the TypeScript equivalent of Java's
sealed types or Rust's exhaustive `match`.

### Runtime validation with zod

```ts
import { z } from "zod"

const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  createdAt: z.coerce.date(),
})
type User = z.infer<typeof UserSchema>

async function fetchUser(id: string): Promise<User> {
  const res = await fetch(`/users/${id}`)
  return UserSchema.parse(await res.json())
}
```

Validate at the boundary; trust the types everywhere inside.
Never pass raw `any` / `unknown` API payloads into business
logic.

### `satisfies` vs type annotation

```ts
// Widens to Record<string, string> — you lose the literal keys
const routesWide: Record<string, string> = {
  home: "/",
  about: "/about",
}

// Keeps literal types for keys and values; still type-checked
const routes = {
  home: "/",
  about: "/about",
} satisfies Record<string, string>

type RouteName = keyof typeof routes  // "home" | "about"
```

### Anti-patterns

- `any` anywhere it isn't strictly required to interop with
  untyped code. Use `unknown` and narrow.
- `Optional` field types via `T | undefined` when the property is
  actually required — use the `?` form only when the property
  may be absent.
- Type assertions (`x as Foo`) used as a refactoring shortcut
  instead of fixing the real type mismatch.
- Returning different shapes from the same function based on
  arguments — use overloads or discriminated unions.
- Catching `Error` and losing the type — re-throw or convert to
  a Result.
- Using `Record<string, V>` when you actually need a `Map<K, V>`
  (iteration order, non-string keys, presence checks).
- Disabling strict mode "temporarily" in a new project.

### Custom error class

```ts
export class NotFoundError extends Error {
  constructor(public readonly resource: string, id: string) {
    super(`${resource} ${id} not found`)
    this.name = "NotFoundError"
  }
}
```

Set `name` explicitly — the default is `"Error"`, which makes
log filtering and `instanceof` checks across realms unreliable.
