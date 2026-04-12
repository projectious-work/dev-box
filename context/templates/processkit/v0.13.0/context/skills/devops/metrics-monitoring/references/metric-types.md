# Prometheus Metric Types Reference

Code examples, naming conventions, label best practices, and PromQL query patterns.

## Counter

A counter is a cumulative metric that only goes up (or resets to zero on restart). Use for: requests, errors, bytes transferred, tasks completed.

```python
from prometheus_client import Counter

REQUEST_COUNT = Counter("http_requests_total", "Total HTTP requests", ["method", "endpoint", "status"])
REQUEST_COUNT.labels(method="GET", endpoint="/api/orders", status="200").inc()
```

```go
var requestCount = promauto.NewCounterVec(prometheus.CounterOpts{
    Name: "http_requests_total", Help: "Total HTTP requests",
}, []string{"method", "endpoint", "status"})
requestCount.WithLabelValues("GET", "/api/orders", "200").Inc()
```

## Gauge

A gauge can go up and down. Use for: temperature, queue depth, active connections, in-flight requests.

```python
from prometheus_client import Gauge

IN_FLIGHT = Gauge("http_requests_in_flight", "Requests currently being processed")

with IN_FLIGHT.track_inprogress():
    handle_request()
```

```go
var inFlight = promauto.NewGauge(prometheus.GaugeOpts{
    Name: "http_requests_in_flight",
    Help: "Requests currently being processed",
})

inFlight.Inc()
defer inFlight.Dec()
```

## Histogram

Observations placed into configurable buckets. Use for: request duration, response size, batch size. Prefer histograms over summaries.

```python
from prometheus_client import Histogram

REQUEST_DURATION = Histogram(
    "http_request_duration_seconds",
    "HTTP request duration in seconds",
    ["method", "endpoint"],
    buckets=[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
)

# Use as decorator or context manager
with REQUEST_DURATION.labels(method="GET", endpoint="/api/orders").time():
    handle_request()
```

## Summary

Pre-calculates quantiles on the client side. Use only when you need exact quantiles and cannot use histograms (rare). Summaries cannot be aggregated across instances.

```python
from prometheus_client import Summary

REQUEST_DURATION = Summary("rpc_duration_seconds", "RPC duration", ["method"])
REQUEST_DURATION.labels(method="GetUser").observe(0.042)
```

## Naming Conventions

Follow Prometheus naming best practices:

| Convention                  | Good                                | Bad                          |
|-----------------------------|-------------------------------------|-------------------------------|
| Snake_case                  | `http_requests_total`               | `httpRequestsTotal`           |
| Unit suffix                 | `request_duration_seconds`          | `request_duration`            |
| `_total` for counters       | `http_requests_total`               | `http_requests`               |
| `_bytes` / `_seconds`       | `response_size_bytes`               | `response_size`               |
| No type in name             | `http_request_duration_seconds`     | `http_request_duration_histogram` |
| Prefix with domain          | `myapp_orders_created_total`        | `orders_created_total`        |

## Label Best Practices

- Keep cardinality low (< 10 values per label in practice)
- Never use user IDs, email addresses, or request paths as label values
- Use bounded categories: `status="2xx"` not `status="200"`, `status="201"`, ...
- Common labels: `method`, `endpoint`, `status`, `service`, `instance`
- Do NOT put metric metadata in labels (team, owner) — use recording rules or external labels

High-cardinality labels cause memory explosion in Prometheus. A metric with 3 labels of 10 values each = 1,000 time series. Add a 4th label with 100 values = 100,000 series.

## PromQL Query Patterns

### Request Rate
```promql
# Requests per second over 5 minutes
rate(http_requests_total[5m])

# Requests per second by endpoint
sum by (endpoint) (rate(http_requests_total[5m]))
```

### Error Rate
```promql
# Error percentage
sum(rate(http_requests_total{status=~"5.."}[5m]))
/
sum(rate(http_requests_total[5m]))
* 100
```

### Latency Percentiles (from histogram)
```promql
# p50 / p95 / p99 — change the first argument
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
```

### Saturation
```promql
# Connection pool utilization
myapp_db_connections_active / myapp_db_connections_max

# Queue saturation
myapp_queue_depth > 100
```

### SLO Burn Rate
```promql
# Error budget burn rate (1h window)
1 - (
  sum(rate(http_requests_total{status!~"5.."}[1h]))
  /
  sum(rate(http_requests_total[1h]))
)
/ (1 - 0.999)  # 99.9% SLO
```

## Grafana Dashboard Layout

Standard rows: (1) Overview — request rate, error rate %, p50/p95/p99 latency, (2) Errors — by endpoint and status code, (3) Saturation — CPU, memory, connection pools, queue depth, (4) Dependencies — RED metrics per downstream, (5) Business Metrics — domain-specific.

Panel types: `stat` for current values, `time series` for trends, `heatmap` for latency distribution. Always add deployment annotations.
