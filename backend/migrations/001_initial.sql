-- Initial database schema for PiNAS

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT UNIQUE NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Settings table (key-value store)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Shares table
CREATE TABLE IF NOT EXISTS shares (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL,
    path TEXT NOT NULL,
    share_type TEXT NOT NULL CHECK(share_type IN ('smb', 'nfs')),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    description TEXT,
    config TEXT, -- JSON configuration
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY NOT NULL,
    level TEXT NOT NULL CHECK(level IN ('info', 'success', 'warning', 'error')),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);

-- Insert default admin user (password: admin)
-- Password hash for "admin" using argon2
INSERT OR IGNORE INTO users (id, username, password_hash, email, is_admin, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$YWRtaW5zYWx0MTIzNDU2$K8EYVrLOIhXf8E3TPP3YvQ',
    'admin@pinas.local',
    TRUE,
    datetime('now'),
    datetime('now')
);
