# ADR-0013: Local Operational Database is SQLite (per-device file)

- Status: Accepted
- Date: 2026-07-17
- Supersedes: DEC-005 / DEC-035 (PostgreSQL local per device), LOCAL_POSTGRESQL_STRATEGY.md, and the PostgreSQL wording in DATA_MODEL.md §3 and EXPLICITLY REJECTED "SQLite as primary operational DB".

## Context

The original blueprint (ADR-0002, DEC-005, DEC-035, LOCAL_POSTGRESQL_STRATEGY.md, DATA_MODEL.md) mandated **PostgreSQL local per device** as the operational database. However, the implemented desktop app (`apps/desktop/src-tauri/src/db.rs`, `Cargo.toml`) uses **SQLite** via `sqlx::sqlite`, and `backup.rs` falls back to a mock because `pg_dump` is not present.

A review on 2026-07-17 confirmed the SQLite implementation already works for the MVP scope (single-cashier, single-device, offline-first). Re-platforming to PostgreSQL would require: rewriting `db.rs`, switching `sqlx` features, rewriting every query (placeholder `$1`, `now()`, `RETURNING`, `FOR UPDATE`), reworking backup/restore to `pg_dump`/`pg_restore`, and bundling/installing PostgreSQL on Windows for UMKM — a large rework with meaningful UX friction and no functional benefit for the MVP's single-writer workload.

## Decision

For the **MVP and Beta** phases, the local operational database is **SQLite** stored as a per-device file (`%APPDATA%/POSQ/posq.db`), accessed through the Tauri/Rust service.

The control-plane server MAY continue to use PostgreSQL (already set up in `docker-compose.yml` / `init.sql`); that choice is independent of the local operational DB.

## Consequences

### Positive
- Zero external DB server to install/run on merchant machines.
- Simpler offline-first reliability, migration, and backup (file copy / `VACUUM INTO`).
- Smaller installer, fewer moving parts for UMKM.
- Aligns implementation with the already-shipped code.

### Negative / Tradeoffs
- Single-writer concurrency model: not suitable for live multi-cashier shared stock without a future outlet-local-server design (deferred, needs a new ADR).
- No native `RETURNING`, `FOR UPDATE`, enum types, or role-based DB users. App enforces RBAC in Rust, not in the DB.
- Currency/monetary math must use `i64` minor units (sen) to avoid floating-point error; `REAL` is only for stock quantities.

### Required follow-ups
- Update `DEC-005` and `DEC-035` to reference this ADR.
- Rewrite `DATA_MODEL.md §3` to use SQLite-compatible types and syntax.
- Rewrite `LOCAL_POSTGRESQL_STRATEGY.md` into a local operational DB strategy that is SQLite-first.
- Keep `init.sql` as **server (control plane) only**.
- Add a CI guard that fails the build if a `pg_dump`/`psql`/PostgreSQL-only construct is introduced into the desktop crate without an approved ADR.

## Out of scope
- Multi-terminal / outlet-local-server mode (future ADR).
- Cloud operational sync (future ADR, enterprise opt-in only).
- Marketplace / payment-gateway sync (rejected for MVP per DEC-016/017).
