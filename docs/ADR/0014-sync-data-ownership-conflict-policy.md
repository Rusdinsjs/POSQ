# ADR-0014: Synchronization Engine Data Ownership and Conflict Policy

- Status: Accepted
- Date: 2026-07-23
- Implements: SYNC-001, Gelombang B of POSQ Master Prompt

## Context

POSQ operates as an offline-first desktop application with local operational storage in SQLite and central consolidated storage in PostgreSQL. To guarantee transactional integrity, data safety, and zero loss of financial or inventory audit trails, POSQ requires a formal data ownership matrix and conflict resolution strategy.

## Decision

### 1. Data Ownership Matrix

| Aggregate Group | Primary Authority | Modification Pattern | Conflict Policy |
|---|---|---|---|
| Sales Orders, Payments, Refunds, Shift Logs, Cash Movements | Originating Device / Outlet | Append-only after posting; Reversal / Adjustment only | Deduplicate by `event_id` / `idempotency_key` |
| Inventory Movement Ledger | Originating Device / Outlet | Append-only ledger entries (`stock_before`, `qty_change`, `stock_after`) | Merge movement ledgers; Server projects consolidated balance |
| Consolidated Inventory Balance | Server Projection | Derived from ledger history | Never overwrite raw `quantity_on_hand` |
| Product, Category, Price List, Customer Master | Primary Server or Authorized Outlet | Versioned master (`aggregate_version`, ETag) | Optimistic Concurrency; Conflict inbox if version mismatch |
| Merchant, Outlet, Device Registration, Entitlement, License | Server Control Plane | Server-authoritative snapshot | Server wins; pushed to local SQLite read-only cache |
| Preset, Capability Definitions, Outlet Profile | Server / Owner Permissioned | Versioned and audited | Server / Owner override; Deterministic resolution |
| Audit Trail Log | Both Local & Server | Append-only; Immutable | Append-only merge |

### 2. Transactional Outbox & Inbox Policy
- Local changes to domain aggregates MUST write a corresponding event into `sync_outbox` within the exact same SQLite transaction.
- Outbound event payloads MUST carry schema version (`schema_version`), aggregate version (`aggregate_version`), device ID, tenant ID, actor ID, UTC timestamp, and UUIDv7 `event_id`.
- Applying inbound events into local SQLite or server PostgreSQL MUST be strictly idempotent using `sync_inbox` deduplication records.

### 3. Inventory Reconciliation Policy
- Devices NEVER send raw stock quantity overrides during normal sales operations.
- Out-of-stock / overselling risks while offline are business risks resolved via reservation policies, post-sync reconciliation adjustments, or operational safety thresholds.

---

## Consequences

- Financial transactions (orders, payments, cash movements) can never be overwritten or silent-dropped.
- Master data conflicts trigger audit records in `sync_conflicts` rather than silent Last-Write-Wins.
- Sync engine behavior is fully predictable and testable across offline-online boundaries.
