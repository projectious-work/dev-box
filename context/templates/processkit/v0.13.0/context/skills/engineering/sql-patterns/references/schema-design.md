# Schema Design Reference

Principles and patterns for designing relational database schemas that balance
integrity, performance, and maintainability.

## Normalization Quick Reference

| Form | Rule | Violation Example |
|------|------|-------------------|
| 1NF | Atomic values, no repeating groups | `tags = "a,b,c"` in a single column |
| 2NF | No partial dependencies on composite keys | Non-key column depends on part of a composite PK |
| 3NF | No transitive dependencies | `zip_code -> city` stored alongside non-address data |
| BCNF | Every determinant is a candidate key | Rare edge cases with overlapping candidate keys |

**Default to 3NF.** Denormalize only when you have measured a performance problem
and documented the trade-off.

## When to Denormalize

Acceptable reasons to break normalization:
- **Read-heavy aggregations** — materialized summary tables refreshed on a schedule
- **Eliminating expensive joins** — embedding a rarely-changing lookup value
- **Caching computed values** — store `order_total` instead of summing line items every read
- **Reporting tables** — star/snowflake schema for analytics workloads

Always document denormalized columns with a comment explaining what they cache
and how they are kept in sync (trigger, application code, or scheduled job).

## Primary Key Strategies

| Strategy | Pros | Cons |
|----------|------|------|
| Auto-increment integer | Simple, compact, fast joins | Predictable, leaks count info |
| UUID v4 | No coordination needed, unpredictable | 16 bytes, poor index locality |
| UUID v7 (time-ordered) | Unpredictable + good index locality | 16 bytes, newer standard |
| Natural key | Self-documenting, no extra column | May change, composite keys are awkward in joins |

Recommendation: use auto-increment for internal IDs, UUID v7 for public-facing
identifiers, natural keys only when truly immutable (ISO country codes, etc.).

## Indexing Strategies

### Composite Index Column Order

The **leftmost prefix rule** determines which queries an index supports:

```sql
-- Index on (status, created_at, category_id)
-- Supports:
WHERE status = 'active'                                  -- yes (leftmost)
WHERE status = 'active' AND created_at > '2025-01-01'   -- yes (prefix)
WHERE status = 'active' AND created_at > '2025-01-01'
  AND category_id = 5                                    -- yes (full index)
-- Does NOT support:
WHERE created_at > '2025-01-01'                          -- no (skips status)
WHERE category_id = 5                                    -- no (skips status, created_at)
```

Put equality columns first, then range columns, then sort columns.

### Partial Indexes

Reduce index size by indexing only relevant rows:

```sql
-- Only 5% of orders are pending, but 90% of queries filter on pending
CREATE INDEX idx_orders_pending ON orders (created_at)
WHERE status = 'pending';
```

### Covering Indexes

Include non-key columns to enable index-only scans:

```sql
CREATE INDEX idx_users_email ON users (email) INCLUDE (name, avatar_url);
-- Query can be answered entirely from the index:
SELECT name, avatar_url FROM users WHERE email = 'user@example.com';
```

## Naming Conventions

Consistent naming prevents confusion and makes auto-generated code predictable.

| Element | Convention | Example |
|---------|-----------|---------|
| Tables | Plural, snake_case | `order_items` |
| Columns | Singular, snake_case | `created_at` |
| Primary key | `id` or `<table_singular>_id` | `id` |
| Foreign key | `<referenced_table_singular>_id` | `order_id` |
| Indexes | `idx_<table>_<columns>` | `idx_orders_status_created` |
| Unique constraints | `uq_<table>_<columns>` | `uq_users_email` |
| Check constraints | `chk_<table>_<description>` | `chk_orders_positive_total` |

## Standard Columns

Add these to every table unless there is a specific reason not to:

```sql
CREATE TABLE example (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- domain columns here
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Use `TIMESTAMPTZ` (timestamp with time zone), never `TIMESTAMP` without zone.

## Migration Patterns

### Adding a Non-Nullable Column

Never add a `NOT NULL` column without a default to an existing table with data.
Use the expand/contract pattern:

1. Add column as nullable: `ALTER TABLE t ADD COLUMN c TYPE NULL;`
2. Backfill data: `UPDATE t SET c = default_value WHERE c IS NULL;`
3. Add NOT NULL constraint: `ALTER TABLE t ALTER COLUMN c SET NOT NULL;`

### Renaming a Column (Zero-Downtime)

1. Add new column, backfill from old column
2. Update application to write to both columns
3. Deploy application to read from new column
4. Drop old column in a later migration

### Foreign Key Considerations

- Always index foreign key columns (PostgreSQL does not auto-index them)
- Use `ON DELETE CASCADE` only when child rows are truly owned by the parent
- Prefer `ON DELETE RESTRICT` (default) to catch application logic bugs
- `ON DELETE SET NULL` for optional relationships

## Common Anti-Patterns

- **EAV (Entity-Attribute-Value)** without strong justification — prefer JSONB for sparse attributes
- **Storing money as floats** — use `DECIMAL(19,4)` or integer cents
- **Overusing soft deletes** — adds complexity to every query; consider an archive table instead
- **Missing foreign keys** to "keep things flexible" — the database should enforce integrity
- **VARCHAR(255) everywhere** — choose meaningful limits or use `TEXT` with check constraints
