---
name: microservice-creation
description: |
  Designs and scaffolds new microservices — covering service boundary
  identification, inter-service communication patterns, containerisation,
  and orchestration. Use when asked to create a new service, extract a
  microservice from a monolith, design a service boundary, choose a
  communication pattern (REST vs gRPC vs events), or set up container
  orchestration for a multi-service system.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-microservice-creation
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: engineering
    uses:
      - skill: software-modularization
        purpose: Apply module boundary heuristics to identify where the service boundary belongs before scaffolding.
      - skill: container-orchestration
        purpose: Configure docker-compose or Kubernetes manifests for the new service.
---

# Microservice Creation

## Intro

A microservice is a module deployed as its own process, with its own
network address, its own data store, and its own release lifecycle. The
decision to create a new service has a higher cost than creating a new
module inside an existing service — it introduces network latency,
distributed failure modes, and operational complexity. Make the decision
deliberately; use `software-modularization` first to confirm the boundary
is right.

## Overview

### When to create a new service (and when not to)

**Create a new service when:**
- The component has a meaningfully different release cadence than the rest
  of the system
- The component needs to scale independently (different resource profile)
- The component has a different trust boundary (security isolation required)
- The team owning the component is genuinely independent and has no shared
  release coordination with other teams
- The component must be implemented in a different language or runtime

**Do NOT create a new service just because:**
- The module is large (decompose it first, within the monolith)
- It would be "cleaner" to have it separate (cleanliness has network cost)
- You want to try a new technology (try it in a module first)
- The current architecture is a monolith and microservices are fashionable

**The rule of thumb:** if you cannot articulate the independent deployment,
scaling, or isolation benefit in one sentence, the boundary is not ready.

### Service boundary identification

Before writing any code:

1. **Name the domain** — what business concept does this service own?
   "User authentication" and "Billing" are good domain names. "Data processing"
   and "Backend" are not — they are too broad.
2. **Define ownership** — what data does this service own? Data owned by a
   service must not be read directly by other services (only via the API).
   If two services share a database table, they are the same service.
3. **Define the API contract** — what does this service expose? Write the
   API contract (OpenAPI / protobuf / AsyncAPI) before writing any
   implementation. The contract is the service.
4. **Identify consumers** — which other services or clients will call this?
   If the answer is "everything," the service may be too fundamental to
   extract safely.

### Inter-service communication patterns

Choose the pattern that matches the interaction semantics:

| Pattern | Use when | Tradeoffs |
|---|---|---|
| **REST (HTTP/JSON)** | Request-response; human-readable; browser clients | Synchronous coupling; retries needed for reliability |
| **gRPC (protobuf)** | High-throughput; typed contracts; service-to-service | Binary protocol; harder to debug; needs proto tooling |
| **Async events (queue/stream)** | Fire-and-forget; pub-sub; eventual consistency OK | Decoupled; harder to trace; requires idempotency |
| **GraphQL** | Flexible queries; multiple consumers with different field needs | Complex server; N+1 risk; overkill for service-to-service |

**Decision rule:**
- If the caller needs an immediate response → REST or gRPC
- If the caller does not need to wait → async events
- If the contract is typed and high-volume → gRPC over REST
- If external/browser clients are involved → REST

### Scaffolding a new service

Minimum scaffold for a containerised microservice:

```
services/<service-name>/
  src/              ← application code
  tests/            ← unit and integration tests
  Dockerfile        ← production image
  pyproject.toml    ← (or package.json, Cargo.toml, go.mod)
  README.md         ← service purpose, API summary, local dev instructions
  openapi.yaml      ← (or .proto file if gRPC)
```

Adding to the project:

```
docker-compose.yml  ← add service entry with health check
context/services/   ← add SERVICE-<name>.md entity for processkit tracking
```

### Containerisation essentials

Every service container must have:

- **Explicit base image tag** — never `FROM python:latest`; pin to
  `python:3.12-slim` or a digest
- **Non-root user** — `RUN adduser --system appuser && USER appuser`
- **Health check** — `HEALTHCHECK CMD curl -f http://localhost:8080/health`
- **Read-only filesystem** where possible — `docker run --read-only`
- **Resource limits** — set in docker-compose or Kubernetes manifests

### Orchestration basics

**docker-compose (local development + small deployments):**

```yaml
services:
  auth:
    build: ./services/auth
    ports: ["8081:8080"]
    environment:
      DATABASE_URL: postgres://...
    depends_on:
      db:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      retries: 3
```

**Kubernetes (production):**
- One `Deployment` per service (with `replicas`, `resources`, `livenessProbe`)
- One `Service` (ClusterIP for internal; LoadBalancer/Ingress for external)
- One `ConfigMap` for non-secret config; `Secret` for credentials
- `HorizontalPodAutoscaler` if independent scaling is the reason for extraction

See `container-orchestration` skill for full Kubernetes patterns.

## Gotchas

- **Extracting a service before the module boundary is clean.** A microservice
  built on top of a tangled module boundary imports the coupling into the
  network layer, where it is far harder to fix. Always run the module
  boundary checklist from `software-modularization` first. If the module
  is not clean inside the monolith, the service will not be clean either.
- **Sharing a database between services.** If two services read and write the
  same database table, they are a distributed monolith — all the operational
  complexity of microservices with none of the independence. Each service must
  own its data exclusively; other services access it only via the API.
- **Choosing synchronous communication for operations that don't need it.**
  Every synchronous call is a dependency: if the downstream service is slow
  or down, the upstream service is slow or down too. For operations where the
  caller does not need an immediate result (sending an email, updating search
  index, generating a report), use async events.
- **No health check endpoint.** A service without a health check cannot be
  safely orchestrated — the orchestrator cannot distinguish a starting service
  from a crashed one. Every service must expose a `/health` or `/healthz`
  endpoint that returns 200 when ready to serve traffic.
- **Forgetting idempotency for async consumers.** Message queues deliver
  at-least-once. A consumer that processes a payment, sends an email, or
  writes a database record must handle duplicate delivery — the second
  delivery must produce the same result as the first. Design for it from the
  start; retrofitting idempotency is painful.
- **No distributed tracing from day one.** Once there are two services, a
  request spans multiple processes. Without a correlation ID propagated
  through every call, debugging a failure means reading logs from multiple
  services with no way to connect them. Add `X-Trace-ID` (or OpenTelemetry)
  before the first cross-service call.
- **Not writing the API contract before the implementation.** A service whose
  API is defined by its implementation (rather than a contract the
  implementation satisfies) will drift as the implementation changes. Write
  the OpenAPI spec or proto file first; implement against it. Contract-first
  design forces clarity about what the service actually promises to do.

## Full reference

### Service readiness checklist

Before extracting a service from the monolith:

- [ ] Domain name defined (one business concept)
- [ ] Data ownership defined (no shared tables)
- [ ] API contract written (OpenAPI / proto) before implementation
- [ ] Consumers identified and have agreed to the contract
- [ ] Independent deployment case articulated (not just "it would be cleaner")
- [ ] Health check endpoint planned
- [ ] Tracing / correlation ID propagation planned
- [ ] Local dev story in docker-compose

### Communication pattern decision tree

```
Caller needs immediate response?
  Yes → gRPC (typed, high volume) or REST (human-readable, browser clients)
  No  → Async events (queue / stream)

Multiple consumers with different field needs?
  Yes → consider GraphQL (but only for external APIs)
  No  → REST or gRPC

Performance-critical, service-to-service only?
  Yes → gRPC
  No  → REST
```

### Minimum viable Dockerfile

```dockerfile
FROM python:3.12-slim

WORKDIR /app
RUN adduser --system --no-create-home appuser

COPY pyproject.toml ./
RUN pip install --no-cache-dir .

COPY src/ ./src/

USER appuser
EXPOSE 8080
HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1
CMD ["python", "-m", "src.main"]
```
