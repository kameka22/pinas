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
/// Row of `list_all_grouped`: id, path, user_id, username, group_id, group_name, permission
type GroupedPermissionRow = (String, String, Option<String>, Option<String>, Option<String>, Option<String>, String);
/// Row of `list_by_folder`: id, user_id, username, group_id, group_name, permission
type FolderPermissionRow = (String, Option<String>, Option<String>, Option<String>, Option<String>, String);

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
        let rows: Vec<GroupedPermissionRow> = sqlx::query_as(
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
        let rows: Vec<FolderPermissionRow> = sqlx::query_as(
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

            let mut query_builder = sqlx::query_as::<_, FolderPermission>(sqlx::AssertSqlSafe(query));
            for gid in &group_ids {
                query_builder = query_builder.bind(gid);
            }

            let group_perms = query_builder.fetch_all(&self.db).await?;
            permissions.extend(group_perms);
        }

        Ok(permissions)
    }

    /// Most specific permission granted to `user_id` (directly or through a
    /// group) on `path` or one of its ancestors; `None` when no entry applies.
    /// On equal specificity the highest level wins.
    pub async fn explicit_permission(
        &self,
        user_id: &str,
        path: &str,
    ) -> Result<Option<PermissionLevel>, PermissionError> {
        let all_perms = self.list_by_user(user_id).await?;
        Ok(most_specific_permission(&all_perms, path))
    }

    /// True when at least one user or group has read or write access configured
    /// exactly on `path` (Samba then restricts the share to them).
    pub async fn folder_has_acl(&self, path: &str) -> Result<bool, PermissionError> {
        Ok(self
            .list_by_folder(path)
            .await?
            .iter()
            .any(|e| matches!(e.permission.as_str(), "read" | "write")))
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

/// Resolve `path` against a user's permission entries (see
/// `PermissionService::list_by_user`): the entry on the longest matching
/// ancestor wins, the highest level on a tie, `None` when nothing applies.
pub fn most_specific_permission(perms: &[FolderPermission], path: &str) -> Option<PermissionLevel> {
    let check_path = path.trim_end_matches('/');
    let mut best: Option<(usize, PermissionLevel)> = None;
    for perm in perms {
        let perm_path = perm.path.trim_end_matches('/');
        let applies = check_path == perm_path
            || check_path
                .strip_prefix(perm_path)
                .is_some_and(|rest| rest.starts_with('/'));
        if !applies {
            continue;
        }
        let level = perm.permission_level();
        let better = match best {
            None => true,
            Some((len, current)) => {
                perm_path.len() > len || (perm_path.len() == len && level as u8 > current as u8)
            }
        };
        if better {
            best = Some((perm_path.len(), level));
        }
    }
    best.map(|(_, level)| level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::permission::PermissionLevel;
    use crate::test_util::{insert_user, migrated_pool};

    #[tokio::test]
    async fn most_specific_path_wins_and_groups_apply() {
        let pool = migrated_pool().await;
        let alice = insert_user(&pool, "alice", false).await;
        let group = crate::services::group::create_group(&pool, "family", None).await.unwrap();
        crate::services::group::add_member(&pool, &group.id, &alice).await.unwrap();

        let svc = PermissionService::new(pool.clone());
        svc.create("/srv/data", None, Some(&group.id), PermissionLevel::Write).await.unwrap();
        svc.create("/srv/data/private", Some(&alice), None, PermissionLevel::Read).await.unwrap();

        // inherited from the group on the parent folder
        assert_eq!(svc.explicit_permission(&alice, "/srv/data/photos").await.unwrap(), Some(PermissionLevel::Write));
        // the more specific user rule downgrades to read-only
        assert_eq!(svc.explicit_permission(&alice, "/srv/data/private/x").await.unwrap(), Some(PermissionLevel::Read));
        assert_eq!(svc.explicit_permission(&alice, "/srv/data/private/").await.unwrap(), Some(PermissionLevel::Read));
        // unrelated path: nothing
        assert_eq!(svc.explicit_permission(&alice, "/srv/other").await.unwrap(), None);
        // a folder that merely shares a prefix is not a child
        assert_eq!(svc.explicit_permission(&alice, "/srv/data2").await.unwrap(), None);
    }

    #[tokio::test]
    async fn explicit_none_denies_and_acl_detection() {
        let pool = migrated_pool().await;
        let carol = insert_user(&pool, "carol", false).await;
        let dave = insert_user(&pool, "dave", false).await;
        let svc = PermissionService::new(pool.clone());

        assert!(!svc.folder_has_acl("/srv/share").await.unwrap());
        svc.create("/srv/share", Some(&carol), None, PermissionLevel::Write).await.unwrap();
        svc.create("/srv/share/secret", Some(&carol), None, PermissionLevel::None).await.unwrap();
        assert!(svc.folder_has_acl("/srv/share").await.unwrap());
        // an explicit "none" on a sub-folder overrides the parent grant
        assert_eq!(svc.explicit_permission(&carol, "/srv/share/secret/a").await.unwrap(), Some(PermissionLevel::None));
        // another user has nothing explicit
        assert_eq!(svc.explicit_permission(&dave, "/srv/share").await.unwrap(), None);
    }

    #[tokio::test]
    async fn duplicate_and_invalid_entries_are_rejected() {
        let pool = migrated_pool().await;
        let bob = insert_user(&pool, "bob", false).await;
        let svc = PermissionService::new(pool.clone());

        svc.create("/srv/a", Some(&bob), None, PermissionLevel::Read).await.unwrap();
        assert!(matches!(
            svc.create("/srv/a", Some(&bob), None, PermissionLevel::Write).await,
            Err(PermissionError::AlreadyExists)
        ));
        assert!(svc.create("/srv/a", None, None, PermissionLevel::Read).await.is_err());

        let entries = svc.list_by_folder("/srv/a").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].username.as_deref(), Some("bob"));
    }
}
