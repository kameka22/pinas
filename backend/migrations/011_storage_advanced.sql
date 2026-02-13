-- Phase 3: Advanced storage features (SMART schedules, power settings, snapshots)

CREATE TABLE IF NOT EXISTS smart_test_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    device_path TEXT NOT NULL,
    device_name TEXT NOT NULL,
    test_type TEXT NOT NULL CHECK(test_type IN ('short', 'long', 'conveyance', 'offline')),
    interval_hours INTEGER NOT NULL DEFAULT 168,
    last_run TEXT,
    next_run TEXT NOT NULL,
    last_result TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS disk_power_settings (
    device_path TEXT PRIMARY KEY NOT NULL,
    device_name TEXT NOT NULL,
    apm_level INTEGER,
    spindown_minutes INTEGER,
    write_cache INTEGER,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS btrfs_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    volume_id TEXT NOT NULL REFERENCES storage_volumes(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    snapshot_type TEXT NOT NULL DEFAULT 'manual' CHECK(snapshot_type IN ('manual', 'scheduled')),
    size_bytes INTEGER,
    created_at TEXT NOT NULL,
    UNIQUE(volume_id, name)
);

CREATE INDEX IF NOT EXISTS idx_smart_schedules_device ON smart_test_schedules(device_path);
CREATE INDEX IF NOT EXISTS idx_smart_schedules_next_run ON smart_test_schedules(next_run);
CREATE INDEX IF NOT EXISTS idx_btrfs_snapshots_volume ON btrfs_snapshots(volume_id);
