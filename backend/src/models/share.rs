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

    /// Parse the config JSON into SmbShareConfig
    pub fn smb_config(&self) -> SmbShareConfig {
        self.config
            .as_ref()
            .and_then(|c| serde_json::from_str(c).ok())
            .unwrap_or_default()
    }

    /// Set SMB config from struct, serializing to JSON
    pub fn set_smb_config(&mut self, config: &SmbShareConfig) {
        self.config = Some(serde_json::to_string(config).unwrap_or_default());
    }
}

/// SMB-specific share configuration stored in shares.config JSON column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbShareConfig {
    #[serde(default)]
    pub guest_ok: bool,
    #[serde(default = "default_true")]
    pub browseable: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default = "default_create_mask")]
    pub create_mask: String,
    #[serde(default = "default_directory_mask")]
    pub directory_mask: String,
    #[serde(default = "default_veto_files")]
    pub veto_files: String,
    #[serde(default)]
    pub recycle_bin: bool,
}

fn default_true() -> bool {
    true
}
fn default_create_mask() -> String {
    "0664".to_string()
}
fn default_directory_mask() -> String {
    "0775".to_string()
}
fn default_veto_files() -> String {
    "/.DS_Store/Thumbs.db/._*/".to_string()
}

impl Default for SmbShareConfig {
    fn default() -> Self {
        Self {
            guest_ok: false,
            browseable: true,
            read_only: false,
            create_mask: "0664".to_string(),
            directory_mask: "0775".to_string(),
            veto_files: "/.DS_Store/Thumbs.db/._*/".to_string(),
            recycle_bin: false,
        }
    }
}

/// Global Samba configuration (stored in settings table with smb.* keys)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbGlobalConfig {
    pub workgroup: String,
    pub server_string: String,
    pub min_protocol: String,
    pub max_protocol: String,
}

impl Default for SmbGlobalConfig {
    fn default() -> Self {
        Self {
            workgroup: "WORKGROUP".to_string(),
            server_string: "PiNAS".to_string(),
            min_protocol: "SMB2".to_string(),
            max_protocol: "SMB3".to_string(),
        }
    }
}

/// Samba service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SambaStatus {
    pub enabled: bool,
    pub running: bool,
    pub share_count: u32,
    pub connected_users: u32,
    pub version: Option<String>,
}

/// Share info for API responses (enriched with parsed config and permissions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub config: SmbShareConfig,
    pub permissions: Vec<SharePermissionEntry>,
    pub created_at: String,
    pub updated_at: String,
}

/// Permission entry for a share (resolved user/group name)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePermissionEntry {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub permission: String, // "none", "read", "write"
}

impl From<Share> for ShareInfo {
    fn from(share: Share) -> Self {
        let config = share.smb_config();
        Self {
            id: share.id,
            name: share.name,
            path: share.path,
            share_type: share.share_type,
            enabled: share.enabled,
            description: share.description,
            config,
            permissions: Vec::new(),
            created_at: share.created_at,
            updated_at: share.updated_at,
        }
    }
}
