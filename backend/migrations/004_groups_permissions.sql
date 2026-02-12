-- Groups and permissions schema for PiNAS

-- User Groups table
CREATE TABLE IF NOT EXISTS user_groups (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Group membership (many-to-many relationship between users and groups)
CREATE TABLE IF NOT EXISTS user_group_members (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id TEXT NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    UNIQUE(user_id, group_id)
);

-- Permissions table for resource-based access control
CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY NOT NULL,
    resource_type TEXT NOT NULL,  -- 'share', 'app', 'system'
    resource_id TEXT,             -- NULL for system-wide permissions
    principal_type TEXT NOT NULL, -- 'user', 'group'
    principal_id TEXT NOT NULL,   -- user_id or group_id
    permission TEXT NOT NULL,     -- 'read', 'write', 'admin'
    created_at TEXT NOT NULL,
    UNIQUE(resource_type, resource_id, principal_type, principal_id)
);

-- Add description field to users (if not exists)
-- SQLite doesn't support IF NOT EXISTS for ALTER TABLE, so we use a workaround
-- This will fail silently if the column already exists in the app

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_user_group_members_user ON user_group_members(user_id);
CREATE INDEX IF NOT EXISTS idx_user_group_members_group ON user_group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_permissions_resource ON permissions(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_permissions_principal ON permissions(principal_type, principal_id);

-- Insert default system groups
INSERT OR IGNORE INTO user_groups (id, name, description, is_system, created_at, updated_at)
VALUES
    ('00000000-0000-0000-0000-000000000001', 'administrators', 'System administrators with full access', TRUE, datetime('now'), datetime('now')),
    ('00000000-0000-0000-0000-000000000002', 'users', 'Regular users with standard access', TRUE, datetime('now'), datetime('now'));

-- Note: Admin user will be added to administrators group during onboarding setup
