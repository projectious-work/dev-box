---
name: sql-patterns
description: |
  SQL query patterns, schema design, and optimization — joins, CTEs, window functions, indexing, and anti-patterns. Use when writing or optimizing SQL queries, designing or reviewing a relational schema, reading EXPLAIN plans, choosing indexes, or implementing pagination, deduplication, or analytical queries.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-sql-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# SQL Patterns

## Intro

SQL rewards thinking in sets and reading execution plans. Build
queries incrementally from the core logic outward, lean on CTEs and
window functions for clarity, and verify cost with `EXPLAIN` before
optimizing anything.

## Overview

### Choose the right join

- `INNER JOIN` — only matching rows in both tables (default, most
  common).
- `LEFT JOIN` — all rows from the left table, NULLs where right has
  no match.
- `RIGHT JOIN` — rare; rewrite as `LEFT JOIN` for clarity.
- `FULL OUTER JOIN` — all rows from both, NULLs on the unmatched
  side.
- `CROSS JOIN` — cartesian product; use intentionally for generating
  combinations.

### CTEs over nested subqueries

CTEs (`WITH` clauses) read top-to-bottom and let each step name its
intent. Prefer them over nested subqueries when a query has more
than two levels.

```sql
WITH active_users AS (
    SELECT user_id, MAX(login_at) AS last_login
    FROM logins
    WHERE login_at > CURRENT_DATE - INTERVAL '30 days'
    GROUP BY user_id
),
user_orders AS (
    SELECT user_id, COUNT(*) AS order_count, SUM(total) AS revenue
    FROM orders
    GROUP BY user_id
)
SELECT u.name, au.last_login, uo.order_count, uo.revenue
FROM users u
JOIN active_users au USING (user_id)
LEFT JOIN user_orders uo USING (user_id);
```

### Window functions

Use windows for rankings, running totals, and per-row comparisons
without collapsing rows:

```sql
SELECT
    department,
    employee,
    salary,
    RANK() OVER (PARTITION BY department ORDER BY salary DESC) AS dept_rank,
    salary - LAG(salary) OVER (PARTITION BY department ORDER BY salary) AS diff_from_prev,
    SUM(salary) OVER (PARTITION BY department) AS dept_total
FROM employees;
```

Remember the evaluation order: `WHERE` filters rows, `GROUP BY`
groups them, `HAVING` filters groups, then window functions run
after `HAVING`.

### Optimization workflow

1. **Read the EXPLAIN plan.** Look for sequential scans on large
   tables, nested loops on big result sets, sort operations without
   indexes.
2. **Check indexes.** Ensure `WHERE`, `JOIN`, and `ORDER BY` columns
   are covered.
3. **Reduce data early.** Filter in `WHERE` before joining; join on
   indexed columns.
4. **Rewrite anti-patterns.** Functions on indexed columns,
   correlated subqueries, `NOT IN` with NULLs, implicit type casts —
   all defeat indexes.

### Indexing strategy

- **B-tree** (default) — equality, range, `ORDER BY`, `LIKE
  'prefix%'`.
- **Hash** — equality only, smaller than B-tree (Postgres).
- **GIN** — full-text search, JSONB containment, array operations.
- **GiST** — geometric data, range types, nearest-neighbor.
- **Composite** — column order matters (leftmost prefix rule).
- **Partial** — `WHERE is_active = true` for selective predicates.
- **Covering** — `INCLUDE` columns to enable index-only scans.

See `references/schema-design.md` for full schema-design patterns
and `references/query-patterns.md` for query recipes.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **SELECT * in production queries.** Selecting all columns returns data the application doesn't use, breaks when columns are added or removed, and prevents index-only scans. Always name the columns you need explicitly so the query is stable, the network payload is minimal, and the planner can use covering indexes.
- **Functions on indexed columns in WHERE clauses.** Writing `WHERE LOWER(email) = ?` or `WHERE DATE(created_at) = ?` forces the planner to evaluate the function on every row, making the index on `email` or `created_at` unusable. Use a functional index, normalize the data at write time, or rewrite the predicate to operate on the raw column.
- **NOT IN with a subquery that can return NULLs.** `WHERE id NOT IN (SELECT user_id FROM orders)` returns zero rows if any `user_id` in `orders` is NULL, because `x NOT IN (..., NULL, ...)` evaluates to NULL for every x. Use `NOT EXISTS` or filter out NULLs in the subquery explicitly.
- **Correlated subqueries that execute once per row.** A subquery in the SELECT list or WHERE clause that references the outer query's row runs once for every row in the outer result set — O(n) database round-trips. Rewrite as a JOIN or a lateral join so the planner can execute it once.
- **Missing LIMIT on exploratory queries run against production.** A full-table scan without a LIMIT clause can lock up a production database for minutes and saturate I/O. Add `LIMIT 100` to any ad hoc query until you know the cardinality, and run exploratory queries against a read replica.
- **Implicit type coercions in join conditions.** Joining `orders.user_id` (INTEGER) to `users.id` (BIGINT) or `VARCHAR` forces the planner to cast one side for every comparison, often disabling index use. Match column types on both sides of every join; fix mismatches in the schema rather than relying on implicit casting.
- **OFFSET pagination for deep pages.** `LIMIT 20 OFFSET 10000` forces the database to scan and discard 10,000 rows to return 20. For large offsets, performance degrades linearly. Use keyset (cursor) pagination — `WHERE id > last_seen_id ORDER BY id LIMIT 20` — which uses the index and runs in constant time regardless of page depth.

## Full reference

### Pagination: keyset over offset

Offset pagination is simple but degrades on deep pages because the
database must read and discard the skipped rows. Keyset (cursor)
pagination is constant-time at any depth:

```sql
-- First page
SELECT id, name, created_at
FROM products
ORDER BY created_at DESC, id DESC
LIMIT 20;

-- Next page (using last row's values as cursor)
SELECT id, name, created_at
FROM products
WHERE (created_at, id) < ('2025-06-15 10:30:00', 4502)
ORDER BY created_at DESC, id DESC
LIMIT 20;
```

Add a composite index on `(created_at DESC, id DESC)` to support
the sort.

### Deduplication

Keep the latest row per key with a window function:

```sql
WITH ranked AS (
    SELECT *,
        ROW_NUMBER() OVER (
            PARTITION BY email ORDER BY updated_at DESC
        ) AS rn
    FROM contacts
)
SELECT * FROM ranked WHERE rn = 1;
```

Or delete duplicates in place:

```sql
DELETE FROM contacts c
USING (
    SELECT email, MAX(updated_at) AS keep_date
    FROM contacts GROUP BY email HAVING COUNT(*) > 1
) dupes
WHERE c.email = dupes.email AND c.updated_at < dupes.keep_date;
```

### Pivot via conditional aggregation

```sql
SELECT
    department,
    COUNT(*) FILTER (WHERE status = 'active')   AS active_count,
    COUNT(*) FILTER (WHERE status = 'inactive') AS inactive_count,
    COUNT(*) FILTER (WHERE status = 'pending')  AS pending_count
FROM employees
GROUP BY department;
```

For engines without `FILTER`, fall back to `SUM(CASE WHEN ... THEN
1 ELSE 0 END)`.

### Recursive CTEs

Tree traversal with a depth limit to prevent runaway recursion on
cyclic data:

```sql
WITH RECURSIVE org_tree AS (
    SELECT id, name, manager_id, 1 AS depth, ARRAY[name] AS path
    FROM employees
    WHERE manager_id IS NULL

    UNION ALL

    SELECT e.id, e.name, e.manager_id, t.depth + 1, t.path || e.name
    FROM employees e
    JOIN org_tree t ON e.manager_id = t.id
    WHERE t.depth < 10
)
SELECT id, name, depth, array_to_string(path, ' > ') AS chain
FROM org_tree
ORDER BY path;
```

### Upsert

```sql
INSERT INTO metrics (sensor_id, reading_date, value)
VALUES ('temp-01', '2025-06-15', 23.5)
ON CONFLICT (sensor_id, reading_date)
DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();
```

MySQL uses `ON DUPLICATE KEY UPDATE`; SQLite uses similar
`ON CONFLICT` syntax.

### Anti-join (rows in A not in B)

Prefer `NOT EXISTS` over `NOT IN` — `NOT IN` returns no rows if any
NULL appears in the subquery result:

```sql
SELECT c.id, c.name
FROM customers c
WHERE NOT EXISTS (
    SELECT 1 FROM orders o WHERE o.customer_id = c.id
);
```

### Composite index column order

The leftmost-prefix rule determines which queries an index supports.
Given `(status, created_at, category_id)`:

```sql
WHERE status = 'active'                                      -- yes
WHERE status = 'active' AND created_at > '2025-01-01'        -- yes
WHERE status = 'active' AND created_at > '2025-01-01'
  AND category_id = 5                                        -- yes
WHERE created_at > '2025-01-01'                              -- no
WHERE category_id = 5                                        -- no
```

Put equality columns first, then range columns, then sort columns.

### Standard table boilerplate

```sql
CREATE TABLE example (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- domain columns here
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Use `TIMESTAMPTZ`, never `TIMESTAMP` without zone. Name constraints
explicitly (`chk_orders_positive_total`, `uq_users_email`) so error
messages are readable.

### Transactions and concurrency

- Use explicit transactions for multi-statement operations.
- Pick the right isolation level: `READ COMMITTED` (default, fine
  for most), `REPEATABLE READ` (consistent snapshots),
  `SERIALIZABLE` (full isolation, retry on conflict).
- Keep transactions short — never wait for user input inside one.
- Use `SELECT ... FOR UPDATE` when reading rows you intend to
  modify.
- Handle deadlocks with retry logic, not longer timeouts.

### Anti-patterns

- **`SELECT *` in production** — fetches columns the caller does not
  need and breaks when schemas evolve.
- **Functions on indexed columns** — `WHERE YEAR(created_at) = 2025`
  defeats the index; use a range instead.
- **`NOT IN` with nullable subqueries** — use `NOT EXISTS`.
- **Correlated subqueries that run per row** — rewrite as a join or
  CTE.
- **Missing `LIMIT` on exploratory queries** — easy way to OOM a
  client.
- **Implicit type conversions** — comparing a `BIGINT` column to a
  string literal can prevent index use.
- **Storing money as floats** — use `DECIMAL(19,4)` or integer
  cents.
- **Missing foreign keys "for flexibility"** — let the database
  enforce integrity.
