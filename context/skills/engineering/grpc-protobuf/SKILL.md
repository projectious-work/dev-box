---
name: grpc-protobuf
description: |
  Protobuf schema design and gRPC service patterns — streaming, error handling, backward compatibility. Use when designing .proto files, implementing gRPC services or clients, choosing a streaming pattern, handling gRPC errors, or evolving a schema without breaking consumers.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-grpc-protobuf
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# gRPC and Protocol Buffers

## Intro

Protobuf schemas are versioned, append-only contracts. gRPC services
use unary RPCs by default and streaming variants only when the
workload genuinely needs them. Field numbers are permanent, enum
zero values are `UNSPECIFIED`, and every RPC gets its own
Request/Response wrapper.

## Overview

### Schema basics

```protobuf
syntax = "proto3";
package myapp.orders.v1;
import "google/protobuf/timestamp.proto";

message Order {
  string order_id = 1;
  string customer_id = 2;
  repeated LineItem items = 3;
  OrderStatus status = 4;
  google.protobuf.Timestamp created_at = 5;
}

enum OrderStatus {
  ORDER_STATUS_UNSPECIFIED = 0;  // zero value = "not set"
  ORDER_STATUS_PENDING = 1;
  ORDER_STATUS_SHIPPED = 2;
}
```

- Always `syntax = "proto3"`. Package format: `<org>.<service>.<version>`.
- Enum zero value must be `UNSPECIFIED` so unset and "explicit zero"
  are distinguishable.
- Use `oneof` for mutually exclusive fields.
- Prefer the well-known types (`google.protobuf.Timestamp`,
  `Duration`, `FieldMask`) over hand-rolled equivalents.

See `references/proto-conventions.md` for the full naming rules,
field-numbering bands, and well-known type table.

### gRPC service patterns

| Pattern              | Use case                                       |
|----------------------|------------------------------------------------|
| Unary                | Standard request-response (CRUD, queries)      |
| Server streaming     | Real-time feeds, long-poll replacement         |
| Client streaming     | Bulk uploads, client-side event aggregation    |
| Bidirectional        | Chat, collaborative editing, real-time sync    |

```protobuf
service OrderService {
  rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
  rpc WatchStatus(WatchStatusRequest) returns (stream StatusEvent);
  rpc UploadItems(stream LineItem) returns (UploadItemsResponse);
  rpc Chat(stream ChatMessage) returns (stream ChatMessage);
}
```

### Request/response wrappers

Wrap every RPC in dedicated `XxxRequest` and `XxxResponse` messages.
Never reuse domain types (`Order`) directly as request or response.
The wrapper lets you add fields (pagination tokens, field masks,
metadata) without touching the domain model.

### Error handling

Return gRPC status codes — never return OK with an error payload.

| Code              | When to use                                |
|-------------------|--------------------------------------------|
| INVALID_ARGUMENT  | Bad client input (validation failure)      |
| NOT_FOUND         | Resource does not exist                    |
| ALREADY_EXISTS    | Duplicate key conflict                     |
| PERMISSION_DENIED | Authenticated but not authorized           |
| UNAUTHENTICATED   | Missing or invalid credentials             |
| UNAVAILABLE       | Transient failure — client should retry    |
| DEADLINE_EXCEEDED | Operation took too long                    |

For richer errors, attach `google.rpc.BadRequest` or `ErrorInfo` in
the `details` field of `google.rpc.Status`.

### Backward compatibility

- **Safe:** add new fields (with new numbers), messages, RPCs, or
  enum values.
- **Breaking:** change a field type or number, reuse a deleted field
  number, change enum numeric values, or remove an RPC.

When removing a field, mark it `reserved` so future authors can't
accidentally reuse the number or name:

```protobuf
message Order {
  reserved 6, 7;
  reserved "legacy_status";
}
```

### Tooling

Prefer **buf** over raw `protoc` — it bundles linting, breaking-change
detection, and code generation, and reads a single `buf.yaml`. Use
**grpcurl** for command-line invocation during debugging. Wire
`buf breaking` into CI to catch incompatible changes before merge.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Reusing a deleted field number.** Field numbers are permanent wire-format identifiers. Reusing a number that was previously assigned to a different field causes old binary-encoded messages to decode the field into the wrong destination. When removing a field, always mark it `reserved`.
- **Enum zero value not named `UNSPECIFIED`.** In proto3, the zero value is the default for any unset enum field. If zero is a real semantic value (e.g., `STATUS_PENDING = 0`), unset and "pending" are indistinguishable on the wire. The zero value must always be `<ENUM_NAME>_UNSPECIFIED`.
- **Using the same domain type as both request and response.** Passing an `Order` directly as a request body means adding request-specific metadata (pagination tokens, field masks, idempotency keys) requires modifying the domain type. Always wrap in `XxxRequest` / `XxxResponse` messages.
- **Returning status `OK` with an error payload in the response body.** Embedding error information inside an OK response breaks client error-handling libraries, monitoring dashboards, and retry logic. Return the appropriate gRPC status code and attach structured error detail using `google.rpc.Status`.
- **No `buf breaking` check in CI.** Without automated breaking-change detection, it is easy to accidentally change a field type, reuse a field number, or remove an RPC and not notice until a client breaks in production. Add `buf breaking --against` to CI.
- **Making every service bidirectionally streaming by default.** Bidirectional streaming holds a connection open for the entire duration of the stream, consuming server resources per client. Use unary RPCs by default and move to streaming only when the workload genuinely requires it.
- **Hand-rolling `int64` timestamps instead of using `google.protobuf.Timestamp`.** Custom timestamp encoding (epoch seconds, epoch milliseconds, strings) produces inconsistency across services and loses timezone awareness. Use the well-known `Timestamp` type and let the SDK handle serialization.

## Full reference

### Naming conventions

- **Files:** `lower_snake_case.proto`. One top-level service per
  file. Directory mirrors the package
  (`myapp/orders/v1/order_service.proto`).
- **Messages:** `UpperCamelCase` (`OrderLineItem`, `CreateUserRequest`).
- **Fields:** `lower_snake_case`, repeated fields use plural names
  (`repeated LineItem items = 3;`).
- **Enums:** type `UpperCamelCase`, values
  `<ENUM_NAME>_UPPER_SNAKE_CASE`. Zero is always `_UNSPECIFIED`.
- **Services:** `UpperCamelCase` + `Service` suffix (`OrderService`).
- **RPCs:** `UpperCamelCase` verb-noun (`CreateOrder`, `ListOrders`).

### Field-numbering rules

- **1-15** encode in one byte — reserve for the most frequent fields.
- **16-2047** encode in two bytes — everything else.
- **19000-19999** are reserved by protobuf itself; never use.
- Field numbers are permanent. They are part of the wire format and
  must never be changed or reused.
- Group field numbers in bands (core 1-10, items 11-19, metadata
  20-29) so future additions land in the right size class.

### Compatibility rules

| Change                     | Safe? | Notes                          |
|----------------------------|-------|--------------------------------|
| Add new field              | Yes   | Use a fresh field number       |
| Add new enum value         | Yes   | Old clients ignore unknown     |
| Add new RPC or message     | Yes   | No effect on existing clients  |
| Rename a field             | Yes   | Wire uses numbers, not names   |
| Change field type          | NO    | Wire encoding differs          |
| Change field number        | NO    | Data maps to wrong field       |
| Reuse deleted field number | NO    | Old data decodes wrong         |
| Change enum numeric value  | NO    | Stored values decode wrong     |
| Remove an RPC              | NO    | Clients get UNIMPLEMENTED      |

**Migration strategy** for a breaking change:

1. Add the new field alongside the old one with a new number.
2. Server populates both fields.
3. Migrate clients to read the new field.
4. Mark the old field `reserved` once usage is zero.
5. Run `buf breaking` in CI to enforce the rules.

### Well-known types

| Type             | Use for                                      |
|------------------|----------------------------------------------|
| `Timestamp`      | Points in time                               |
| `Duration`       | Time spans                                   |
| `Empty`          | RPCs with no request or response             |
| `FieldMask`      | Partial updates (which fields to write)      |
| `Any`            | Embedding arbitrary messages                 |
| `Struct`/`Value` | Untyped JSON-like data (use sparingly)       |

### Common patterns

**Pagination** (Google AIP-158 style):

```protobuf
message ListOrdersRequest {
  int32 page_size = 1;
  string page_token = 2;
}
message ListOrdersResponse {
  repeated Order orders = 1;
  string next_page_token = 2;
}
```

**Partial updates with FieldMask:**

```protobuf
message UpdateOrderRequest {
  Order order = 1;
  google.protobuf.FieldMask update_mask = 2;
}
```

The server modifies only fields listed in `update_mask`, which lets
clients send a full message without overwriting unrelated fields.

**Resource names** (AIP-122):

```protobuf
message GetOrderRequest {
  string name = 1;  // "projects/{project}/orders/{order}"
}
```

### Interceptors

Cross-cutting concerns belong in interceptors, chained in this order:

```
auth -> logging -> metrics -> validation -> handler
```

- **Auth** validates tokens and sets user context.
- **Logging** records method, duration, status code, request ID.
- **Metrics** emit latency histograms and error rates per method.
- **Validation** runs `buf validate` rules generated from proto
  annotations, so the handler never sees invalid input.

Client-side interceptors handle retries (only on `UNAVAILABLE` and
`DEADLINE_EXCEEDED`, with exponential backoff and a propagated
deadline), connection pooling, and tracing headers.

### Anti-patterns to avoid

- Reusing domain types as request/response messages.
- Skipping `_UNSPECIFIED` enum zero values.
- Returning errors in the response body with status OK.
- Reusing deleted field numbers (reserve them instead).
- Making libraries streaming-by-default — pick unary unless the
  workload genuinely needs streaming.
- Hand-rolling timestamps as `int64` epoch seconds when
  `google.protobuf.Timestamp` exists.
