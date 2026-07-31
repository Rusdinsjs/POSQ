CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE merchants (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE merchant_users (
    id UUID PRIMARY KEY,
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE admin_users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE devices (
    id UUID PRIMARY KEY,
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    name TEXT NOT NULL,
    hardware_id TEXT UNIQUE NOT NULL,
    status TEXT NOT NULL DEFAULT 'active', -- active, revoked
    registered_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE plans (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    price_monthly INTEGER NOT NULL,
    features JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY,
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    plan_id UUID NOT NULL REFERENCES plans(id),
    status TEXT NOT NULL, -- active, grace_period, restricted_expired
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE device_licenses (
    id UUID PRIMARY KEY,
    device_id UUID NOT NULL REFERENCES devices(id),
    signed_token TEXT NOT NULL,
    issued_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE backup_metadata (
    id UUID PRIMARY KEY,
    merchant_id UUID NOT NULL REFERENCES merchants(id),
    device_id UUID NOT NULL REFERENCES devices(id),
    storage_path TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    uploaded_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE app_versions (
    id UUID PRIMARY KEY,
    version TEXT UNIQUE NOT NULL,
    release_notes TEXT,
    download_url TEXT NOT NULL,
    signature TEXT NOT NULL,
    published_at TIMESTAMPTZ DEFAULT NOW()
);
