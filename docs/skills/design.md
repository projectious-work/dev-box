# Design & Visual Skills

Skills for frontend development, visual design, and creative production.

---

### excalidraw

> Generates Excalidraw diagrams programmatically as JSON. Use when creating architecture diagrams, flowcharts, or hand-drawn-style visuals for documentation.

**Triggers:** Creating architecture diagrams, flowcharts, system diagrams, or hand-drawn-style visuals for documentation
**Tools:** None
**References:** `json-schema.md`

Key capabilities:

- Generate Excalidraw JSON files with proper structure (elements, appState, version 2 format)
- Create element types: rectangles, ellipses, diamonds, lines, arrows, text, with configurable styles
- Bind text labels to shapes for labeled diagrams
- Follow layout guidelines: grid alignment (multiples of 20), consistent spacing, readable font sizes
- Apply a semantic color palette (primary, secondary, success, warning, danger, neutral)
- Produce architecture diagrams, flowcharts, and sequence-style diagrams
- Embed in documentation as `.excalidraw`, SVG, or PNG

??? example "Example usage"
    "Create an architecture diagram for a web app with React frontend, Node API, and PostgreSQL" -- Generates Excalidraw JSON with three labeled rectangles arranged left-to-right, connected by arrows labeled "HTTP/REST" and "SQL", using blue for frontend, green for API, yellow for database.

---

### frontend-design

> Frontend architecture and UI design -- component hierarchies, accessibility, performance, state management. Use when designing or reviewing frontend applications.

**Triggers:** Designing frontend architecture, building component hierarchies, implementing accessibility, optimizing performance, structuring React/Next.js projects
**Tools:** None
**References:** `accessibility-checklist.md`

Key capabilities:

- Design component architecture with single-responsibility, container vs. presentational separation, and composition over configuration
- Apply semantic HTML first (landmarks, correct elements, ARIA only when needed)
- Implement WCAG 2.2 AA accessibility: keyboard navigation, focus management, color contrast, target size, motion preferences
- Choose state management by complexity: useState, Context, Zustand/Jotai/Redux, TanStack Query, React Hook Form
- Apply React/Next.js patterns: Server Components, Client Components, Suspense streaming, SSG/SSR/ISR
- Optimize Core Web Vitals: LCP, INP, CLS with specific techniques
- Select styling approaches: Tailwind CSS, CSS Modules, Vanilla CSS, CSS-in-JS with trade-off analysis
- Structure Next.js App Router projects with route groups, layouts, and feature-based organization

??? example "Example usage"
    "Design the component structure for a dashboard" -- Proposes a layout with Server Component shell (DashboardLayout), Suspense boundaries around data widgets, client components only for interactive charts and filters, and shared ui/ primitives for cards, tables, and badges.

---

### infographics

> Creates data-driven infographics and charts as SVG. Use when visualizing data, creating charts, or designing informational graphics.

**Triggers:** Creating charts, graphs, data visualizations, infographics, or visual summaries from data
**Tools:** None
**References:** `best-practices.md`

Key capabilities:

- Generate standalone SVG files with proper viewBox, responsive scaling, and semantic structure
- Map data to visuals: identify the message, choose encoding (position, length, angle, area, color), apply visual hierarchy
- Select chart types based on data relationship: line for trends, bar for comparison, histogram for distribution, scatter for correlation, treemap for hierarchy
- Apply visual design: 3-5 color palette, typography hierarchy, generous whitespace, direct data labels
- Ensure accessibility: 4.5:1 contrast ratio, patterns alongside color, `<title>` and `<desc>` for screen readers
- Avoid common pitfalls: truncated y-axis, 3D effects, pie charts with many slices, dual y-axes, rainbow colormaps
- Alternative formats when appropriate: Mermaid, ASCII art, CSV with narrative

??? example "Example usage"
    "Create a bar chart comparing these quarterly revenues" -- Generates a horizontal bar chart SVG with labeled axes, consistent color, data labels on each bar, a clear title, and source note. Uses a single brand color with opacity variation for visual hierarchy.

---

### logo-design

> Creates SVG logos with proper scalability, color theory, and variant generation. Use when designing logos, icons, or brand marks.

**Triggers:** Designing logos or brand marks, generating favicons, reviewing logo scalability, applying color theory to branding
**Tools:** None
**References:** `design-principles.md`

Key capabilities:

- Apply logo design principles: simplicity (recognizable at 16x16), memorability, timelessness, versatility, appropriateness
- Construct SVG logos with clean geometric shapes, proper viewBox, optimized paths, and meaningful groups
- Apply color theory: monochromatic, complementary, analogous, triadic, split-complementary schemes with 2-3 color maximum
- Handle typography in logos: font personality, custom ligatures, text-to-path conversion, optical spacing
- Generate logo variants: full logo, icon/mark only, favicon (16x16, 32x32), monochrome, reversed (dark mode), social banner
- Test and validate: render at multiple sizes, test on varied backgrounds, simulate colorblindness, verify single-color reproduction

??? example "Example usage"
    "Create a logo for my CLI tool called 'flux'" -- Designs a geometric mark suggesting flow/movement, pairs it with a clean sans-serif wordmark. Generates SVG with full logo, icon-only, monochrome, and reversed variants. Explains color choices and scaling behavior.

---

### tailwind

> Tailwind CSS v4 patterns -- utility-first styling, responsive design, dark mode, component extraction. Use when building or reviewing Tailwind-based UIs.

**Triggers:** Building UIs with Tailwind CSS, styling components, setting up Tailwind, implementing responsive layouts or dark mode
**Tools:** None
**References:** `cheatsheet.md`

Key capabilities:

- Set up Tailwind v4 projects with `@import "tailwindcss"` and `@theme` design tokens (no config file needed)
- Apply utility-first principles: compose styles in markup, group by concern, avoid arbitrary values
- Extract components in the framework (React/Vue/Svelte), not with `@apply`
- Implement responsive design: mobile-first breakpoints, container queries, content width constraints
- Configure dark mode with semantic color tokens and `.dark` class overrides
- Use OKLCH color space, `color-mix()` via opacity modifiers, and multi-brand theming
- Optimize performance: automatic unused CSS elimination, avoid dynamic class construction
- Ensure accessibility: `focus-visible:` styles, `motion-reduce:`, `sr-only`, contrast compliance

??? example "Example usage"
    "Make this layout responsive" -- Starts with a single-column mobile layout, adds `sm:` and `lg:` breakpoints for multi-column grids, uses container queries for self-contained components, and tests at 320px minimum width.

---

### pixijs-gamedev

> PixiJS 2D rendering and game development including sprites, animations, interactions, and WebGL/Canvas rendering. Use when building PixiJS applications, creating 2D games, or implementing interactive graphics.

**Triggers:** Building 2D games or interactive graphics with PixiJS, managing sprites, animation loops, event handling, or WebGL rendering
**Tools:** `Bash(npm:*) Bash(npx:*) Read Write`
**References:** `api-cheatsheet.md`

Key capabilities:

- Set up PixiJS Application with `await app.init()`, responsive canvas, HiDPI rendering
- Manage sprites and textures: `Assets.load()`, Spritesheet atlases, anchor centering, texture caching
- Build display hierarchies with Containers, child transforms, zIndex sorting, ParticleContainer for bulk sprites
- Implement animation: `app.ticker.add()` with deltaTime, GSAP tweening, AnimatedSprite frame playback
- Handle interaction and events: `eventMode`, pointer events, custom hitArea, drag patterns, cursor styles
- Draw vector graphics with `Graphics()`: shapes, chained fills/strokes, shared GraphicsContext
- Apply filters and effects: BlurFilter, ColorMatrixFilter, DisplacementFilter, AlphaFilter
- Load assets with bundles, progress callbacks, and lazy loading for secondary assets
- Optimize performance: ParticleContainer, texture atlases, object pooling, GPU memory management

??? example "Example usage"
    "Set up a basic PixiJS game with a moving character" -- Creates an Application, loads a character spritesheet via Assets, creates an AnimatedSprite, adds it to the stage, and uses `app.ticker.add()` to update position based on keyboard input.

---

### mobile-app-design

> Mobile app UX design including touch targets, navigation patterns, platform conventions, and accessibility. Use when designing mobile interfaces, reviewing mobile UX, or adapting web designs for mobile.

**Triggers:** Designing mobile interfaces, reviewing mobile UX, adapting web designs for iOS and Android, implementing touch interactions
**Tools:** None
**References:** `platform-guidelines.md`

Key capabilities:

- Size touch targets correctly: 44x44pt (iOS) / 48x48dp (Android) minimum, with 8pt gaps between targets
- Choose navigation patterns: tab bar (3-5 destinations), navigation drawer (5+), stack navigation, modal sheets
- Design responsive layouts: smallest screen first (375pt/360dp), 4pt/8pt spacing grid, Dynamic Type support
- Follow iOS vs. Android conventions: back navigation, button styles, alerts, typography, icons
- Implement gesture patterns: tap, long press, swipe, pull to refresh, pinch to zoom -- with visible alternatives
- Ensure accessibility: screen reader support, 4.5:1 contrast, font scaling, reduced motion, VoiceOver/TalkBack testing
- Design offline-first: cached content, offline indicators, action queuing, optimistic UI, conflict resolution
- Handle push notifications: contextual permission requests, grouping, deep linking, in-app controls
- Create onboarding flows: 3-5 screens max, show value first, progressive disclosure, skip option on every screen

??? example "Example usage"
    "Design the navigation for a banking app" -- Recommends a bottom tab bar with 4 tabs (Accounts, Transfers, Cards, More), stack navigation for account details, a modal bottom sheet for quick transfer, and biometric authentication before sensitive actions. Places the primary CTA in the thumb-reachable zone.
