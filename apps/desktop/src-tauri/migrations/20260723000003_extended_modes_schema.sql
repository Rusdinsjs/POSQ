-- Migration: 20260723000003_extended_modes_schema.sql
-- Description: Schema for Grocery/Weighted, Wholesale, Rental, Membership, Consignment, and Digital Voucher modes

-- 1. Grocery & FEFO Lot tracking
CREATE TABLE IF NOT EXISTS inventory_lots (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    lot_number TEXT NOT NULL,
    expiry_date TEXT NOT NULL,
    qty_on_hand REAL NOT NULL DEFAULT 0.0,
    supplier_id TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_inventory_lots_expiry ON inventory_lots(product_id, expiry_date);

-- 2. Wholesale UOM & Credit Limits
CREATE TABLE IF NOT EXISTS uom_conversions (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    from_uom TEXT NOT NULL,
    to_uom TEXT NOT NULL,
    factor REAL NOT NULL DEFAULT 1.0,
    UNIQUE(product_id, from_uom, to_uom)
);

CREATE TABLE IF NOT EXISTS customer_credit_limits (
    customer_id TEXT PRIMARY KEY,
    credit_limit INTEGER NOT NULL DEFAULT 0,
    current_ar INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

-- 3. Rental Asset Contracts
CREATE TABLE IF NOT EXISTS rental_contracts (
    id TEXT PRIMARY KEY,
    asset_id TEXT NOT NULL,
    customer_id TEXT,
    deposit_amount INTEGER NOT NULL DEFAULT 0,
    start_at TEXT NOT NULL,
    due_at TEXT NOT NULL,
    returned_at TEXT,
    late_fee INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'reserved', -- reserved, checked_out, returned, overdue
    created_at TEXT NOT NULL
);

-- 4. Membership & Entitlements
CREATE TABLE IF NOT EXISTS membership_subscriptions (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    plan_name TEXT NOT NULL,
    remaining_credits INTEGER NOT NULL DEFAULT 0,
    valid_until TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

-- 5. Consignment & Seller Settlement
CREATE TABLE IF NOT EXISTS consignment_settlements (
    id TEXT PRIMARY KEY,
    consignor_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    total_sales INTEGER NOT NULL DEFAULT 0,
    commission_amount INTEGER NOT NULL DEFAULT 0,
    net_payout INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft', -- draft, approved, paid
    created_at TEXT NOT NULL
);
