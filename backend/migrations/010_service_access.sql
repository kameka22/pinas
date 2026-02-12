-- Service access control per user/group (SMB, NFS, FTP)
-- Default: no record = no access (disabled by default)
CREATE TABLE IF NOT EXISTS service_access (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT,
    group_id TEXT,
    service TEXT NOT NULL CHECK(service IN ('smb', 'nfs', 'ftp')),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES user_groups(id) ON DELETE CASCADE,
    CHECK (
        (user_id IS NOT NULL AND group_id IS NULL) OR
        (user_id IS NULL AND group_id IS NOT NULL)
    ),
    UNIQUE(user_id, service),
    UNIQUE(group_id, service)
);
