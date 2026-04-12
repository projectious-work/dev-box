# REST Conventions Reference

## Resource Naming Conventions

### Rules

1. **Plural nouns** for collections: `/users`, `/products`, `/orders`.
2. **Singular identifiers** in path params: `/users/{user_id}`.
3. **Kebab-case** for multi-word segments: `/line-items`, `/payment-methods`.
4. **Lowercase only** — never mix cases in URIs.
5. **No trailing slashes** — `/users` not `/users/`.
6. **No file extensions** — use `Accept` headers instead of `.json`.
7. **Max 3 nesting levels**: `/users/{id}/orders/{order_id}/items`.

### Naming Patterns

```
# Collections
GET  /v1/users
GET  /v1/products

# Single resource
GET  /v1/users/{user_id}

# Sub-resources (ownership relationship)
GET  /v1/users/{user_id}/orders
POST /v1/users/{user_id}/orders

# Actions (when CRUD doesn't fit)
POST /v1/orders/{order_id}/cancel
POST /v1/users/{user_id}/verify-email

# Search (cross-resource queries)
GET  /v1/search/products?q=keyboard&category=electronics
```

### Anti-Patterns to Avoid

```
# Verbs in paths
GET  /v1/getUsers           -> GET  /v1/users
POST /v1/createOrder        -> POST /v1/orders
POST /v1/deleteUser/{id}    -> DELETE /v1/users/{id}

# Singular collections
GET  /v1/user               -> GET  /v1/users

# Deep nesting (>3 levels)
GET  /v1/users/{id}/orders/{oid}/items/{iid}/reviews
-> GET /v1/order-items/{iid}/reviews
```

## HTTP Method Mapping

| Method  | Semantics       | Idempotent | Safe | Request Body | Typical Use            |
|---------|-----------------|------------|------|--------------|------------------------|
| GET     | Read            | Yes        | Yes  | No           | Fetch resource(s)      |
| POST    | Create / Action | No         | No   | Yes          | Create resource, RPC   |
| PUT     | Full replace    | Yes        | No   | Yes          | Replace entire resource|
| PATCH   | Partial update  | No*        | No   | Yes          | Update specific fields |
| DELETE  | Remove          | Yes        | No   | No           | Delete resource        |
| HEAD    | Metadata        | Yes        | Yes  | No           | Check existence        |
| OPTIONS | Capabilities    | Yes        | Yes  | No           | CORS preflight         |

*PATCH can be made idempotent with JSON Merge Patch (RFC 7396).

## Status Code Reference

### Success (2xx)

| Code | Name        | When to Use                                      |
|------|-------------|--------------------------------------------------|
| 200  | OK          | GET, PUT, PATCH with response body               |
| 201  | Created     | POST that creates a resource (include Location)  |
| 202  | Accepted    | Async operation started, not yet complete         |
| 204  | No Content  | DELETE success, PUT/PATCH with no response body   |

### Client Errors (4xx)

| Code | Name                  | When to Use                               |
|------|-----------------------|-------------------------------------------|
| 400  | Bad Request           | Malformed syntax, invalid JSON            |
| 401  | Unauthorized          | Missing or invalid authentication         |
| 403  | Forbidden             | Authenticated but lacks permission        |
| 404  | Not Found             | Resource does not exist                   |
| 405  | Method Not Allowed    | HTTP method not supported on endpoint     |
| 409  | Conflict              | State conflict (duplicate, version clash) |
| 422  | Unprocessable Entity  | Valid syntax but semantic errors           |
| 429  | Too Many Requests     | Rate limit exceeded                       |

### Server Errors (5xx)

| Code | Name                  | When to Use                               |
|------|-----------------------|-------------------------------------------|
| 500  | Internal Server Error | Unhandled exception                       |
| 502  | Bad Gateway           | Upstream service returned invalid response|
| 503  | Service Unavailable   | Overloaded or maintenance                 |
| 504  | Gateway Timeout       | Upstream service timed out                |

## Pagination Patterns

### Cursor-Based (Preferred)

Best for large datasets, real-time data, consistent results during writes.

```
GET /v1/orders?limit=25&cursor=eyJpZCI6MTAwfQ

Response:
{
  "data": [...],
  "pagination": {
    "next_cursor": "eyJpZCI6MTI1fQ",
    "prev_cursor": "eyJpZCI6NzZ9",
    "has_more": true
  }
}
```

The cursor is an opaque, base64-encoded token. Clients must not parse it.

### Offset-Based

Simpler but suffers from drift on writes. Use only for small, stable datasets.

```
GET /v1/products?limit=25&offset=50

Response:
{
  "data": [...],
  "pagination": {
    "total": 342,
    "limit": 25,
    "offset": 50
  }
}
```

## Error Response Format

Standard error envelope — use consistently across all endpoints:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "One or more fields failed validation.",
    "details": [
      {
        "field": "email",
        "reason": "required",
        "message": "Email address is required."
      },
      {
        "field": "age",
        "reason": "minimum",
        "message": "Age must be at least 18.",
        "metadata": { "minimum": 18, "actual": 15 }
      }
    ],
    "request_id": "req_abc123"
  }
}
```

### Common Error Codes

| Code                  | HTTP Status | Description                     |
|-----------------------|-------------|---------------------------------|
| VALIDATION_ERROR      | 400 / 422   | Field validation failed         |
| AUTHENTICATION_ERROR  | 401         | Invalid or missing credentials  |
| AUTHORIZATION_ERROR   | 403         | Insufficient permissions        |
| NOT_FOUND             | 404         | Resource does not exist         |
| CONFLICT              | 409         | Resource state conflict         |
| RATE_LIMITED          | 429         | Too many requests               |
| INTERNAL_ERROR        | 500         | Unexpected server error         |
