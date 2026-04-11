# Protocol Buffers Conventions Reference

Quick reference for `.proto` file conventions, field numbering, compatibility rules,
and common patterns.

## Naming Conventions

### Files
- File names: `lower_snake_case.proto` (e.g., `order_service.proto`).
- One top-level service per file. Shared messages in separate files.
- Directory mirrors package: `myapp/orders/v1/order_service.proto`.

### Messages and Fields
- Messages: `UpperCamelCase` — `OrderLineItem`, `CreateUserRequest`.
- Fields: `lower_snake_case` — `order_id`, `created_at`.
- Repeated fields use plural names: `repeated LineItem items = 3;`.

### Enums
- Type: `UpperCamelCase` — `OrderStatus`.
- Values: `UPPER_SNAKE_CASE` prefixed with enum name — `ORDER_STATUS_PENDING`.
- Zero value: `<ENUM_NAME>_UNSPECIFIED = 0;`.

### Services and RPCs
- Service: `UpperCamelCase` + `Service` suffix — `OrderService`.
- RPCs: `UpperCamelCase` verb-noun — `CreateOrder`, `ListOrders`.
- Every RPC gets its own Request/Response messages.

### Packages
- Org-based with version: `myapp.orders.v1`.
- Map to directory structure for `buf`/`protoc` compatibility.

## Field Numbering Rules

- **1-15**: frequent fields (1-byte tag encoding).
- **16-2047**: less common fields (2-byte encoding).
- **19000-19999**: reserved by Protobuf — never use.
- Numbers are permanent — never change or reuse them.
- Mark deleted fields as `reserved`.

```protobuf
message Order {
  // Core (1-10)
  string order_id = 1;
  string customer_id = 2;
  OrderStatus status = 3;
  // Items (11-15)
  repeated LineItem items = 11;
  // Metadata (20-25)
  google.protobuf.Timestamp created_at = 20;
  // Reserved
  reserved 30, 31;
  reserved "legacy_tracking_id";
}
```

## Compatibility Rules

### Safe Changes (non-breaking)

| Change                     | Notes                                  |
|----------------------------|----------------------------------------|
| Add new field              | Use a new field number                 |
| Add new enum value         | Clients ignore unknown values          |
| Add new RPC or message     | No impact on existing clients          |
| Rename a field             | Wire format uses numbers, not names    |

### Unsafe Changes (breaking)

| Change                     | Why it breaks                          |
|----------------------------|----------------------------------------|
| Change field type          | Wire encoding differs                  |
| Change field number        | Data maps to wrong field               |
| Reuse deleted field number | Old data decodes into wrong field      |
| Change enum numeric values | Stored values decode wrong             |
| Remove an RPC              | Clients get UNIMPLEMENTED              |

### Migration Strategy
1. Add new field alongside old one.
2. Server populates both fields.
3. Migrate clients to new field.
4. Mark old field `reserved` after removal.
5. Run `buf breaking` in CI.

## Well-Known Types

| Type            | Use for                                    |
|-----------------|--------------------------------------------|
| `Timestamp`     | Points in time                             |
| `Duration`      | Time spans                                 |
| `Empty`         | RPCs with no request/response              |
| `FieldMask`     | Partial updates (specify which fields)     |
| `Any`           | Embedding arbitrary messages               |
| `Struct`/`Value`| Untyped JSON-like data (use sparingly)     |

## Common Patterns

### Pagination
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

### Partial Updates with FieldMask
```protobuf
message UpdateOrderRequest {
  Order order = 1;
  google.protobuf.FieldMask update_mask = 2;
}
```
Server modifies only fields listed in `update_mask`.

### Resource Names (AIP pattern)
```protobuf
message GetOrderRequest {
  string name = 1;  // "projects/{project}/orders/{order}"
}
```
