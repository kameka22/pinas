use sqlx::SqlitePool;
use std::collections::HashMap;
use thiserror::Error;

use crate::models::permission::{
    FolderPermission, FolderPermissions, PermissionEntry, PermissionLevel,
};

/// Permission service errors
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Permission not found")]
    NotFound,

    #[error("Invalid permission: must specify either user_id or group_id")]
    InvalidPermission,

    #[error("Permission already exists")]
    AlreadyExists,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Permission Service for managing folder access permissions
pub struct PermissionService {
    db: SqlitePool,
}

impl PermissionService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create a new folder permission
    pub async fn create(
        &self,
        path: &str,
        user_id: Option<&str>,
        group_id: Option<&str>,
        permission: PermissionLevel,
    ) -> Result<FolderPermission, PermissionError> {
        // Validate: must have exactly one of user_id or group_id
        if (user_id.is_some() && group_id.is_some()) || (user_id.is_none() && group_id.is_none()) {
            return Err(PermissionError::InvalidPermission);
        }

        // Check if permission already exists
        let existing = if let Some(uid) = user_id {
            self.get_by_path_and_user(path, uid).await?
        } else if let Some(gid) = group_id {
            self.get_by_path_and_group(path, gid).await?
        } else {
            None
        };

        if existing.is_some() {
            return Err(PermissionError::AlreadyExists);
        }

        let perm = FolderPermission::new(
            path.to_string(),
            user_id.map(|s| s.to_string()),
            group_id.map(|s| s.to_string()),
            permission,
        );

        sqlx::query(
            r#"
            INSERT INTO folder_permissions (id, path, user_id, group_id, permission, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&perm.id)
        .bind(&perm.path)
        .bind(&perm.user_id)
        .bind(&perm.group_id)
        .bind(&perm.permission)
        .bind(&perm.created_at)
        .bind(&perm.updated_at)
        .execute(&self.db)
        .await?;

        Ok(perm)
    }

    /// Update an existing permission
    pub async fn update(
        &self,
        id: &str,
        permission: PermissionLevel,
    ) -> Result<FolderPermission, PermissionError> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE folder_permissions SET permission = ?, updated_at = ? WHERE id = ?",
        )
        .bind(permission.as_str())
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PermissionError::NotFound);
        }

        self.get_by_id(id).await?.ok_or(PermissionError::NotFound)
    }

    /// Delete a permission
    pub async fn delete(&self, id: &str) -> Result<(), PermissionError> {
        let result = sqlx::query("DELETE FROM folder_permissions WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PermissionError::NotFound);
        }

        Ok(())
    }

    /// Get permission by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<FolderPermission>, PermissionError> {
        let perm = sqlx::query_as::<_, FolderPermission>(
            "SELECT * FROM folder_permissions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(perm)
    }

    /// Get permission by path and user
    pub async fn get_by_path_and_user(
        &self,
        path: &str,
        user_id: &str,
    ) -> Result<Option<FolderPermission>, PermissionError> {
        let perm = sqlx::query_as::<_, FolderPermission>(
            "SELECT * FROM folder_permissions WHERE path = ? AND user_id = ?",
        )
        .bind(path)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(perm)
    }

    /// Get permission by path and group
    pub async fn get_by_path_and_group(
        &self,
        path: &str,
        group_id: &str,
    ) -> Result<Option<FolderPermission>, PermissionError> {
        let perm = sqlx::query_as::<_, FolderPermission>(
            "SELECT * FROM folder_permissions WHERE path = ? AND group_id = ?",
        )
        .bind(path)
        .bind(group_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(perm)
    }

    /// List all permissions grouped by folder
    pub async fn list_all_grouped(&self) -> Result<Vec<FolderPermissions>, PermissionError> {
        let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, Option<String>, String)> = sqlx::query_as(
            r#"
            SELECT
                fp.id,
                fp.path,
                fp.user_id,
                u.username,
                fp.group_id,
                g.name as group_name,
                fp.permission
            FROM folder_permissions fp
            LEFT JOIN users u ON fp.user_id = u.id
            LEFT JOIN user_groups g ON fp.group_id = g.id
            ORDER BY fp.path, u.username, g.name
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        // Group by path
        let mut grouped: HashMap<String, Vec<PermissionEntry>> = HashMap::new();
        for (id, path, user_id, username, group_id, group_name, permission) in rows {
            let entry = PermissionEntry {
                id,
                user_id,
                username,
                group_id,
                group_name,
                permission,
            };
            grouped.entry(path).or_default().push(entry);
        }

        let result: Vec<FolderPermissions> = grouped
            .into_iter()
            .map(|(path, permissions)| FolderPermissions { path, permissions })
            .collect();

        Ok(result)
    }

    /// List permissions for a specific folder
    pub async fn list_by_folder(&self, path: &str) -> Result<Vec<PermissionEntry>, PermissionError> {
        let rows: Vec<(String, Option<String>, Option<String>, Option<String>, Option<String>, String)> = sqlx::query_as(
            r#"
            SELECT
                fp.id,
                fp.user_id,
                u.username,
                fp.group_id,
                g.name as group_name,
                fp.permission
            FROM folder_permissions fp
            LEFT JOIN users u ON fp.user_id = u.id
            LEFT JOIN user_groups g ON fp.group_id = g.id
            WHERE fp.path = ?
            ORDER BY u.username, g.name
            "#,
        )
        .bind(path)
        .fetch_all(&self.db)
        .await?;

        let entries: Vec<PermissionEntry> = rows
            .into_iter()
            .map(|(id, user_id, username, group_id, group_name, permission)| PermissionEntry {
                id,
                user_id,
                username,
                group_id,
                group_name,
                permission,
            })
            .collect();

        Ok(entries)
    }

    /// List permissions for a specific user (direct + via groups)
    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<FolderPermission>, PermissionError> {
        // Get user's groups
        let group_ids: Vec<(String,)> = sqlx::query_as(
            "SELECT group_id FROM user_group_members WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let group_ids: Vec<String> = group_ids.into_iter().map(|(id,)| id).collect();

        // Get direct permissions
        let mut permissions: Vec<FolderPermission> = sqlx::query_as(
            "SELECT * FROM folder_permissions WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        // Get group permissions
        if !group_ids.is_empty() {
            let placeholders = group_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT * FROM folder_permissions WHERE group_id IN ({})",
                placeholders
            );

            let mut query_builder = sqlx::query_as::<_, FolderPermission>(&query);
            for gid in &group_ids {
                query_builder = query_builder.bind(gid);
            }

            let group_perms = query_builder.fetch_all(&self.db).await?;
            permissions.extend(group_perms);
        }

        Ok(permissions)
    }

    /// Get effective permission for a user on a path
    /// Returns the highest permission level found (write > read > none)
    pub async fn get_effective_permission(
        &self,
        user_id: &str,
        path: &str,
    ) -> Result<PermissionLevel, PermissionError> {
        // Get all permissions for this user
        let all_perms = self.list_by_user(user_id).await?;

        // Find the best matching permission for this path
        // A permission matches if the path starts with the permission path
        let mut best_permission = PermissionLevel::None;
        let mut best_match_len = 0;

        for perm in all_perms {
            // Check if this permission applies to the path
            // Path must start with the permission path
            let perm_path = perm.path.trim_end_matches('/');
            let check_path = path.trim_end_matches('/');

            if check_path == perm_path || check_path.starts_with(&format!("{}/", perm_path)) {
                let match_len = perm_path.len();

                // Use the most specific (longest) matching permission
                // If same length, use the highest permission level
                if match_len > best_match_len
                    || (match_len == best_match_len
                        && perm.permission_level() as u8 > best_permission as u8)
                {
                    best_permission = perm.permission_level();
                    best_match_len = match_len;
                }
            }
        }

        Ok(best_permission)
    }

    /// Check if user can access a path (read permission)
    pub async fn can_read(&self, user_id: &str, path: &str) -> Result<bool, PermissionError> {
        let perm = self.get_effective_permission(user_id, path).await?;
        Ok(perm.can_read())
    }

    /// Check if user can write to a path
    pub async fn can_write(&self, user_id: &str, path: &str) -> Result<bool, PermissionError> {
        let perm = self.get_effective_permission(user_id, path).await?;
        Ok(perm.can_write())
    }

    /// Get list of all folders with permissions configured
    pub async fn list_configured_folders(&self) -> Result<Vec<String>, PermissionError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT path FROM folder_permissions ORDER BY path",
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|(path,)| path).collect())
    }

    /// Create or update a permission (upsert)
    pub async fn upsert(
        &self,
        path: &str,
        user_id: Option<&str>,
        group_id: Option<&str>,
        permission: PermissionLevel,
    ) -> Result<FolderPermission, PermissionError> {
        // Validate: must have exactly one of user_id or group_id
        if (user_id.is_some() && group_id.is_some()) || (user_id.is_none() && group_id.is_none()) {
            return Err(PermissionError::InvalidPermission);
        }

        // Check if permission already exists
        let existing = if let Some(uid) = user_id {
            self.get_by_path_and_user(path, uid).await?
        } else if let Some(gid) = group_id {
            self.get_by_path_and_group(path, gid).await?
        } else {
            None
        };

        if let Some(existing) = existing {
            // Update existing
            self.update(&existing.id, permission).await
        } else {
            // Create new
            self.create(path, user_id, group_id, permission).await
        }
    }
}
