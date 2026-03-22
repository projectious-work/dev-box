---
name: tailwind
description: Tailwind CSS v4 patterns — utility-first styling, responsive design, dark mode, component extraction. Use when building or reviewing Tailwind-based UIs.
---

# Tailwind CSS

## When to Use

When the user is building UIs with Tailwind CSS, asks about styling patterns,
responsive layouts, dark mode, or says "style this component" or "set up Tailwind".

## Instructions

### 1. Project Setup (v4)

- Install: `npm i tailwindcss @tailwindcss/postcss` (or use `@tailwindcss/vite`)
- CSS entry point needs only `@import "tailwindcss";` — no config file required
- Define design tokens with `@theme` in CSS, not `tailwind.config.js`:
  ```css
  @import "tailwindcss";

  @theme {
    --color-brand-500: oklch(0.65 0.19 260);
    --font-heading: "Inter", sans-serif;
    --breakpoint-3xl: 1920px;
  }
  ```
- Content detection is automatic (respects `.gitignore`); override with `@source`
- For Vite projects, prefer the first-party Vite plugin over PostCSS

### 2. Utility-First Principles

- Compose styles directly in markup — avoid premature abstraction
- Keep class lists readable: group by concern (layout, spacing, typography, color)
- Use arbitrary values sparingly: `w-[347px]` signals a missing design token
- Tailwind v4 supports any spacing value (`mt-13`, `w-17`) without configuration
- Prefer semantic HTML elements over `div` soup with utility classes

### 3. Component Extraction

- Extract components in your framework (React/Vue/Svelte), not with `@apply`
- The `@apply` directive still works but is discouraged for maintainability
- When you must share raw CSS (e.g., markdown-rendered content), use `@apply`
- Keep extracted components small — a button, a card, not an entire page section
- Document component variants with props, not class string concatenation

### 4. Responsive Design

- Mobile-first: base styles apply to all sizes, add breakpoints upward
- Breakpoints: `sm:` (40rem), `md:` (48rem), `lg:` (64rem), `xl:` (80rem), `2xl:` (96rem)
- Use container queries for component-level responsiveness:
  ```html
  <div class="@container">
    <div class="grid grid-cols-1 @sm:grid-cols-2 @lg:grid-cols-4">
  ```
- Prefer `max-w-*` with `mx-auto` for content width constraints
- Use `reflow` utilities and test at 320px width for WCAG 1.4.10 compliance

### 5. Dark Mode

- Use the `dark:` variant with CSS `prefers-color-scheme` (default) or class strategy
- Define semantic color tokens that map to light/dark values:
  ```css
  @theme {
    --color-surface: oklch(0.99 0 0);
    --color-on-surface: oklch(0.15 0 0);
  }
  .dark {
    --color-surface: oklch(0.15 0 0);
    --color-on-surface: oklch(0.95 0 0);
  }
  ```
- Apply via: `bg-surface text-on-surface` — no `dark:` prefix needed per element
- Use `color-scheme-dark` / `color-scheme-light` for native element theming
- Always test both modes; check contrast ratios meet 4.5:1 (WCAG AA)

### 6. Colors and Theming

- Tailwind v4 uses OKLCH color space — more vibrant and perceptually uniform
- Use `color-mix()` via opacity modifiers: `bg-blue-500/50`
- Define brand colors in `@theme`; avoid hardcoded hex values in markup
- Multi-brand theming: swap CSS variables at `[data-theme]` level, no rebuild needed

### 7. Performance

- Automatic unused CSS elimination — no manual purge config in v4
- Full builds complete in ~100ms; incremental in ~5ms (Rust-based engine)
- Avoid dynamically constructing class names (`text-${color}-500`) — the scanner
  cannot detect them. Use complete class names or safelist with `@source`
- Lazy-load heavy components; Tailwind adds zero runtime JS

### 8. Common Mistakes to Avoid

- **Class string builders**: `text-${size}` breaks detection. Use lookup maps instead
- **Overusing arbitrary values**: signals missing design tokens — add them to `@theme`
- **Nesting utilities in CSS**: defeats the utility-first approach
- **Ignoring accessibility**: add focus-visible styles, sufficient contrast, motion-reduce
- **Not using `@theme`**: hardcoded values scattered across markup create inconsistency

### 9. Accessibility with Tailwind

- Always include `focus-visible:` ring/outline styles on interactive elements
- Use `motion-reduce:` to respect `prefers-reduced-motion`
- Use `sr-only` for screen-reader-only text (icon-only buttons need labels)
- Ensure color contrast: text on `bg-*` must meet WCAG AA (4.5:1 normal, 3:1 large)
- Click targets should be at least 24x24 CSS pixels (WCAG 2.5.8)

## References

- `references/cheatsheet.md` — Most-used utility classes by category
- [Tailwind CSS v4 announcement](https://tailwindcss.com/blog/tailwindcss-v4)
- [Official documentation](https://tailwindcss.com/docs)

## Examples

**User:** "Add dark mode to this card component"
**Agent:** Defines semantic color tokens in `@theme` with `.dark` overrides, applies
token-based classes (`bg-surface text-on-surface`), adds `dark:` variant only where
semantic tokens do not cover the case, and tests contrast in both modes.

**User:** "Make this layout responsive"
**Agent:** Starts with a single-column mobile layout, adds `sm:` and `lg:` breakpoints
for multi-column grids, uses container queries for self-contained components, and
tests at 320px minimum width.
