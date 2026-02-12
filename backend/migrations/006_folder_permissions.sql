-- Folder permissions table
CREATE TABLE IF NOT EXISTS folder_permissions (
    id TEXT PRIMARY KEY NOT NULL,
    path TEXT NOT NULL,
    user_id TEXT,
    group_id TEXT,
    permission TEXT NOT NULL CHECK (permission IN ('none', 'read', 'write')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES user_groups(id) ON DELETE CASCADE,
    CHECK ((user_id IS NOT NULL AND group_id IS NULL) OR (user_id IS NULL AND group_id IS NOT NULL))
);

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_folder_permissions_path ON folder_permissions(path);
CREATE INDEX IF NOT EXISTS idx_folder_permissions_user ON folder_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_folder_permissions_group ON folder_permissions(group_id);

-- Unique constraint: one permission per user/group per path
CREATE UNIQUE INDEX IF NOT EXISTS idx_folder_permissions_unique_user ON folder_permissions(path, user_id) WHERE user_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_folder_permissions_unique_group ON folder_permissions(path, group_id) WHERE group_id IS NOT NULL;
