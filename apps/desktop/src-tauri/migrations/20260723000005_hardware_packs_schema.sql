-- Migration: 20260723000005_hardware_packs_schema.sql
-- Description: Schema for Parking Gate & Fuel/Energy Hardware Integration Packs

-- 1. Parking Gate Tickets & Vehicle Events
CREATE TABLE IF NOT EXISTS parking_tickets (
    id TEXT PRIMARY KEY,
    ticket_code TEXT UNIQUE NOT NULL,
    vehicle_type TEXT NOT NULL, -- motorcycle, car, bus_truck
    license_plate TEXT,
    entry_time TEXT NOT NULL,
    exit_time TEXT,
    duration_minutes INTEGER NOT NULL DEFAULT 0,
    total_fee INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'inside', -- inside, paid_exited, lost_ticket
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_parking_tickets_code ON parking_tickets(ticket_code);
CREATE INDEX IF NOT EXISTS idx_parking_tickets_status ON parking_tickets(status);

-- 2. Fuel & Energy Dispenser Pump Readings
CREATE TABLE IF NOT EXISTS fuel_pump_readings (
    id TEXT PRIMARY KEY,
    pump_id TEXT NOT NULL,
    fuel_type TEXT NOT NULL, -- pertalite, pertamax, solar, ev_kwh
    initial_meter REAL NOT NULL,
    final_meter REAL NOT NULL,
    volume_liters REAL NOT NULL,
    shift_id TEXT,
    recorded_at TEXT NOT NULL
);
