---
name: flutter-development
description: |
  Flutter and Dart development — widget architecture, state management, navigation, and cross-platform layout. Use when building Flutter apps, choosing a state-management approach, designing responsive layouts, or reviewing widget code.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-flutter-development
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Flutter Development

## Intro

In Flutter, everything is a widget — compose small `StatelessWidget`s
and `StatefulWidget`s into screens, prefer `const` constructors, and
keep `build()` methods lightweight. Pick the simplest state-management
solution that fits the complexity, not the most fashionable one.

## Overview

### Widget architecture

Use `StatelessWidget` when there's no mutable state and
`StatefulWidget` when the widget owns animations, controllers, or
form input. Extract sub-widgets into top-level classes (not helper
methods) so they can be `const` and rebuild independently. Compose by
wrapping, never by extending. Name widgets by purpose
(`UserProfileCard`), not by position or sequence.

### Layout system

`Row` and `Column` are the primary axis-based layouts; `Expanded` and
`Flexible` distribute space among children. `Stack` plus `Positioned`
handles overlap and absolute placement. Use `ListView.builder` /
`GridView.builder` for any list that may grow — they only build
visible items. `SizedBox` for fixed gaps, `Spacer` for flexible gaps,
`Padding`/`Container` for spacing and decoration. Reach for
`LayoutBuilder` and `MediaQuery` when the layout depends on the
viewport.

### State management

Pick the smallest tool that fits:

- Local widget state → `StatefulWidget` + `setState`
- Shared state across a few screens → Provider or Riverpod
- Complex async flows with many events → BLoC

Riverpod is type-safe and avoids `BuildContext` plumbing; BLoC enforces
event-driven structure for large apps. Avoid putting all state in a
single global provider — split by domain so unrelated screens don't
rebuild together.

### Navigation with GoRouter

Define routes declaratively and navigate with `context.go(...)` to
replace or `context.push(...)` to stack. Use `ShellRoute` /
`StatefulShellRoute` for tabbed layouts that need to preserve state
per tab. Authentication guards live in the `redirect` callback. Deep
linking works automatically once routes are declared.

### Theming

Configure theme on `MaterialApp` with
`ThemeData(useMaterial3: true, colorScheme: ColorScheme.fromSeed(...))`.
Read theme values via `Theme.of(context)` rather than hardcoding font
sizes or colors. Provide a matching `darkTheme` and let Flutter pick
based on platform brightness.

### Networking and data

Use `http` or `dio` for REST. Serialize JSON via `json_serializable`
and `build_runner` for type-safe parsing. Wrap APIs in repositories
(`UserRepository` over a raw client). In the UI, render
loading/error/data states explicitly with `FutureBuilder` or
`StreamBuilder` — never show a blank screen while waiting.

### Testing

Three layers: pure-Dart unit tests with `test()`, widget tests with
`testWidgets()` and `find.byType()`, and full integration tests with
the `integration_test` package. Use `pumpWidget` to mount and `pump`
to advance frames. Mock external boundaries with `mocktail` or
`mockito`. Use golden tests (`matchesGoldenFile`) for visual
regression on critical screens.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Nested `Scaffold` widgets.** A `Scaffold` inside another `Scaffold` produces double app bars, broken navigation behavior, and unpredictable layout. Each route gets exactly one `Scaffold` at the top level.
- **`setState` called after `dispose()`.** Calling `setState` on a widget that has been removed from the tree throws a `FlutterError` in debug mode and causes silent bugs in release mode. Guard all async callbacks with `if (!mounted) return;` before calling `setState`.
- **Forgetting to call `dispose()` on controllers, animations, and stream subscriptions.** Objects that own platform resources or listeners are not garbage-collected automatically. Not disposing them causes memory leaks and "stream already closed" errors. Override `dispose()` and clean up every controller and subscription.
- **Using helper methods instead of top-level `StatelessWidget` classes for sub-widgets.** A helper method returning a `Widget` rebuilds on every parent rebuild regardless of whether its inputs changed. A top-level widget class enables `const` instantiation and independent rebuild boundaries.
- **Unbounded `Text` or `Column` inside a `Row`.** A `Text` widget without a size constraint in a `Row` throws a "RenderFlex overflowed" error. Wrap with `Expanded` or `Flexible` to constrain it, or use `SizedBox` with a fixed width.
- **Blocking the event loop in a widget's `build()` method.** `build()` is called every frame. Any synchronous heavy computation inside it drops frames and causes jank. Move expensive work into `initState`, an async handler, or a compute isolate.
- **Using `ListView()` with all items materialized for a long list.** `ListView()` with a `children` list builds all items up front. Use `ListView.builder` so only the visible items are built; this scales to thousands of items.

## Full reference

### Layout widget recipes

```dart
// Row — horizontal layout
Row(
  mainAxisAlignment: MainAxisAlignment.spaceBetween,
  crossAxisAlignment: CrossAxisAlignment.center,
  children: [Text('Left'), Text('Right')],
)

// Column — vertical layout
Column(
  mainAxisSize: MainAxisSize.min,  // shrink-wrap
  children: [
    Text('Title'),
    SizedBox(height: 8),
    Text('Subtitle'),
  ],
)

// Stack — overlapping children
Stack(
  children: [
    Image.network(url),
    Positioned(
      bottom: 16, left: 16,
      child: Text('Overlay text'),
    ),
  ],
)

// Wrap — flow layout with wrapping
Wrap(
  spacing: 8, runSpacing: 4,
  children: tags.map((t) => Chip(label: Text(t))).toList(),
)
```

### Scrollable widgets

```dart
// ListView.builder — efficient for large lists
ListView.builder(
  itemCount: items.length,
  itemBuilder: (context, index) => ListTile(
    title: Text(items[index].name),
    subtitle: Text(items[index].description),
    leading: CircleAvatar(child: Text(items[index].initials)),
    trailing: Icon(Icons.chevron_right),
    onTap: () => context.push('/items/${items[index].id}'),
  ),
)

// GridView.builder
GridView.builder(
  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
    crossAxisCount: 2, crossAxisSpacing: 8, mainAxisSpacing: 8,
  ),
  itemCount: products.length,
  itemBuilder: (context, i) => ProductCard(products[i]),
)

// CustomScrollView with slivers
CustomScrollView(
  slivers: [
    SliverAppBar(floating: true, title: Text('Feed')),
    SliverList.builder(
      itemCount: posts.length,
      itemBuilder: (context, i) => PostCard(posts[i]),
    ),
  ],
)
```

### Form input

```dart
// TextField
TextField(
  controller: _controller,
  decoration: InputDecoration(
    labelText: 'Email',
    hintText: 'you@example.com',
    prefixIcon: Icon(Icons.email),
    border: OutlineInputBorder(),
    errorText: _hasError ? 'Invalid email' : null,
  ),
  keyboardType: TextInputType.emailAddress,
  onChanged: (value) => setState(() => _email = value),
)

// Form with validation
Form(
  key: _formKey,
  child: Column(children: [
    TextFormField(
      validator: (v) => v == null || v.isEmpty ? 'Required' : null,
    ),
    ElevatedButton(
      onPressed: () {
        if (_formKey.currentState!.validate()) { /* submit */ }
      },
      child: Text('Submit'),
    ),
  ]),
)
```

### Async builders

```dart
// FutureBuilder — one-shot async
FutureBuilder<User>(
  future: _userFuture,
  builder: (context, snapshot) {
    if (snapshot.connectionState == ConnectionState.waiting) {
      return CircularProgressIndicator();
    }
    if (snapshot.hasError) return Text('Error: ${snapshot.error}');
    final user = snapshot.data!;
    return Text(user.name);
  },
)

// StreamBuilder — continuous updates
StreamBuilder<int>(
  stream: _counterStream,
  builder: (context, snapshot) {
    return Text('Count: ${snapshot.data ?? 0}');
  },
)
```

### Common patterns

```dart
// Responsive layout
LayoutBuilder(
  builder: (context, constraints) {
    if (constraints.maxWidth > 600) {
      return Row(children: [Sidebar(), Expanded(child: Content())]);
    }
    return Content();  // mobile: no sidebar
  },
)

// Pull-to-refresh
RefreshIndicator(
  onRefresh: () async => await _loadData(),
  child: ListView.builder(...),
)

// Empty state
items.isEmpty
    ? Center(child: Text('No items yet'))
    : ListView.builder(
        itemCount: items.length,
        itemBuilder: (ctx, i) => ItemTile(items[i]),
      )
```

### Material 3 navigation

```dart
NavigationBar(
  selectedIndex: _currentIndex,
  onDestinationSelected: (i) => setState(() => _currentIndex = i),
  destinations: [
    NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
    NavigationDestination(icon: Icon(Icons.search), label: 'Search'),
    NavigationDestination(icon: Icon(Icons.person), label: 'Profile'),
  ],
)
```

### Platform channels

Prefer existing packages (`camera`, `geolocator`, `path_provider`)
over hand-rolled channels. When you must call native code, use
`MethodChannel` for one-off calls and `EventChannel` for streams.
Platform-specific source goes in `android/` and `ios/`. Use
`Platform.isIOS` / `Platform.isAndroid` for branching in Dart.

### Performance

- Use `const` widgets aggressively — they short-circuit rebuilds
- `ListView.builder` for any non-trivial list
- Avoid `setState` high in the tree; it rebuilds every descendant
- Wrap expensive painters in `RepaintBoundary` to isolate them
- Prefer `Opacity` only when necessary — it forces an off-screen
  layer
- Cache network images with `CachedNetworkImage`
- Profile rebuild counts and frame times in Flutter DevTools

### Common mistakes

- **Nested `Scaffold`s** — leads to double app bars and broken
  navigation
- **`setState` after dispose** — guard with `mounted`, cancel timers
  and stream subscriptions in `dispose()`
- **Unbounded children in `Row`/`Column`** — wrap with `Expanded` or
  `Flexible`
- **Forgetting `dispose()`** — controllers, animation controllers,
  and stream subscriptions all need explicit disposal
- **Hardcoded user-facing strings** — use `intl` or
  `easy_localization`

### Further reading

- [Flutter Documentation](https://docs.flutter.dev/)
- [Flutter Widget Catalog](https://docs.flutter.dev/ui/widgets)
