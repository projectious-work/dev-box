---
name: tailwind
description: |
  Tailwind CSS v4 — utility-first styling, responsive design, dark mode, and component extraction. Use when building or reviewing Tailwind-based UIs, setting up Tailwind v4, or designing responsive or dark-mode styling.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-tailwind
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Tailwind CSS

## Intro

Tailwind v4 is CSS-first: install the package, `@import "tailwindcss"`
once, and declare design tokens inside a `@theme` block. Compose
styles directly in markup with utility classes, extract components in
your framework (not with `@apply`), and lean on semantic tokens so
dark mode and theming don't fight you.

## Overview

### Project setup (v4)

Install `tailwindcss` plus the appropriate adapter
(`@tailwindcss/postcss` or `@tailwindcss/vite`). The CSS entry point
only needs:

```css
@import "tailwindcss";

@theme {
  --color-brand-500: oklch(0.65 0.19 260);
  --font-heading: "Inter", sans-serif;
  --breakpoint-3xl: 1920px;
}
```

There's no `tailwind.config.js`. Content detection is automatic
(respecting `.gitignore`); override it with `@source` when you need to
include extra files. For Vite projects, prefer the first-party Vite
plugin over PostCSS.

### Utility-first principles

Compose styles in markup and avoid premature abstraction. Group
classes by concern (layout → spacing → typography → color) to keep
long class lists scannable. Tailwind v4 accepts any integer spacing
(`mt-13`, `w-17`) without configuration — arbitrary values like
`w-[347px]` should be a code smell that signals a missing design
token.

### Component extraction

Extract in your UI framework (React/Vue/Svelte), not with `@apply`.
`@apply` still works and remains appropriate when you need real CSS
(e.g., styling HTML rendered from markdown), but prefer framework
components for everything else. Keep extracted components small — a
button or card, not an entire page section.

### Responsive design

Mobile-first: base styles apply to all sizes; add breakpoints upward
with `sm:`, `md:`, `lg:`, `xl:`, `2xl:`. For component-level
responsiveness, use container queries:

```html
<div class="@container">
  <div class="grid grid-cols-1 @sm:grid-cols-2 @lg:grid-cols-4">
```

Constrain content with `max-w-*` + `mx-auto`. Test at 320px wide to
stay WCAG 1.4.10 compliant.

### Dark mode

Use the `dark:` variant with either the default
`prefers-color-scheme` strategy or a `.dark` class. Best pattern:
define semantic tokens once and override them in `.dark`, then apply
token-based classes without any `dark:` prefix:

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

Then write `bg-surface text-on-surface` everywhere and both modes
follow automatically.

### Colors and theming

Tailwind v4 uses OKLCH, which is more vibrant and perceptually
uniform than sRGB. Opacity modifiers work via `color-mix()`:
`bg-blue-500/50`. Define brand colors in `@theme` rather than
scattering hex values through markup. Multi-brand apps can swap CSS
variables at the `[data-theme]` level at runtime without a rebuild.

### Performance

The v4 engine is Rust-based: full builds run in around 100ms and
incremental rebuilds in single-digit ms. Unused CSS is eliminated
automatically. Never build class names dynamically
(`` `text-${color}-500` ``) — the scanner can't see them. Use a
lookup map or safelist the class with `@source`.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Building class names dynamically with template literals.** Tailwind's scanner detects class names by static string matching. A class like `` `text-${color}-500` `` or `"bg-" + shade` is invisible to the scanner and will be missing from the production build. Use a lookup object with full class name strings, or add the generated classes to a safelist.
- **Overusing arbitrary values.** `w-[347px]` or `text-[13px]` signals a design token that is missing from `@theme`. One arbitrary value is a problem to fix; many arbitrary values mean the design system has diverged from the implementation. Add the value to `@theme` and reference it by name.
- **Using `@apply` for everything instead of component extraction.** `@apply` scatters styles across CSS files and breaks the direct relationship between markup and styling that makes Tailwind easy to maintain. Extract reusable patterns as framework components (React, Vue, Svelte), not as CSS utilities.
- **Not using `focus-visible:` for focus styles.** `focus:outline-none` removes focus styles for all users, including keyboard users who depend on them. Use `focus-visible:ring-2 focus-visible:ring-blue-500` to show focus indicators for keyboard navigation while hiding them for mouse users.
- **Skipping `@theme` and hardcoding colors inline.** Using arbitrary color values like `text-[#3b82f6]` instead of defining `--color-brand-primary` in `@theme` makes dark mode and multi-brand theming require global search-and-replace instead of a single token change.
- **Not testing at 320px viewport width.** Tailwind's mobile-first default base styles apply at all widths. A layout that works at 375px may break at 320px, which is the WCAG 1.4.10 (Reflow) minimum. Always check that no horizontal scrolling appears at 320px.
- **Using the v3 `tailwind.config.js` pattern with v4.** Tailwind v4 removes `tailwind.config.js`; configuration moves to CSS `@theme` blocks. Mixing v3 config with v4 styles causes tokens to be silently ignored. Check the version and use the appropriate configuration pattern.

## Full reference

### Layout utilities

| Purpose          | Classes                                                                |
|------------------|------------------------------------------------------------------------|
| Display          | `block` `inline-block` `flex` `inline-flex` `grid` `hidden` `contents` |
| Position         | `static` `relative` `absolute` `fixed` `sticky`                        |
| Flex direction   | `flex-row` `flex-col` `flex-row-reverse` `flex-col-reverse`            |
| Flex wrap        | `flex-wrap` `flex-nowrap`                                              |
| Flex grow/shrink | `grow` `shrink` `grow-0` `shrink-0` `basis-1/2`                        |
| Justify content  | `justify-start` `justify-center` `justify-between` `justify-end`       |
| Align items      | `items-start` `items-center` `items-end` `items-stretch`               |
| Align self       | `self-auto` `self-start` `self-center` `self-end`                      |
| Grid template    | `grid-cols-1` ... `grid-cols-12` `grid-rows-1` ... `grid-rows-6`       |
| Grid span        | `col-span-1` ... `col-span-full` `row-span-1` ... `row-span-full`      |
| Gap              | `gap-0` `gap-1` ... `gap-8` `gap-x-4` `gap-y-4`                        |
| Overflow         | `overflow-auto` `overflow-hidden` `overflow-scroll` `overflow-visible` |
| Z-index          | `z-0` `z-10` `z-20` `z-30` `z-40` `z-50` `z-auto`                      |

### Spacing and sizing

| Purpose       | Classes                                                                    |
|---------------|----------------------------------------------------------------------------|
| Padding       | `p-0` `p-1` ... `p-8` `px-4` `py-2` `pt-4` `pr-4` `pb-4` `pl-4`            |
| Margin        | `m-0` `m-1` ... `m-8` `mx-auto` `my-4` `-mt-4`                             |
| Space between | `space-x-4` `space-y-4`                                                    |
| Width         | `w-0` `w-px` `w-1` ... `w-96` `w-auto` `w-full` `w-fit` `w-1/2` `w-1/3`    |
| Max width     | `max-w-xs` `max-w-sm` ... `max-w-7xl` `max-w-full` `max-w-prose`           |
| Min width     | `min-w-0` `min-w-full` `min-w-min` `min-w-max`                             |
| Height        | `h-0` `h-1` ... `h-96` `h-auto` `h-full` `h-screen` `h-fit`                |
| Min/Max height| `min-h-screen` `max-h-96`                                                  |

In v4, any integer works: `p-13`, `mt-17`, `gap-22` — no config.

### Typography

| Purpose         | Classes                                                                |
|-----------------|------------------------------------------------------------------------|
| Font size       | `text-xs` `text-sm` `text-base` `text-lg` ... `text-9xl`               |
| Font weight     | `font-thin` `font-light` `font-normal` `font-medium` `font-semibold` `font-bold` |
| Font style      | `italic` `not-italic`                                                  |
| Font family     | `font-sans` `font-serif` `font-mono`                                   |
| Text align      | `text-left` `text-center` `text-right` `text-justify`                  |
| Text color      | `text-black` `text-white` `text-{color}-{shade}`                       |
| Line height     | `leading-none` `leading-tight` `leading-snug` `leading-normal`         |
| Letter spacing  | `tracking-tight` `tracking-normal` `tracking-wide`                     |
| Text decoration | `underline` `overline` `line-through` `no-underline`                   |
| Text transform  | `uppercase` `lowercase` `capitalize` `normal-case`                     |
| Text overflow   | `truncate` `text-ellipsis` `text-clip`                                 |
| Whitespace      | `whitespace-normal` `whitespace-nowrap` `whitespace-pre`               |

### Backgrounds and borders

| Purpose       | Classes                                                                  |
|---------------|--------------------------------------------------------------------------|
| Background    | `bg-white` `bg-{color}-{shade}` `bg-transparent` `bg-blue-500/50`        |
| Gradient      | `bg-linear-to-r` `bg-linear-45` `from-{color}` `via-{color}` `to-{color}`|
| Border width  | `border` `border-0` `border-2` `border-4` `border-t` `border-r`          |
| Border color  | `border-gray-200` `border-{color}-{shade}` `border-transparent`          |
| Border style  | `border-solid` `border-dashed` `border-dotted` `border-none`             |
| Radius        | `rounded-none` `rounded-sm` `rounded` `rounded-md` `rounded-lg` `rounded-full` |
| Ring          | `ring` `ring-2` `ring-{color}-{shade}` `ring-offset-2`                   |

v4 renamed `bg-gradient-*` to `bg-linear-*`.

### Effects and motion

| Purpose       | Classes                                                                  |
|---------------|--------------------------------------------------------------------------|
| Shadow        | `shadow-sm` `shadow` `shadow-md` `shadow-lg` `shadow-xl` `shadow-none`   |
| Opacity       | `opacity-0` `opacity-25` `opacity-50` `opacity-75` `opacity-100`         |
| Blur          | `blur-none` `blur-sm` `blur` `blur-md` `blur-lg`                         |
| Backdrop blur | `backdrop-blur-sm` `backdrop-blur` `backdrop-blur-md`                    |
| Transition    | `transition-none` `transition` `transition-colors` `transition-transform`|
| Duration      | `duration-75` `duration-150` `duration-300` `duration-500` `duration-1000` |
| Timing        | `ease-linear` `ease-in` `ease-out` `ease-in-out`                         |
| Animation     | `animate-spin` `animate-ping` `animate-pulse` `animate-bounce`           |
| Transform     | `scale-50` ... `scale-150` `rotate-45` `translate-x-4` `translate-y-4`   |

### Breakpoints

| Prefix | Min-width | Typical use             |
|--------|-----------|-------------------------|
| (none) | 0px       | Mobile-first base       |
| `sm:`  | 640px     | Large phones, landscape |
| `md:`  | 768px     | Tablets                 |
| `lg:`  | 1024px    | Laptops                 |
| `xl:`  | 1280px    | Desktops                |
| `2xl:` | 1536px    | Large screens           |

### State variants

| Variant            | Trigger                                  |
|--------------------|------------------------------------------|
| `hover:`           | Mouse hover                              |
| `focus:`           | Element focused                          |
| `focus-visible:`   | Keyboard focus (recommended over focus:) |
| `active:`          | Being clicked/pressed                    |
| `disabled:`        | Disabled form element                    |
| `first:` `last:`   | First/last child                         |
| `odd:` `even:`     | Odd/even children                        |
| `dark:`            | Dark mode                                |
| `motion-reduce:`   | Prefers reduced motion                   |
| `@sm:` `@md:` etc. | Container query breakpoints (v4)         |
| `not-hover:`       | Inverse of hover (v4)                    |

### Accessibility utilities

| Purpose            | Classes                                             |
|--------------------|-----------------------------------------------------|
| Screen reader only | `sr-only` `not-sr-only`                             |
| Focus ring         | `focus-visible:ring-2 focus-visible:ring-blue-500`  |
| Reduced motion     | `motion-reduce:transition-none`                     |
| Forced colors      | `forced-colors:border`                              |

Always include `focus-visible:` ring/outline styles on interactive
elements. Use `sr-only` for labels on icon-only buttons. Text on
colored backgrounds must meet WCAG AA contrast (4.5:1 normal, 3:1
large). Click targets should be at least 24×24 CSS pixels
(WCAG 2.5.8).

### Anti-patterns

- **Class string builders** (`` `text-${size}` ``) — the scanner
  can't detect them; use a lookup map
- **Overusing arbitrary values** — treat `w-[347px]` as a missing
  token and add it to `@theme`
- **Nesting utilities in CSS** — defeats the utility-first approach
- **Ignoring accessibility** — skipping focus states, contrast, or
  `motion-reduce:` will come back as audits
- **Not using `@theme`** — scattered hardcoded values destroy
  consistency and make dark mode painful

### Further reading

- [Tailwind CSS v4 announcement](https://tailwindcss.com/blog/tailwindcss-v4)
- [Official documentation](https://tailwindcss.com/docs)
