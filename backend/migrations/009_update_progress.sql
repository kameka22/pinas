-- Add progress tracking columns to system_updates
ALTER TABLE system_updates ADD COLUMN progress_percent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE system_updates ADD COLUMN current_step TEXT;
