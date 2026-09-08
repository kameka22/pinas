-- Security: audit trail of login attempts (REMEDIATION_PLAN P4.3)
CREATE TABLE IF NOT EXISTS login_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    ip TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_login_attempts_created ON login_attempts(created_at DESC);

-- Hardware & Power: scheduled reboot / shutdown (REMEDIATION_PLAN P4.2)
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('reboot', 'shutdown')),
    time TEXT NOT NULL,          -- "HH:MM" local time
    days TEXT NOT NULL,          -- comma-separated: mon,tue,wed,thu,fri,sat,sun
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    last_run_at TEXT,
    created_at TEXT NOT NULL
);
