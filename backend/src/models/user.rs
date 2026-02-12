use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    pub fn new(username: String, password_hash: String, email: Option<String>, is_admin: bool) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            password_hash,
            email,
            is_admin,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
