---
name: frontend-design
description: |
  Frontend architecture and UI design — component hierarchies, accessibility, performance, state management. Use when designing frontend architecture, building component hierarchies, making something accessible, optimizing Core Web Vitals, or choosing a state management approach.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-frontend-design
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: design
---

# Frontend Design

## Intro

Good frontend design starts with semantic HTML, small
single-responsibility components, and WCAG 2.2 AA accessibility, then
layers on performance, state management, and rendering strategy as the
app grows. Apply these conventions when designing or reviewing any
frontend application.

## Overview

### Component architecture

Build a tree of small, single-responsibility components. Separate
container (data/logic) from presentational (UI) components, and
co-locate related files (`Button/Button.tsx`, `Button.test.tsx`,
`Button.module.css`). Prefer composition — children and slots — over
deep prop configuration. Name components by what they are
(`UserAvatar`), not where they live (`SidebarIcon`).

### Semantic HTML first

Start with the correct HTML element before reaching for ARIA:

- `<nav>`, `<main>`, `<aside>`, `<header>`, `<footer>` for landmarks.
- `<button>` for actions, `<a>` for navigation — never
  `<div onClick>`.
- `<ul>` / `<ol>` for lists, `<table>` for tabular data.
- `<h1>` through `<h6>` in logical order, no skipping levels.
- `<dialog>` for modals, `<details>` for disclosure widgets.

Every `<img>` needs `alt` (use `alt=""` for decorative images). Every
form input needs a visible `<label>` with matching `for` / `id`. Add
ARIA only when HTML semantics are insufficient.

### Accessibility (WCAG 2.2 AA)

- Keyboard navigation: all interactive elements reachable and
  operable via keyboard.
- Focus management: visible indicator, logical tab order, no focus
  traps.
- Color contrast: 4.5:1 normal text, 3:1 large text and UI
  components.
- Target size: at least 24x24 CSS pixels (WCAG 2.5.8).
- Motion: respect `prefers-reduced-motion`; no auto-playing
  animations over 5 seconds.
- Error handling: identify errors clearly, suggest corrections,
  prevent data loss.
- Screen readers: `aria-live` for dynamic updates, `aria-label` for
  icon buttons.
- Authentication: no cognitive-function tests (WCAG 3.3.8).

See Level 3 for the full WCAG 2.2 AA checklist.

### State management

Pick the simplest approach that fits the complexity:

| Complexity        | Solution                                               |
|-------------------|--------------------------------------------------------|
| Local UI state    | `useState`, `useReducer`                               |
| Shared across few | Lift state up, prop drilling (if shallow)              |
| Shared across many| React Context (low-frequency updates)                  |
| Complex global    | Zustand (simple API), Jotai (atomic), Redux (large teams) |
| Server state      | TanStack Query / SWR (caching, revalidation, dedup)    |
| URL state         | Search params via router (`useSearchParams`)           |
| Form state        | React Hook Form or Conform (validation, performance)   |

Avoid putting everything in global state — most state is local.
Server state is not client state; use a dedicated data-fetching
library. Derive computed values instead of storing them separately.

### React / Next.js patterns

Server Components (default in App Router) fetch data on the server
and ship zero JS. Add `"use client"` only when you need interactivity,
hooks, or browser APIs. Use `<Suspense>` boundaries to stream UI
progressively. Fetch data in Server Components and use TanStack Query
for client mutations. Pick a rendering strategy per route: SSG for
static, SSR for personalized, ISR for hybrid. Export `metadata` or
`generateMetadata` for SEO on each route.

### Performance (Core Web Vitals)

Optimize to the three Core Web Vitals thresholds:

- **LCP (< 2.5s):** preload hero images, use `next/image` with
  `priority`, minimize render-blocking CSS/JS, inline critical CSS,
  lean on Server Components to cut client JS.
- **INP (< 200ms):** avoid long tasks on the main thread, break work
  with `startTransition`, debounce expensive handlers, virtualize
  long lists, use `useOptimistic` for instant feedback.
- **CLS (< 0.1):** set explicit `width` / `height` on media, reserve
  space for dynamic content with skeletons, never inject content
  above existing content.

Analyze bundles with `@next/bundle-analyzer`, lazy-load heavy
components, and load fonts via `next/font` for zero layout shift.

### Styling approaches

| Approach       | Best for                        | Trade-offs                       |
|----------------|---------------------------------|----------------------------------|
| Tailwind CSS   | Rapid development, consistency  | Long class strings, learning curve |
| CSS Modules    | Scoped styles, small bundles    | More files, less dynamic         |
| Vanilla CSS    | Simple projects, standards      | No scoping, manual organization  |
| CSS-in-JS      | Dynamic styles, co-location     | Runtime cost, SSR complexity     |

Prefer Tailwind or CSS Modules for new projects (zero runtime cost).
Avoid runtime CSS-in-JS in Server Components. Use CSS custom
properties for theming and keep design tokens in one place.

### Project structure (Next.js App Router)

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

Group by feature for large apps, by type for small ones. Keep
`components/ui/` framework-agnostic where possible, and export barrel
files (`index.ts`) only for public component APIs.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Using `<div onClick>` instead of `<button>` for interactive elements.** A `<div>` is not keyboard-focusable, has no implicit ARIA role, and does not fire click events on Enter/Space. Keyboard users and assistive technology users cannot interact with it. Use the correct semantic HTML element first.
- **Putting all state in a global store.** Global stores cause unrelated components to re-render on every update and make the data flow hard to trace. Most state is local; lift it up only when genuinely shared; use a dedicated data-fetching library for server state.
- **Adding `"use client"` to a Next.js component to "make it work" without understanding why.** Every `"use client"` boundary ships JavaScript to the browser. Adding it to a parent makes every descendant a client component. Diagnose why the component needs interactivity and push the boundary as far down the tree as possible.
- **Hardcoding colors, font sizes, or spacing values in components.** Hardcoded values bypass the design token system and make theming, dark mode, and accessibility changes require a search-and-replace across the codebase. Define tokens in one place and reference them everywhere.
- **Omitting `alt` on images or using generic text like "image".** An `<img alt="image">` is meaningless to a screen reader user. Decorative images need `alt=""` to be skipped; meaningful images need descriptive text that conveys their content.
- **No `aria-live` for dynamic updates.** When content changes without a page reload (search results, error messages, toast notifications), assistive technology does not announce the change unless a live region is used. Wrap dynamic status updates in an `aria-live="polite"` region.
- **Optimizing Core Web Vitals last.** LCP, INP, and CLS regressions are cheapest to fix when the component is first written. Setting explicit `width`/`height` on images, using `priority` on above-fold images, and avoiding long main-thread tasks are design decisions, not afterthoughts.

## Full reference

### WCAG 2.2 AA checklist

Criteria marked (A) are Level A; (AA) are Level AA. Both are required
for AA conformance.

**Perceivable — Text alternatives:**

- 1.1.1 Non-text Content (A) — all images, icons, and buttons have
  descriptive `alt`; decorative images use `alt=""` or CSS.

**Perceivable — Time-based media:**

- 1.2.1 Audio/Video Only (A) — transcripts for audio-only,
  transcripts or audio descriptions for video-only.
- 1.2.2 Captions Prerecorded (A) — synchronized captions on
  prerecorded video.
- 1.2.3 Audio Description (A) — audio description or descriptive
  transcript for video.
- 1.2.4 Captions Live (AA) — real-time captions for live audio.

**Perceivable — Adaptable:**

- 1.3.1 Info and Relationships (A) — semantic HTML for headings,
  landmarks, lists, table headers; do not rely on visual styling.
- 1.3.2 Meaningful Sequence (A) — DOM order matches visual reading
  order.
- 1.3.3 Sensory Characteristics (A) — instructions do not rely solely
  on shape, color, size, or location.
- 1.3.4 Orientation (AA) — content works in portrait and landscape.
- 1.3.5 Identify Input Purpose (AA) — form fields use appropriate
  `autocomplete` attributes.

**Perceivable — Distinguishable:**

- 1.4.1 Use of Color (A) — color is not the only cue.
- 1.4.2 Audio Control (A) — auto-playing audio over 3s can be paused
  or adjusted.
- 1.4.3 Contrast Minimum (AA) — 4.5:1 normal text, 3:1 large text
  (18pt+ or 14pt+ bold).
- 1.4.4 Resize Text (AA) — readable and functional at 200% zoom.
- 1.4.5 Images of Text (AA) — use real text, not images of text.
- 1.4.10 Reflow (AA) — no horizontal scrolling at 320px viewport
  (1280px at 400% zoom).
- 1.4.11 Non-text Contrast (AA) — 3:1 for UI components and graphical
  objects.
- 1.4.12 Text Spacing (AA) — no content loss with overridden line
  height (1.5x), paragraph spacing (2x), letter spacing (0.12em),
  word spacing (0.16em).
- 1.4.13 Content on Hover/Focus (AA) — tooltips dismissible,
  hoverable, persistent until dismissed.

**Operable — Keyboard:**

- 2.1.1 Keyboard (A) — all functionality keyboard-accessible.
- 2.1.2 No Keyboard Trap (A) — focus can always move away.

**Operable — Enough time & seizures:**

- 2.2.1 Timing Adjustable (A) — time limits can be turned off or
  extended.
- 2.2.2 Pause, Stop, Hide (A) — auto-moving content over 5s can be
  paused.
- 2.3.1 Three Flashes (A) — no more than 3 flashes per second.

**Operable — Navigable:**

- 2.4.1 Bypass Blocks (A) — "skip to main content" link.
- 2.4.2 Page Titled (A) — descriptive `<title>`.
- 2.4.3 Focus Order (A) — logical tab order.
- 2.4.4 Link Purpose (A) — link text describes the destination.
- 2.4.5 Multiple Ways (AA) — at least two ways to find pages.
- 2.4.6 Headings and Labels (AA) — descriptive headings and labels.
- 2.4.7 Focus Visible (AA) — clearly visible focus indicator.
- 2.4.11 Focus Not Obscured (AA) — focused elements not hidden by
  sticky UI.

**Operable — Input modalities:**

- 2.5.1 Pointer Gestures (A) — multi-finger/path gestures have
  single-pointer alternatives.
- 2.5.2 Pointer Cancellation (A) — actions fire on pointer up and can
  be aborted.
- 2.5.3 Label in Name (A) — visible label text is in the accessible
  name.
- 2.5.4 Motion Actuation (A) — shake/tilt has button alternatives.
- 2.5.7 Dragging Movements (AA) — drag-and-drop has a non-drag
  alternative.
- 2.5.8 Target Size Minimum (AA) — interactive targets at least
  24x24 CSS pixels.

**Understandable:**

- 3.1.1 Language of Page (A) — `<html lang="...">`.
- 3.1.2 Language of Parts (AA) — `lang` attribute on foreign content.
- 3.2.1 On Focus (A) — focus does not trigger unexpected changes.
- 3.2.2 On Input (A) — input changes do not cause unexpected
  navigation.
- 3.2.3 Consistent Navigation (AA) — navigation order consistent
  across pages.
- 3.2.4 Consistent Identification (AA) — same functionality uses same
  labels.
- 3.3.1 Error Identification (A) — errors clearly described in text.
- 3.3.2 Labels or Instructions (A) — visible labels or instructions.
- 3.3.3 Error Suggestion (AA) — suggest how to fix errors.
- 3.3.4 Error Prevention (AA) — reversible, verified, or confirmed
  for legal/financial data.
- 3.3.7 Redundant Entry (A) — previously entered info auto-populated.
- 3.3.8 Accessible Authentication (AA) — no cognitive tests, allow
  paste, support password managers.

**Robust:**

- 4.1.2 Name, Role, Value (A) — custom components expose name, role,
  state via ARIA; prefer native HTML.
- 4.1.3 Status Messages (AA) — dynamic status announced via
  `aria-live` or `role="alert"`.

### ARIA patterns

```html
<!-- Icon button -->
<button aria-label="Close dialog">
  <svg aria-hidden="true">...</svg>
</button>

<!-- Live region for dynamic updates -->
<div aria-live="polite" aria-atomic="true">
  3 items in cart
</div>

<!-- Form with error -->
<label for="email">Email</label>
<input id="email" type="email" aria-invalid="true" aria-describedby="email-error">
<p id="email-error" role="alert">Please enter a valid email address.</p>

<!-- Modal dialog -->
<dialog aria-labelledby="dialog-title">
  <h2 id="dialog-title">Confirm deletion</h2>
  ...
</dialog>
```

Base HTML skeleton:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Descriptive Page Title</title>
</head>
<body>
  <a href="#main" class="sr-only focus:not-sr-only">Skip to main content</a>
  <header><nav aria-label="Main">...</nav></header>
  <main id="main">...</main>
  <footer>...</footer>
</body>
</html>
```

### Testing tools

- **axe DevTools** — browser extension for automated checks.
- **Lighthouse** — accessibility audit in Chrome DevTools.
- **NVDA** (Windows) / **VoiceOver** (macOS) — screen reader testing.
- **Keyboard-only navigation** — unplug the mouse, test every flow.
- **Colour Contrast Analyser** — verify contrast ratios.
- **WAVE** — web accessibility evaluation tool.

### Further reading

- `references/accessibility-checklist.md` — source for the WCAG 2.2
  AA checklist above.
- [React documentation](https://react.dev)
- [Next.js documentation](https://nextjs.org/docs)
- [Web Content Accessibility Guidelines 2.2](https://www.w3.org/TR/WCAG22/)
