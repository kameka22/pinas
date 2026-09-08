//! Shared helpers for unit tests: an in-memory SQLite database with the real migrations applied.
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

/// Fresh in-memory database with every migration applied.
/// `max_connections(1)`: each SQLite in-memory connection is its own database.
pub async fn migrated_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite");
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrations");
    pool
}

/// Insert a user row directly (no password hashing) and return its id.
pub async fn insert_user(pool: &SqlitePool, username: &str, is_admin: bool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, email, is_admin, created_at, updated_at) VALUES (?, ?, 'x', NULL, ?, ?, ?)",
    )
    .bind(&id).bind(username).bind(is_admin).bind(&now).bind(&now)
    .execute(pool).await.expect("insert user");
    id
}
