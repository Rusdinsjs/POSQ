-- Migration: User Management Permissions
-- Adds permissions required for the User Management module (RBAC)

-- 1. Add user & role management permissions
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'user.view',       'Lihat daftar dan detail pengguna')
    ON CONFLICT(key) DO NOTHING;
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'user.create',     'Tambah pengguna baru')
    ON CONFLICT(key) DO NOTHING;
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'user.edit',       'Edit nama dan status pengguna')
    ON CONFLICT(key) DO NOTHING;
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'user.deactivate', 'Nonaktifkan pengguna')
    ON CONFLICT(key) DO NOTHING;
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'user.reset_pin',  'Reset PIN pengguna')
    ON CONFLICT(key) DO NOTHING;
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'role.view',       'Lihat daftar role dan permission')
    ON CONFLICT(key) DO NOTHING;
INSERT INTO permissions (id, key, description) VALUES
    (lower(hex(randomblob(16))), 'role.assign',     'Assign atau cabut role pengguna')
    ON CONFLICT(key) DO NOTHING;

-- 2. Grant all user management permissions to 'owner' role (all merchants)
INSERT INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.name = 'owner'
      AND p.key IN ('user.view', 'user.create', 'user.edit', 'user.deactivate', 'user.reset_pin', 'role.view', 'role.assign')
    ON CONFLICT DO NOTHING;

-- 3. Grant limited user management permissions to 'manager' role
INSERT INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.name = 'manager'
      AND p.key IN ('user.view', 'user.edit', 'user.deactivate', 'user.reset_pin', 'role.view')
    ON CONFLICT DO NOTHING;

-- 4. Also update user_outlet_roles for owner and manager if they already exist
-- (propagate new permissions to existing outlet-level role assignments automatically through role_permissions join)
