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

/// Permission model for resource-based access control
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub principal_type: String,
    pub principal_id: String,
    pub permission: String,
    pub created_at: String,
}

impl Permission {
    pub fn new(
        resource_type: String,
        resource_id: Option<String>,
        principal_type: String,
        principal_id: String,
        permission: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            resource_type,
            resource_id,
            principal_type,
            principal_id,
            permission,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Permission level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    Read,
    Write,
    Admin,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::Read => write!(f, "read"),
            PermissionLevel::Write => write!(f, "write"),
            PermissionLevel::Admin => write!(f, "admin"),
        }
    }
}

/// Principal type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrincipalType {
    User,
    Group,
}

impl std::fmt::Display for PrincipalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrincipalType::User => write!(f, "user"),
            PrincipalType::Group => write!(f, "group"),
        }
    }
}

/// Resource type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Share,
    App,
    System,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Share => write!(f, "share"),
            ResourceType::App => write!(f, "app"),
            ResourceType::System => write!(f, "system"),
        }
    }
}
