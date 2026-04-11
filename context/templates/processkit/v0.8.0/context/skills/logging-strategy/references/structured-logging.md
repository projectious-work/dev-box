# Structured Logging Reference

Language-specific patterns for structured JSON logging with common fields and aggregation.

## Common Fields

Every structured log entry should include these fields:

| Field       | Description                          | Example                                |
|-------------|--------------------------------------|----------------------------------------|
| `timestamp` | ISO 8601 with timezone               | `2026-03-22T14:30:00.123Z`            |
| `level`     | Log severity                         | `info`, `error`, `debug`               |
| `service`   | Service or application name          | `order-service`                        |
| `message`   | Human-readable description           | `Order created successfully`           |
| `trace_id`  | Distributed trace identifier         | `4bf92f3577b34da6a3ce929d0e0e4736`     |
| `span_id`   | Span within the trace                | `00f067aa0ba902b7`                     |
| `request_id`| Unique ID for the incoming request   | `req_01HXYZ...`                        |
| `duration_ms`| Operation duration in milliseconds  | `142`                                  |
| `error`     | Error type or message (if applicable)| `ConnectionTimeout`                    |

## Python — structlog

```python
import structlog, logging

structlog.configure(
    processors=[
        structlog.contextvars.merge_contextvars,
        structlog.processors.add_log_level,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.format_exc_info,
        structlog.processors.JSONRenderer(),
    ],
    wrapper_class=structlog.make_filtering_bound_logger(logging.INFO),
    logger_factory=structlog.PrintLoggerFactory(),
)

log = structlog.get_logger()
structlog.contextvars.bind_contextvars(request_id="req_abc123", service="order-service")
log.info("order.created", order_id="ord_456", amount=99.99)
```

## Node.js — pino

```javascript
const pino = require("pino");

const logger = pino({
  level: process.env.LOG_LEVEL || "info",
  timestamp: pino.stdTimeFunctions.isoTime,
  base: { service: "order-service" },
});

const reqLogger = logger.child({ request_id: "req_abc123" });
reqLogger.info({ order_id: "ord_456", amount: 99.99 }, "order.created");
```

## Go — slog (standard library, Go 1.21+)

```go
logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))
reqLogger := logger.With(slog.String("service", "order-service"), slog.String("request_id", "req_abc123"))
reqLogger.Info("order.created", slog.String("order_id", "ord_456"), slog.Float64("amount", 99.99))
```

## Rust — tracing + tracing-subscriber

```rust
use tracing::{info, span, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

tracing_subscriber::registry()
    .with(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
    .with(fmt::layer().json().with_target(false))
    .init();

let span = span!(Level::INFO, "request", request_id = "req_abc123", service = "order-service");
let _guard = span.enter();
info!(order_id = "ord_456", amount = 99.99, "order.created");
```

## ELK Stack Patterns

**Filebeat** ships logs to Elasticsearch. For JSON logs, minimal config:

```yaml
# filebeat.yml
filebeat.inputs:
  - type: container
    paths: ["/var/lib/docker/containers/*/*.log"]
    json.keys_under_root: true
    json.add_error_key: true

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "logs-%{[service]}-%{+yyyy.MM.dd}"
```

Key index patterns:
- `logs-*` — all services
- `logs-order-service-*` — single service
- Filter by `request_id` to trace a single request across services

## Grafana Loki Patterns

Loki uses labels for indexing and LogQL for querying. Common LogQL queries:

```
{service="order-service"} | json | level="error"          # All errors for a service
{service=~".+"} | json | request_id="req_abc123"           # Trace a request across services
{service="api-gateway"} | json | duration_ms > 1000        # Slow requests (> 1s)
sum(rate({service="order-service"} | json | level="error" [5m]))  # Error rate over time
```

## Naming Conventions

Use dot-delimited event names for consistent, searchable log messages:

- `http.request.received` / `http.response.sent` — HTTP lifecycle
- `db.query.executed` — database query completed
- `cache.hit` / `cache.miss` — cache operations
- `order.created` / `order.cancelled` — business events
- `auth.login.success` / `auth.login.failed` — authentication events
- `queue.message.published` / `queue.message.consumed` — message queue events
