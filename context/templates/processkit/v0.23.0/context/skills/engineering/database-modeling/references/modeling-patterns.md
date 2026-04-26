# Data Modeling Patterns Reference

Recurring patterns for handling complex relationships and structural challenges
in relational databases.

## Polymorphic Associations

One table needs to reference rows in multiple parent tables.

**Approach A: Type + ID columns (application-enforced)**
```sql
CREATE TABLE comments (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    body TEXT NOT NULL,
    commentable_type TEXT NOT NULL,  -- 'post', 'image', 'video'
    commentable_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_comments_target ON comments (commentable_type, commentable_id);
```
Pros: simple, single table. Cons: no foreign key enforcement; the database cannot
validate that `commentable_id` exists in the referenced table.

**Approach B: Separate foreign keys (nullable)**
```sql
CREATE TABLE comments (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    body TEXT NOT NULL,
    post_id BIGINT REFERENCES posts(id),
    image_id BIGINT REFERENCES images(id),
    video_id BIGINT REFERENCES videos(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_single_parent CHECK (
        (post_id IS NOT NULL)::int + (image_id IS NOT NULL)::int
        + (video_id IS NOT NULL)::int = 1
    )
);
```
Pros: real foreign keys. Cons: grows awkward beyond 3-4 parent types; NULLable columns.

**Approach C: Shared base table**
```sql
CREATE TABLE commentable_entities (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    entity_type TEXT NOT NULL
);
-- posts, images, videos each have a FK to commentable_entities
CREATE TABLE comments (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    body TEXT NOT NULL,
    commentable_entity_id BIGINT NOT NULL REFERENCES commentable_entities(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```
Pros: real FK, extensible. Cons: extra join, indirection.

## Entity-Attribute-Value (EAV)

Stores arbitrary key-value pairs per entity. Use sparingly.

```sql
CREATE TABLE product_attributes (
    product_id BIGINT NOT NULL REFERENCES products(id),
    attribute_name TEXT NOT NULL,
    attribute_value TEXT NOT NULL,
    PRIMARY KEY (product_id, attribute_name)
);
```

Problems: no type safety, no constraints per attribute, expensive queries that
filter on multiple attributes. **Prefer JSONB** for sparse attributes:

```sql
ALTER TABLE products ADD COLUMN attributes JSONB NOT NULL DEFAULT '{}';
-- Query: products with color = 'red'
SELECT * FROM products WHERE attributes @> '{"color": "red"}';
-- Index for containment queries:
CREATE INDEX idx_products_attrs ON products USING GIN (attributes);
```

## Adjacency List (Tree/Hierarchy)

Each row points to its parent.

```sql
CREATE TABLE categories (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id BIGINT REFERENCES categories(id)
);
CREATE INDEX idx_categories_parent ON categories (parent_id);
```

Querying subtrees requires recursive CTEs:
```sql
WITH RECURSIVE subtree AS (
    SELECT id, name, 0 AS depth FROM categories WHERE id = 10
    UNION ALL
    SELECT c.id, c.name, s.depth + 1
    FROM categories c JOIN subtree s ON c.parent_id = s.id
    WHERE s.depth < 20
)
SELECT * FROM subtree;
```

Best for: shallow trees, frequent inserts/moves, infrequent subtree queries.

## Nested Sets

Each node stores `lft` and `rgt` values that define its subtree range.

```sql
CREATE TABLE categories (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    lft INT NOT NULL,
    rgt INT NOT NULL
);
CREATE INDEX idx_categories_lft_rgt ON categories (lft, rgt);

-- All descendants of node with lft=2, rgt=11:
SELECT * FROM categories WHERE lft > 2 AND rgt < 11 ORDER BY lft;

-- All ancestors of node with lft=5, rgt=6:
SELECT * FROM categories WHERE lft < 5 AND rgt > 6 ORDER BY lft;
```

Best for: read-heavy trees. Cons: inserts and moves require renumbering many rows.

## Materialized Path

Store the full path from root as a string.

```sql
CREATE TABLE categories (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL  -- e.g., '/1/4/10/'
);
CREATE INDEX idx_categories_path ON categories USING btree (path text_pattern_ops);

-- All descendants of category 4:
SELECT * FROM categories WHERE path LIKE '/1/4/%';

-- Depth of a node:
SELECT (LENGTH(path) - LENGTH(REPLACE(path, '/', ''))) - 1 AS depth
FROM categories WHERE id = 10;
```

Best for: display paths, breadcrumbs, moderate-depth trees. Cons: path updates
cascade to all descendants.

## Temporal / Bi-Temporal Tables

Track historical state of data.

**Effective-time (business time):**
```sql
CREATE TABLE prices (
    product_id BIGINT NOT NULL REFERENCES products(id),
    price DECIMAL(10,2) NOT NULL,
    valid_from DATE NOT NULL,
    valid_to DATE NOT NULL DEFAULT '9999-12-31',
    PRIMARY KEY (product_id, valid_from),
    CONSTRAINT chk_valid_range CHECK (valid_from < valid_to),
    EXCLUDE USING gist (product_id WITH =, daterange(valid_from, valid_to) WITH &&)
);
-- Current price:
SELECT price FROM prices
WHERE product_id = 42 AND CURRENT_DATE >= valid_from AND CURRENT_DATE < valid_to;
```

The `EXCLUDE` constraint prevents overlapping validity ranges for the same product.

## Star Schema (Analytics)

Separate facts (events/measures) from dimensions (descriptive attributes):

```sql
-- Dimension tables (slowly changing)
CREATE TABLE dim_product (product_key SERIAL PRIMARY KEY, name TEXT, category TEXT);
CREATE TABLE dim_date (date_key INT PRIMARY KEY, full_date DATE, year INT, quarter INT, month INT);

-- Fact table (append-mostly)
CREATE TABLE fact_sales (
    sale_id BIGINT GENERATED ALWAYS AS IDENTITY,
    product_key INT REFERENCES dim_product,
    date_key INT REFERENCES dim_date,
    quantity INT NOT NULL,
    revenue DECIMAL(12,2) NOT NULL
);
CREATE INDEX idx_fact_sales_date ON fact_sales (date_key);
CREATE INDEX idx_fact_sales_product ON fact_sales (product_key);
```

Best for: analytical queries with GROUP BY on dimension attributes. Keep fact
tables narrow (keys + measures only).

## JSONB Columns (Semi-Structured Data)

Use JSONB for data that varies per row but does not need relational integrity:

```sql
ALTER TABLE events ADD COLUMN payload JSONB NOT NULL DEFAULT '{}';

-- Querying nested values:
SELECT payload->>'action' AS action,
       (payload->'metadata'->>'duration')::int AS duration_ms
FROM events
WHERE payload @> '{"source": "api"}';

-- Indexing strategies:
CREATE INDEX idx_events_payload ON events USING GIN (payload);           -- general
CREATE INDEX idx_events_source ON events ((payload->>'source'));         -- specific key
```

Guidelines:
- Use JSONB for 0-5 keys that vary per row
- If you query a JSONB key in every query, promote it to a real column
- Validate structure at the application layer or with CHECK constraints
