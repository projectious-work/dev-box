# SQL Query Patterns Reference

Common query recipes for recurring problems. All examples use standard SQL with
PostgreSQL extensions noted where applicable.

## Pagination

**Offset-based** (simple but slow for deep pages):
```sql
SELECT id, name, created_at
FROM products
ORDER BY created_at DESC, id DESC
LIMIT 20 OFFSET 60;  -- page 4
```

**Keyset/cursor-based** (consistent performance at any depth):
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
Keyset pagination requires a unique, ordered column combination. Add an index on
`(created_at DESC, id DESC)` to support the sort efficiently.

## Running Totals

```sql
SELECT
    transaction_date,
    amount,
    SUM(amount) OVER (ORDER BY transaction_date ROWS UNBOUNDED PRECEDING) AS running_total,
    AVG(amount) OVER (ORDER BY transaction_date ROWS BETWEEN 6 PRECEDING AND CURRENT ROW) AS rolling_7day_avg
FROM transactions
WHERE account_id = 42
ORDER BY transaction_date;
```

Use `ROWS UNBOUNDED PRECEDING` (not `RANGE`) when the ordering column may have
duplicates and you want deterministic results.

## Gaps and Islands

Find consecutive groups ("islands") in sequential data:

```sql
WITH numbered AS (
    SELECT
        event_date,
        status,
        event_date - (ROW_NUMBER() OVER (PARTITION BY status ORDER BY event_date) * INTERVAL '1 day') AS grp
    FROM daily_status
)
SELECT
    status,
    MIN(event_date) AS island_start,
    MAX(event_date) AS island_end,
    COUNT(*) AS duration_days
FROM numbered
GROUP BY status, grp
ORDER BY island_start;
```

This works by subtracting a row number from the date; consecutive dates in the
same status produce the same `grp` value.

## Pivot / Crosstab

**Standard SQL (conditional aggregation):**
```sql
SELECT
    department,
    COUNT(*) FILTER (WHERE status = 'active') AS active_count,
    COUNT(*) FILTER (WHERE status = 'inactive') AS inactive_count,
    COUNT(*) FILTER (WHERE status = 'pending') AS pending_count
FROM employees
GROUP BY department;
```

Without `FILTER` (MySQL, older engines):
```sql
SELECT
    department,
    SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) AS active_count,
    SUM(CASE WHEN status = 'inactive' THEN 1 ELSE 0 END) AS inactive_count,
    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) AS pending_count
FROM employees
GROUP BY department;
```

## Deduplication

**Keep the latest row per group:**
```sql
DELETE FROM contacts c
USING (
    SELECT email, MAX(updated_at) AS keep_date
    FROM contacts
    GROUP BY email
    HAVING COUNT(*) > 1
) dupes
WHERE c.email = dupes.email
AND c.updated_at < dupes.keep_date;
```

**Select distinct rows without deleting (window approach):**
```sql
WITH ranked AS (
    SELECT *,
        ROW_NUMBER() OVER (PARTITION BY email ORDER BY updated_at DESC) AS rn
    FROM contacts
)
SELECT * FROM ranked WHERE rn = 1;
```

## Recursive CTEs (Hierarchies)

**Org chart / tree traversal:**
```sql
WITH RECURSIVE org_tree AS (
    -- Base case: top-level managers
    SELECT id, name, manager_id, 1 AS depth, ARRAY[name] AS path
    FROM employees
    WHERE manager_id IS NULL

    UNION ALL

    -- Recursive case: subordinates
    SELECT e.id, e.name, e.manager_id, t.depth + 1, t.path || e.name
    FROM employees e
    JOIN org_tree t ON e.manager_id = t.id
    WHERE t.depth < 10  -- safety limit
)
SELECT id, name, depth, array_to_string(path, ' > ') AS chain
FROM org_tree
ORDER BY path;
```

Always include a depth limit to prevent infinite recursion on circular data.

## Upsert (INSERT ON CONFLICT)

```sql
INSERT INTO metrics (sensor_id, reading_date, value)
VALUES ('temp-01', '2025-06-15', 23.5)
ON CONFLICT (sensor_id, reading_date)
DO UPDATE SET
    value = EXCLUDED.value,
    updated_at = NOW();
```

MySQL equivalent uses `ON DUPLICATE KEY UPDATE`. SQLite uses `ON CONFLICT` with
similar syntax.

## Finding Missing Data

**Anti-join (rows in A not in B):**
```sql
-- Preferred: NOT EXISTS
SELECT c.id, c.name
FROM customers c
WHERE NOT EXISTS (
    SELECT 1 FROM orders o WHERE o.customer_id = c.id
);

-- Alternative: LEFT JOIN + IS NULL
SELECT c.id, c.name
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.id
WHERE o.id IS NULL;
```

Avoid `NOT IN` when the subquery column is nullable — it returns no rows if any
NULL exists in the subquery result.
