use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Permission level for folder access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    None,
    Read,
    Write,
}

impl PermissionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionLevel::None => "none",
            PermissionLevel::Read => "read",
            PermissionLevel::Write => "write",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(PermissionLevel::None),
            "read" => Some(PermissionLevel::Read),
            "write" => Some(PermissionLevel::Write),
            _ => None,
        }
    }

    /// Check if this permission allows reading
    pub fn can_read(&self) -> bool {
        matches!(self, PermissionLevel::Read | PermissionLevel::Write)
    }

    /// Check if this permission allows writing
    pub fn can_write(&self) -> bool {
        matches!(self, PermissionLevel::Write)
    }
}

impl Default for PermissionLevel {
    fn default() -> Self {
        PermissionLevel::None
    }
}

/// Folder permission record from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FolderPermission {
    pub id: String,
    pub path: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub permission: String,
    pub created_at: String,
    pub updated_at: String,
}

impl FolderPermission {
    pub fn new(
        path: String,
        user_id: Option<String>,
        group_id: Option<String>,
        permission: PermissionLevel,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            user_id,
            group_id,
            permission: permission.as_str().to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::from_str(&self.permission).unwrap_or_default()
    }
}

/// Permission with user/group details for API responses
#[derive(Debug, Clone, Serialize)]
pub struct FolderPermissionWithDetails {
    pub id: String,
    pub path: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub permission: String,
}

/// Grouped permissions by folder for API responses
#[derive(Debug, Clone, Serialize)]
pub struct FolderPermissions {
    pub path: String,
    pub permissions: Vec<PermissionEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    pub permission: String,
}
