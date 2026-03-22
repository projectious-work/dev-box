---
name: frontend-design
description: Frontend architecture and UI design — component hierarchies, accessibility, performance, state management. Use when designing or reviewing frontend applications.
---

# Frontend Design

## When to Use

When the user is designing frontend architecture, building component hierarchies,
asking about accessibility, performance optimization, or says "how should I structure
this UI?" or "make this accessible".

## Instructions

### 1. Component Architecture

- Build a tree of small, single-responsibility components
- Separate concerns: container (data/logic) vs. presentational (UI) components
- Co-locate related files: `Button/Button.tsx`, `Button.test.tsx`, `Button.module.css`
- Use composition over configuration — prefer children/slots over excessive props
- Name components by what they are, not where they go: `UserAvatar`, not `SidebarIcon`

### 2. Semantic HTML First

- Start with correct HTML elements before adding ARIA:
  - `<nav>`, `<main>`, `<aside>`, `<header>`, `<footer>` for landmarks
  - `<button>` for actions, `<a>` for navigation, never `<div onClick>`
  - `<ul>`/`<ol>` for lists, `<table>` for tabular data
  - `<h1>` through `<h6>` in logical order (no skipping levels)
  - `<dialog>` for modals, `<details>` for disclosure widgets
- Add ARIA only when HTML semantics are insufficient
- Every `<img>` needs `alt`; decorative images use `alt=""`
- Every form input needs a visible `<label>` with matching `for`/`id`

### 3. Accessibility (WCAG 2.2 AA)

- **Keyboard navigation**: all interactive elements reachable and operable via keyboard
- **Focus management**: visible focus indicator, logical tab order, no focus traps
- **Color contrast**: 4.5:1 for normal text, 3:1 for large text and UI components
- **Target size**: interactive targets at least 24x24 CSS pixels
- **Motion**: respect `prefers-reduced-motion`; no auto-playing animations >5s
- **Error handling**: identify errors clearly, suggest corrections, prevent data loss
- **Screen readers**: use `aria-live` for dynamic updates, `aria-label` for icon buttons
- **Authentication**: avoid cognitive function tests (WCAG 3.3.8)
- See `references/accessibility-checklist.md` for the full WCAG 2.2 AA checklist

### 4. State Management

Choose the simplest approach that fits the complexity:

| Complexity        | Solution                                               |
|-------------------|--------------------------------------------------------|
| Local UI state    | `useState`, `useReducer`                               |
| Shared across few | Lift state up, prop drilling (if shallow)              |
| Shared across many| React Context (low-frequency updates)                  |
| Complex global    | Zustand (simple API), Jotai (atomic), Redux (large teams) |
| Server state      | TanStack Query / SWR (caching, revalidation, dedup)    |
| URL state         | Search params via router (`useSearchParams`)           |
| Form state        | React Hook Form or Conform (validation, performance)   |

- Avoid putting everything in global state — most state is local
- Server state is not client state; use a dedicated data-fetching library
- Derive computed values instead of storing them separately

### 5. React / Next.js Patterns

- **Server Components** (default in App Router): fetch data on the server, zero JS shipped
- **Client Components**: add `"use client"` only when needed (interactivity, hooks, browser APIs)
- **Streaming**: use `<Suspense>` boundaries to progressively load UI
- **Data fetching**: fetch in Server Components; use TanStack Query for client mutations
- **Rendering strategies**: SSG for static content, SSR for personalized, ISR for hybrid
- **File-based routing**: leverage layouts, loading states, error boundaries, and not-found
- **Metadata**: export `metadata` or `generateMetadata` for SEO in each route

### 6. Performance (Core Web Vitals)

- **LCP (Largest Contentful Paint < 2.5s)**:
  - Preload hero images, use `next/image` with priority
  - Minimize render-blocking CSS/JS; inline critical CSS
  - Use Server Components to reduce client JS bundle
- **INP (Interaction to Next Paint < 200ms)**:
  - Avoid long tasks on main thread; break work with `startTransition`
  - Debounce expensive handlers; virtualize long lists
  - Use `useOptimistic` for instant feedback on mutations
- **CLS (Cumulative Layout Shift < 0.1)**:
  - Set explicit `width`/`height` on images and videos
  - Reserve space for dynamic content (skeleton loaders)
  - Avoid injecting content above existing content
- **Bundle size**: analyze with `@next/bundle-analyzer`; lazy-load heavy components
- **Fonts**: use `next/font` for zero-layout-shift font loading

### 7. Styling Approaches

| Approach       | Best for                        | Trade-offs                       |
|----------------|---------------------------------|----------------------------------|
| Tailwind CSS   | Rapid development, consistency  | Long class strings, learning curve |
| CSS Modules    | Scoped styles, small bundles    | More files, less dynamic         |
| Vanilla CSS    | Simple projects, standards      | No scoping, manual organization  |
| CSS-in-JS      | Dynamic styles, co-location    | Runtime cost, SSR complexity     |

- Prefer Tailwind or CSS Modules for new projects (zero runtime cost)
- Avoid CSS-in-JS libraries with runtime overhead in Server Components
- Use CSS custom properties for theming regardless of approach
- Design tokens should live in one place, not scattered across files

### 8. Project Structure (Next.js App Router)

```
src/
  app/                    # Routes, layouts, pages
    (marketing)/          # Route groups
    dashboard/
      page.tsx
      loading.tsx
      error.tsx
    layout.tsx
    globals.css
  components/
    ui/                   # Shared primitives (Button, Input, Card)
    features/             # Feature-specific components
  lib/                    # Utilities, helpers, constants
  hooks/                  # Custom React hooks
  types/                  # Shared TypeScript types
```

- Group by feature for large apps, by type for small ones
- Keep `components/ui/` framework-agnostic where possible
- Export barrel files (`index.ts`) only for public component APIs

## References

- `references/accessibility-checklist.md` — WCAG 2.2 AA checklist
- [React documentation](https://react.dev)
- [Next.js documentation](https://nextjs.org/docs)
- [Web Content Accessibility Guidelines 2.2](https://www.w3.org/TR/WCAG22/)

## Examples

**User:** "Design the component structure for a dashboard"
**Agent:** Proposes a layout with Server Component shell (`DashboardLayout`),
Suspense boundaries around data widgets, client components only for interactive
charts and filters, and shared `ui/` primitives for cards, tables, and badges.

**User:** "Make this form accessible"
**Agent:** Adds visible labels to all inputs, associates them with `htmlFor`/`id`,
adds `aria-describedby` for help text, implements inline validation with
`aria-invalid` and `aria-errormessage`, ensures keyboard navigation works,
and tests with a screen reader.
