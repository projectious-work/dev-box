# Estimation Cheatsheet

Quick reference for back-of-envelope calculations in system design.

## Powers of 2

| Power | Value | Approx | Common Usage |
|-------|-------|--------|--------------|
| 2^10  | 1,024 | 1 Thousand | 1 KB |
| 2^20  | 1,048,576 | 1 Million | 1 MB |
| 2^30  | 1,073,741,824 | 1 Billion | 1 GB |
| 2^40  | ~1.1 Trillion | 1 Trillion | 1 TB |

Useful shortcuts: 1 million seconds ~ 12 days. 1 billion seconds ~ 32 years.

## Latency Numbers Every Engineer Should Know

| Operation | Time |
|-----------|------|
| L1 cache reference | 1 ns |
| L2 cache reference | 4 ns |
| Main memory reference | 100 ns |
| SSD random read | 16 us |
| HDD random read | 2 ms |
| Read 1 MB sequentially from SSD | 50 us |
| Read 1 MB sequentially from HDD | 825 us |
| Read 1 MB sequentially from memory | 3 us |
| Round trip within same datacenter | 500 us |
| Round trip CA to Netherlands | 150 ms |
| Mutex lock/unlock | 17 ns |

Key takeaways:
- Memory is 100x faster than SSD for random access
- SSD is 100x faster than HDD for random access
- Network within datacenter ~ 0.5 ms
- Cross-continent network ~ 150 ms

## DAU to QPS Conversion

Formula: `QPS = DAU * actions_per_user_per_day / 86400`

| DAU | Actions/User/Day | QPS (avg) | QPS (peak, 3x) |
|-----|-------------------|-----------|-----------------|
| 1M | 10 | ~115 | ~350 |
| 10M | 10 | ~1,150 | ~3,500 |
| 100M | 10 | ~11,500 | ~35,000 |
| 1B | 10 | ~115,000 | ~350,000 |

Rule of thumb: 1M DAU with 10 actions/day ~ 100 QPS average, 300-500 QPS peak.

## Storage Estimation

| Data Type | Typical Size |
|-----------|-------------|
| User profile (text fields) | 1-5 KB |
| Tweet/short post | 250 bytes - 1 KB |
| Chat message | 100-500 bytes |
| Photo (compressed) | 200 KB - 2 MB |
| Video (1 min, compressed) | 5-50 MB |
| Log entry | 200-500 bytes |
| Database row (typical) | 500 bytes - 2 KB |

Storage formula: `total = records_per_day * avg_size * retention_days`

Example: 10M messages/day * 500 bytes * 365 days * 5 years = ~9 TB

## Bandwidth Estimation

Formula: `bandwidth = QPS * avg_request_or_response_size`

Example: 10K QPS * 5 KB avg response = 50 MB/s = 400 Mbps outbound

## Common Capacity Patterns

**1M DAU social app:**
- ~100 QPS avg, ~500 QPS peak
- ~10 GB/day new data (posts, metadata)
- ~5 TB storage over 2 years
- 2-5 application servers, 1-2 DB servers, Redis cache

**100M DAU messaging app:**
- ~50K QPS avg, ~200K QPS peak
- ~500 GB/day new messages
- ~500 TB storage over 3 years
- Sharded DB cluster, dedicated cache layer, WebSocket gateway fleet

**1B DAU content platform (read-heavy):**
- ~500K QPS avg, ~2M QPS peak
- Multi-region CDN mandatory
- Read replicas + heavy caching (95%+ cache hit rate)
- Terabytes of bandwidth per day

## Server Capacity Rules of Thumb

| Resource | Typical Limit |
|----------|--------------|
| Single web server (simple API) | 1K-10K QPS |
| Single Redis instance | 100K+ ops/s |
| Single PostgreSQL (tuned) | 5K-20K QPS (depends on query complexity) |
| Single Kafka broker | 100K+ messages/s |
| Single machine network | 1-10 Gbps |
| Single machine connections | 10K-100K concurrent |

## Quick Estimation Framework

1. Start with DAU and read:write ratio
2. Calculate average and peak QPS
3. Estimate storage per year
4. Estimate bandwidth (QPS * size)
5. Divide QPS by per-server capacity to get instance count
6. Add 2-3x headroom for growth and spikes
7. Consider geographic distribution needs
