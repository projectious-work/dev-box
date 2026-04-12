# Flutter Widget Catalog

## Layout Widgets

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

## Scrollable Widgets

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

## Input Widgets

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

// DropdownMenu (Material 3)
DropdownMenu<String>(
  initialSelection: 'Option A',
  dropdownMenuEntries: ['Option A', 'Option B', 'Option C']
      .map((e) => DropdownMenuEntry(value: e, label: e))
      .toList(),
  onSelected: (value) => setState(() => _selected = value),
)
```

## Display Widgets

```dart
// Card with content
Card(
  elevation: 2,
  child: Padding(
    padding: EdgeInsets.all(16),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Title', style: Theme.of(context).textTheme.titleMedium),
        SizedBox(height: 8),
        Text('Body text here'),
      ],
    ),
  ),
)

// Image with placeholder and error
Image.network(
  url,
  fit: BoxFit.cover,
  loadingBuilder: (context, child, progress) =>
      progress == null ? child : CircularProgressIndicator(),
  errorBuilder: (context, error, stack) => Icon(Icons.broken_image),
)

// Badge
Badge(
  label: Text('3'),
  child: Icon(Icons.notifications),
)
```

## Navigation Widgets

```dart
// NavigationBar (Material 3 bottom nav)
NavigationBar(
  selectedIndex: _currentIndex,
  onDestinationSelected: (i) => setState(() => _currentIndex = i),
  destinations: [
    NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
    NavigationDestination(icon: Icon(Icons.search), label: 'Search'),
    NavigationDestination(icon: Icon(Icons.person), label: 'Profile'),
  ],
)

// NavigationDrawer
NavigationDrawer(
  selectedIndex: _index,
  onDestinationSelected: (i) => setState(() => _index = i),
  children: [
    NavigationDrawerDestination(icon: Icon(Icons.inbox), label: Text('Inbox')),
    NavigationDrawerDestination(icon: Icon(Icons.send), label: Text('Sent')),
  ],
)

// TabBar + TabBarView
TabBar(
  controller: _tabController,
  tabs: [Tab(text: 'Tab 1'), Tab(text: 'Tab 2')],
)
TabBarView(
  controller: _tabController,
  children: [Page1(), Page2()],
)
```

## Async Widgets

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

## Common Patterns

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
