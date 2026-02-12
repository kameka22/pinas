use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

impl Session {
    pub fn new(user_id: String, token: String, expires_at: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            token,
            expires_at,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
