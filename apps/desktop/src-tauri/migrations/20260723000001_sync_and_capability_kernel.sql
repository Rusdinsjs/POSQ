-- Migration: 20260723000001_sync_and_capability_kernel.sql
-- Description: Creates SQLite tables for Sync Outbox/Inbox engine and Capability Kernel

-- Sync Outbox for local transactional events
CREATE TABLE IF NOT EXISTS sync_outbox (
    id TEXT PRIMARY KEY,
    event_id TEXT UNIQUE NOT NULL,
    event_type TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    aggregate_version INTEGER NOT NULL DEFAULT 1,
    schema_version INTEGER NOT NULL DEFAULT 1,
    merchant_id TEXT NOT NULL DEFAULT 'default_merchant',
    outlet_id TEXT NOT NULL DEFAULT 'default_outlet',
    device_id TEXT NOT NULL DEFAULT 'default_device',
    actor_id TEXT,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, pushed, failed
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TEXT NOT NULL,
    pushed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_sync_outbox_status ON sync_outbox(status);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_created ON sync_outbox(created_at);

-- Sync Inbox for inbound deduplication and event processing
CREATE TABLE IF NOT EXISTS sync_inbox (
    id TEXT PRIMARY KEY,
    event_id TEXT UNIQUE NOT NULL,
    event_type TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'applied', -- applied, failed
    applied_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_inbox_event_id ON sync_inbox(event_id);

-- Monotonic pull cursors
CREATE TABLE IF NOT EXISTS sync_cursors (
    cursor_key TEXT PRIMARY KEY,
    last_position INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

-- Conflict records for unresolvable master data edit collisions
CREATE TABLE IF NOT EXISTS sync_conflicts (
    id TEXT PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    client_version INTEGER NOT NULL,
    server_version INTEGER NOT NULL,
    conflict_type TEXT NOT NULL,
    client_payload TEXT NOT NULL,
    server_payload TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unresolved', -- unresolved, resolved_client, resolved_server, resolved_manual
    resolved_at TEXT,
    resolved_by TEXT,
    created_at TEXT NOT NULL
);

-- Dead letter queue for events failing threshold retries
CREATE TABLE IF NOT EXISTS sync_dead_letters (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    failure_reason TEXT NOT NULL,
    retry_count INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

-- Local device registration snapshot
CREATE TABLE IF NOT EXISTS device_registration (
    device_id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL,
    outlet_id TEXT NOT NULL,
    device_name TEXT NOT NULL,
    device_token TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    registered_at TEXT NOT NULL,
    last_sync_at TEXT
);

-- Business presets definitions
CREATE TABLE IF NOT EXISTS business_presets (
    id TEXT PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    default_capabilities_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Capability definitions
CREATE TABLE IF NOT EXISTS capability_definitions (
    key TEXT PRIMARY KEY,
    domain TEXT NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    dependencies_json TEXT NOT NULL DEFAULT '[]',
    conflicts_json TEXT NOT NULL DEFAULT '[]',
    default_config_json TEXT NOT NULL DEFAULT '{}'
);

-- Outlet profiles
CREATE TABLE IF NOT EXISTS outlet_profiles (
    outlet_id TEXT PRIMARY KEY,
    primary_preset_code TEXT NOT NULL DEFAULT 'general_flexible',
    preset_version INTEGER NOT NULL DEFAULT 1,
    config_version INTEGER NOT NULL DEFAULT 1,
    activated_at TEXT NOT NULL,
    activated_by TEXT
);

-- Effective capability assignments per outlet
CREATE TABLE IF NOT EXISTS outlet_capabilities (
    outlet_id TEXT NOT NULL,
    capability_key TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    source TEXT NOT NULL DEFAULT 'preset', -- preset, user_override, plan_entitlement
    config_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    PRIMARY KEY (outlet_id, capability_key)
);

-- Audit log for outlet profile & capability state modifications
CREATE TABLE IF NOT EXISTS outlet_profile_change_log (
    id TEXT PRIMARY KEY,
    outlet_id TEXT NOT NULL,
    before_json TEXT NOT NULL,
    after_json TEXT NOT NULL,
    reason TEXT NOT NULL,
    approved_by TEXT,
    created_at TEXT NOT NULL
);
