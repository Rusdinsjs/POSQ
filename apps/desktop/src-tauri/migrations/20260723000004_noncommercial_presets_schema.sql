-- Migration: 20260723000004_noncommercial_presets_schema.sql
-- Description: Schema for Non-Commercial & Institutional Presets (Donation, Cooperative, Public Service Fee, Internal Warehouse)

-- 1. Nonprofit & Donation Records
CREATE TABLE IF NOT EXISTS donation_records (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    donor_name TEXT NOT NULL,
    donor_phone TEXT,
    campaign_name TEXT NOT NULL DEFAULT 'General Fund',
    fund_type TEXT NOT NULL DEFAULT 'unrestricted', -- unrestricted, restricted, zakat, infaq
    amount INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- 2. Cooperative Members & Patronage Points
CREATE TABLE IF NOT EXISTS cooperative_members (
    id TEXT PRIMARY KEY,
    member_no TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    phone TEXT,
    patronage_points INTEGER NOT NULL DEFAULT 0,
    current_savings INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- 3. Internal Warehouse Issues
CREATE TABLE IF NOT EXISTS internal_warehouse_issues (
    id TEXT PRIMARY KEY,
    cost_center TEXT NOT NULL,
    requester_name TEXT NOT NULL,
    product_id TEXT NOT NULL,
    qty REAL NOT NULL DEFAULT 1.0,
    unit_cost INTEGER NOT NULL DEFAULT 0,
    issued_at TEXT NOT NULL
);
