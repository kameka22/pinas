use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User Group model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl UserGroup {
    pub fn new(name: String, description: Option<String>, is_system: bool) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            is_system,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Group membership model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub id: String,
    pub user_id: String,
    pub group_id: String,
    pub created_at: String,
}

impl GroupMember {
    pub fn new(user_id: String, group_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            group_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

