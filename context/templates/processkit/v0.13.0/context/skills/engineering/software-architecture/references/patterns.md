# Architecture Patterns Reference

The top 10 software architecture patterns, when to use each, and key trade-offs.

## 1. Layered (N-Tier)

Organizes code into horizontal layers (presentation, business logic, data access), each calling only the layer below it.

- **Use when:** Building traditional business applications, migrating legacy systems, or when the team is familiar with layered design.
- **Trade-offs:** Simple to understand and implement. Can lead to "sinkhole" anti-pattern where layers just pass data through. Changes often cut across multiple layers.
- **Example:** Presentation -> Service -> Repository -> Database

## 2. Hexagonal (Ports & Adapters)

Places domain logic at the center, surrounded by ports (interfaces) and adapters (implementations). External systems connect through adapters.

- **Use when:** You need testable, infrastructure-independent domain logic. Good for applications that integrate with multiple external systems.
- **Trade-offs:** Excellent testability and flexibility in swapping infrastructure. More boilerplate (port/adapter pairs). Requires discipline to keep dependencies pointing inward.
- **Key principle:** Domain never imports from infrastructure. Infrastructure implements domain-defined ports.

## 3. Clean Architecture

Concentric rings: Entities (innermost), Use Cases, Interface Adapters, Frameworks & Drivers (outermost). Dependencies point inward only.

- **Use when:** Complex business domains where domain logic must be protected from framework changes.
- **Trade-offs:** Clear separation of concerns, highly testable core. Can feel over-engineered for simple CRUD apps. Lots of mapping between layers.
- **Relation:** Shares core principles with hexagonal and onion architecture (dependency inversion, domain at center).

## 4. Microservices

Decomposes the system into small, independently deployable services, each owning its data and implementing a bounded context.

- **Use when:** Large teams needing independent deployment, complex domains with clear bounded contexts, high scalability requirements.
- **Trade-offs:** Independent scaling and deployment, technology diversity. Significant operational complexity (service discovery, distributed tracing, data consistency). Requires mature DevOps practices.

## 5. Event-Driven

Components communicate through events (publish/subscribe). Producers emit events without knowing consumers.

- **Use when:** Systems requiring real-time processing, loose coupling between components, or complex event flows (IoT, financial systems, notification pipelines).
- **Trade-offs:** Excellent decoupling and scalability. Harder to debug (event flows are implicit), eventual consistency challenges, potential event ordering issues.
- **Variants:** Event sourcing (store state as event stream), CQRS (separate read/write models).

## 6. CQRS (Command Query Responsibility Segregation)

Separates read operations (queries) from write operations (commands) into different models, potentially different data stores.

- **Use when:** Read and write workloads have very different scaling or complexity requirements. Often paired with event sourcing.
- **Trade-offs:** Optimized read/write paths, natural fit for event sourcing. Added complexity of maintaining two models, eventual consistency between read and write sides.

## 7. Pipe-and-Filter

Data flows through a series of processing stages (filters), connected by pipes. Each filter transforms its input and passes output to the next stage.

- **Use when:** Data processing pipelines, compiler/transpiler chains, ETL workflows, CLI tools that process streams.
- **Trade-offs:** Highly composable, easy to add/remove/reorder stages. Each filter must be independent. Not suitable for interactive or stateful processing.

## 8. Modular Monolith

A single deployable unit with strict module boundaries. Modules communicate through defined interfaces, not direct internal access.

- **Use when:** Starting a new project that may evolve into microservices, or when operational complexity of microservices is not justified.
- **Trade-offs:** Simpler deployment and operations than microservices, clearer boundaries than a traditional monolith. Requires discipline to enforce module boundaries. Can extract modules into services later.

## 9. Service-Oriented Architecture (SOA)

Organizes functionality into reusable services with well-defined contracts, often using an enterprise service bus (ESB) for communication.

- **Use when:** Enterprise environments integrating multiple legacy systems, organizations needing shared services across departments.
- **Trade-offs:** Promotes reuse across the organization. ESB can become a bottleneck and single point of failure. Heavier governance overhead than microservices.

## 10. Serverless / Function-as-a-Service

Individual functions triggered by events, managed entirely by the cloud provider. No server management.

- **Use when:** Event-driven workloads with variable traffic, rapid prototyping, glue code between cloud services.
- **Trade-offs:** Zero infrastructure management, pay-per-execution. Cold start latency, vendor lock-in, limited execution duration, harder to test locally.

---

## Pattern Selection Checklist

When choosing a pattern, evaluate:

1. **Team size and skill**: Smaller teams benefit from simpler patterns (layered, modular monolith)
2. **Domain complexity**: Complex domains need stronger boundaries (hexagonal, clean architecture)
3. **Scale requirements**: High scale favors microservices or event-driven
4. **Deployment frequency**: Frequent deploys favor microservices or serverless
5. **Data consistency needs**: Strong consistency favors monoliths; eventual consistency enables distribution
6. **Operational maturity**: Microservices and event-driven require mature DevOps
7. **Existing codebase**: Migrations favor incremental patterns (modular monolith as stepping stone)

## Further Reading

- Alistair Cockburn, "Hexagonal Architecture" (2005) - https://alistair.cockburn.us/hexagonal-architecture/
- Robert C. Martin, "Clean Architecture" (2017) - book
- Martin Fowler, "Patterns of Enterprise Application Architecture" (2002) - book
- Sam Newman, "Building Microservices" (2021, 2nd ed.) - book
- Simon Brown, "The C4 Model" - https://c4model.com/
- Michael Nygard, "Documenting Architecture Decisions" (2011) - https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions
- Microsoft Azure Architecture Center - https://learn.microsoft.com/en-us/azure/architecture/guide/architecture-styles/
