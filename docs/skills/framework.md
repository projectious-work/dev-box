# Framework & SEO Skills

Skills for specific frameworks and search engine optimization.

---

### reflex-python

> Reflex Python web framework for building full-stack apps in pure Python. Components, state management, and deployment. Use when building Reflex apps, designing component hierarchies, or managing app state.

**Triggers:** When building web apps with Reflex, designing components, managing state, routing, or creating full-stack Python web applications.
**Tools:** `Bash(reflex:*)` `Bash(python:*)` `Read` `Write`
**References:** `component-reference.md`

Key capabilities:

- App structure: initialization, entry points, page decorators, file-based routing, configuration
- Component system: layout (box, flex, grid), display (text, heading, image), input (input, select, checkbox), feedback (alert, toast, spinner)
- State management with `rx.State` classes, typed vars, event handlers, computed vars, and substates
- Event handling: on_click, on_change, two-way binding, background tasks, event chaining
- Styling with Radix UI design tokens, responsive props, light/dark themes
- Routing with dynamic segments, programmatic navigation, on_load events, and 404 handling
- Database integration via built-in SQLModel with automatic migrations
- Deployment to Reflex Cloud or self-hosted via Docker

??? example "Example usage"
    **Build a todo app:** Defines a `TodoState` with a list of todos and input field, creates event handlers for add/delete/toggle, builds UI with `rx.input`, `rx.button`, and `rx.foreach(TodoState.todos, render_todo)` to render the list dynamically.

---

### fastapi-patterns

> FastAPI patterns including dependency injection, Pydantic models, async endpoints, middleware, and testing. Use when building FastAPI applications, designing API endpoints, or reviewing FastAPI code.

**Triggers:** When building APIs with FastAPI, designing endpoints, implementing dependency injection, authentication, or testing FastAPI applications.
**Tools:** `Bash(python:*)` `Bash(uvicorn:*)` `Read` `Write`
**References:** `endpoint-patterns.md`

Key capabilities:

- Route definitions with HTTP method decorators, path/query parameters, and APIRouter modules
- Pydantic models for request/response: separate Create/Update/Response schemas, validation with Field()
- Dependency injection with `Depends()`, chained dependencies, `Annotated` for reusable deps, yield-based cleanup
- Async endpoint patterns: when to use `async def` vs `def`, pairing with async libraries
- Middleware: CORS, custom timing, trusted hosts, GZip compression
- Authentication: OAuth2 password flow, JWT decoding, API key headers, scopes
- Background tasks, WebSocket support, and structured error handling
- Testing with TestClient, dependency overrides, async tests, and WebSocket testing

??? example "Example usage"
    **REST API for a blog:** Creates Pydantic schemas for `PostCreate`, `PostResponse`, `CommentCreate`, defines APIRouter modules for `/posts` and `/posts/{id}/comments`, implements CRUD handlers with SQLAlchemy dependency injection, and adds pagination to list endpoints.

---

### pandas-polars

> DataFrame operations with pandas and polars including groupby, joins, reshaping, and performance optimization. Use when manipulating tabular data, choosing between pandas and polars, or optimizing DataFrame code.

**Triggers:** When working with tabular data, performing DataFrame operations, data transformations, cleaning data, aggregating by group, or choosing between pandas and polars.
**Tools:** `Bash(python:*)` `Read` `Write`
**References:** `api-comparison.md`

Key capabilities:

- Choosing between pandas (mature ecosystem, exploratory work, <1GB) and polars (faster, lower memory, lazy evaluation, 1GB+)
- DataFrame I/O: Parquet over CSV, dtype enforcement, chunked reading, column selection
- Selection and filtering with expressions (polars) and `.loc[]`/`.iloc[]` (pandas)
- GroupBy and aggregation with named columns, window functions (`over()` in polars)
- Joins and merges with explicit join types, duplicate checking, anti-joins
- Reshaping with pivot and melt/unpivot for wide and long formats
- Missing data handling: null counts, fill strategies, interpolation
- String and datetime operations across both libraries
- Performance optimization: lazy evaluation, avoiding `apply()`, categorical dtypes

??? example "Example usage"
    **Process a large CSV with group statistics:** Uses polars lazy mode to scan the CSV, applies filters before collection, groups by the requested column with multiple aggregations in one `.agg()` call, sorts results, and writes output to Parquet for downstream use.

---

### flutter-development

> Flutter/Dart development including widget architecture, state management, navigation, and cross-platform patterns. Use when building Flutter apps, choosing state management, or designing responsive mobile layouts.

**Triggers:** When building mobile or cross-platform apps with Flutter, designing widgets, managing state, adding navigation, or creating responsive layouts.
**Tools:** `Bash(flutter:*)` `Bash(dart:*)` `Read` `Write`
**References:** `widget-catalog.md`

Key capabilities:

- Widget architecture: StatelessWidget vs StatefulWidget, composition over inheritance, const constructors
- Layout system: Row, Column, Expanded, Flexible, Stack, ListView.builder, responsive design with LayoutBuilder
- State management options: setState (local), Provider (lightweight DI), Riverpod (type-safe), BLoC (event-driven)
- Navigation with GoRouter: declarative routes, nested navigation with ShellRoute, redirect guards, deep linking
- Theming with Material 3, ColorScheme.fromSeed, dark mode support, custom TextTheme
- Networking with http/dio, json_serializable, FutureBuilder/StreamBuilder, repository pattern
- Platform channels for native code integration (MethodChannel, EventChannel)
- Testing: unit, widget (testWidgets, pumpWidget), integration, and golden tests
- Performance: const widgets, ListView.builder, RepaintBoundary, DevTools profiling

??? example "Example usage"
    **Product list with search and pull-to-refresh:** Creates a StatefulWidget with a search TextField, uses ListView.builder for efficient rendering, implements RefreshIndicator for pull-to-refresh, fetches products from a repository, and shows loading/error/empty states.

---

### seo-optimization

> SEO optimization including on-page SEO, technical SEO, Core Web Vitals, structured data, and mobile-first indexing. Use when optimizing websites for search engines, implementing structured data, or improving page performance.

**Triggers:** When optimizing a website for search engines, working with meta tags, structured data, page speed, Core Web Vitals, or mobile-first indexing.
**Tools:** `Bash(curl:*)` `Bash(lighthouse:*)` `Read` `Write`
**References:** `technical-seo-checklist.md`

Key capabilities:

- On-page SEO: title tags, meta descriptions, heading hierarchy, alt text, internal links, URL structure
- Technical SEO: robots.txt, XML sitemaps, canonical URLs, hreflang, redirect chains, index control
- Structured data with JSON-LD: Article, Product, FAQ, BreadcrumbList, Organization schemas
- Core Web Vitals optimization: LCP (<2.5s), INP (<200ms), CLS (<0.1) with specific fix strategies
- Page speed: render-blocking resources, image compression (WebP/AVIF), CDN, minification, Brotli/gzip
- Mobile-first indexing: responsive design, viewport meta, touch targets, content parity
- Internal linking strategy: content hubs, descriptive anchor text, orphan page detection
- Security and trust signals: HTTPS, HSTS, E-E-A-T authorship

??? example "Example usage"
    **SEO audit of a Next.js site:** Checks meta tags on key pages, verifies sitemap.xml and robots.txt, validates structured data with Rich Results Test, runs Lighthouse for Core Web Vitals, checks canonical URLs, and produces a prioritized list of fixes sorted by expected impact.
