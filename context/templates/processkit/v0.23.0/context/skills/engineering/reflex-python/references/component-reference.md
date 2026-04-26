# Reflex Component Reference

## Layout Components

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

# Stack (vertical flex shorthand)
rx.vstack(rx.text("Top"), rx.text("Bottom"), spacing="3")
rx.hstack(rx.text("Left"), rx.text("Right"), spacing="3")

# Center content
rx.center(rx.spinner(), height="100vh")
```

## Common Display Components

```python
# Text and headings
rx.text("Body text", size="3", color="gray.11")
rx.heading("Page Title", size="7", weight="bold")

# Image
rx.image(src="/logo.png", alt="Logo", width="200px")

# Badge and code
rx.badge("NEW", color_scheme="green", variant="soft")
rx.code_block(code, language="python", show_line_numbers=True)

# Card
rx.card(
    rx.text("Card content"),
    rx.button("Action"),
    size="3",
)
```

## Input Components

```python
# Text input with two-way binding
rx.input(
    placeholder="Search...",
    value=State.query,
    on_change=State.set_query,
    size="3",
)

# Text area
rx.text_area(
    value=State.content,
    on_change=State.set_content,
    rows="5",
)

# Select dropdown
rx.select(
    ["Option A", "Option B", "Option C"],
    value=State.selected,
    on_change=State.set_selected,
    placeholder="Choose one",
)

# Checkbox and switch
rx.checkbox("Accept terms", checked=State.accepted, on_change=State.set_accepted)
rx.switch(checked=State.dark_mode, on_change=State.toggle_dark)

# Button variants
rx.button("Primary", color_scheme="blue")
rx.button("Outline", variant="outline")
rx.button("Loading", loading=State.is_loading, on_click=State.submit)
```

## State Patterns

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

## Event Handler Patterns

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

## Conditional and Dynamic Rendering

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

# Match (multi-branch conditional)
rx.match(
    State.status,
    ("loading", rx.spinner()),
    ("error", rx.text("Error!", color="red")),
    ("success", rx.text("Done!", color="green")),
    rx.text("Unknown"),  # default
)
```

## Page and Routing

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

# Navigation
rx.link("Go Home", href="/")
rx.link(rx.button("Dashboard"), href="/dashboard")
```

## Form Handling

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
