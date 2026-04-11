# Gang of Four Design Patterns Reference

The 23 GoF patterns organized by category, with modern usage notes. Patterns are rated by frequency of use in modern codebases: (high), (medium), (low).

## Creational Patterns

How objects are created.

### Factory Method (high)
- **Intent:** Define an interface for creating objects, letting subclasses decide which class to instantiate.
- **Modern use:** Constructor functions, `new_*` methods in Rust, `create_*` functions. Centralizes creation logic and hides concrete types.
- **Apply when:** Object creation involves conditional logic or configuration, or you want to decouple callers from concrete types.
- **Example:** `fn create_parser(format: Format) -> Box<dyn Parser>` instead of match statements at every call site.

### Abstract Factory (medium)
- **Intent:** Create families of related objects without specifying concrete classes.
- **Modern use:** Platform-specific UI toolkits, database driver families, test fixture factories.
- **Apply when:** You need to create groups of related objects that must be used together.

### Builder (high)
- **Intent:** Construct complex objects step-by-step, separating construction from representation.
- **Modern use:** Extremely common. `XxxBuilder` structs in Rust, fluent APIs in all languages, config builders.
- **Apply when:** Objects have many optional fields, or construction requires validation/sequencing.
- **Example:** `ServerBuilder::new().port(8080).tls(true).timeout(30).build()`

### Prototype (low)
- **Intent:** Create new objects by cloning existing instances.
- **Modern use:** `Clone` trait in Rust, `copy()`/`deepcopy()` in Python. Useful for object pools or templates.
- **Apply when:** Object creation is expensive, or you need variants of a base configuration.

### Singleton (low -- often an anti-pattern)
- **Intent:** Ensure a class has exactly one instance with global access.
- **Modern use:** Largely replaced by dependency injection. Still seen in loggers, configuration, thread pools.
- **Caution:** Makes testing difficult, hides dependencies, causes issues in concurrent code. Prefer passing instances explicitly.

## Structural Patterns

How objects are composed into larger structures.

### Adapter (high)
- **Intent:** Convert one interface to another that clients expect.
- **Modern use:** Wrapping third-party libraries behind your own trait/interface, integrating legacy code.
- **Apply when:** You need to use a class that does not match your expected interface. Essential in hexagonal architecture (adapters implement ports).
- **Example:** Wrapping an HTTP client library behind a `trait HttpClient` you define.

### Bridge (low)
- **Intent:** Decouple an abstraction from its implementation so both can vary independently.
- **Modern use:** Platform abstraction layers, driver architectures.
- **Apply when:** You have multiple dimensions of variation (e.g., shape x renderer, message x transport).

### Composite (medium)
- **Intent:** Compose objects into tree structures and treat individual objects and compositions uniformly.
- **Modern use:** File systems, UI component trees, AST nodes, recursive data structures.
- **Apply when:** You have a part-whole hierarchy where clients should treat leaves and branches the same.

### Decorator (high)
- **Intent:** Attach additional responsibilities to an object dynamically.
- **Modern use:** Middleware stacks (HTTP, logging, auth), Python decorators, Rust newtype wrappers with `Deref`.
- **Apply when:** You need to add behavior without modifying existing code (Open/Closed Principle).
- **Example:** `LoggingMiddleware(AuthMiddleware(Handler))` -- each layer adds behavior.

### Facade (medium)
- **Intent:** Provide a simplified interface to a complex subsystem.
- **Modern use:** API clients, library wrappers, service layers that hide internal complexity.
- **Apply when:** A subsystem has grown complex and callers only need a subset of functionality.

### Flyweight (low)
- **Intent:** Share common state between many objects to reduce memory usage.
- **Modern use:** String interning, icon/sprite caches, shared configuration objects.
- **Apply when:** You have thousands of similar objects where most state can be shared.

### Proxy (medium)
- **Intent:** Provide a surrogate that controls access to another object.
- **Modern use:** Lazy initialization, caching proxies, access control wrappers, smart pointers (`Rc`, `Arc` in Rust).
- **Apply when:** You need to add access control, caching, or lazy loading transparently.

## Behavioral Patterns

How objects communicate and distribute responsibility.

### Chain of Responsibility (medium)
- **Intent:** Pass a request along a chain of handlers until one handles it.
- **Modern use:** Middleware pipelines, event bubbling, validation chains.
- **Apply when:** Multiple objects might handle a request, and the handler is not known in advance.

### Command (medium)
- **Intent:** Encapsulate a request as an object, enabling parameterization, queuing, and undo.
- **Modern use:** Undo/redo systems, task queues, transaction logs, CLI subcommands.
- **Apply when:** You need to decouple "what to do" from "when to do it," or need undo support.

### Iterator (high)
- **Intent:** Provide a way to access elements of a collection sequentially without exposing its structure.
- **Modern use:** Built into every modern language (`for..in`, `.iter()`, generators). Rarely implemented manually.
- **Apply when:** Custom collections need standard traversal. Usually the language provides this.

### Mediator (medium)
- **Intent:** Define an object that encapsulates how a set of objects interact.
- **Modern use:** Event buses, message brokers, UI controllers that coordinate widgets.
- **Apply when:** Many objects interact in complex ways and you want to centralize the interaction logic.

### Memento (low)
- **Intent:** Capture and restore an object's internal state without violating encapsulation.
- **Modern use:** Undo systems, state snapshots, serialization checkpoints.
- **Apply when:** You need to save/restore state (often combined with Command pattern for undo).

### Observer (high)
- **Intent:** Define a one-to-many dependency so that when one object changes state, all dependents are notified.
- **Modern use:** Event emitters, reactive streams, UI data binding, pub/sub systems.
- **Apply when:** Changes in one object need to trigger updates in others without tight coupling.

### State (medium)
- **Intent:** Allow an object to alter its behavior when its internal state changes.
- **Modern use:** State machines, connection states (connected/disconnected), workflow engines.
- **Apply when:** An object's behavior depends on its state, and you have complex conditional logic around state transitions.
- **Example:** Replace `if state == Connected { ... } else if state == Disconnected { ... }` with state objects.

### Strategy (high)
- **Intent:** Define a family of algorithms, encapsulate each one, and make them interchangeable.
- **Modern use:** Sort comparators, serialization formats, authentication methods, pricing rules.
- **Apply when:** You have multiple algorithms for the same task and need to select one at runtime.
- **Example:** `trait Compressor { fn compress(&self, data: &[u8]) -> Vec<u8>; }` with Gzip, Zstd, LZ4 implementations.

### Template Method (medium)
- **Intent:** Define the skeleton of an algorithm in a base class, letting subclasses override specific steps.
- **Modern use:** Framework hooks, test fixtures (setup/teardown), processing pipelines with customizable steps.
- **Apply when:** Multiple classes share the same algorithm structure but differ in specific steps.

### Visitor (low)
- **Intent:** Separate an algorithm from the object structure it operates on.
- **Modern use:** AST walkers, serializers, code generators. In Rust, often replaced by `match` on enums.
- **Apply when:** You need to add many operations to a class hierarchy without modifying the classes.

---

## Pattern Selection Guide

Before introducing a GoF pattern, ask:

1. **Is there a concrete problem?** Never add patterns speculatively.
2. **Does it reduce complexity?** If the pattern adds more code than it removes, skip it.
3. **Is it idiomatic?** Prefer language-native solutions (Rust enums over Visitor, closures over Strategy objects, traits over abstract classes).
4. **Can the team understand it?** A pattern nobody understands adds no value.
5. **Rule of three:** Wait until you see a need in three places before abstracting.

## Common Smell-to-Pattern Mappings

| Code Smell | Candidate Pattern |
|---|---|
| Complex conditional logic on types | Strategy, State, or polymorphism |
| Growing switch/match statements | Factory Method + polymorphism |
| Duplicated algorithm with variations | Template Method |
| Complex object construction | Builder |
| Third-party integration coupling | Adapter |
| Need to add behavior to existing code | Decorator |
| Multiple objects reacting to events | Observer |
| Complex state-dependent behavior | State |
| Request needs undo/queue/log | Command |
| Many interacting objects | Mediator |

## References

- Erich Gamma, Richard Helm, Ralph Johnson, John Vlissides, "Design Patterns: Elements of Reusable Object-Oriented Software" (1994)
- https://refactoring.guru/design-patterns
- Joshua Kerievsky, "Refactoring to Patterns" (2004)
- https://sourcemaking.com/design_patterns
