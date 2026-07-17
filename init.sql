-- =============================================================================
-- POSQ Control-Plane Server Database Initialization (PostgreSQL)
-- =============================================================================
-- NOTE (ADR-0013, 2026-07-17): This script provisions the CONTROL-PLANE server
-- database ONLY (merchant accounts, devices, licenses, subscriptions, updates,
-- backup metadata). It does NOT contain operational POS tables (orders, payments,
-- inventory, stock movements). The local operational DB is SQLite per device.
--
-- This script runs when the PostgreSQL container starts for the first time

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable pgcrypto for additional cryptographic functions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create custom types
DO $$ BEGIN
    CREATE TYPE subscription_status AS ENUM ('active', 'grace_period', 'restricted_expired', 'suspended', 'cancelled');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE device_status AS ENUM ('active', 'revoked', 'suspended');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE license_status AS ENUM ('active', 'expired', 'revoked', 'rotating');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE key_status AS ENUM ('active', 'rotating', 'retired', 'compromised');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE runtime_mode AS ENUM ('active', 'grace', 'restricted_expired', 'revoked', 'suspicious_time');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE job_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'cancelled');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE admin_role AS ENUM ('admin', 'super_admin', 'release_manager');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- =============================================================================
-- Core Tables
-- =============================================================================

-- Merchants table
CREATE TABLE IF NOT EXISTS merchants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    email TEXT,
    phone TEXT,
    address TEXT,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Merchant users (owners and staff)
CREATE TABLE IF NOT EXISTS merchant_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'owner',
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_secret TEXT,
    last_login_at TIMESTAMPTZ,
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Admin users (POSQ internal staff)
CREATE TABLE IF NOT EXISTS admin_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role admin_role NOT NULL DEFAULT 'admin',
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_secret TEXT,
    last_login_at TIMESTAMPTZ,
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Device & License Tables
-- =============================================================================

-- Devices registered to merchants
CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    install_id_hash TEXT NOT NULL,
    device_fingerprint_hash TEXT,
    device_public_key_thumbprint TEXT,
    status device_status NOT NULL DEFAULT 'active',
    app_version TEXT,
    os TEXT,
    last_heartbeat_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(merchant_id, install_id_hash)
);

-- Device activation challenges
CREATE TABLE IF NOT EXISTS device_activation_challenges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    challenge_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Device nonces for heartbeat replay protection
CREATE TABLE IF NOT EXISTS device_nonces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    nonce_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- License signing keys
CREATE TABLE IF NOT EXISTS license_signing_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key_id TEXT UNIQUE NOT NULL,
    algorithm TEXT NOT NULL DEFAULT 'Ed25519',
    public_key TEXT NOT NULL,
    encrypted_private_key_ref TEXT,
    status key_status NOT NULL DEFAULT 'active',
    rotated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Update signing keys (separate from license keys)
CREATE TABLE IF NOT EXISTS update_signing_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key_id TEXT UNIQUE NOT NULL,
    algorithm TEXT NOT NULL DEFAULT 'Ed25519',
    public_key TEXT NOT NULL,
    encrypted_private_key_ref TEXT,
    status key_status NOT NULL DEFAULT 'active',
    rotated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Device licenses (issued tokens)
CREATE TABLE IF NOT EXISTS device_licenses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    token_version INTEGER NOT NULL DEFAULT 1,
    token_hash TEXT NOT NULL,
    signing_key_id UUID NOT NULL REFERENCES license_signing_keys(id),
    runtime_mode runtime_mode NOT NULL DEFAULT 'active',
    valid_until TIMESTAMPTZ NOT NULL,
    grace_until TIMESTAMPTZ,
    issued_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- =============================================================================
-- Subscription & Plans Tables
-- =============================================================================

-- Available subscription plans
CREATE TABLE IF NOT EXISTS plans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    price_monthly INTEGER NOT NULL,
    price_yearly INTEGER,
    features JSONB NOT NULL DEFAULT '{}',
    max_devices INTEGER NOT NULL DEFAULT 1,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Merchant subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES plans(id),
    status subscription_status NOT NULL DEFAULT 'active',
    valid_until TIMESTAMPTZ NOT NULL,
    grace_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Subscription events (audit trail)
CREATE TABLE IF NOT EXISTS subscription_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    old_status subscription_status,
    new_status subscription_status,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Entitlements (feature access per plan)
CREATE TABLE IF NOT EXISTS entitlements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    feature_key TEXT NOT NULL,
    feature_value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(plan_id, feature_key)
);

-- =============================================================================
-- Update & Backup Tables
-- =============================================================================

-- App versions (for updates)
CREATE TABLE IF NOT EXISTS app_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version TEXT NOT NULL,
    channel TEXT NOT NULL DEFAULT 'stable',
    os TEXT NOT NULL,
    min_supported_version TEXT,
    sha256 TEXT NOT NULL,
    signature TEXT NOT NULL,
    signing_key_id UUID NOT NULL REFERENCES update_signing_keys(id),
    download_url TEXT NOT NULL,
    release_notes TEXT,
    critical BOOLEAN DEFAULT false,
    published_at TIMESTAMPTZ DEFAULT NOW()
);

-- Backup metadata (no payload stored)
CREATE TABLE IF NOT EXISTS backup_metadata (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_id UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    backup_id TEXT UNIQUE NOT NULL,
    destination_type TEXT NOT NULL DEFAULT 'cloud',
    logical_storage_ref TEXT,
    size_bytes BIGINT NOT NULL,
    checksum TEXT NOT NULL,
    encryption_algorithm TEXT NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT true,
    app_version TEXT,
    db_schema_version TEXT,
    status TEXT NOT NULL DEFAULT 'uploaded',
    failure_code TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Operations Tables
-- =============================================================================

-- Idempotency keys for retryable operations
CREATE TABLE IF NOT EXISTS idempotency_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key TEXT NOT NULL,
    scope TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    response JSONB,
    status TEXT NOT NULL DEFAULT 'processing',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(key, scope)
);

-- Admin audit logs
CREATE TABLE IF NOT EXISTS admin_audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    admin_id UUID,
    merchant_id UUID,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    old_value JSONB,
    new_value JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Job queue for background processing
CREATE TABLE IF NOT EXISTS job_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    priority INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    retry_count INTEGER DEFAULT 0,
    last_error TEXT,
    run_after TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Indexes
-- =============================================================================

-- Merchant users indexes
CREATE INDEX IF NOT EXISTS idx_merchant_users_email ON merchant_users(email);
CREATE INDEX IF NOT EXISTS idx_merchant_users_merchant ON merchant_users(merchant_id);

-- Devices indexes
CREATE INDEX IF NOT EXISTS idx_devices_merchant ON devices(merchant_id);
CREATE INDEX IF NOT EXISTS idx_devices_install_id ON devices(merchant_id, install_id_hash);
CREATE INDEX IF NOT EXISTS idx_devices_fingerprint ON devices(device_fingerprint_hash);
CREATE INDEX IF NOT EXISTS idx_devices_public_key ON devices(device_public_key_thumbprint);
CREATE INDEX IF NOT EXISTS idx_devices_status ON devices(merchant_id, status);

-- Activation challenges indexes
CREATE INDEX IF NOT EXISTS idx_challenges_merchant ON device_activation_challenges(merchant_id);
CREATE INDEX IF NOT EXISTS idx_challenges_expires ON device_activation_challenges(expires_at);

-- Device nonces indexes
CREATE INDEX IF NOT EXISTS idx_nonces_device ON device_nonces(device_id);
CREATE INDEX IF NOT EXISTS idx_nonces_created ON device_nonces(created_at);

-- Subscriptions indexes
CREATE INDEX IF NOT EXISTS idx_subscriptions_merchant ON subscriptions(merchant_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(merchant_id, status);

-- Device licenses indexes
CREATE INDEX IF NOT EXISTS idx_licenses_device ON device_licenses(device_id);
CREATE INDEX IF NOT EXISTS idx_licenses_version ON device_licenses(device_id, token_version);

-- App versions indexes
CREATE INDEX IF NOT EXISTS idx_versions_os_channel ON app_versions(os, channel, version);

-- Backup metadata indexes
CREATE INDEX IF NOT EXISTS idx_backup_merchant ON backup_metadata(merchant_id);
CREATE INDEX IF NOT EXISTS idx_backup_device ON backup_metadata(device_id);
CREATE INDEX IF NOT EXISTS idx_backup_created ON backup_metadata(merchant_id, device_id, created_at);
CREATE INDEX IF NOT EXISTS idx_backup_id ON backup_metadata(backup_id);

-- Idempotency keys indexes
CREATE INDEX IF NOT EXISTS idx_idempotency_key ON idempotency_keys(key, scope);

-- Admin audit logs indexes
CREATE INDEX IF NOT EXISTS idx_audit_merchant ON admin_audit_logs(merchant_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON admin_audit_logs(merchant_id, created_at);

-- Job queue indexes
CREATE INDEX IF NOT EXISTS idx_job_status ON job_queue(status, run_after);
CREATE INDEX IF NOT EXISTS idx_job_type ON job_queue(job_type, status);

-- =============================================================================
-- Seed Default Plans
-- =============================================================================

INSERT INTO plans (id, name, slug, price_monthly, price_yearly, features, max_devices) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'Free', 'free', 0, 0, '{"products": 50, "reports": "basic", "support": "community"}', 1),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'Basic', 'basic', 199000, 1990000, '{"products": 500, "reports": "standard", "support": "email", "kds": true, "fb_tables": true}', 3),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'Professional', 'professional', 499000, 4990000, '{"products": -1, "reports": "advanced", "support": "priority", "kds": true, "fb_tables": true, "multi_outlet": true, "inventory": true}', 10),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'Enterprise', 'enterprise', 999000, 9990000, '{"products": -1, "reports": "advanced", "support": "dedicated", "kds": true, "fb_tables": true, "multi_outlet": true, "inventory": true, "api_access": true, "custom_integration": true}', 50)
ON CONFLICT (slug) DO NOTHING;

-- =============================================================================
-- Seed Default Entitlements
-- =============================================================================

-- Free plan entitlements
INSERT INTO entitlements (plan_id, feature_key, feature_value) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'max_products', '50'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'max_devices', '1'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'kds_enabled', 'false'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'fb_tables_enabled', 'false'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'inventory_enabled', 'false'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'multi_outlet_enabled', 'false')
ON CONFLICT (plan_id, feature_key) DO NOTHING;

-- Basic plan entitlements
INSERT INTO entitlements (plan_id, feature_key, feature_value) VALUES
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'max_products', '500'),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'max_devices', '3'),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'kds_enabled', 'true'),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'fb_tables_enabled', 'true'),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'inventory_enabled', 'false'),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'multi_outlet_enabled', 'false')
ON CONFLICT (plan_id, feature_key) DO NOTHING;

-- Professional plan entitlements
INSERT INTO entitlements (plan_id, feature_key, feature_value) VALUES
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'max_products', '-1'),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'max_devices', '10'),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'kds_enabled', 'true'),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'fb_tables_enabled', 'true'),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'inventory_enabled', 'true'),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'multi_outlet_enabled', 'true')
ON CONFLICT (plan_id, feature_key) DO NOTHING;

-- Enterprise plan entitlements
INSERT INTO entitlements (plan_id, feature_key, feature_value) VALUES
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'max_products', '-1'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'max_devices', '50'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'kds_enabled', 'true'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'fb_tables_enabled', 'true'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'inventory_enabled', 'true'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'multi_outlet_enabled', 'true'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'api_access_enabled', 'true'),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'custom_integration_enabled', 'true')
ON CONFLICT (plan_id, feature_key) DO NOTHING;

-- =============================================================================
-- Seed Default Admin User (password: admin123 - CHANGE IN PRODUCTION!)
-- =============================================================================
-- Password hash generated with Argon2id
INSERT INTO admin_users (id, email, password_hash, name, role) VALUES
    ('f0eebc99-9c0b-4ef8-bb6d-6bb9bd380a55', 'admin@posq.id', '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$somehashhere', 'POSQ Admin', 'super_admin')
ON CONFLICT (email) DO NOTHING;
