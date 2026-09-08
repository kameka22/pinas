//! Persisted notifications + live broadcast to WebSocket clients.
//!
//! Producers call `NotificationService::new(db).notify(...)`; the WebSocket layer
//! subscribes to `NotificationService::subscribe()` to push new entries to browsers.

use std::sync::OnceLock;

use sqlx::SqlitePool;
use thiserror::Error;
use tokio::sync::broadcast;

use crate::models::notification::Notification;

static BROADCAST: OnceLock<broadcast::Sender<Notification>> = OnceLock::new();

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Success,
    Warning,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Info => "info",
            Level::Success => "success",
            Level::Warning => "warning",
            Level::Error => "error",
        }
    }
}

pub struct NotificationService {
    db: SqlitePool,
}

impl NotificationService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Install the process-wide broadcast channel (called once by `main`).
    pub fn init_broadcast(capacity: usize) {
        let (tx, _) = broadcast::channel(capacity);
        let _ = BROADCAST.set(tx);
    }

    /// Live stream of newly created notifications (for the WebSocket handler)
    pub fn subscribe() -> Option<broadcast::Receiver<Notification>> {
        BROADCAST.get().map(|tx| tx.subscribe())
    }

    /// Create a notification. With a `dedupe_key`, an existing *unread* entry for the same
    /// key is left untouched (returns `None`); a *read* one is refreshed and re-surfaced.
    pub async fn notify(
        &self,
        level: Level,
        source: &str,
        title: &str,
        message: &str,
        dedupe_key: Option<&str>,
    ) -> Result<Option<Notification>, NotificationError> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(key) = dedupe_key {
            let existing: Option<Notification> =
                sqlx::query_as("SELECT * FROM notifications WHERE dedupe_key = ?")
                    .bind(key)
                    .fetch_optional(&self.db)
                    .await?;
            if let Some(existing) = existing {
                if !existing.read {
                    return Ok(None);
                }
                sqlx::query(
                    "UPDATE notifications SET level = ?, title = ?, message = ?, read = FALSE, created_at = ? WHERE id = ?",
                )
                .bind(level.as_str())
                .bind(title)
                .bind(message)
                .bind(&now)
                .bind(&existing.id)
                .execute(&self.db)
                .await?;
                let refreshed = Notification {
                    level: level.as_str().to_string(),
                    title: title.to_string(),
                    message: message.to_string(),
                    read: false,
                    created_at: now,
                    ..existing
                };
                Self::broadcast(&refreshed);
                return Ok(Some(refreshed));
            }
        }

        let notification = Notification {
            id: uuid::Uuid::new_v4().to_string(),
            level: level.as_str().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            source: source.to_string(),
            read: false,
            created_at: now,
            dedupe_key: dedupe_key.map(|k| k.to_string()),
        };
        sqlx::query(
            "INSERT INTO notifications (id, level, title, message, source, read, created_at, dedupe_key) VALUES (?, ?, ?, ?, ?, FALSE, ?, ?)",
        )
        .bind(&notification.id)
        .bind(&notification.level)
        .bind(&notification.title)
        .bind(&notification.message)
        .bind(&notification.source)
        .bind(&notification.created_at)
        .bind(&notification.dedupe_key)
        .execute(&self.db)
        .await?;

        Self::broadcast(&notification);
        Ok(Some(notification))
    }

    fn broadcast(notification: &Notification) {
        if let Some(tx) = BROADCAST.get() {
            let _ = tx.send(notification.clone());
        }
    }

    pub async fn list(&self, limit: i64, unread_only: bool) -> Result<Vec<Notification>, NotificationError> {
        let rows = if unread_only {
            sqlx::query_as("SELECT * FROM notifications WHERE read = FALSE ORDER BY created_at DESC LIMIT ?")
                .bind(limit)
                .fetch_all(&self.db)
                .await?
        } else {
            sqlx::query_as("SELECT * FROM notifications ORDER BY created_at DESC LIMIT ?")
                .bind(limit)
                .fetch_all(&self.db)
                .await?
        };
        Ok(rows)
    }

    pub async fn unread_count(&self) -> Result<i64, NotificationError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE read = FALSE")
            .fetch_one(&self.db)
            .await?;
        Ok(count)
    }

    pub async fn mark_read(&self, id: &str) -> Result<(), NotificationError> {
        let result = sqlx::query("UPDATE notifications SET read = TRUE WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(NotificationError::NotFound);
        }
        Ok(())
    }

    pub async fn mark_all_read(&self) -> Result<u64, NotificationError> {
        let result = sqlx::query("UPDATE notifications SET read = TRUE WHERE read = FALSE")
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(&self, id: &str) -> Result<(), NotificationError> {
        let result = sqlx::query("DELETE FROM notifications WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(NotificationError::NotFound);
        }
        Ok(())
    }

    pub async fn clear(&self) -> Result<u64, NotificationError> {
        let result = sqlx::query("DELETE FROM notifications").execute(&self.db).await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::migrated_pool;

    #[tokio::test]
    async fn create_list_read_delete() {
        let pool = migrated_pool().await;
        let svc = NotificationService::new(pool);
        svc.notify(Level::Info, "system", "Hello", "first", None).await.unwrap();
        svc.notify(Level::Error, "storage", "Pool degraded", "sda missing", None).await.unwrap();

        assert_eq!(svc.unread_count().await.unwrap(), 2);
        let all = svc.list(10, false).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].title, "Pool degraded", "newest first");

        svc.mark_read(&all[0].id).await.unwrap();
        assert_eq!(svc.unread_count().await.unwrap(), 1);
        assert_eq!(svc.list(10, true).await.unwrap().len(), 1);

        svc.delete(&all[1].id).await.unwrap();
        assert!(matches!(svc.delete(&all[1].id).await, Err(NotificationError::NotFound)));
        assert_eq!(svc.mark_all_read().await.unwrap(), 0);
        assert_eq!(svc.clear().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn dedupe_key_collapses_repeats_until_read() {
        let pool = migrated_pool().await;
        let svc = NotificationService::new(pool);
        let first = svc.notify(Level::Warning, "update", "Update", "0.11 available", Some("update:0.11")).await.unwrap();
        assert!(first.is_some());
        // same condition while still unread: silently ignored
        let again = svc.notify(Level::Warning, "update", "Update", "0.11 available", Some("update:0.11")).await.unwrap();
        assert!(again.is_none());
        assert_eq!(svc.list(10, false).await.unwrap().len(), 1);

        // once read, the condition re-surfaces as unread with the new message
        svc.mark_all_read().await.unwrap();
        let resurfaced = svc.notify(Level::Warning, "update", "Update", "still there", Some("update:0.11")).await.unwrap().unwrap();
        assert_eq!(resurfaced.id, first.unwrap().id);
        assert!(!resurfaced.read);
        assert_eq!(svc.list(10, true).await.unwrap()[0].message, "still there");
    }
}
