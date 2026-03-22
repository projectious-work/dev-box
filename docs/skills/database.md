# Database Skills

Skills for SQL, data modeling, NoSQL patterns, and schema migrations.

---

### sql-patterns

> SQL query patterns, schema design, and optimization. Joins, CTEs, window functions, indexing, and anti-patterns. Use when writing SQL queries, designing schemas, optimizing database performance, or reviewing database code.

**Triggers:** Writing or optimizing SQL queries (joins, CTEs, window functions), designing or reviewing schemas, analyzing EXPLAIN plans, choosing indexing strategies, fixing slow queries, implementing pagination or analytical queries.
**Tools:** `Bash` `Read` `Write`
**References:** `query-patterns.md`, `schema-design.md`

Key capabilities:

- Query construction: join types (INNER, LEFT, FULL OUTER, CROSS), CTEs over nested subqueries, window functions (RANK, LAG, SUM OVER)
- Aggregation evaluation order: WHERE, GROUP BY, HAVING, window functions
- Query optimization process: EXPLAIN plan reading, index verification, early filtering, anti-pattern avoidance
- Anti-patterns: SELECT *, functions on indexed columns, NOT IN with NULLs, correlated subqueries, missing LIMIT, implicit type conversions
- Indexing strategy: B-tree, Hash, GIN, GiST, composite (leftmost prefix rule), partial, covering (INCLUDE)
- Transactions and concurrency: isolation levels (READ COMMITTED, REPEATABLE READ, SERIALIZABLE), SELECT FOR UPDATE, deadlock handling
- Schema design basics: normalize to 3NF by default, surrogate keys, timestamps, foreign keys, named constraints

??? example "Example usage"
    A query is slow. The agent runs EXPLAIN ANALYZE, identifies a sequential scan
    on a 2M-row table caused by a function call on an indexed column in the WHERE
    clause. Rewrites the condition to use a range comparison, confirms the index
    is now used, and shows before/after execution times.

---

### database-modeling

> Data modeling approaches including ER diagrams, normalization, denormalization trade-offs, and schema evolution. Use when designing database schemas, evaluating data models, or planning schema migrations.

**Triggers:** Designing a database schema for a new feature or project, evaluating an existing data model, choosing between normalized and denormalized designs, modeling complex relationships, planning schema evolution.
**Tools:** None
**References:** `modeling-patterns.md`

Key capabilities:

- Requirements gathering: entities, cardinality, access patterns, data volume, consistency requirements
- Conceptual modeling with Mermaid ER diagrams and standard cardinality notation
- Normalization to 3NF: atomic values (1NF), full key dependency (2NF), no transitive dependencies (3NF)
- Denormalization trade-offs: summary tables, embedded lookups, materialized views -- only when measured performance problems exist
- Relationship patterns: junction tables, polymorphic associations, single/class-table inheritance, adjacency lists, nested sets, materialized paths, JSONB columns
- Schema evolution: expand/contract pattern, nullable additions, additive-only changes, independent API/schema versioning
- Polyglot persistence decisions: PostgreSQL as default, with guidance for Elasticsearch, Redis, TimescaleDB, Neo4j, MongoDB, Kafka by data type

??? example "Example usage"
    Model a comment system where comments can belong to posts, images, or videos.
    The agent presents three options: polymorphic association with commentable_type
    + commentable_id, separate junction tables per parent type, and shared parent
    table with class-table inheritance. Recommends separate junction tables for
    foreign key integrity, with a union view for display queries.

---

### nosql-patterns

> NoSQL database patterns for document, key-value, graph, and wide-column stores. Access-pattern-driven design and consistency models. Use when choosing or designing NoSQL data models.

**Triggers:** Choosing between NoSQL database types, designing document/key-value/graph/wide-column data models, optimizing access patterns, understanding consistency models, migrating between relational and NoSQL.
**Tools:** None
**References:** None

Key capabilities:

- Store type selection: document (MongoDB), key-value (Redis, DynamoDB), wide-column (Cassandra), graph (Neo4j) -- matched to access patterns
- Access-pattern-driven design: list operations first, design data to serve queries directly, accept duplication
- Document store patterns: embedding vs referencing decision criteria, bucket pattern for time-series
- Key-value patterns: colon-separated hierarchical key naming, TTL caching, rate limiting with INCR+EXPIRE, sorted sets, streams
- DynamoDB single-table design: composite PK/SK keys, multiple entity types per table, GSIs for alternate access patterns
- Consistency models: strong, eventual, causal, read-your-writes -- with practical guidance on when to use each
- CAP theorem: AP vs CP trade-offs, default to eventual consistency unless stale reads cause financial or safety harm

??? example "Example usage"
    Design a Redis caching layer for an API. The agent proposes cache-aside with
    TTL-based expiration, keys following resource:id:variant naming, SET with EX
    for individual resources, sorted sets for paginated list caches, DEL on
    write-through for invalidation, and a circuit breaker fallback to the database
    if Redis is unreachable.

---

### database-migration

> Schema migration workflows including zero-downtime migrations, data backfills, and rollback strategies. Use when planning database migrations, reviewing migration scripts, or troubleshooting migration failures.

**Triggers:** Writing or reviewing schema migrations, planning zero-downtime migrations for production, backfilling data after schema changes, setting up migration tooling, rolling back failed migrations, avoiding locking and data loss.
**Tools:** `Bash` `Read` `Write`
**References:** None

Key capabilities:

- Migration file structure: up/down scripts, sequential versioning, one logical change per migration, immutable once applied
- Zero-downtime expand/contract pattern: add new structure, backfill and update code, remove old structure
- Safe operations by database: PostgreSQL and MySQL compatibility matrix for ALTER TABLE operations
- PostgreSQL-specific: CREATE INDEX CONCURRENTLY, NOT NULL with NOT VALID + VALIDATE CONSTRAINT
- Data backfills: batch processing (1,000-10,000 rows), throttling with sleep between batches, replication lag monitoring
- Rollback strategies: down migrations, forward-fix, point-in-time recovery (PITR) as last resort
- Pre-production checklist: backup confirmation, staging test with production-like data, execution time measurement, rollback command ready
- Version tracking with schema_migrations table; tool support for Flyway, Alembic, Knex, Diesel, Ecto, and others
- Common pitfalls: large table locking, missing FK indexes, DDL inside long transactions, NULL handling during backfill, irreversible migrations without backup

??? example "Example usage"
    Rename the `username` column to `handle` without downtime. The agent plans a
    3-step expand/contract migration: add `handle` column, deploy dual-write code
    and backfill in 5,000-row batches, then drop `username` once all reads are
    migrated. Writes three migration files with up/down scripts.
