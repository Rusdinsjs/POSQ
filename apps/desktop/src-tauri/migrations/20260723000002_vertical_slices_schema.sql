-- Migration: 20260723000002_vertical_slices_schema.sql
-- Description: Creates schema for Retail Serialized, Appointments, and Repair/Workshop modes

-- 1. Retail Serialized & IMEI Tracking
CREATE TABLE IF NOT EXISTS serialized_units (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    serial_no TEXT UNIQUE NOT NULL,
    imei1 TEXT,
    imei2 TEXT,
    status TEXT NOT NULL DEFAULT 'in_stock', -- in_stock, sold, returned, reserved
    unit_cost INTEGER NOT NULL DEFAULT 0,
    warranty_months INTEGER NOT NULL DEFAULT 12,
    received_at TEXT NOT NULL,
    sold_at TEXT,
    order_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_serialized_units_product ON serialized_units(product_id);
CREATE INDEX IF NOT EXISTS idx_serialized_units_status ON serialized_units(status);

CREATE TABLE IF NOT EXISTS trade_in_records (
    id TEXT PRIMARY KEY,
    order_id TEXT,
    customer_id TEXT,
    device_name TEXT NOT NULL,
    serial_no TEXT NOT NULL,
    condition_grade TEXT NOT NULL, -- grade_a, grade_b, grade_c
    agreed_value INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- 2. Appointment & Resource Service
CREATE TABLE IF NOT EXISTS staff_resources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'staff',
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS appointments (
    id TEXT PRIMARY KEY,
    customer_id TEXT,
    staff_id TEXT NOT NULL REFERENCES staff_resources(id),
    service_product_id TEXT NOT NULL,
    scheduled_at TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL DEFAULT 60,
    status TEXT NOT NULL DEFAULT 'scheduled', -- scheduled, checked_in, in_service, completed, cancelled, no_show
    deposit_amount INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_appointments_scheduled ON appointments(scheduled_at);

-- 3. Repair / Workshop Service
CREATE TABLE IF NOT EXISTS customer_assets (
    id TEXT PRIMARY KEY,
    customer_id TEXT,
    asset_type TEXT NOT NULL, -- smartphone, laptop, vehicle, camera
    brand_model TEXT NOT NULL,
    serial_no TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS repair_tickets (
    id TEXT PRIMARY KEY,
    ticket_number TEXT UNIQUE NOT NULL,
    asset_id TEXT NOT NULL REFERENCES customer_assets(id),
    customer_id TEXT,
    problem_description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'received', -- received, diagnosing, awaiting_approval, approved, repairing, ready, collected
    estimated_cost INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
