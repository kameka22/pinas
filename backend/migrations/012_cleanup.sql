-- Cleanup: the resource-based `permissions` table (004) was superseded by
-- `folder_permissions` (006) and was never read or written by the backend.
DROP TABLE IF EXISTS permissions;
