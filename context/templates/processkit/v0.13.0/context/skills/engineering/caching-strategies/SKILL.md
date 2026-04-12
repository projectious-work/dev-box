---
name: caching-strategies
description: |
  Caching patterns — cache-aside, write-through, TTL strategies, invalidation, HTTP caching. Use when adding caching to reduce latency or DB load, choosing between caching patterns, configuring HTTP cache headers, debugging stale data or stampedes, or designing an invalidation strategy.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-caching-strategies
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Caching Strategies

## Intro

Pick a caching pattern from read/write ratio and consistency
requirements, bound staleness with TTLs, and accept that invalidation
is hard. Always set a TTL as a safety net even when you invalidate
explicitly. Protect popular keys from stampedes.

## Overview

### Choose the right pattern

**Cache-Aside (Lazy Loading)** — most common, application manages the
cache. On read, check cache; on miss, load from DB and populate. On
write, update DB and invalidate the cache (do not write through).
Best for read-heavy workloads tolerating brief staleness.

**Read-Through** — the cache library loads from the data source on
miss. Reduces application complexity by moving load logic into the
cache.

**Write-Through** — writes go through the cache to the data source
synchronously. Higher write latency but the cache is always fresh.
Best when downstream reads must never see a stale value.

**Write-Behind (Write-Back)** — the cache writes to the data source
asynchronously and returns immediately. Best for write-heavy
workloads, but you risk data loss if the cache fails before flush.

### TTL strategy

TTL is the primary tool for bounding staleness:

- **Short (1-60 s)** — rapidly changing data: stock prices, sessions
- **Medium (5-60 min)** — user profiles, product listings, API
  responses
- **Long (hours-days)** — static assets, configuration, reference
  data
- **No TTL (manual)** — data that changes only through known events

Apply jitter (e.g. ±10%) to TTLs so a wave of entries does not expire
together and stampede the database.

### Invalidation

The two hard problems in computer science: cache invalidation and
naming things. Three approaches:

1. **Event-driven** (preferred) — on write/update/delete, explicitly
   invalidate or update the cache entry. Use pub/sub or CDC to
   propagate across services.
2. **Tag-based** — tag entries by entity (`user:123`, `product:456`)
   and invalidate by tag.
3. **Versioned keys** — `user:{id}:v{version}`; bump version on
   change so old entries expire naturally via TTL.

Rules: prefer invalidation over update (avoids races); always set a
TTL as safety net; accept that distributed cache invalidation is
eventually consistent.

### HTTP caching

```
# Immutable static assets (hashed filenames)
Cache-Control: public, max-age=31536000, immutable

# API responses that vary by user
Cache-Control: private, max-age=0, must-revalidate
ETag: "a1b2c3"

# Shared content (CDN-cacheable)
Cache-Control: public, s-maxage=3600, max-age=60, stale-while-revalidate=300
```

Use ETag or Last-Modified for conditional revalidation — clients send
`If-None-Match` and the server returns `304 Not Modified` when
unchanged. Use `s-maxage` to control CDN TTL independently of the
browser TTL. Use `Surrogate-Key` for tag-based purging on
Fastly/Varnish.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Per-instance in-memory cache in a horizontal fleet.** Each replica holds a different cache state, so the same key returns different values depending on which instance handles the request. Either use a shared external cache or accept that per-instance caches are only valid for data that is safe to be inconsistent across replicas (e.g., non-critical local rate limiting).
- **Updating cached values instead of invalidating them.** A write path that updates the cache entry directly while the database write is in-flight creates a window where the cache holds a value that the database doesn't yet confirm. Invalidate on write; let the next read repopulate from the authoritative source.
- **No TTL with manual invalidation as the only expiry mechanism.** When invalidation logic is missed in any write path, stale data lives forever. Always set a TTL as a safety net — even a long one (hours or days) — so the worst case is bounded. Manual invalidation is a supplement, not a replacement.
- **Caching error responses.** Caching a 404 or 500 from an upstream service means a transient failure becomes permanent until the entry expires or is manually cleared. Never cache error responses; only cache successful results from authoritative sources.
- **No jitter on TTLs for a large key population.** If every key expires at the same second (e.g., all set with TTL 3600 at the same time), the resulting cache miss storm hits the origin simultaneously. Add per-key random jitter (±10–20%) to TTLs to spread expiry events.
- **Key collisions across environments or tenants.** Using bare entity IDs as cache keys (`user:123`) without a namespace prefix causes staging data to collide with production data, or one tenant's data to collide with another's. Always prefix keys with environment and tenant identifiers.
- **Not warming the cache before routing traffic.** A cold cache after a deployment or restart causes a thundering herd — all requests miss simultaneously and hammer the origin. Pre-warm critical keys before switching traffic, or use a staggered rollout that lets the cache fill naturally.

## Full reference

### Stampede prevention

When a popular key expires, hundreds of requests can hit the database
at once. Three mitigations:

**Locking (mutex)**:

```python
value = cache.get(key)
if value is None:
    if cache.acquire_lock(f"lock:{key}", timeout=5):
        value = db.query(key)
        cache.set(key, value, ttl=300)
        cache.release_lock(f"lock:{key}")
    else:
        value = cache.get(key)  # retry — another process is loading
```

**Probabilistic early expiration (XFetch)** — recompute before TTL
expires, with rising probability as expiry approaches:

```python
if time_remaining < ttl * 0.1 * random.random():
    recompute_and_cache(key)
```

**Stale-while-revalidate** — serve the stale value immediately while
refreshing in the background. Built into HTTP via the
`stale-while-revalidate` directive.

### Cache warming

Pre-populate caches before traffic hits. On deploy, warm the top-N
most accessed keys. On cache flush, run a warming script that replays
recent access patterns. For predictable access patterns, pre-compute
during off-peak hours. Do not warm everything — warm only keys with
high access frequency and expensive computation.

### Anti-patterns

- **Per-instance in-memory caches** in a horizontally scaled fleet —
  hit rates collapse and one instance can hold a stale value while
  another invalidates. Use a shared cache (Redis, Memcached) with
  consistent hashing.
- **Updating the cache from writers** — racy. Invalidate instead.
- **No TTL with manual invalidation** — one missed invalidation
  becomes a permanent bug.
- **Caching errors** — a downstream 500 cached for 10 minutes turns a
  blip into an outage. Cache only successful responses, or use a much
  shorter negative-cache TTL.
- **Cache key collisions** — always namespace keys by service and
  version.

### Operational checklist

Track hit rate per cache, hit rate per key family, eviction rate, and
P99 cache latency. Alert when hit rate drops sharply — that usually
means an upstream change broke key generation. Cap entry sizes; a
single huge value can blow out your cache memory budget.
