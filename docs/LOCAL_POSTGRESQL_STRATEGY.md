# LOCAL OPERATIONAL DB STRATEGY (SQLite)

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan strategi database operasional lokal untuk MVP, beta, dan fase multi-terminal.

> **Amended 2026-07-17 (ADR-0013):** The MVP local operational DB is **SQLite** (one file per
> device), not PostgreSQL. The Postgres-required language below has been revised. PostgreSQL remains
> only the control-plane server database.

## 1. Decision Summary

MVP menggunakan SQLite lokal per device (satu file `posq.db` per mesin, WAL mode).

Artinya:

- Setiap komputer POS memiliki database SQLite lokal sendiri (`%LOCALAPPDATA%/POSQ/posq.db`).
- Checkout, shift, inventory, report, audit, license cache, dan backup berjalan dari database lokal tersebut.
- Server tetap control plane, bukan database transaksi.
- Multi-terminal dengan satu database outlet ditunda sampai fondasi single-device stabil.

## 2. Why Per-Device SQLite for MVP

Alasan:

- Instalasi nol-friction: tidak perlu install/setup PostgreSQL di Windows UMKM.
- Risiko concurrency multi-terminal lebih rendah (single-writer WAL cocok untuk kasir tunggal).
- Checkout offline lebih mudah dijamin.
- Migration dan backup lebih mudah diuji (`VACUUM INTO` + file copy).
- Cocok untuk target awal UMKM dan kasir tunggal.
- Mengurangi scope sebelum core POS stabil.

Catatan tradeoff (ADR-0013):

- SQLite adalah single-writer. Tidak cocok untuk beberapa kasir menulis stok real-time bersamaan
  tanpa desain tambahan (ditunda ke multi-terminal ADR).
- Tidak ada `RETURNING`, `FOR UPDATE`, enum, atau role-DB. Gunakan `INTEGER`/`TEXT`/`REAL`,
  `text` status, dan `ON CONFLICT` SQLite.
- Backup pakai `VACUUM INTO` + file copy, dienkripsi AES-256-GCM client-side.

Tradeoff:

- Tidak cocok untuk beberapa kasir aktif di outlet yang sama jika harus berbagi stok real-time.
- Data antar device tidak otomatis sama.
- Multi-outlet/multi-terminal perlu desain tambahan.

## 3. Deployment Phases

| Phase | Mode | Status | Notes |
|---|---|---|---|
| MVP | SQLite per device | Required | Single cashier/device, WAL mode |
| Beta | Optional outlet local server | Planned | Butuh ADR multi-terminal |
| Business | Outlet local server + multiple terminals | Future | Butuh LAN discovery, locking, backup central |
| Enterprise | Outlet local server + optional cloud operational sync | Future | Butuh ADR cloud operational sync |

## 4. MVP Architecture

```text
Tauri Desktop App
  -> Rust local service
  -> SQLite local on same machine (posq.db, WAL)
  -> Control plane API for license/subscription/update/backup metadata
  -> Object storage optional for encrypted backup
```

Rules:

- Checkout writes only to local DB.
- Reports read only from local DB.
- Backup reads local DB and app config.
- License state is cached locally.
- Server does not store orders/payments/inventory.

## 5. Local SQLite Operational File

The local operational DB is a single SQLite file managed by the app:

- Default path: `%LOCALAPPDATA%/POSQ/posq.db` (Windows), equivalent per-OS data dir via `dirs::data_dir`.
- Journal mode: WAL (`journal_mode=WAL`), synchronous=Normal, busy_timeout=5000ms (see `db.rs`).
- Created automatically with `create_if_missing(true)` on first launch.
- No separate DB user concept: the app process owns the file; protect with OS file permissions.

Backup/restore uses `VACUUM INTO '<backup>.db'` for an atomic snapshot, then AES-256-GCM encryption
of the snapshot file when a recovery key is supplied (see `backup.rs`). There is no `pg_dump`/`psql`
dependency.

MVP recommendation:

1. File is created and migrated automatically on first launch.
2. Installer must NOT delete the existing `posq.db` on reinstall/upgrade (reinstall preserves data).
3. Production: installer provides a repair/reset flow that first takes a backup before any destructive action.

## 6. Database Users

Not applicable to SQLite (single file owned by the app process). Operational DB has no role/DB-user
model. Access control is enforced in Rust (RBAC via `security_policy.rs` + `auth::has_permission`),
not at the DB layer. (If a future multi-terminal design adopts a server DB, revisit DB users per a
new ADR.)

## 7. Connection Configuration

Default local config (resolved in `db.rs`):

```text
DATABASE_URL=sqlite://%LOCALAPPDATA%/POSQ/posq.db   # or via dirs::data_dir
journal_mode=WAL
synchronous=Normal
busy_timeout=5000ms
max_connections=5
```

Storage:

- Override via `DATABASE_URL` environment variable if needed (e.g. portable/USB install).
- No password to store for the local file.
- Redact any path containing usernames in logs and diagnostics (SEC-003).

## 8. Health Check

Local SQLite health check (see `db.rs::check_db_health`) must verify:

- Server reachable.
- Database exists.
- App user can connect.
- Migration table exists.
- Current schema version is compatible.
- Required tables exist.
- Disk space is sufficient.
- Last backup status is visible.

Health states:

```text
OK
WARNING
ACTION_REQUIRED
BLOCKED
```

## 9. Migration Strategy

Rules:

- All schema changes use versioned migrations.
- Migration must be idempotent where possible.
- App creates backup before destructive migration.
- Migration log is required.
- Failed migration must not destroy current data.
- App must show recovery instructions.

Migration table (managed by sqlx `migrate!`, SQLite):

```text
_schema_migrations(
  version text primary key,
  description text not null,
  installed_on text not null,
  success boolean not null
)
```

## 10. Backup Strategy for Local SQLite

Backup required before:

- Migration.
- Restore.
- Major update.
- Repair operation.

Backup must include:

- `VACUUM INTO` snapshot of `posq.db` (atomic).
- App config manifest.
- Schema version.
- App version.
- Device id.
- Checksum.
- Encryption metadata if encrypted.

Implementation: see `backup.rs` (`create_local_backup`, `restore_local_backup`). The pre-restore
safety backup is mandatory before overwriting the live DB.

## 11. Multi-Terminal Future

Outlet local server mode must not be bolted on casually.

Future ADR must decide:

- How terminals discover outlet DB server.
- How shift ownership works.
- How stock locking works.
- How receipt/order numbering works.
- How backup works for shared outlet DB.
- How offline terminal works if LAN server unavailable.
- Whether local server needs service installer.

Until then:

- Do not implement cross-device operational sync.
- Do not use server control plane as transaction database.
- Do not pretend per-device DB supports live multi-cashier stock consistency.

## 12. Acceptance Criteria

- MVP can run with local SQLite per device.
- App can detect unavailable SQLite and show actionable error.
- App can run migration on fresh DB.
- App can store dummy order locally.
- App can backup local DB (VACUUM INTO + AES-256-GCM).
- Reinstall does not delete existing DB.
- Server schema does not store operational POS data by default.
