use sqlx::SqlitePool;
use thiserror::Error;

use crate::models::user::User;
use crate::services::auth::{hash_password, AuthError};

/// User service errors
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,

    #[error("Username already exists")]
    DuplicateUsername,

    #[error("Cannot delete yourself")]
    CannotDeleteSelf,

    #[error("Cannot delete the last admin")]
    CannotDeleteLastAdmin,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
}

/// User update struct for partial updates
#[derive(Debug, Default)]
pub struct UserUpdate {
    pub email: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub is_admin: Option<bool>,
}

/// Create a new user
pub async fn create_user(
    db: &SqlitePool,
    username: &str,
    password: &str,
    email: Option<String>,
    is_admin: bool,
) -> Result<User, UserError> {
    // Check if username already exists
    let existing = get_user_by_username(db, username).await?;
    if existing.is_some() {
        return Err(UserError::DuplicateUsername);
    }

    // Hash the password
    let password_hash = hash_password(password)?;

    // Create the user
    let user = User::new(username.to_string(), password_hash, email, is_admin);

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, email, is_admin, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.email)
    .bind(user.is_admin)
    .bind(&user.created_at)
    .bind(&user.updated_at)
    .execute(db)
    .await?;

    Ok(user)
}

/// Get a user by ID
pub async fn get_user_by_id(db: &SqlitePool, id: &str) -> Result<Option<User>, UserError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?;

    Ok(user)
}

/// Get a user by username
pub async fn get_user_by_username(
    db: &SqlitePool,
    username: &str,
) -> Result<Option<User>, UserError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(db)
        .await?;

    Ok(user)
}

/// List all users
pub async fn list_users(db: &SqlitePool) -> Result<Vec<User>, UserError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(db)
        .await?;

    Ok(users)
}

/// Update a user
pub async fn update_user(
    db: &SqlitePool,
    id: &str,
    updates: UserUpdate,
) -> Result<User, UserError> {
    // Get the existing user first
    let existing = get_user_by_id(db, id).await?.ok_or(UserError::NotFound)?;

    let now = chrono::Utc::now().to_rfc3339();

    // Build dynamic update query
    // updates.email: None = don't change, Some(None) = set to null, Some(Some(v)) = set to v
    let email = updates.email.unwrap_or_else(|| existing.email.clone());
    let is_admin = updates.is_admin.unwrap_or(existing.is_admin);

    sqlx::query(
        r#"
        UPDATE users
        SET email = ?, is_admin = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&email)
    .bind(is_admin)
    .bind(&now)
    .bind(id)
    .execute(db)
    .await?;

    // Return updated user
    get_user_by_id(db, id)
        .await?
        .ok_or(UserError::NotFound)
}

/// Delete a user
pub async fn delete_user(
    db: &SqlitePool,
    id: &str,
    current_user_id: &str,
) -> Result<(), UserError> {
    // Check if user exists
    let user = get_user_by_id(db, id).await?.ok_or(UserError::NotFound)?;

    // Cannot delete yourself
    if id == current_user_id {
        return Err(UserError::CannotDeleteSelf);
    }

    // Cannot delete the last admin
    if user.is_admin {
        let admin_count = count_admins(db).await?;
        if admin_count <= 1 {
            return Err(UserError::CannotDeleteLastAdmin);
        }
    }

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;

    Ok(())
}

/// Change a user's password
pub async fn change_password(
    db: &SqlitePool,
    id: &str,
    new_password: &str,
) -> Result<(), UserError> {
    // Check if user exists
    let _ = get_user_by_id(db, id).await?.ok_or(UserError::NotFound)?;

    // Hash the new password
    let password_hash = hash_password(new_password)?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(&password_hash)
        .bind(&now)
        .bind(id)
        .execute(db)
        .await?;

    Ok(())
}

/// Count total users
pub async fn count_users(db: &SqlitePool) -> Result<i64, UserError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await?;

    Ok(count.0)
}

/// Count admin users
pub async fn count_admins(db: &SqlitePool) -> Result<i64, UserError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_admin = TRUE")
        .fetch_one(db)
        .await?;

    Ok(count.0)
}

/// Check if any users exist
pub async fn has_any_users(db: &SqlitePool) -> Result<bool, UserError> {
    let count = count_users(db).await?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE users (
                id TEXT PRIMARY KEY NOT NULL,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                email TEXT,
                is_admin BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "testuser", "password123", Some("test@example.com".to_string()), false)
            .await
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(!user.is_admin);

        let fetched = get_user_by_id(&pool, &user.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().username, "testuser");
    }

    #[tokio::test]
    async fn test_duplicate_username() {
        let pool = setup_test_db().await;

        create_user(&pool, "testuser", "password123", None, false)
            .await
            .unwrap();

        let result = create_user(&pool, "testuser", "password456", None, false).await;
        assert!(matches!(result, Err(UserError::DuplicateUsername)));
    }

    #[tokio::test]
    async fn test_list_users() {
        let pool = setup_test_db().await;

        create_user(&pool, "user1", "pass1", None, false).await.unwrap();
        create_user(&pool, "user2", "pass2", None, true).await.unwrap();

        let users = list_users(&pool).await.unwrap();
        assert_eq!(users.len(), 2);
    }
}
