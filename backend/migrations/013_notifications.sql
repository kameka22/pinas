-- Notifications become a real feature (REMEDIATION_PLAN P3): where they come from,
-- and a dedupe key so recurring conditions (degraded pool, update available) don't pile up.
ALTER TABLE notifications ADD COLUMN source TEXT NOT NULL DEFAULT 'system';
ALTER TABLE notifications ADD COLUMN dedupe_key TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_notifications_dedupe ON notifications(dedupe_key) WHERE dedupe_key IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at DESC);
