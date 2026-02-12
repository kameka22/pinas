-- System updates history
CREATE TABLE IF NOT EXISTS system_updates (
    id TEXT PRIMARY KEY NOT NULL,
    version TEXT NOT NULL,
    previous_version TEXT NOT NULL,
    update_type TEXT NOT NULL CHECK(update_type IN ('patch', 'minor', 'major')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'downloading', 'applying', 'completed', 'failed', 'reboot_required')),
    changelog TEXT,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
