---
name: database-migration
description: |
  Schema migration workflows — zero-downtime expand/contract, batched backfills, and rollback strategies. Use when planning a schema migration, reviewing migration scripts, troubleshooting a slow or failed migration, or rolling back a deployed change.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-database-migration
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Database Migration

## Intro

Schema changes are deploys against live state, so they need the same
discipline as application releases. Use immutable, reversible
migrations and the expand/contract pattern to keep production reads
and writes flowing while the schema moves underneath them.

## Overview

### Migration file structure

Every migration has an `up` (apply) and `down` (revert) script. Tools
order them by sequential version or timestamp:

```
migrations/
  001_create_users.up.sql
  001_create_users.down.sql
  002_add_user_email_index.up.sql
  002_add_user_email_index.down.sql
```

Rules of the road:

- One logical change per migration; do not mix unrelated alterations.
- The `down` migration must fully reverse the `up` migration.
- Migrations are immutable once applied — never edit a deployed
  migration; write a new one.
- Test both directions: `up` then `down` then `up` again must leave
  the schema intact.

### Expand / migrate / contract

Never make a breaking change in a single step. The expand/contract
pattern decouples schema and code so each can ship independently.

**Phase 1 — Expand:** add the new structure alongside the old.

```sql
ALTER TABLE users ADD COLUMN display_name TEXT;
```

**Phase 2 — Migrate:** backfill data and update the application to
write to both columns and read from the new one.

```sql
UPDATE users SET display_name = name
WHERE display_name IS NULL AND id BETWEEN 1 AND 10000;
```

**Phase 3 — Contract:** remove the old structure once all readers
have moved over.

```sql
ALTER TABLE users ALTER COLUMN display_name SET NOT NULL;
-- Later, in a separate migration:
ALTER TABLE users DROP COLUMN name;
```

### Batched backfills

Large backfills must be batched to avoid long locks, replication lag,
and runaway WAL growth. Process 1,000–10,000 rows at a time, commit
between batches, and add a short sleep so replicas can catch up. Run
during low-traffic windows for very large tables and monitor lag the
whole way.

### Version tracking

Migration tools record applied versions in a metadata table:

```sql
CREATE TABLE schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Common tools — Flyway, Liquibase, golang-migrate, Alembic, Knex,
Diesel, Ecto — all follow the same pattern: ordered files plus a
tracking table.

### Example: rename a column with no downtime

To rename `username` to `handle`:

1. Add `handle` (nullable), backfill from `username` in batches.
2. Deploy code that writes both columns and reads from `handle`.
3. Add `NOT NULL` on `handle`, then drop `username` in a later
   migration once nothing reads it.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Locking a large table with ADD COLUMN ... DEFAULT.** In databases that rewrite the table to fill in the default (older PostgreSQL, MySQL without INSTANT algorithm), adding a column with a non-null default on a multi-million-row table takes a full table lock for minutes. Use a nullable column first, backfill in batches, then add the NOT NULL constraint once all rows are populated.
- **Missing index on a foreign key column.** Adding a foreign key constraint without a corresponding index on the referencing column means every delete or update on the referenced table causes a full sequential scan of the child table to check referential integrity. Always create the index before or with the foreign key.
- **DDL inside a long transaction with other writes.** A migration that runs inside the same transaction as application writes holds DDL locks for the entire transaction duration, blocking all other writers. Run DDL in short, isolated transactions; use concurrent index builds where the database supports them.
- **Forgetting NULL during a backfill before NOT NULL constraint.** The sequence is: add column nullable → backfill → add NOT NULL. If the backfill script misses any rows (e.g., soft-deleted records, rows in an archive table), adding NOT NULL will fail or silently coerce nulls to a default. Verify zero nulls remain before adding the constraint.
- **Irreversible migrations without a backup or rollback plan.** A migration that drops a column, truncates a table, or changes a type destructively cannot be undone after the fact. Always take a point-in-time backup, write a down migration before running the up migration, and validate the down migration on a staging copy before touching production.
- **Editing a deployed migration instead of writing a new one.** Changing a migration file that has already run on any environment means that environment's schema history no longer matches the file. Write a new corrective migration; treat deployed migrations as immutable history.
- **No dry-run on staging before production.** Running a migration directly on production without first running it on a staging environment of similar size means that unexpected lock durations, constraint violations, or data issues surface in production. Always run on staging with production-representative data volume first.

## Full reference

### Safe operations by database

Not all `ALTER TABLE` operations are safe; the same statement can be
free on one engine and table-rewriting on another.

| Operation | PostgreSQL | MySQL (InnoDB) |
|-----------|-----------|----------------|
| Add nullable column | Fast, no lock | Fast (8.0+), locks in 5.7 |
| Add column with default | Fast (11+) | Rewrites table in 5.7, fast in 8.0+ |
| Drop column | Fast (marks invisible) | Rewrites table |
| Add index | `CONCURRENTLY` avoids lock | `ALGORITHM=INPLACE` for online |
| Rename column | Fast | Fast (8.0+) |
| Change column type | Rewrites table, locks | Rewrites table, locks |

PostgreSQL-specific tips:

- Always use `CREATE INDEX CONCURRENTLY` to avoid blocking writes.
- `CONCURRENTLY` cannot run inside a transaction; configure the
  migration tool to disable transaction wrapping for that file.
- Add `NOT NULL` constraints with `NOT VALID` to skip checking
  existing rows, then `VALIDATE CONSTRAINT` later in a separate step.

### Backfill recipe

```sql
DO $$
DECLARE
    batch_size INT := 5000;
    max_id BIGINT;
BEGIN
    SELECT MAX(id) INTO max_id FROM users;
    FOR i IN 0..max_id/batch_size LOOP
        UPDATE users
        SET display_name = name
        WHERE display_name IS NULL
          AND id BETWEEN i * batch_size + 1 AND (i + 1) * batch_size;
        COMMIT;
        PERFORM pg_sleep(0.1);  -- throttle to reduce load
    END LOOP;
END $$;
```

Most teams run backfills from application code or a one-off script
rather than inside the migration tool, so they can pause, resume, and
report progress.

### Rollback strategies

Plan for failure before running anything in production:

- **Down migration** — the standard path. Must be tested before
  deploying.
- **Forward-fix** — sometimes rolling back is riskier than fixing
  forward (especially after a destructive migration). Write a new
  migration that corrects the issue.
- **Point-in-time recovery (PITR)** — last resort. Restores the entire
  database to a timestamp; everything written after that point is
  lost.

Pre-flight checklist:

1. Take a backup or confirm PITR is enabled.
2. Run the migration on a staging copy with production-like data.
3. Measure execution time on staging — a 2-second migration on
   staging may take 20 minutes on production.
4. Have the rollback command ready to execute before you start.

### Concurrent index example

```sql
-- up
CREATE INDEX CONCURRENTLY idx_orders_customer
    ON orders (customer_id);

-- down
DROP INDEX CONCURRENTLY idx_orders_customer;
```

### Anti-patterns

- **Locking a large table** with `ADD COLUMN ... DEFAULT x` on older
  PostgreSQL/MySQL. Use nullable + backfill instead.
- **Missing index on a new foreign key.** PostgreSQL does not create
  one automatically; deletes on the parent table will become slow
  scans.
- **DDL inside a long transaction.** PostgreSQL DDL takes
  `ACCESS EXCLUSIVE` locks; combined with long-running queries this
  cascades into lock waits across the cluster.
- **Forgetting NULL during backfill.** Application code deployed
  before the backfill completes must handle NULL in the new column.
- **Irreversible migrations without a backup.** Dropping columns or
  tables with no down migration and no PITR is data loss waiting to
  happen.
- **Editing a deployed migration.** Add a new one instead — staging
  and production hashes will diverge otherwise.
