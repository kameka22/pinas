-- Migration 005: Storage Manager (Pools & Volumes)
-- Gestion des pools de stockage et volumes pour disques additionnels

-- Storage pools (agrégation de disques)
CREATE TABLE IF NOT EXISTS storage_pools (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    raid_type TEXT NOT NULL CHECK(raid_type IN ('basic', 'jbod', 'raid0', 'raid1', 'raid5', 'raid10', 'btrfs-single', 'btrfs-raid0', 'btrfs-raid1', 'btrfs-raid10')),
    status TEXT NOT NULL DEFAULT 'normal' CHECK(status IN ('normal', 'degraded', 'rebuilding', 'error', 'creating')),
    devices TEXT NOT NULL,              -- JSON array of device paths (by-id preferred)
    total_size INTEGER DEFAULT 0,       -- Total size in bytes
    metadata TEXT,                      -- JSON metadata (btrfs UUID, mdadm array info, etc.)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Volumes (filesystems within pools)
CREATE TABLE IF NOT EXISTS storage_volumes (
    id TEXT PRIMARY KEY NOT NULL,
    pool_id TEXT NOT NULL REFERENCES storage_pools(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    fs_type TEXT NOT NULL CHECK(fs_type IN ('ext4', 'btrfs', 'xfs', 'f2fs')),
    mount_point TEXT NOT NULL UNIQUE,   -- /storage/pools/{pool_name}/{volume_name}
    size_bytes INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'mounted' CHECK(status IN ('mounted', 'unmounted', 'error', 'creating')),
    mount_options TEXT,                 -- Mount options (defaults, noatime, etc.)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(pool_id, name)
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_storage_volumes_pool_id ON storage_volumes(pool_id);
CREATE INDEX IF NOT EXISTS idx_storage_volumes_mount_point ON storage_volumes(mount_point);
CREATE INDEX IF NOT EXISTS idx_storage_pools_status ON storage_pools(status);
