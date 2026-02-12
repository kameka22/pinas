use sqlx::SqlitePool;
use thiserror::Error;

use crate::models::service_access::{ServiceAccess, UserServiceAccess};

#[derive(Debug, Error)]
pub enum ServiceAccessError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Invalid service: {0}")]
    InvalidService(String),
}

pub struct ServiceAccessService {
    db: SqlitePool,
}

impl ServiceAccessService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Get aggregated service access for a single user
    pub async fn get_user_access(&self, user_id: &str) -> Result<UserServiceAccess, ServiceAccessError> {
        let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&self.db)
            .await
            .unwrap_or_else(|_| "unknown".to_string());

        let rows = sqlx::query_as::<_, ServiceAccess>(
            "SELECT * FROM service_access WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let mut access = UserServiceAccess {
            user_id: user_id.to_string(),
            username,
            smb: false,
            nfs: false,
            ftp: false,
        };

        for row in rows {
            match row.service.as_str() {
                "smb" => access.smb = row.enabled,
                "nfs" => access.nfs = row.enabled,
                "ftp" => access.ftp = row.enabled,
                _ => {}
            }
        }

        Ok(access)
    }

    /// List aggregated service access for all users
    pub async fn list_all_access(&self) -> Result<Vec<UserServiceAccess>, ServiceAccessError> {
        let users: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, username FROM users ORDER BY username",
        )
        .fetch_all(&self.db)
        .await?;

        let all_access = sqlx::query_as::<_, ServiceAccess>(
            "SELECT * FROM service_access WHERE user_id IS NOT NULL",
        )
        .fetch_all(&self.db)
        .await?;

        let mut result = Vec::with_capacity(users.len());

        for (uid, uname) in &users {
            let mut access = UserServiceAccess {
                user_id: uid.clone(),
                username: uname.clone(),
                smb: false,
                nfs: false,
                ftp: false,
            };

            for row in &all_access {
                if row.user_id.as_deref() == Some(uid.as_str()) {
                    match row.service.as_str() {
                        "smb" => access.smb = row.enabled,
                        "nfs" => access.nfs = row.enabled,
                        "ftp" => access.ftp = row.enabled,
                        _ => {}
                    }
                }
            }

            result.push(access);
        }

        Ok(result)
    }

    /// Set service access for a user (upsert)
    pub async fn set_user_access(
        &self,
        user_id: &str,
        service: &str,
        enabled: bool,
    ) -> Result<(), ServiceAccessError> {
        self.validate_service(service)?;

        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO service_access (id, user_id, group_id, service, enabled, created_at, updated_at)
            VALUES (?, ?, NULL, ?, ?, ?, ?)
            ON CONFLICT(user_id, service) DO UPDATE SET enabled = excluded.enabled, updated_at = excluded.updated_at
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(service)
        .bind(enabled)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Set service access for a group (upsert)
    pub async fn set_group_access(
        &self,
        group_id: &str,
        service: &str,
        enabled: bool,
    ) -> Result<(), ServiceAccessError> {
        self.validate_service(service)?;

        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO service_access (id, user_id, group_id, service, enabled, created_at, updated_at)
            VALUES (?, NULL, ?, ?, ?, ?, ?)
            ON CONFLICT(group_id, service) DO UPDATE SET enabled = excluded.enabled, updated_at = excluded.updated_at
            "#,
        )
        .bind(&id)
        .bind(group_id)
        .bind(service)
        .bind(enabled)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get list of usernames that have SMB access enabled
    pub async fn get_smb_authorized_usernames(&self) -> Result<Vec<String>, ServiceAccessError> {
        let usernames: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT u.username
            FROM users u
            INNER JOIN service_access sa ON sa.user_id = u.id
            WHERE sa.service = 'smb' AND sa.enabled = TRUE
            ORDER BY u.username
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(usernames.into_iter().map(|(u,)| u).collect())
    }

    /// Check if a specific user has SMB access enabled
    pub async fn is_user_smb_enabled(&self, user_id: &str) -> Result<bool, ServiceAccessError> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT enabled FROM service_access WHERE user_id = ? AND service = 'smb'",
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|(e,)| e).unwrap_or(false))
    }

    fn validate_service(&self, service: &str) -> Result<(), ServiceAccessError> {
        match service {
            "smb" | "nfs" | "ftp" => Ok(()),
            _ => Err(ServiceAccessError::InvalidService(service.to_string())),
        }
    }
}
