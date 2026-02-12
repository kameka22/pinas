use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database row for service_access table
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ServiceAccess {
    pub id: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub service: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Aggregated service access for a user (used in API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserServiceAccess {
    pub user_id: String,
    pub username: String,
    pub smb: bool,
    pub nfs: bool,
    pub ftp: bool,
}
