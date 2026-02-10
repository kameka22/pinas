use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use thiserror::Error;

use crate::models::session::Session;

/// Hash a token with SHA-256 for secure storage
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Session service errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Session expired")]
    Expired,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Create a new session (stores hashed token in DB)
pub async fn create_session(
    db: &SqlitePool,
    user_id: &str,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<Session, SessionError> {
    let token_hash = hash_token(token);
    let session = Session::new(
        user_id.to_string(),
        token_hash,
        expires_at.to_rfc3339(),
    );

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, token, expires_at, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.user_id)
    .bind(&session.token)
    .bind(&session.expires_at)
    .bind(&session.created_at)
    .execute(db)
    .await?;

    Ok(session)
}

/// Get a session by token (looks up by hash)
pub async fn get_session_by_token(
    db: &SqlitePool,
    token: &str,
) -> Result<Option<Session>, SessionError> {
    let token_hash = hash_token(token);
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE token = ?")
        .bind(&token_hash)
        .fetch_optional(db)
        .await?;

    Ok(session)
}

/// Delete a session by token (looks up by hash)
pub async fn delete_session(db: &SqlitePool, token: &str) -> Result<(), SessionError> {
    let token_hash = hash_token(token);
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(&token_hash)
        .execute(db)
        .await?;

    Ok(())
}

/// Delete all sessions for a user
pub async fn delete_user_sessions(db: &SqlitePool, user_id: &str) -> Result<(), SessionError> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(())
}

/// Cleanup expired sessions
pub async fn cleanup_expired_sessions(db: &SqlitePool) -> Result<u64, SessionError> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
        .bind(&now)
        .execute(db)
        .await?;

    Ok(result.rows_affected())
}

/// Check if a session is valid (exists and not expired)
pub async fn is_session_valid(db: &SqlitePool, token: &str) -> Result<bool, SessionError> {
    let session = get_session_by_token(db, token).await?;

    match session {
        Some(s) => {
            let expires_at = DateTime::parse_from_rfc3339(&s.expires_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(expires_at > Utc::now())
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE sessions (
                id TEXT PRIMARY KEY NOT NULL,
                user_id TEXT NOT NULL,
                token TEXT UNIQUE NOT NULL,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let pool = setup_test_db().await;
        let expires_at = Utc::now() + Duration::hours(24);

        let session = create_session(&pool, "user-123", "token-abc", expires_at)
            .await
            .unwrap();

        assert_eq!(session.user_id, "user-123");
        // Token is stored as SHA-256 hash, not plaintext
        assert_ne!(session.token, "token-abc");
        assert_eq!(session.token, hash_token("token-abc"));

        // Lookup by original token still works (hashed internally)
        let fetched = get_session_by_token(&pool, "token-abc").await.unwrap();
        assert!(fetched.is_some());
    }

    #[tokio::test]
    async fn test_delete_session() {
        let pool = setup_test_db().await;
        let expires_at = Utc::now() + Duration::hours(24);

        create_session(&pool, "user-123", "token-abc", expires_at)
            .await
            .unwrap();

        delete_session(&pool, "token-abc").await.unwrap();

        let fetched = get_session_by_token(&pool, "token-abc").await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_is_session_valid() {
        let pool = setup_test_db().await;

        // Valid session
        let expires_at = Utc::now() + Duration::hours(24);
        create_session(&pool, "user-123", "valid-token", expires_at)
            .await
            .unwrap();
        assert!(is_session_valid(&pool, "valid-token").await.unwrap());

        // Expired session
        let expired_at = Utc::now() - Duration::hours(1);
        create_session(&pool, "user-456", "expired-token", expired_at)
            .await
            .unwrap();
        assert!(!is_session_valid(&pool, "expired-token").await.unwrap());

        // Non-existent session
        assert!(!is_session_valid(&pool, "nonexistent").await.unwrap());
    }
}
