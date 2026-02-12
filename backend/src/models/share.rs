use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Share {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: String, // "smb" or "nfs"
    pub enabled: bool,
    pub description: Option<String>,
    pub config: Option<String>, // JSON config
    pub created_at: String,
    pub updated_at: String,
}

impl Share {
    pub fn new(name: String, path: String, share_type: String, description: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            path,
            share_type,
            enabled: true,
            description,
            config: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
