---
name: reflex-python
description: |
  Reflex Python web framework for full-stack apps in pure Python — components, state, routing, and deployment. Use when building Reflex apps, designing component hierarchies, managing Reflex state, or deploying a Reflex application.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-reflex-python
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Reflex Python

## Intro

Reflex lets you build full-stack web apps in pure Python: UI is
composed from `rx.*` components (Python wrappers over React), state
lives in `rx.State` subclasses, and event handlers are ordinary
methods that mutate state. The framework compiles the frontend and
runs a Python backend that owns all business logic.

## Overview

### App structure

`reflex init` scaffolds `rxconfig.py` and the app package. The entry
point is `app/app.py` with `app = rx.App()`. Each page is a function
decorated with `@rx.page(route="/path", title="...")`. Files dropped
in `app/pages/` become routes automatically. Configuration (app name,
database URL) lives in `rxconfig.py`. Static assets in `assets/` are
served at `/filename`.

### Components

Every piece of UI is an `rx.*` call. Layout helpers (`rx.box`,
`rx.flex`, `rx.grid`, `rx.stack`, `rx.vstack`, `rx.hstack`,
`rx.center`) nest display and input components. Props are keyword
arguments; children are positional. Reflex ships Radix-based
primitives for inputs, feedback, navigation, and display.

### State management

Define state as a class inheriting from `rx.State`. State vars are
typed class attributes (`count: int = 0`) — the type annotation is
mandatory because Reflex uses it for serialization. Event handlers
are methods that modify `self`. Computed vars use `@rx.var`. Each
browser session gets its own state instance, and substates
(`class FormState(rx.State)`) keep large apps modular.

### Event handlers and binding

Attach handlers to component events: `rx.button("Add",
on_click=State.increment)`. Reflex auto-generates `set_<var>` for
every state var, enabling two-way binding like `rx.input(
value=State.query, on_change=State.set_query)`. Handlers can `yield`
between mutations to push intermediate UI updates. Long-running work
goes in `@rx.event(background=True)` handlers that use `async with
self:` to mutate state safely.

### Styling

Props drive styling: `rx.box(padding="4", bg="blue.500",
border_radius="md")`. Values come from Radix design tokens (color
scales `blue.1`–`blue.12`, spacing, radii). Responsive props take a
list that maps to `[mobile, tablet, desktop]`. Global styles live in
`rx.App(style={...})`. Dark mode uses `rx.color_mode.toggle` and
`rx.color_mode_cond()`.

### Routing

`@rx.page(route="/users/[user_id]")` declares a dynamic route;
parameters are accessed via `self.router.page.params["user_id"]`.
`rx.link("Home", href="/")` produces navigation links and
`rx.redirect("/dashboard")` returned from a handler performs a
programmatic navigation. Attach `on_load=State.load_data` to the page
decorator for per-visit initialization.

### Database

Reflex ships SQLModel integration: `class User(rx.Model, table=True):
...`. Query inside handlers via `with rx.session() as session: ...`.
`reflex db migrate` handles schema migrations. Default SQLite for
development; configure PostgreSQL in `rxconfig.py` for production.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Mutating state outside an event handler.** Reflex tracks state mutations only inside event handler methods. Changing a state var directly from a background task or external callback without `async with self:` silently drops the mutation — the UI never updates. All state mutations must go through event handlers or the `async with self:` context manager in background tasks.
- **Missing type annotations on state vars.** Reflex uses type annotations for serialization and code generation. A state var without a type annotation (`count = 0` instead of `count: int = 0`) causes runtime serialization errors or is silently ignored. Every state var requires a type annotation.
- **Non-serializable values in state.** Reflex must serialize state to JSON to sync it between backend and frontend. Storing a SQLAlchemy model instance, a file handle, or a datetime object without a JSON-compatible type in a state var causes silent failures or serialization errors. State vars must be JSON-serializable; derive and cache computed values via `@rx.var`.
- **Blocking the event loop with CPU or I/O work inside a regular event handler.** A slow synchronous event handler (network call, database query, heavy computation) blocks all other events for that session. Move blocking work into `@rx.event(background=True)` async handlers.
- **One giant global state class for the entire app.** All components subscribed to the same state class rebuild on any change. A single large state means unrelated UI panels rebuild together. Split state by feature domain into substates (`class CartState(rx.State)`, `class UserState(rx.State)`) to isolate rebuilds.
- **Not calling `yield` in a long-running synchronous handler.** A handler that runs for several seconds without `yield` blocks UI updates for the entire duration — the user sees a frozen screen. `yield` between steps to push intermediate state updates to the frontend.
- **Hardcoding route paths as strings in multiple places.** Route paths like `"/dashboard"` scattered across `rx.redirect(...)` calls and `@rx.page(route=...)` decorators become maintenance debt when routes change. Define route constants once and reference them everywhere.

## Full reference

### Layout components

```python
# Flex container (horizontal by default)
rx.flex(
    rx.box("Item 1"),
    rx.box("Item 2"),
    direction="row",      # "row" | "column"
    gap="4",
    align="center",       # cross-axis alignment
    justify="between",    # main-axis distribution
    wrap="wrap",
)

# Grid layout
rx.grid(
    rx.box("A"), rx.box("B"), rx.box("C"), rx.box("D"),
    columns="2",
    spacing="4",
)

# Stack shorthands
rx.vstack(rx.text("Top"), rx.text("Bottom"), spacing="3")
rx.hstack(rx.text("Left"), rx.text("Right"), spacing="3")

# Center content
rx.center(rx.spinner(), height="100vh")
```

### Display and input

```python
rx.text("Body text", size="3", color="gray.11")
rx.heading("Page Title", size="7", weight="bold")
rx.image(src="/logo.png", alt="Logo", width="200px")
rx.badge("NEW", color_scheme="green", variant="soft")
rx.code_block(code, language="python", show_line_numbers=True)

rx.card(
    rx.text("Card content"),
    rx.button("Action"),
    size="3",
)

rx.input(
    placeholder="Search...",
    value=State.query,
    on_change=State.set_query,
    size="3",
)

rx.text_area(value=State.content, on_change=State.set_content, rows="5")

rx.select(
    ["Option A", "Option B", "Option C"],
    value=State.selected,
    on_change=State.set_selected,
    placeholder="Choose one",
)

rx.checkbox("Accept terms", checked=State.accepted, on_change=State.set_accepted)
rx.switch(checked=State.dark_mode, on_change=State.toggle_dark)

rx.button("Primary", color_scheme="blue")
rx.button("Outline", variant="outline")
rx.button("Loading", loading=State.is_loading, on_click=State.submit)
```

### State patterns

```python
import reflex as rx

class AppState(rx.State):
    # Simple vars (set_<name> auto-generated)
    count: int = 0
    items: list[str] = []
    query: str = ""

    # Event handlers
    def increment(self):
        self.count += 1

    def add_item(self):
        if self.query:
            self.items.append(self.query)
            self.query = ""

    def remove_item(self, item: str):
        self.items = [i for i in self.items if i != item]

    # Computed var (read-only, auto-updates)
    @rx.var
    def item_count(self) -> int:
        return len(self.items)

    @rx.var
    def filtered_items(self) -> list[str]:
        if not self.query:
            return self.items
        return [i for i in self.items if self.query.lower() in i.lower()]
```

### Event handler patterns

```python
class FormState(rx.State):
    form_data: dict = {}

    # Handler with event data
    def handle_submit(self, data: dict):
        self.form_data = data

    # Yielding for progress updates
    def process(self):
        self.status = "Starting..."
        yield
        self.status = "Processing..."
        yield
        self.status = "Done"

    # Background task (non-blocking)
    @rx.event(background=True)
    async def fetch_data(self):
        import httpx
        async with httpx.AsyncClient() as client:
            resp = await client.get("https://api.example.com/data")
            async with self:
                self.data = resp.json()

    # Redirect after action
    def login(self):
        if self.authenticate():
            return rx.redirect("/dashboard")
```

### Conditional and dynamic rendering

```python
# Conditional rendering
rx.cond(
    State.logged_in,
    rx.text("Welcome back!"),
    rx.button("Log in", on_click=State.show_login),
)

# Rendering lists
rx.foreach(
    State.items,
    lambda item: rx.hstack(
        rx.text(item),
        rx.icon_button("x", on_click=State.remove_item(item)),
    ),
)

# Multi-branch
rx.match(
    State.status,
    ("loading", rx.spinner()),
    ("error", rx.text("Error!", color="red")),
    ("success", rx.text("Done!", color="green")),
    rx.text("Unknown"),  # default
)
```

### Pages and routing

```python
@rx.page(route="/", title="Home", on_load=AppState.load_data)
def index() -> rx.Component:
    return rx.box(
        rx.heading("Home"),
        rx.text("Welcome"),
    )

@rx.page(route="/users/[user_id]", title="User Profile")
def user_profile() -> rx.Component:
    return rx.box(
        rx.text(f"User: {AppState.router.page.params['user_id']}"),
    )

rx.link("Go Home", href="/")
rx.link(rx.button("Dashboard"), href="/dashboard")
```

### Form handling

```python
rx.form(
    rx.vstack(
        rx.input(name="email", placeholder="Email", required=True),
        rx.input(name="password", type="password", required=True),
        rx.button("Submit", type="submit"),
        spacing="3",
    ),
    on_submit=FormState.handle_submit,
    reset_on_submit=True,
)
```

### Deployment

- **Reflex Cloud**: `reflex deploy` for managed hosting
- **Self-host**: `reflex export --frontend-only` produces a static
  frontend plus a backend you run with `uvicorn`
- **Docker**: export, then serve the frontend via nginx and the
  backend via uvicorn
- Set `REFLEX_ENV=prod` for production optimizations

### Common mistakes

- **Mutating state outside event handlers** — all state mutation
  must happen inside a handler method
- **Missing type annotations on state vars** — Reflex needs them for
  serialization and code generation
- **Non-serializable values in state** — every state var must be
  JSON-serializable
- **Blocking the event loop** — push CPU/IO-heavy work into
  `@rx.event(background=True)` handlers
- **One giant global state** — split into substates by feature

### Further reading

- [Reflex Documentation](https://reflex.dev/docs/)
- [Component Library](https://reflex.dev/docs/library/)
